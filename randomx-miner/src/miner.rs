use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::string;

use core::errors::MinerError;
use core::config::MinerConfig;
use core::{Miner, Stats, ControlMessage, Solution};

use randomx::{RxState, mine};


pub struct RxMiner {
    // randomx mining state
    pub state: Arc<RwLock<RxState>>,

    /// Job control tx
	control_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// solver loop tx
	solver_loop_txs: Vec<mpsc::Sender<ControlMessage>>,

	/// Solver has stopped and cleanly shutdown
	solver_stopped_rxs: Vec<mpsc::Receiver<ControlMessage>>,
}

impl RxMiner {
    fn teste() {}
}

unsafe impl Send for RxMiner{}
unsafe impl Sync for RxMiner{}

impl Miner for RxMiner {

    fn new(configs: &MinerConfig) -> RxMiner {
        RxMiner {
            state: Arc::new(RwLock::new(RxState::new())),
            control_txs: vec![],
            solver_loop_txs: vec![],
            solver_stopped_rxs: vec![],
        }
    }

    fn start_solvers(&mut self) -> Result<(), MinerError> {
        Ok(())
    }

    fn get_solutions(&self) -> Option<Vec<Solution>> {
        None
    }

    fn get_stats(&self) -> Result<Vec<Stats>, MinerError>{
        Ok(vec![])
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
			let _ = 
            t.send(ControlMessage::Pause);
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