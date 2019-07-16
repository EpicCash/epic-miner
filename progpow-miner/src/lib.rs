#[macro_use]
extern crate lazy_static;
extern crate bigint;
extern crate keccak_hash;

extern crate epic_miner_core as core;
extern crate epic_miner_util as util;
#[cfg(feature = "opencl")]
extern crate progpow_opencl as progpow;
#[cfg(feature = "cuda")]
extern crate progpow_cuda as progpow;

pub mod miner;
pub use miner::PpMiner;
