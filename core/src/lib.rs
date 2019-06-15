extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate grin_miner_util as util;

pub mod errors;
pub mod types;
pub mod config;
pub mod miner;

pub use errors::MinerError;
pub use miner::Miner;
pub use types::{
    Stats,
    Solution,
    Algorithm,
    AlgorithmParams,
    ControlMessage};