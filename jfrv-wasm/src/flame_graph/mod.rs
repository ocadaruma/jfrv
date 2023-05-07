use crate::execution_sample::Profile;
use crate::profile::{FrameType, StackFrame, StackTrace};
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
pub mod render;

pub struct FlameGraph {
    pub depth: usize,
    pub root: Frame,
}

impl Default for FlameGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FlameGraph {
    pub fn new() -> Self {
        Self {
            depth: 0,
            root: Frame {
                base_type: FrameType::Native,
                ..Default::default()
            },
        }
    }

    pub fn from_execution_sample(profile: &Profile) -> Self {
        let mut flame = Self::new();

        let mut pre_aggregation = FxHashMap::default();
        for thread in profile.threads() {
            if let Some(samples) = profile.per_thread_samples.get(&thread.os_thread_id) {
                for sample in samples {
                    if profile.is_valid_sample(sample) {
                        let count = pre_aggregation
                            .get(&sample.stack_trace_key)
                            .unwrap_or(&0usize)
                            + 1;
                        pre_aggregation.insert(sample.stack_trace_key, count);
                    }
                }
            }
        }

        for (k, &v) in pre_aggregation.iter() {
            if let Some(trace) = profile.stack_trace_pool.get(k) {
                flame.add_sample(trace, v);
            }
        }

        flame
    }

    pub fn add_sample(&mut self, stack_trace: &StackTrace, count: usize) {
        let mut frame = &mut self.root;
        for f in stack_trace.frames.iter().rev() {
            frame = frame.add_child(f, count);
        }
        frame.add_leaf(count);
        self.depth = self.depth.max(stack_trace.frames.len());
    }
}

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct FrameId {
    pub name: String,
    category: FrameCategory,
}

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum FrameCategory {
    Native,
    Java,
    Kernel,
}

#[derive(Default)]
pub struct Frame {
    base_type: FrameType,
    pub children: BTreeMap<FrameId, Frame>,
    pub total_count: usize,
    pub self_count: usize,
    pub inlined_count: usize,
    pub c1_count: usize,
    pub interpreted_count: usize,
}

impl Frame {
    pub fn calculated_type(&self) -> FrameType {
        if self.inlined_count * 3 >= self.total_count {
            FrameType::Inlined
        } else if self.c1_count * 2 >= self.total_count {
            FrameType::C1Compiled
        } else if self.interpreted_count * 2 >= self.total_count {
            FrameType::Interpreted
        } else {
            self.base_type
        }
    }

    pub fn add_child(&mut self, frame: &StackFrame, count: usize) -> &mut Frame {
        self.total_count += count;

        let (base_type, category) = match frame.frame_type {
            FrameType::Interpreted
            | FrameType::JitCompiled
            | FrameType::Inlined
            | FrameType::C1Compiled => (FrameType::JitCompiled, FrameCategory::Java),
            FrameType::Cpp => (FrameType::Cpp, FrameCategory::Native),
            FrameType::Kernel => (FrameType::Kernel, FrameCategory::Kernel),
            _ => (FrameType::Native, FrameCategory::Native),
        };

        let id = FrameId {
            name: frame.name().to_string(),
            category,
        };

        let mut child = self.children.entry(id).or_insert_with(|| Frame {
            base_type,
            ..Default::default()
        });

        match frame.frame_type {
            FrameType::Interpreted => child.interpreted_count += count,
            FrameType::Inlined => child.inlined_count += count,
            FrameType::C1Compiled => child.c1_count += count,
            _ => {}
        };

        child
    }

    pub fn add_leaf(&mut self, count: usize) {
        self.total_count += count;
        self.self_count += count;
    }
}
