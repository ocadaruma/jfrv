use std::collections::HashMap;
use std::hash::Hash;
use std::io::{BufReader, Cursor};
use jfrs::reader::{JfrReader};
use jfrs::reader::event::Accessor;
use jfrs::reader::value_descriptor::ValueDescriptor;
// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console::error;
use three_d::{Object2D, Rad, Vector2};

use log::{info, warn};
use three_d::{Color, ColorMaterial, degrees, Rectangle, vec2, Window, WindowSettings};
use tsify::Tsify;
use serde::{Serialize, Deserialize};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug);
    //
    info!("Logging works!");
    //

    info!("hello, world from wasm");
    main();
    Ok(())
}

fn main() -> Result<(), ()> {
    Ok(())
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub width: f32,
    pub height: f32
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ChartConfig {
    pub header_width: f32,
    pub font_size: f32,
    pub margin: f32,
    pub border_width: f32,
    pub border_color: String,
    pub sample_render_size: Dimension
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SampledThread {
    pub id: i64,
    pub name: String
}

#[derive(Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DateInterval {
    pub start_millis: i64,
    pub end_millis: i64,
    pub duration_millis: i64,
}

impl DateInterval {
    pub fn new(start_millis: i64, end_millis: i64) -> Self {
        Self {
            start_millis,
            end_millis,
            duration_millis: end_millis - start_millis,
        }
    }
}

#[derive(Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ThreadProfile {
    pub interval: DateInterval,
    pub samples: HashMap<i64, Vec<Sample>>,
    pub max_sample_num: usize,
    pub threads: Vec<SampledThread>
}

#[derive(Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub timestamp: i64,
    pub thread_id: i64,
    pub thread_state_id: i32,
    pub stack_trace_id: i32,
}

#[derive(Deserialize, Serialize, Tsify, Eq, PartialEq, Hash)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub type_name: String,
    pub method_name: String,
}

#[derive(Default, Deserialize, Serialize, Tsify, Eq, PartialEq, Hash)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct StackTrace {
    pub frames: Vec<Frame>
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub samples: Vec<Sample>,
    pub stack_trace_pool: HashMap<i32, StackTrace>,
    pub thread_name_pool: HashMap<i64, String>,
    pub thread_state_pool: HashMap<i32, String>,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Thread {
    pub id: i32,
    pub name: String
}

#[wasm_bindgen]
pub struct JfrRenderer {
}

#[wasm_bindgen]
impl JfrRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_jfr(&mut self, bytes: Vec<u8>) -> Option<Profile> {
        info!("passed. byte size: {}", bytes.len());

        // let canvas = web_sys::window()
        //     .and_then(|w| w.document())
        //     .and_then(|d| d.get_element_by_id("thread-chart-sample-view"))
        //     .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        //     .unwrap();
        //
        // canvas.set_width(800);
        // canvas.set_height(600);
        //
        // let window = Window::new(WindowSettings {
        //     canvas: Some(canvas),
        //     ..WindowSettings::default()
        // }).unwrap();
        // let context = window.gl();
        //
        // let mut rectangle = Rectangle::new_with_material(
        //     &context,
        //     vec2(50.0, 100.0),
        //     Rad(0.0),
        //     100.0,
        //     200.0,
        //     ColorMaterial {
        //         color: Color::RED,
        //         ..Default::default()
        //     },
        // );
        // rectangle.render(window.viewport());

        let mut stack_trace_pool: HashMap<i32, StackTrace> = HashMap::new();
        let mut thread_name_pool: HashMap<i64, String> = HashMap::new();
        let mut thread_state_pool: HashMap<i32, String> = HashMap::new();
        let mut samples: Vec<Sample> = Vec::new();

        let mut event_count = 0;
        let mut reader = JfrReader::new(Cursor::new(bytes));
        for reader in reader.chunks() {
            if let Ok((reader, chunk)) = reader {
                let mut stack_trace_cp_pool: Pool = Pool::default();
                let mut thread_state_cp_pool: Pool = Pool::default();

                for event in reader.events(&chunk) {
                    if let Ok(event) = event {
                        if event.class.name() != "jdk.ExecutionSample" {
                            continue;
                        }
                        event_count += 1;

                        let stack_trace_cp_index = match event.value().get_field("stackTrace").map(|t| t.value) {
                            Some(ValueDescriptor::ConstantPool {class_id, constant_index}) => {
                                (*class_id, *constant_index)
                            }
                            _ => continue
                        };

                        let t = event.value().get_field("sampledThread");
                        let thread_name = if let Some(name) = t.as_ref()
                            .and_then(|t| t.get_field("javaName"))
                            .and_then(|n| <&str>::try_from(n.value).ok()) {
                            name.to_string()
                        } else if let Some(name) = t.as_ref()
                            .and_then(|t| t.get_field("osName"))
                            .and_then(|n| <&str>::try_from(n.value).ok()) {
                            name.to_string()
                        } else {
                            "unknown".to_string()
                        };
                        let thread_id = t.as_ref()
                            .and_then(|t| t.get_field("osThreadId"))
                            .and_then(|i| i64::try_from(i.value).ok()).unwrap();

                        thread_name_pool.insert(thread_id, thread_name);

                        let thread_state_cp_index = match event.value().get_field("state").map(|s| s.value) {
                            Some(ValueDescriptor::ConstantPool {class_id, constant_index}) => {
                                (*class_id, *constant_index)
                            }
                            _ => continue
                        };

                        let start_time = event.value().get_field("startTime")
                            .and_then(|s| i64::try_from(s.value).ok()).unwrap();

                        samples.push(Sample {
                            timestamp: start_time,
                            thread_id,
                            thread_state_id: thread_state_cp_pool.register(thread_state_cp_index),
                            stack_trace_id: stack_trace_cp_pool.register(stack_trace_cp_index)
                        })
                    } else {
                        warn!("Failed to read event");
                        return None;
                    }
                }

                for (k, v) in thread_state_cp_pool.cache {
                    let desc = ValueDescriptor::ConstantPool {class_id: k.0, constant_index: k.1};
                    let accessor = Accessor::new(&chunk, &desc);
                    let str = accessor
                        .get_field("name")
                        .and_then(|s| <&str>::try_from(s.value).ok()).unwrap().to_string();
                    thread_state_pool.insert(v, str);
                }

                for (k, v) in stack_trace_cp_pool.cache {
                    let desc = ValueDescriptor::ConstantPool { class_id: k.0, constant_index: k.1 };
                    let accessor = Accessor::new(&chunk, &desc);
                    let mut frames: Vec<Frame> = Vec::new();
                    for f in accessor
                        .get_field("frames")
                        .and_then(|f| f.as_iter())
                        .unwrap() {
                        frames.push(Frame {
                            type_name: f.get_field("method")
                                .and_then(|m| m.get_field("type"))
                                .and_then(|t| t.get_field("name"))
                                .and_then(|n| n.get_field("string"))
                                .and_then(|s| <&str>::try_from(s.value).ok()).unwrap().to_string(),
                            method_name: f.get_field("method")
                                .and_then(|t| t.get_field("name"))
                                .and_then(|n| n.get_field("string"))
                                .and_then(|s| <&str>::try_from(s.value).ok()).unwrap().to_string(),
                        })
                    }
                    stack_trace_pool.insert(v, StackTrace { frames });
                }

            } else {
                warn!("Failed to read chunk");
                return None;
            }
        }
        info!("event_count: {}", event_count);

        let profile = Profile {
            samples,
            stack_trace_pool,
            thread_name_pool,
            thread_state_pool,
        };

        Some(profile)
    }
}

#[derive(Default)]
struct Pool {
    id: i32,
    cache: HashMap<(i64, i64), i32>
}

impl Pool {
    fn register(&mut self, v: (i64, i64)) -> i32 {
        if let Some(i) = self.cache.get(&v) {
            return *i;
        }
        self.cache.insert(v, self.id);
        let prev = self.id;
        self.id += 1;
        prev
    }
}
