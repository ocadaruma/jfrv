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
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::io::Cursor;
#[cfg(target_arch = "wasm32")]
use tsify::Tsify;

/// Conditions to filter data
#[derive(Default, Deserialize, Serialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub thread_name_regex: Option<String>,
    pub stack_trace_match_regex: Option<String>,
    pub stack_trace_reject_regex: Option<String>,
}

#[derive(Default)]
pub struct Profile {
    threads: Vec<Thread>,
    filtered_threads: Vec<Thread>,
    filtered_stack_trace_keys: FxHashSet<ConstantPoolKey>,
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
            let (mut reader, chunk) = reader?;
            let start_nanos = chunk.header.start_time_nanos;
            let start_ticks = chunk.header.start_ticks;
            let ticks_per_nanos = (chunk.header.ticks_per_second as f64) / 1_000_000_000.0;

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

                let tick = event
                    .value()
                    .get_field("startTime")
                    .and_then(|s| i64::try_from(s.value).ok())
                    .ok_or_else(|| anyhow!("Failed to get startTime"))?;
                let timestamp_nanos =
                    start_nanos + ((tick - start_ticks) as f64 / ticks_per_nanos) as i64;
                let timestamp_millis = timestamp_nanos / 1000 / 1000;

                let samples = per_thread_samples
                    .entry(os_thread_id)
                    .or_insert_with(Vec::new);
                samples.push(ExecutionSample {
                    timestamp_nanos,
                    state,
                    stack_trace_key,
                });

                interval.start_millis = interval.start_millis.min(timestamp_millis);
                interval.end_millis = interval.end_millis.max(timestamp_millis);
                column_count = column_count.max(samples.len());
            }
        }

        self.threads = thread_pool.into_values().collect();
        self.threads.sort_by(|a, b| a.name.cmp(&b.name));
        self.filtered_threads = self.threads.to_vec();
        self.stack_trace_pool = stack_trace_pool;
        self.filtered_stack_trace_keys = self.stack_trace_pool.keys().cloned().collect();
        self.per_thread_samples = per_thread_samples;
        for (_, v) in self.per_thread_samples.iter_mut() {
            v.sort_by_key(|s| s.timestamp_nanos)
        }
        self.column_count = column_count;
        self.interval = interval;

        info!("Loaded {} events", event_count);

        Ok(())
    }

    pub fn filtered_threads(&self) -> &Vec<Thread> {
        &self.filtered_threads
    }

    pub fn is_valid_sample(&self, sample: &ExecutionSample) -> bool {
        self.filtered_stack_trace_keys
            .contains(&sample.stack_trace_key)
    }

    pub fn apply_filter(&mut self, filter: Filter) -> Result<()> {
        if let Some(regex) = &filter.thread_name_regex {
            let regex = Regex::new(regex.as_str())?;
            info!("regex: {:?}", regex);
            self.filtered_threads = self
                .threads
                .iter()
                .filter(|t| regex.is_match(t.name.as_str()))
                .cloned()
                .collect();
            info!("filtered");
        } else {
            self.filtered_threads = self.threads.to_vec();
        }

        let stack_trace_match_regex = if let Some(regex) = &filter.stack_trace_match_regex {
            Some(Regex::new(regex.as_str())?)
        } else {
            None
        };
        let stack_trace_reject_regex = if let Some(regex) = &filter.stack_trace_reject_regex {
            Some(Regex::new(regex.as_str())?)
        } else {
            None
        };

        if stack_trace_match_regex.is_none() && stack_trace_reject_regex.is_none() {
            // fast path
            self.filtered_stack_trace_keys = self.stack_trace_pool.keys().cloned().collect();
        } else {
            self.filtered_stack_trace_keys = self
                .stack_trace_pool
                .iter()
                .map(|(k, v)| {
                    (
                        k,
                        v.frames
                            .iter()
                            .map(|f| format!("{}.{}", f.type_name, f.method_name))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                })
                .filter(|(_k, v)| {
                    stack_trace_match_regex
                        .as_ref()
                        .map(|r| r.is_match(v))
                        .unwrap_or(true)
                        && stack_trace_reject_regex
                            .as_ref()
                            .map(|r| !r.is_match(v))
                            .unwrap_or(true)
                })
                .map(|(k, _v)| k)
                .cloned()
                .collect();
        }

        Ok(())
    }

    fn get_constant_pool(
        chunk_seq: usize,
        accessor: &Accessor<'_>,
        field_name: &'static str,
    ) -> Result<ConstantPoolKey> {
        match accessor.get_field_raw(field_name).map(|v| v.value) {
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
            frames.push(StackFrame::new(
                f.get_field("method")
                    .and_then(|m| m.get_field("type"))
                    .and_then(|t| t.get_field("name"))
                    .and_then(|n| n.get_field("string"))
                    .and_then(|s| <&str>::try_from(s.value).ok())
                    .ok_or_else(|| anyhow!("failed to get type name"))?
                    .to_string(),
                f.get_field("method")
                    .and_then(|t| t.get_field("name"))
                    .and_then(|n| n.get_field("string"))
                    .and_then(|s| <&str>::try_from(s.value).ok())
                    .ok_or_else(|| anyhow!("failed to get method name"))?
                    .to_string(),
                f.get_field("type")
                    .and_then(|t| t.get_field("description"))
                    .and_then(|s| <&str>::try_from(s.value).ok())
                    .map(|s| s.into())
                    .ok_or_else(|| anyhow!("failed to get frame type"))?,
                f.get_field("lineNumber")
                    .and_then(|l| <i32>::try_from(l.value).ok())
                    .ok_or_else(|| anyhow!("failed to get line number"))?,
            ));
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
    use crate::execution_sample::Profile;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn test_load() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/profiler-wall.jfr");
        let mut bytes = vec![];
        File::open(path).unwrap().read_to_end(&mut bytes).unwrap();

        let mut profile = Profile::default();
        assert!(profile.load(bytes).is_ok());
        // these values can be checked by JMC
        assert_eq!(profile.threads.len(), 30);
        assert_eq!(profile.stack_trace_pool.len(), 25);
    }

    #[test]
    fn test_load_multichunk() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/profiler-multichunk.jfr");
        let mut bytes = vec![];
        File::open(path).unwrap().read_to_end(&mut bytes).unwrap();

        let mut profile = Profile::default();
        assert!(profile.load(bytes).is_ok());
        // Check that even when there are multiple chunks, threads are identified correctly
        assert_eq!(profile.threads.len(), 30);
        assert_eq!(profile.stack_trace_pool.len(), 87);

        let chunks: HashSet<usize> = profile
            .stack_trace_pool
            .keys()
            .map(|k| k.chunk_seq)
            .collect();
        assert_eq!(chunks.len(), 3);
    }
}
