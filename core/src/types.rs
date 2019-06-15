use std::string;
use std::ffi::CString;
use std::collections::HashMap;

const MAX_NAME_LEN: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Algorithm {
    Cuckoo,
    RandomX
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmParams {
    // edge_bits, nonces
	Cuckoo(u32, Vec<u64>),

    // hash
	RandomX([u8; 32]),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
	/// Stop everything, pull down, exis
	Stop,
	/// Stop current mining iteration, set solver threads to paused
	Pause,
	/// Resume
	Resume,
	/// Solver reporting stopped
	SolverStopped(usize),
}

#[derive(Clone)]
pub struct Stats {
	pub device_id: u32,
	pub edge_bits: u32,
	pub plugin_name: [u8; MAX_NAME_LEN],
	pub device_name: [u8; MAX_NAME_LEN],
	pub has_errored: bool,
	pub error_reason: [u8; MAX_NAME_LEN],
	pub iterations: u32,
	pub last_start_time: u64,
	pub last_end_time: u64,
	pub last_solution_time: u64,
}

impl Stats {
	fn get_name(&self, c_str: &[u8; MAX_NAME_LEN]) -> String {
		// remove all null zeroes
		let v = c_str.clone().to_vec();
		let mut i = 0;
		for j in 0..v.len() {
			if v.get(j) == Some(&0) {
				i = j;
				break;
			}
		}
		let v = v.split_at(i).0;
		match CString::new(v) {
			Ok(s) => s.to_str().unwrap().to_owned(),
			Err(_) => String::from("Unknown Device Name"),
		}
	}
	/// return device name as rust string
	pub fn get_device_name(&self) -> String {
		self.get_name(&self.device_name)
	}
	/// return plugin name as rust string
	pub fn get_plugin_name(&self) -> String {
		self.get_name(&self.plugin_name)
	}
	/// return plugin name as rust string
	pub fn get_error_reason(&self) -> String {
		self.get_name(&self.error_reason)
	}
	/// set plugin name
	pub fn set_plugin_name(&mut self, name: &str) {
		let c_vec = CString::new(name).unwrap().into_bytes();
		for i in 0..c_vec.len() {
			self.plugin_name[i] = c_vec[i];
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution(u64, u64, AlgorithmParams);

impl Solution {
    pub fn new(id: u64, nonce: u64, algo_params: AlgorithmParams) -> Self {
        Solution(id, nonce, algo_params)
    }

    pub fn get_id(&self) -> u64 {
        self.0
    }

    pub fn get_nonce(&self) -> u64 {
        self.1
    }

    pub fn get_algorithm_params(&self) -> AlgorithmParams {
        self.2.clone()
    }
}
