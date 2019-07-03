#[macro_use]
extern crate lazy_static;
extern crate bigint;
extern crate keccak_hash;

extern crate grin_miner_core as core;
extern crate grin_miner_util as util;
extern crate progpow;

pub mod miner;
pub use miner::PpMiner;
