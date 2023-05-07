//! The module implements the renderer for jvm-blocking-monitor logs.

#[cfg(target_arch = "wasm32")]
pub mod render;

use crate::profile::{FrameType, OffCpu, StackFrame, StackTrace, Thread, ThreadState};
use crate::TimeInterval;
use anyhow::{anyhow, Result};
use chrono::{Local, NaiveDateTime, TimeZone};
use log::info;
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::Entry;
use std::io::{BufRead, BufReader, Cursor};
#[cfg(target_arch = "wasm32")]
use tsify::Tsify;

/// Conditions to filter data
#[derive(Default, Deserialize, Serialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct JbmFilter {
    pub thread_name_regex: Option<String>,
}

#[derive(Default)]
pub struct Profile {
    threads: Vec<Thread>,
    filtered_threads: Vec<Thread>,
    pub stack_trace_pool: FxHashMap<i32, StackTrace>,
    pub per_thread_samples: FxHashMap<i64, Vec<OffCpu>>,
    pub interval: TimeInterval,
}

impl Profile {
    pub fn load(&mut self, bytes: Vec<u8>) -> Result<()> {
        let reader = BufReader::new(Cursor::new(bytes));
        let sample_start = Regex::new(
            r"^=== ([-: 0-9]+)\.([0-9]+) PID: [0-9]+, TID: ([0-9]+) \(([^),]*)\), DURATION: ([0-9]+) us",
        )?;
        let stack_frame = Regex::new(
            r"^(?:(\s*[0-9]+: \[0x[0-9A-Fa-f]+].*)|(-----+)|(JVM Stack).*|(Native Stack:).*)$",
        )?;

        let mut interval = TimeInterval::new(i64::MAX, 0);
        let mut threads = FxHashSet::default();
        let mut stack_trace_id: i32 = 0;
        let mut stack_trace_pool: FxHashMap<StackTrace, i32> = FxHashMap::default();
        let mut per_thread_samples: FxHashMap<i64, Vec<OffCpu>> = FxHashMap::default();
        let mut parsing_sample: Option<ParsingSample> = None;

        for line in reader.lines() {
            let line = line?;

            if let Some(captures) = sample_start.captures(line.as_str()) {
                // flush
                if let Some(sample) = parsing_sample.take() {
                    Self::flush_sample(
                        sample,
                        &mut stack_trace_id,
                        &mut stack_trace_pool,
                        &mut per_thread_samples,
                    );
                }

                let ts = &captures[1];
                let millis_fragment = &captures[2].parse::<i64>()?;
                let tid = captures[3].parse::<i64>()?;
                let thread_name = &captures[4];
                let duration: i64 = captures[5].parse::<i64>()? / 1000;

                let t = Thread {
                    name: format!("{} [tid=0x{:x}]", thread_name, tid),
                    os_thread_id: tid,
                };
                threads.insert(t);
                let timestamp = Local
                    .from_local_datetime(&NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")?)
                    .earliest()
                    .ok_or_else(|| anyhow!("failed to parse timestamp"))?
                    .timestamp_millis()
                    + *millis_fragment
                    - duration;

                interval.start_millis = interval.start_millis.min(timestamp);
                interval.end_millis = interval.end_millis.max(timestamp + duration);
                parsing_sample = Some(ParsingSample {
                    timestamp,
                    duration,
                    thread_id: tid,
                    state: ThreadState::Unknown,
                    frames: vec![],
                });
                continue;
            }
            if let Some(captures) = stack_frame.captures(line.as_str()) {
                let mut frame = "";
                for i in 1..=4 {
                    if let Some(f) = captures.get(i) {
                        frame = f.as_str();
                        break;
                    }
                }
                if let Some(sample) = &mut parsing_sample {
                    sample.frames.push(StackFrame::new(
                        "".to_string(),
                        frame.to_string(),
                        FrameType::Unknown,
                        0,
                    ));
                }
                continue;
            }
        }

        if let Some(sample) = parsing_sample.take() {
            Self::flush_sample(
                sample,
                &mut stack_trace_id,
                &mut stack_trace_pool,
                &mut per_thread_samples,
            );
        }

        let mut threads: Vec<Thread> = threads.into_iter().collect();
        threads.sort_by(|a, b| a.name.cmp(&b.name));

        let mut inverted = FxHashMap::default();
        for (k, v) in stack_trace_pool {
            inverted.insert(v, k);
        }
        for (_, samples) in per_thread_samples.iter_mut() {
            samples.sort_by_key(|s| s.timestamp);
        }

        self.threads = threads;
        self.filtered_threads = self.threads.to_vec();
        self.stack_trace_pool = inverted;
        self.per_thread_samples = per_thread_samples;
        self.interval = interval;

        Ok(())
    }

    fn flush_sample(
        sample: ParsingSample,
        stack_trace_id: &mut i32,
        stack_trace_pool: &mut FxHashMap<StackTrace, i32>,
        per_thread_samples: &mut FxHashMap<i64, Vec<OffCpu>>,
    ) {
        let stack_trace = StackTrace {
            frames: sample.frames,
        };
        let id = match stack_trace_pool.entry(stack_trace) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let id = e.insert(*stack_trace_id);
                *stack_trace_id += 1;
                *id
            }
        };
        let samples = per_thread_samples
            .entry(sample.thread_id)
            .or_insert_with(Vec::new);
        samples.push(OffCpu {
            timestamp: sample.timestamp,
            duration_millis: sample.duration,
            state: sample.state,
            stack_trace_key: id,
        });
    }

    pub fn threads(&self) -> &Vec<Thread> {
        &self.filtered_threads
    }

    pub fn apply_filter(&mut self, filter: JbmFilter) -> Result<()> {
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
        Ok(())
    }
}

struct ParsingSample {
    timestamp: i64,
    duration: i64,
    thread_id: i64,
    state: ThreadState,
    frames: Vec<StackFrame>,
}

#[cfg(test)]
mod tests {
    use crate::jbm::Profile;

    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn test_load() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data/jbm.log");
        let mut bytes = vec![];
        File::open(path).unwrap().read_to_end(&mut bytes).unwrap();

        let mut profile = Profile::default();
        assert!(profile.load(bytes).is_ok());
        assert_eq!(2, profile.threads.len());
    }
}
