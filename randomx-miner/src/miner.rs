use std::string;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use core::config::{MinerConfig, RxConfig};
use core::errors::MinerError;
use core::miner::Miner;
use core::types::AlgorithmParams;
use core::util;
use core::{ControlMessage, JobSharedData, JobSharedDataType, Solution, Stats};

use bigint::uint::U256;
use randomx::{calculate, RxState, RxAction};
use util::LOGGER;

const MAX_HASHS: u64 = 100;
const ALGORITHM_NAME: &str = "randomx";

fn timestamp() -> u64 {
	let start = SystemTime::now();
	let since_the_epoch = start
		.duration_since(UNIX_EPOCH)
		.expect("Time went backwards");
	since_the_epoch.as_millis() as u64
}


#[derive(Debug, Clone, PartialEq, Eq)]
enum EpochState {
	Waiting,
	Loading,
	Loaded,
	Running,
	Failed(String),
}

#[derive(Debug, Clone)]
struct EpochSeed {
	start_height: u64,
	end_height: u64,
	seed: [u8; 32],
	state: EpochState,
}

impl EpochSeed {
	fn new(
		start_height: u64,
		end_height: u64,
		seed: [u8; 32], ) -> Self
	{
		EpochSeed {
			start_height,
			end_height,
			seed,
			state: EpochState::Waiting,
		}
	}
}

pub struct RxMiner {
	/// Data shared across threads
	pub shared_data: Arc<RwLock<JobSharedData>>,

	// randomx mining state
	state: Arc<RwLock<RxState>>,

	/// Job control tx
	control_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// solver loop tx
	solver_loop_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// Solver has stopped and cleanly shutdown
	solver_stopped_rxs: Vec<mpsc::Receiver<ControlMessage>>,

	current_seed: [u8; 32],

	epochs: Arc<RwLock<Vec<EpochSeed>>>,

	config: RxConfig,
}

unsafe impl Send for RxMiner {}
unsafe impl Sync for RxMiner {}

impl RxMiner {
	fn create_rx_state(config: &RxConfig) -> Arc<RwLock<RxState>> {
		let mut rx_state = RxState::new();

		rx_state.full_mem = true;

		rx_state.hard_aes = config.hard_aes;
		rx_state.large_pages = config.large_pages;
		rx_state.jit_compiler = config.jit;

		Arc::new(RwLock::new(rx_state))
	}

	fn load_next_dataset(&mut self) -> Result<(), MinerError>
	{
		let mut epochs = self.epochs.clone();
		let current_seed = self.current_seed.clone();
		let threads = self.config.threads;
		let rx_state = self.state.clone();

		let is_loading = {
			let mut epochs = epochs.read().unwrap();
			(*epochs)
				.iter()
				.filter(|x| x.state == EpochState::Loading || x.state == EpochState::Loaded)
				.count() > 0
			||
			(*epochs)
				.iter()
				.filter(|x| x.state == EpochState::Waiting)
				.count() == 0
		};
		
		if is_loading {
			return Ok(());
		}

		thread::spawn(move || {
			let mut seed = [0u8; 32];
			let mut seed_changed = false;

			{
				let mut epochs = epochs.write().unwrap();
				let mut epoch_first = (*epochs)
					.iter_mut()
					.filter(|x| x.state == EpochState::Waiting && x.seed != current_seed)
					.next();

				if let Some(ref mut epoch) = epoch_first {
					seed = epoch.seed.clone();
					seed_changed = true;
					epoch.state = EpochState::Loading;
				}
			}

			if seed_changed {
				let mut result = EpochState::Loaded;
				let mut rx = rx_state.write().unwrap();

				if let Ok(RxAction::Changed) = rx.init_cache(&seed) {
					if let Err(e) = rx.init_dataset(threads as u8) {
						result = EpochState::Failed(e.to_owned());
					}
				} else {
					result = EpochState::Failed("We can't initialize a new dataset".to_owned());
				}

				let mut epochs = epochs.write().unwrap();
				let mut epoch_first = (*epochs)
					.iter_mut()
					.filter(|x| x.state == EpochState::Loading)
					.next();
	
				if let Some(ref mut epoch) = epoch_first {
					epoch.state = result;
				}
			}
		});

		Ok(())
	}

	fn swap_dataset(&mut self, height: u64) -> Result<(), &'static str> {
		let mut epochs = self.epochs.write().unwrap();
		let mut epoch = epochs
			.iter_mut()
			.filter(|x| x.start_height < height && x.end_height >= height)
			.next();

		if let Some(ref mut e) = epoch {
			match e.state.clone() {
				EpochState::Failed(e) => {
					panic!(e);
				},
				EpochState::Loaded => {},
				_ => return Ok(()),
			}

			e.state = EpochState::Running;
			self.current_seed = e.seed.clone();
			let mut rx = self.state.write().unwrap();
			rx.update_vms();
		}

		Ok(())
	}

	fn solver_thread(
		instance: usize,
		threads: u8,
		state: Arc<RwLock<RxState>>,
		shared_data: JobSharedDataType,
		epochs: Arc<RwLock<Vec<EpochSeed>>>,
		control_rx: mpsc::Receiver<ControlMessage>,
		solver_loop_rx: mpsc::Receiver<ControlMessage>,
		solver_stopped_tx: mpsc::Sender<ControlMessage>,
	) {
		{
			let mut s = shared_data.write().unwrap();
			s.stats[instance].set_plugin_name(ALGORITHM_NAME);
			s.stats[instance].set_device_name("CPU");
		}

		let mut iter_count = 0;
		let mut last_solution_time = 0;
		let mut paused = true;
		let mut vm = None;

		loop {
			if let Some(message) = solver_loop_rx.try_iter().next() {
				//debug!(LOGGER, "solver_thread - solver_loop_rx got msg: {:?}", message);
				match message {
					ControlMessage::Stop => break,
					ControlMessage::Pause => paused = true,
					ControlMessage::Resume => paused = false,
					_ => {}
				}
			}

			if paused {
				{
					let mut s = shared_data.write().unwrap();
					s.stats[instance].set_plugin_name(ALGORITHM_NAME);
					s.stats[instance].hashes_per_sec = 0;
				}

				thread::sleep(time::Duration::from_micros(100));
				continue;
			}
			{
				let mut s = shared_data.write().unwrap();
				s.stats[instance].set_plugin_name(ALGORITHM_NAME);
			}

			if let None = vm {
				let mut rx = state.write().unwrap();

				if !rx.is_initialized() {
					continue;
				}
		
				vm = Some(rx.create_vm().unwrap().clone());
			}

			let header_pre = { shared_data.read().unwrap().pre_nonce.clone() };
			let header_post = { shared_data.read().unwrap().post_nonce.clone() };
			let height = { shared_data.read().unwrap().height.clone() };
			let epochs_state = { epochs.read().unwrap().iter().filter(|x| x.state == EpochState::Running && x.start_height < height && x.end_height >= height ).count() == 0 };
			
			if epochs_state {
				{
					let mut s = shared_data.write().unwrap();
					s.stats[instance].hashes_per_sec = 0;
				}

				thread::sleep(time::Duration::from_micros(100));
				continue;
			}

			let job_id = { shared_data.read().unwrap().job_id.clone() };
			let target_difficulty = { shared_data.read().unwrap().difficulty.clone() };
			let header = util::get_next_header_data(&header_pre, &header_post);
			let nonce = header.0;
			let mut header = header.1;

			let boundary = U256::max_value()
				/ U256::from(if target_difficulty > 0 {
					target_difficulty
				} else {
					1
				});

			let vm_ref = vm.as_ref().unwrap();
			let start = timestamp();
			let results = (0..MAX_HASHS)
				.map(|x| calculate(vm_ref, &mut header, nonce + x))
				.collect::<Vec<U256>>();
			let end = timestamp();

			iter_count += MAX_HASHS;
			let still_valid = { height == shared_data.read().unwrap().height };
			if still_valid {
				let mut s = shared_data.write().unwrap();

				for (i, hash) in results.iter().enumerate().filter(|(i, &x)| x <= boundary) {
					last_solution_time = timestamp();
					s.solutions.push(Solution::new(
						job_id as u64,
						nonce + i as u64,
						AlgorithmParams::RandomX(hash.clone().into()),
					));
					break;
				}

				let mut stats = Stats {
					last_start_time: start,
					last_end_time: end,
					last_solution_time: last_solution_time,
					iterations: iter_count as u32,
					hashes_per_sec: (MAX_HASHS * 1000) / (end - start),
					..Default::default()
				};

				stats.set_plugin_name(ALGORITHM_NAME);
				stats.set_device_name("cpu");
				s.stats[instance] = stats;
			}
		}

		let _ = solver_stopped_tx.send(ControlMessage::SolverStopped(instance));
	}
}

impl Miner for RxMiner {
	fn new(configs: &MinerConfig) -> RxMiner {
		RxMiner {
			state: RxMiner::create_rx_state(&configs.randomx_config),
			control_txs: vec![],
			solver_loop_txs: vec![],
			solver_stopped_rxs: vec![],
			config: configs.randomx_config.clone(),
			shared_data: Arc::new(RwLock::new(JobSharedData::new(
				configs.randomx_config.threads as usize,
			))),
			current_seed: [u8::max_value(); 32],
			epochs: Arc::new(RwLock::new(vec![])),
		}
	}

	fn start_solvers(&mut self) -> Result<(), MinerError> {
		let s = self.state.clone();
		let threads = self.config.threads;

		for i in 0..(self.config.threads as usize) {
			let state = self.state.clone();
			let shared_data = self.shared_data.clone();
			let epochs = self.epochs.clone();

			let (control_tx, control_rx) = mpsc::channel::<ControlMessage>();
			let (solver_tx, solver_rx) = mpsc::channel::<ControlMessage>();
			let (solver_stopped_tx, solver_stopped_rx) = mpsc::channel::<ControlMessage>();

			self.control_txs.push(control_tx);
			self.solver_loop_txs.push(solver_tx);
			self.solver_stopped_rxs.push(solver_stopped_rx);

			thread::spawn(move || {
				let _ = RxMiner::solver_thread(
					i,
					threads as u8,
					state,
					shared_data,
					epochs,
					control_rx,
					solver_rx,
					solver_stopped_tx,
				);
			});
		}

		Ok(())
	}

	fn get_solutions(&self) -> Option<Vec<Solution>> {
		let mut s = self.shared_data.write().unwrap();

		let solutions = s.solutions.clone();
		s.solutions.clear();

		Some(solutions)
	}

	fn get_stats(&self) -> Result<Vec<Stats>, MinerError> {
		Ok(self.shared_data.read().unwrap().stats.clone())
	}

	fn notify(
		&mut self,
		job_id: u32,      // Job id
		height: u64,      // Job height
		pre_nonce: &str,  // Pre-nonce portion of header
		post_nonce: &str, // Post-nonce portion of header
		difficulty: u64,  /* The target difficulty, only sols greater than this difficulty will
		                   * be returned. */
	) -> Result<(), MinerError> {
		let mut paused = false;
		{
			let mut sd = self.shared_data.write().unwrap();
			if height != sd.height {
				// stop/pause any existing jobs if job is for a new
				// height
				self.pause_solvers();
				paused = true;
			}

			sd.job_id = job_id;
			sd.height = height;
			sd.pre_nonce = pre_nonce.to_owned();
			sd.post_nonce = post_nonce.to_owned();
			sd.difficulty = difficulty;
		}

		if paused {
			self.swap_dataset(height);
			self.load_next_dataset();

			self.resume_solvers();
		}

		Ok(())
	}

	fn add_epoch(
		&mut self,
		start_height: u64,
		end_height: u64,
		next_seed: [u8; 32],
	){
		let mut epochs = self.epochs.write().unwrap();
		if epochs.iter().filter(|x| x.seed == next_seed).count() == 0 {
			epochs.push(
				EpochSeed::new(start_height, end_height, next_seed, ));
		}
	}

	/// #Description
	///
	/// Stops the current job, and signals for the loaded plugin to stop
	/// processing and perform any cleanup it needs to do.
	///
	/// #Returns
	///
	/// Nothing
	fn stop_solvers(&self) {
		for t in self.control_txs.iter() {
			let _ = t.send(ControlMessage::Stop);
		}
		for t in self.solver_loop_txs.iter() {
			let _ = t.send(ControlMessage::Stop);
		}
		//debug!(LOGGER, "Stop message sent");
	}

	/// Tells current solvers to stop and wait
	fn pause_solvers(&self) {
		for t in self.control_txs.iter() {
			let _ = t.send(ControlMessage::Pause);
		}
		for t in self.solver_loop_txs.iter() {
			let _ = t.send(ControlMessage::Pause);
		}
		//debug!(LOGGER, "Pause message sent");
	}

	/// Tells current solvers to stop and wait
	fn resume_solvers(&self) {
		for t in self.control_txs.iter() {
			let _ = t.send(ControlMessage::Resume);
		}
		for t in self.solver_loop_txs.iter() {
			let _ = t.send(ControlMessage::Resume);
		}
		//debug!(LOGGER, "Resume message sent");
	}

	/// block until solvers have all exited
	fn wait_for_solver_shutdown(&self) {
		for r in self.solver_stopped_rxs.iter() {
			while let Some(message) = r.iter().next() {
				match message {
					ControlMessage::SolverStopped(i) => {
						//debug!(LOGGER, "Solver stopped: {}", i);
						break;
					}
					_ => {}
				}
			}
		}
	}
}
