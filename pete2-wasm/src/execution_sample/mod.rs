//! The module implements the renderer for execution samples.
//! The root module should compile without browser env so should not contain any wasm dependencies.

#[cfg(target_arch = "wasm32")]
pub mod render;

use crate::profile::{
    ConstantPoolKey, ExecutionSample, StackFrame, StackTrace, Thread, ThreadState,
};
use crate::TimeInterval;
use anyhow::{anyhow, Result};
use jfrs::reader::event::Accessor;
use jfrs::reader::value_descriptor::ValueDescriptor;
use jfrs::reader::{Chunk, JfrReader};
use log::info;
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;

use std::io::Cursor;

#[derive(Default)]
pub struct Profile {
    pub threads: Vec<Thread>,
    pub stack_trace_pool: FxHashMap<ConstantPoolKey, StackTrace>,
    pub per_thread_samples: FxHashMap<i64, Vec<ExecutionSample>>,
    pub column_count: usize,
    pub interval: TimeInterval,
}

impl Profile {
    /// Load and parse execution-sample recording file.
    pub fn load(&mut self, bytes: Vec<u8>) -> Result<()> {
        info!("loading JFR file of {} bytes", bytes.len());

        let mut event_count = 0;
        let mut reader = JfrReader::new(Cursor::new(bytes));

        let mut stack_trace_pool = FxHashMap::default();
        let mut thread_state_pool: FxHashMap<ConstantPoolKey, String> = FxHashMap::default();
        let mut thread_pool = FxHashMap::default();
        let mut per_thread_samples = FxHashMap::default();
        let mut column_count = 0;
        let mut interval = TimeInterval::new(i64::MAX, 0);

        for (chunk_seq, reader) in reader.chunks().enumerate() {
            let (reader, chunk) = reader?;

            for event in reader.events(&chunk) {
                let event = event?;
                if event.class.name() != "jdk.ExecutionSample" {
                    continue;
                }
                event_count += 1;

                let stack_trace_key =
                    Self::get_constant_pool(chunk_seq, &event.value(), "stackTrace")?;
                let thread_state_key = Self::get_constant_pool(chunk_seq, &event.value(), "state")?;
                let thread_accessor = event
                    .value()
                    .get_field("sampledThread")
                    .ok_or_else(|| anyhow!("Failed to get sampled thread"))?;
                let os_thread_id = thread_accessor
                    .get_field("osThreadId")
                    .and_then(|i| i64::try_from(i.value).ok())
                    .ok_or_else(|| anyhow!("Failed to get osThreadId"))?;

                if let Entry::Vacant(e) = stack_trace_pool.entry(stack_trace_key) {
                    e.insert(Self::parse_stack_trace(&stack_trace_key, &chunk)?);
                }
                // FIXME: Thread name doesn't show correctly if same os_thread_id is reused
                if let Entry::Vacant(e) = thread_pool.entry(os_thread_id) {
                    e.insert(Self::parse_thread(os_thread_id, &thread_accessor)?);
                }
                // we can't use or_insert_with because parse_thread_state may fail
                let state = match thread_state_pool.get(&thread_state_key) {
                    Some(state) => state.as_str().into(),
                    None => {
                        let state_str = Self::parse_thread_state(&thread_state_key, &chunk)?;
                        let state: ThreadState = state_str.as_str().into();
                        thread_state_pool.insert(thread_state_key, state_str);
                        state
                    }
                };

                let timestamp = event
                    .value()
                    .get_field("startTime")
                    .and_then(|s| i64::try_from(s.value).ok())
                    .ok_or_else(|| anyhow!("Failed to get startTime"))?;

                let samples = per_thread_samples
                    .entry(os_thread_id)
                    .or_insert_with(Vec::new);
                samples.push(ExecutionSample {
                    timestamp,
                    state,
                    stack_trace_key,
                });

                interval.start_millis = interval.start_millis.min(timestamp);
                interval.end_millis = interval.end_millis.max(timestamp);
                column_count = column_count.max(samples.len());
            }
        }

        self.threads = thread_pool.into_values().collect();
        self.threads.sort_by(|a, b| a.name.cmp(&b.name));
        self.stack_trace_pool = stack_trace_pool;
        self.per_thread_samples = per_thread_samples;
        for (_, v) in self.per_thread_samples.iter_mut() {
            v.sort_by_key(|s| s.timestamp)
        }
        self.column_count = column_count;
        self.interval = interval;

        info!("Loaded {} events", event_count);

        Ok(())
    }

    fn get_constant_pool(
        chunk_seq: usize,
        accessor: &Accessor<'_>,
        field_name: &'static str,
    ) -> Result<ConstantPoolKey> {
        match accessor.get_field(field_name).map(|v| v.value) {
            Some(ValueDescriptor::ConstantPool {
                class_id,
                constant_index,
            }) => Ok(ConstantPoolKey::new(chunk_seq, *class_id, *constant_index)),
            _ => Err(anyhow!("Field {} is not constant pool", field_name)),
        }
    }

    fn parse_stack_trace(key: &ConstantPoolKey, chunk: &Chunk) -> Result<StackTrace> {
        let desc = ValueDescriptor::ConstantPool {
            class_id: key.class_id,
            constant_index: key.constant_pool_index,
        };
        let accessor = Accessor::new(chunk, &desc);
        let mut frames = vec![];

        for f in accessor
            .get_field("frames")
            .and_then(|f| f.as_iter())
            .ok_or_else(|| anyhow!("failed to get stack frames"))?
        {
            frames.push(StackFrame {
                type_name: f
                    .get_field("method")
                    .and_then(|m| m.get_field("type"))
                    .and_then(|t| t.get_field("name"))
                    .and_then(|n| n.get_field("string"))
                    .and_then(|s| <&str>::try_from(s.value).ok())
                    .ok_or_else(|| anyhow!("failed to get type name"))?
                    .to_string(),
                method_name: f
                    .get_field("method")
                    .and_then(|t| t.get_field("name"))
                    .and_then(|n| n.get_field("string"))
                    .and_then(|s| <&str>::try_from(s.value).ok())
                    .ok_or_else(|| anyhow!("failed to get method name"))?
                    .to_string(),
            });
        }

        Ok(StackTrace { frames })
    }

    fn parse_thread_state(key: &ConstantPoolKey, chunk: &Chunk) -> Result<String> {
        let desc = ValueDescriptor::ConstantPool {
            class_id: key.class_id,
            constant_index: key.constant_pool_index,
        };
        let accessor = Accessor::new(chunk, &desc);

        accessor
            .get_field("name")
            .and_then(|s| <&str>::try_from(s.value).ok())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("failed to get thread state"))
    }

    fn parse_thread(os_thread_id: i64, accessor: &Accessor<'_>) -> Result<Thread> {
        let name = if let Some(name) = accessor
            .get_field("javaName")
            .and_then(|n| <&str>::try_from(n.value).ok())
        {
            name.to_string()
        } else if let Some(name) = accessor
            .get_field("osName")
            .and_then(|n| <&str>::try_from(n.value).ok())
        {
            name.to_string()
        } else {
            return Err(anyhow!("Failed to get thread name"));
        };
        Ok(Thread { os_thread_id, name })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load() {}
}
