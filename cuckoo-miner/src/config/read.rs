use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use core::config::{MinerConfig, EpicMinerPluginConfig};
use core::errors::MinerError;
use util::LOGGER;

use {PluginConfig};

/// Transforms a set of epic-miner plugin configs to cuckoo-miner plugins configs
pub fn read_configs(
	plugin_dir: Option<PathBuf>,
	conf_in: Vec<EpicMinerPluginConfig>,
) -> Result<Vec<PluginConfig>, MinerError> {
	// Resolve a final plugin path, either config-provided or from the current executable path
	let plugin_dir_absolute_path = match plugin_dir {
		Some(path) => {
			let absolute_path = path.canonicalize().map_err(MinerError::from);
			if let Ok(path) = &absolute_path {
				debug!(
					LOGGER,
					"Using mining plugin dir provided by config: {:?}", path
				);
			};
			absolute_path
		}
		None => {
			let absolute_path =
				env::current_exe()
					.map_err(MinerError::from)
					.map(|mut env_path| {
						env_path.pop();
						// cargo test exes are a directory further down
						if env_path.ends_with("deps") {
							env_path.pop();
						}
						env_path.push("plugins");
						env_path
					});
			if let Ok(path) = &absolute_path {
				debug!(
					LOGGER,
					"No mining plugin dir provided by config. Using default plugin dir: {:?}", path
				);
			};
			absolute_path
		}
	}?;

	let mut return_vec = vec![];
	for conf in conf_in {
		let res = PluginConfig::new(plugin_dir_absolute_path.clone(), &conf.plugin_name);
		match res {
			Err(e) => {
				error!(LOGGER, "Error reading plugin config: {:?}", e);
				return Err(e);
			}
			Ok(mut c) => {
				if conf.parameters.is_some() {
					let params = conf.parameters.unwrap();
					for k in params.keys() {
						resolve_param(&mut c, k, *params.get(k).unwrap());
					}
				}
				return_vec.push(c)
			}
		}
	}
	Ok(return_vec)
}

/// resolve a read parameter to a solver param, (or not if it isn't found)
fn resolve_param(config: &mut PluginConfig, name: &str, value: u32) {
	match name {
		"nthreads" => config.params.nthreads = value,
		"ntrims" => config.params.ntrims = value,
		"cpuload" => {
			config.params.cpuload = match value {
				1 => true,
				_ => false,
			}
		}
		"device" => config.params.device = value,
		"blocks" => config.params.blocks = value,
		"tbp" => config.params.tpb = value,
		"expand" => config.params.expand = value,
		"genablocks" => config.params.genablocks = value,
		"genatpb" => config.params.genatpb = value,
		"genbtpb" => config.params.genbtpb = value,
		"trimtpb" => config.params.trimtpb = value,
		"tailtpb" => config.params.tailtpb = value,
		"recoverblocks" => config.params.recoverblocks = value,
		"recovertpb" => config.params.recovertpb = value,
		"platform" => config.params.platform = value,
		"edge_bits" => config.params.edge_bits = value,
		n => {
			warn!(LOGGER, "Configuration param: {} unknown. Ignored.", n);
		}
	};
}