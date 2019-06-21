#[macro_use]
extern crate lazy_static;
extern crate bigint;

extern crate grin_miner_util as util;
extern crate grin_miner_core as core;
extern crate randomx;

//pub mod plugin;
pub mod miner;
pub use miner::RxMiner;
