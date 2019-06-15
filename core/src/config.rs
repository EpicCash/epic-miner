use std::collections::HashMap;
use std::path::PathBuf;
use types::Algorithm;

/// CuckooMinerPlugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrinMinerPluginConfig {
	/// The type of plugin to load (i.e. filters on filename)
	pub plugin_name: String,

	///
	pub parameters: Option<HashMap<String, u32>>,
}

impl Default for GrinMinerPluginConfig {
	fn default() -> GrinMinerPluginConfig {
		GrinMinerPluginConfig {
			plugin_name: String::new(),
			parameters: None,
		}
	}
}

/// basic mining configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerConfig {
	/// Algorithm will be use to miner
	pub algorithm: Option<Algorithm>,

	/// Whether to run the tui
	pub run_tui: bool,

	/// mining loop by adding a sleep to the thread
	pub stratum_server_addr: String,

	/// login for the stratum server
	pub stratum_server_login: Option<String>,

	/// password for the stratum server
	pub stratum_server_password: Option<String>,

	/// whether tls is enabled for the stratum server
	pub stratum_server_tls_enabled: Option<bool>,

	/// plugin dir
	pub miner_plugin_dir: Option<PathBuf>,

	/// Cuckoo miner plugin configuration, one for each plugin
	pub miner_plugin_config: Vec<GrinMinerPluginConfig>,
}

impl Default for MinerConfig {
	fn default() -> MinerConfig {
		MinerConfig {
			algorithm: Some(Algorithm::RandomX),
			run_tui: false,
			miner_plugin_dir: None,
			miner_plugin_config: vec![],
			stratum_server_addr: String::from("http://127.0.0.1:13416"),
			stratum_server_login: None,
			stratum_server_password: None,
			stratum_server_tls_enabled: None,
		}
	}
}
