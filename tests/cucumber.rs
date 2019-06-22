#[macro_use]
extern crate cucumber_rust;
extern crate cuckoo_miner as cuckoo;
extern crate randomx_miner as randomx;
extern crate rand;
extern crate grin_miner_core as core;
mod common;

use core::Miner;
use core::config::{GrinMinerPluginConfig, MinerConfig};
use core::types::Algorithm;

pub enum TargetMiner {
	Cuckoo(cuckoo::CuckooMiner),
	RandomX(randomx::RxMiner),
}

pub struct MinerWorld {
	// You can use this struct for mutable context in scenarios.
	pub plugin: String,
	pub time_in_seconds: Option<i64>,
	pub algorithm: Algorithm,
	pub miner: Option<TargetMiner>,
}

impl cucumber_rust::World for MinerWorld {}
impl std::default::Default for MinerWorld {
	fn default() -> MinerWorld {
		// This function is called every time a new scenario is started
		MinerWorld {
			plugin: String::new(),
			time_in_seconds: None,
			algorithm: Algorithm::Cuckoo,
			miner: None
		}
	}
}

mod miner_test {

	use super::*;
	use common::{mine_async_for_duration, mining_plugin_dir_for_tests};
	use cuckoo::PluginConfig;
	// Any type that implements cucumber_rust::World + Default can be the world
	steps!(crate::MinerWorld => {
        given regex r"I define my mining plugin as <(.*)>" |world, matches, _step| {
            println!("{:?}", _step);
            let plugin = matches[1].clone();
            world.plugin = plugin.as_str().to_lowercase();
            println!("I'll execute the plugin:{}", world.plugin);
            
        };

		given regex r"I choose <([a-zA-Z0-9]+)> algorithm" |world, matches, _step| {
			let algo: String = matches[1].clone().parse().unwrap();
			let mut miner_config: MinerConfig = MinerConfig::default();

			match algo.as_str() {
				"randomx" => {
					world.miner = Some(TargetMiner::RandomX(randomx::RxMiner::new(&miner_config)));
				},
				"cuckoo" => {
					//let plugin = PluginConfig::new(mining_plugin_dir_for_tests(), &world.plugin).unwrap();
					miner_config.miner_plugin_dir = Some(mining_plugin_dir_for_tests());
					miner_config.miner_plugin_config = vec![GrinMinerPluginConfig{ plugin_name: world.plugin.clone(), ..Default::default()}];
					world.miner = Some(TargetMiner::Cuckoo(cuckoo::CuckooMiner::new(&miner_config)));
				},
				_ => {
					panic!("Algorithm not supported");
				}
			}
		};

        then regex r"Mine async for the duration of <(.*)> seconds" |world, matches, _step| {
            println!("I'll execute the async mining!");
            world.time_in_seconds = Some(matches[1].clone().parse().unwrap());

			if let Some(ref mut miner) = &mut world.miner {
				match miner {
					TargetMiner::Cuckoo(ref mut m) => {
						mine_async_for_duration(m, world.time_in_seconds.unwrap());
					},
					TargetMiner::RandomX(ref mut m) => {
						mine_async_for_duration(m, world.time_in_seconds.unwrap());
					}
				}
			}
        };
    });
}

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {

});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {

});

// A setup function to be called before everything else
fn setup() {}

cucumber! {
	features: "./features", // Path to our feature files
	world: ::MinerWorld, // The world needs to be the same for steps and the main cucumber call
	steps: &[
		miner_test::steps // the `steps!` macro creates a `steps` function in a module
	],
	setup: setup, // Optional; called once before everything
	before: &[
		a_before_fn // Optional; called before each scenario
	],
	after: &[
		an_after_fn // Optional; called after each scenario
	]
}
