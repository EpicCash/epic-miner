use errors::MinerError;
use config::MinerConfig;
use types::{Stats, Solution};

pub trait Miner: Send + Sync {

    /// Creates a new instance of a CuckooMiner with the given configuration.
	/// One PluginConfig per device
    fn new(configs: &MinerConfig) -> Self;

    /// An asynchronous -esque version of the plugin miner, which takes
	/// parts of the header and the target difficulty as input, and begins
	/// asyncronous processing to find a solution. The loaded plugin is
	/// responsible
	/// for how it wishes to manage processing or distribute the load. Once
	/// called
	/// this function will continue to find solutions over the target difficulty
	/// for the given inputs and place them into its output queue until
	/// instructed to stop.
    fn notify(
        &mut self,
		job_id: u32,      // Job id
		height: u64,      // Job height
		pre_nonce: &str,  // Pre-nonce portion of header
		post_nonce: &str, // Post-nonce portion of header
		difficulty: u64,
		seed: [u8; 32], ) -> Result<(), MinerError>;

    /// Starts solvers, ready for jobs via job control
    fn start_solvers(&mut self) -> Result<(), MinerError>;

    /// get stats for all running solvers
    fn get_stats(&self) -> Result<Vec<Stats>, MinerError>;

    /// Returns solutions if currently waiting.
    fn get_solutions(&self) -> Option<Vec<Solution>>;

    /// #Description
	///
	/// Stops the current job, and signals for the loaded plugin to stop
	/// processing and perform any cleanup it needs to do.
	///
	/// #Returns
	///
	/// Nothing
    fn stop_solvers(&self);

    /// Tells current solvers to stop and wait
    fn pause_solvers(&self);

    /// Tells current solvers to stop and wait
    fn resume_solvers(&self);

    /// block until solvers have all exited
    fn wait_for_solver_shutdown(&self);
}