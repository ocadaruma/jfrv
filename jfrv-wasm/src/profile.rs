//! Data structures to represent JFR profiles.
//! Should not contain any wasm dependencies.

use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use tsify::Tsify;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct StackFrame {
    pub type_name: String,
    pub method_name: String,
    pub frame_type: FrameType,
    pub line_number: i32,
    name: String,
}

impl StackFrame {
    pub fn new(
        type_name: String,
        method_name: String,
        frame_type: FrameType,
        line_number: i32,
    ) -> Self {
        let name = match frame_type {
            FrameType::Interpreted
            | FrameType::JitCompiled
            | FrameType::Inlined
            | FrameType::C1Compiled
                if !type_name.is_empty() =>
            {
                let line_num = if line_number > 0 {
                    format!(":{}", line_number)
                } else {
                    "".to_string()
                };
                format!("{}.{}{}", type_name, method_name, line_num)
            }
            _ => method_name.clone(),
        };

        Self {
            type_name,
            method_name,
            frame_type,
            line_number,
            name,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub os_thread_id: i64,
    pub name: String,
}

#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
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

impl ThreadState {
    const THREAD_STATE_RUNNING: &'static str = "STATE_RUNNABLE";
    const THREAD_STATE_SLEEPING: &'static str = "STATE_SLEEPING";
}

impl From<&str> for ThreadState {
    fn from(s: &str) -> Self {
        match s {
            Self::THREAD_STATE_RUNNING => Self::Runnable,
            Self::THREAD_STATE_SLEEPING => Self::Sleeping,
            _ => Self::Unknown,
        }
    }
}

/// Compact representation of frame type (which is originally String)
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
#[repr(u8)]
pub enum FrameType {
    Interpreted = 0,
    JitCompiled,
    Inlined,
    Native,
    Cpp,
    Kernel,
    C1Compiled,
    #[default]
    Unknown = 255,
}

impl FrameType {
    const FRAME_INTERPRETED: &'static str = "Interpreted";
    const FRAME_JIT_COMPILED: &'static str = "JIT compiled";
    const FRAME_INLINED: &'static str = "Inlined";
    const FRAME_NATIVE: &'static str = "Native";
    const FRAME_CPP: &'static str = "C++";
    const FRAME_KERNEL: &'static str = "Kernel";
    const FRAME_C1_COMPILED: &'static str = "C1 compiled";
}

impl From<&str> for FrameType {
    fn from(s: &str) -> Self {
        match s {
            Self::FRAME_INTERPRETED => Self::Interpreted,
            Self::FRAME_JIT_COMPILED => Self::JitCompiled,
            Self::FRAME_INLINED => Self::Inlined,
            Self::FRAME_NATIVE => Self::Native,
            Self::FRAME_CPP => Self::Cpp,
            Self::FRAME_KERNEL => Self::Kernel,
            Self::FRAME_C1_COMPILED => Self::C1Compiled,
            _ => Self::Unknown,
        }
    }
}

pub struct ExecutionSample {
    pub timestamp_nanos: i64,
    pub state: ThreadState,
    pub stack_trace_key: ConstantPoolKey,
}

pub struct OffCpu {
    pub timestamp: i64,
    pub duration_millis: i64,
    pub state: ThreadState,
    pub stack_trace_key: i32,
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
