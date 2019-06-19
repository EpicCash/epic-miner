extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rand;
extern crate byteorder;

//extern crate grin_miner_util as util;

pub mod errors;
pub mod types;
pub mod config;
pub mod miner;
pub mod util;

pub use errors::MinerError;
pub use miner::Miner;
pub use types::{
    Stats,
    Solution,
    Algorithm,
    AlgorithmParams,
    ControlMessage,
    JobSharedData,
    JobSharedDataType};