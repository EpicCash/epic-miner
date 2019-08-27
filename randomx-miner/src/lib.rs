#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate slog;
extern crate bigint;

extern crate epic_miner_util as util;
extern crate epic_miner_core as core;
extern crate randomx;


//pub mod plugin;
pub mod miner;
pub use miner::RxMiner;
