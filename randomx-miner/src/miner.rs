use std::string;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use core::config::MinerConfig;
use core::errors::MinerError;
use core::miner::Miner;
use core::types::AlgorithmParams;
use core::util;
use core::{ControlMessage, JobSharedData, JobSharedDataType, Solution, Stats};

use bigint::uint::U256;
use randomx::{calculate, RxState};
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

pub struct RxMiner {
	/// Data shared across threads
	pub shared_data: Arc<RwLock<JobSharedData>>,

	cpu_threads: u8,

	// randomx mining state
	state: Arc<RwLock<RxState>>,

	/// Job control tx
	control_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// solver loop tx
	solver_loop_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// Solver has stopped and cleanly shutdown
	solver_stopped_rxs: Vec<mpsc::Receiver<ControlMessage>>,
}

unsafe impl Send for RxMiner {}
unsafe impl Sync for RxMiner {}

impl RxMiner {
	fn solver_thread(
		instance: usize,
		state: Arc<RwLock<RxState>>,
		shared_data: JobSharedDataType,
		control_rx: mpsc::Receiver<ControlMessage>,
		solver_loop_rx: mpsc::Receiver<ControlMessage>,
		solver_stopped_tx: mpsc::Sender<ControlMessage>,
	) {
		{
			let mut s = shared_data.write().unwrap();
			s.stats[instance].set_plugin_name(ALGORITHM_NAME);
			s.stats[instance].set_device_name("cpu");
		}

		let mut iter_count = 0;
		let mut last_solution_time = 0;
		let mut paused = true;

		let vm = {
			unsafe {
				let mut rx = state.write().unwrap();
				rx.create_vm().unwrap()
			}
		};

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
				thread::sleep(time::Duration::from_micros(100));
				continue;
			}
			{
				let mut s = shared_data.write().unwrap();
				s.stats[instance].set_plugin_name(ALGORITHM_NAME);
			}

			let header_pre = { shared_data.read().unwrap().pre_nonce.clone() };
			let header_post = { shared_data.read().unwrap().post_nonce.clone() };
			let height = { shared_data.read().unwrap().height.clone() };
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

			let start = timestamp();
			let results = (0..MAX_HASHS)
				.map(|x| calculate(&vm, &mut header, nonce + x))
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
		let mut rx_state = RxState::new();
		rx_state.full_mem = true;
		rx_state.hard_aes = true;
		rx_state.jit_compiler = true;
		RxMiner {
			state: Arc::new(RwLock::new(rx_state)),
			shared_data: Arc::new(RwLock::new(JobSharedData::new(
				configs.cpu_threads as usize,
			))),
			control_txs: vec![],
			solver_loop_txs: vec![],
			solver_stopped_rxs: vec![],
			cpu_threads: configs.cpu_threads,
		}
	}

	fn start_solvers(&mut self) -> Result<(), MinerError> {
		let s = self.state.clone();
		let cpu_threads = self.cpu_threads.clone();

		let th = thread::spawn(move || {
			let mut rx = s.write().unwrap();
			unsafe {
				rx.init_cache(&[0; 32], true).expect("Isn't possible initialize RandomX cache!");
				rx.init_dataset(cpu_threads).expect("Isn't possible initialize RandomX dataset!");
			}
		});

		th.join();

		for i in 0..(cpu_threads as usize) {
			let state = self.state.clone();
			let shared_data = self.shared_data.clone();

			let (control_tx, control_rx) = mpsc::channel::<ControlMessage>();
			let (solver_tx, solver_rx) = mpsc::channel::<ControlMessage>();
			let (solver_stopped_tx, solver_stopped_rx) = mpsc::channel::<ControlMessage>();

			self.control_txs.push(control_tx);
			self.solver_loop_txs.push(solver_tx);
			self.solver_stopped_rxs.push(solver_stopped_rx);

			thread::spawn(move || {
				let _ = RxMiner::solver_thread(
					i,
					state,
					shared_data,
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
		//println!("{:?}", self.shared_data.read().unwrap().stats[0].get_plugin_name());
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
		let mut sd = self.shared_data.write().unwrap();
		let mut paused = false;
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
		if paused {
			self.resume_solvers();
		}
		Ok(())
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
