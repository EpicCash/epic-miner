use std::collections::HashMap;
use std::path::PathBuf;
use types::Algorithm;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
	pub device: u32,
	pub driver: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RxConfig {
	#[serde(default = "default_threads")]
	pub threads: u64,
	#[serde(default = "default_rx_bool")]
	pub jit: bool,
	#[serde(default = "default_rx_bool")]
	pub hard_aes: bool,
	#[serde(default = "default_rx_bool")]
	pub large_pages: bool,
}

fn default_threads() -> u64 {
	1
}

fn default_rx_bool() -> bool {
	false
}

impl Default for RxConfig {
	fn default() -> Self {
		RxConfig {
			jit: false,
			hard_aes: false,
			large_pages: false,
			threads: 1
		}
	}
}

/// CuckooMinerPlugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpicMinerPluginConfig {
	/// The type of plugin to load (i.e. filters on filename)
	pub plugin_name: String,

	///
	pub parameters: Option<HashMap<String, u32>>,
}

impl Default for EpicMinerPluginConfig {
	fn default() -> EpicMinerPluginConfig {
		EpicMinerPluginConfig {
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

	pub randomx_config: RxConfig,

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
	pub miner_plugin_config: Vec<EpicMinerPluginConfig>,

	// gpu devices
	pub gpu_config: Vec<GpuConfig>,
}

impl Default for MinerConfig {
	fn default() -> MinerConfig {
		MinerConfig {
			algorithm: Some(Algorithm::RandomX),
			randomx_config: RxConfig::default(),
			run_tui: false,
			miner_plugin_dir: None,
			miner_plugin_config: vec![],
			stratum_server_addr: String::from("http://127.0.0.1:13416"),
			stratum_server_login: None,
			stratum_server_password: None,
			stratum_server_tls_enabled: None,
			gpu_config: vec![],
		}
	}
}
