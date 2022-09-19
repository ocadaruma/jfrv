//! Data structures to represent JFR profiles.
//! Should not contain any wasm dependencies.

use serde::{Deserialize, Serialize};
use tsify::Tsify;

pub const THREAD_STATE_RUNNING: &str = "STATE_RUNNABLE";
pub const THREAD_STATE_SLEEPING: &str = "STATE_SLEEPING";

#[derive(Clone, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    pub type_name: String,
    pub method_name: String,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub os_thread_id: i64,
    pub name: String,
}

#[derive(Clone, Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
}

/// Compact representation of thread state (which is originally String)
pub enum ThreadState {
    Unknown,
    Runnable,
    Sleeping,
}

impl From<&str> for ThreadState {
    fn from(s: &str) -> Self {
        match s {
            THREAD_STATE_RUNNING => Self::Runnable,
            THREAD_STATE_SLEEPING => Self::Sleeping,
            _ => Self::Unknown,
        }
    }
}

pub struct ExecutionSample {
    pub timestamp: i64,
    pub state: ThreadState,
    pub stack_trace_key: ConstantPoolKey,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct ConstantPoolKey {
    pub chunk_seq: usize,
    pub class_id: i64,
    pub constant_pool_index: i64,
}

impl ConstantPoolKey {
    pub fn new(chunk_seq: usize, class_id: i64, constant_pool_index: i64) -> Self {
        Self {
            chunk_seq,
            class_id,
            constant_pool_index,
        }
    }
}
