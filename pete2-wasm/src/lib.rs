mod execution;

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

use log::{info, warn};
use tsify::Tsify;
use serde::{Serialize, Deserialize};
use speedy2d::color::Color;
use speedy2d::dimen::UVec2;
use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug);
    info!("Renderer loaded");
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

#[derive(Deserialize, Serialize, Default, Tsify)]
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
    pub threads: Vec<SampledThread>,
    pub stack_trace_pool: HashMap<i32, StackTrace>,
    pub thread_state_pool: HashMap<i32, String>,
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

#[derive(Clone, Deserialize, Serialize, Tsify, Eq, PartialEq, Hash)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub type_name: String,
    pub method_name: String,
}

#[derive(Clone, Default, Deserialize, Serialize, Tsify, Eq, PartialEq, Hash)]
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

#[wasm_bindgen]
#[derive(Default)]
pub struct JfrRenderer {
    samples: Vec<Rect>,
    threads: Vec<SampledThread>,
    per_thread_samples: HashMap<i64, Vec<Sample>>,
    sample_view_width: f32,
    row_height: f32,
    interval: DateInterval,
    chart_config: ChartConfig,
    stack_trace_pool: HashMap<i32, StackTrace>,

    highlighted_thread_id: Option<i64>,
    highlighted_sample_idx: Option<usize>,
}

#[wasm_bindgen]
impl JfrRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_jfr(&mut self, bytes: Vec<u8>, chart_config: ChartConfig) -> Option<ThreadProfile> {
        info!("loading JFR file of {} bytes", bytes.len());

        let mut stack_trace_pool: HashMap<i32, StackTrace> = HashMap::new();
        let mut thread_name_pool: HashMap<i64, String> = HashMap::new();
        let mut thread_state_pool: HashMap<i32, String> = HashMap::new();
        let mut per_thread_samples: HashMap<i64, Vec<Sample>> = HashMap::new();

        let mut event_count = 0;
        let mut start_millis = i64::MAX;
        let mut end_millis = -1i64;
        let mut max_samples = 0;

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
                        start_millis = start_millis.min(start_time);
                        end_millis = end_millis.max(start_time);

                        per_thread_samples
                            .entry(thread_id)
                            .or_insert(vec![])
                            .push(Sample {
                                timestamp: start_time,
                                thread_id,
                                thread_state_id: thread_state_cp_pool.register(thread_state_cp_index),
                                stack_trace_id: stack_trace_cp_pool.register(stack_trace_cp_index)
                            });
                        max_samples = max_samples.max(
                            per_thread_samples.get(&thread_id)
                                .map(|v| v.len()).unwrap_or(0));
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
        info!("Done read chunk");

        for (k, v) in per_thread_samples.iter_mut() {
            v.sort_by_key(|s| s.timestamp);
        }
        let mut threads = vec![];
        for (k, v) in thread_name_pool {
            threads.push(SampledThread { id: k, name: v });
        }
        threads.sort_by(|a, b| a.name.cmp(&b.name));

        let interval = DateInterval { start_millis, end_millis, duration_millis: end_millis - start_millis };
        let sample_view_width = chart_config.sample_render_size.width *
            (max_samples as f32);
        let row_height = chart_config.font_size + (chart_config.margin * 2.0);

        let document = web_sys::window()
            .and_then(|w| w.document()).unwrap();

        let header = Self::get_element_by_id::<web_sys::SvgGraphicsElement>("header").unwrap();
        let header_overlay = Self::get_element_by_id::<web_sys::HtmlCanvasElement>("header-overlay").unwrap();
        let canvas = Self::get_element_by_id::<web_sys::HtmlCanvasElement>("thread-chart-sample-view").unwrap();
        let chart_overlay = Self::get_element_by_id::<web_sys::HtmlCanvasElement>("chart-overlay").unwrap();

        let mut shapes: Vec<Rect> = vec![];
        for (i, thread) in threads.iter().enumerate() {
            if let Some(samples) = per_thread_samples.get(&thread.id) {
                for (j, sample) in samples.iter().enumerate() {
                    let x = sample_view_width * (sample.timestamp - interval.start_millis) as f32
                        / interval.duration_millis as f32;
                    let y = row_height * i as f32 + (row_height - chart_config.sample_render_size.height) / 2.0;

                    let state_name = thread_state_pool.get(&sample.thread_state_id).unwrap();
                    let color = match state_name.as_str() {
                        "STATE_RUNNABLE" => 0x6cba1e,
                        "STATE_SLEEPING" => 0x8d3eee,
                        _ => 0x6f6d72
                    };

                    shapes.push(Rect {
                        x, y,
                        width: chart_config.sample_render_size.width,
                        height: chart_config.sample_render_size.height,
                        color: Color::from_hex_rgb(color)
                    });
                }
            }

            let text = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "text").unwrap();
            let text_node = document.create_text_node(&thread.name);
            text.set_attribute("x", chart_config.margin.to_string().as_str());
            // y is the baseline of the text.
            // so we add fontSize to the current offset.
            // also add margin to allocate the margin-top.
            text.set_attribute("y", (row_height * i as f32 + chart_config.font_size + chart_config.margin).to_string().as_str());
            text.append_child(&text_node);
            header.append_child(&text);
        }
        info!("Done convert");

        let header_width = header.get_b_box().map(|b| b.width()).unwrap();

        header.set_attribute("width", header_width.to_string().as_str());
        header.set_attribute("height", (row_height * threads.len() as f32).to_string().as_str());
        header_overlay.set_width(header_width as u32);
        header_overlay.set_height((row_height * threads.len() as f32) as u32);
        canvas.set_width(sample_view_width as u32);
        canvas.set_height((row_height * threads.len() as f32) as u32);
        chart_overlay.set_width(sample_view_width as u32);
        chart_overlay.set_height((row_height * threads.len() as f32) as u32);

        for i in 0..threads.len() {
            let y = row_height * i as f32;
            if i < threads.len() -1 {
                let y = y + row_height;
                let line = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "line").unwrap();
                line.set_attribute("x1", "0");
                line.set_attribute("y1", y.to_string().as_str());
                line.set_attribute("x2", header_width.to_string().as_str());
                line.set_attribute("y2", y.to_string().as_str());
                line.set_attribute("stroke-width", chart_config.border_width.to_string().as_str());
                line.set_attribute("stroke", chart_config.border_color.as_str());
                header.append_child(&line);
            }
        }

        self.samples = shapes;
        self.threads = threads;
        self.sample_view_width = sample_view_width;
        self.row_height = row_height;
        self.per_thread_samples = per_thread_samples;
        self.interval = interval;
        self.chart_config = chart_config;
        self.stack_trace_pool = stack_trace_pool;

        info!("Done render");

        info!("event_count: {}", event_count);
        None
    }

    #[wasm_bindgen]
    pub fn render(&self) {
        let mut renderer = speedy2d::GLRenderer::new_for_web_canvas_by_id(
            (self.sample_view_width as u32, (self.row_height * self.threads.len() as f32) as u32),
            "thread-chart-sample-view").unwrap();
        renderer.draw_frame(|graphics| {
            graphics.clear_screen(Color::from_hex_rgb(0xf2f5f9));
            // let f = self.scale_factor as f32;

            for sample in self.samples.iter() {
                graphics.draw_rectangle(Rectangle::from_tuples(
                    (sample.x, sample.y), (sample.x + sample.width, sample.y + sample.height)
                ), sample.color);
            }
        });
    }

    #[wasm_bindgen]
    pub fn clear(&self) {
        let mut renderer = speedy2d::GLRenderer::new_for_web_canvas_by_id(
            (self.sample_view_width as u32, (self.row_height * self.threads.len() as f32) as u32),
            "thread-chart-sample-view").unwrap();
        renderer.draw_frame(|g| {
           g.clear_screen(Color::WHITE);
        });
    }

    #[wasm_bindgen]
    pub fn on_chart_mouse_move(&mut self, x: f32, y: f32) {
        self.on_mouse_move(Some(x), y);
    }

    #[wasm_bindgen]
    pub fn on_header_mouse_move(&mut self, x: f32, y: f32) {
        self.on_mouse_move(None, y);
    }

    fn on_mouse_move(&mut self, x: Option<f32>, y: f32) {
        let thread_idx = (y / self.row_height) as usize;
        let thread_id = self.threads.get(thread_idx).map(|t| t.id);

        let mut highlighted_sample = None;
        if let Some(thread_id) = thread_id {
            match (self.per_thread_samples.get(&thread_id), x) {
                (Some(samples), Some(x)) => {
                    // TODO: binary search
                    for (i, sample) in samples.iter().enumerate() {
                        let sample_x = self.sample_view_width * (sample.timestamp - self.interval.start_millis) as f32
                            / self.interval.duration_millis as f32;
                        let mut right_bound = sample_x + self.chart_config.sample_render_size.width;
                        if let Some(next_sample) = samples.get(i + 1) {
                            right_bound = self.sample_view_width * (next_sample.timestamp - self.interval.start_millis) as f32
                                / self.interval.duration_millis as f32;
                        }
                        if sample_x <= x && x <= right_bound {
                            highlighted_sample = Some(
                                (i, sample_x, thread_idx as f32 * self.row_height));
                            break
                        }
                    }
                }
                _ => {}
            }
        }

        let sample_idx = highlighted_sample.map(|s| s.0);
        if thread_id != self.highlighted_thread_id || sample_idx != self.highlighted_sample_idx {
            Self::clear_overlay();
            match (Self::get_canvas_ctx("chart-overlay"), Self::get_canvas_ctx("header-overlay")) {
                (Some(chart_ctx), Some(header_ctx)) => {
                    chart_ctx.set_fill_style(&JsValue::from("#40404040"));
                    header_ctx.set_fill_style(&JsValue::from("#40404040"));

                    chart_ctx.fill_rect(
                        0.0,
                        (thread_idx as f32 * self.row_height) as f64,
                        chart_ctx.canvas().unwrap().width() as f64,
                        self.row_height as f64);
                    header_ctx.fill_rect(
                        0.0,
                        (thread_idx as f32 * self.row_height) as f64,
                        header_ctx.canvas().unwrap().width() as f64,
                        self.row_height as f64);

                    if let Some((_, x, y)) = highlighted_sample {
                        chart_ctx.set_fill_style(&JsValue::from("#f04074"));
                        chart_ctx.fill_rect(
                            x as f64,
                            y as f64,
                            self.chart_config.sample_render_size.width as f64,
                            self.row_height as f64
                        );
                    }
                }
                _ => {}
            }
        }
        self.highlighted_sample_idx = sample_idx;
        self.highlighted_thread_id = thread_id;
    }

    #[wasm_bindgen]
    pub fn on_mouse_out(&mut self) {
        Self::clear_overlay();
        self.highlighted_thread_id = None;
        self.highlighted_sample_idx = None;
    }

    #[wasm_bindgen]
    pub fn on_chart_click(&self) -> Option<StackTrace> {
        match (self.highlighted_thread_id, self.highlighted_sample_idx) {
            (Some(thread_id), Some(sample_idx)) => {
                self.per_thread_samples.get(&thread_id)
                    .and_then(|s| s.get(sample_idx))
                    .and_then(|s| self.stack_trace_pool.get(&s.stack_trace_id))
                    .cloned()
            },
            _ => None
        }
    }

    fn clear_overlay() {
        Self::clear_canvas("header-overlay");
        Self::clear_canvas("chart-overlay");
    }

    fn get_element_by_id<T: JsCast>(id: &str) -> Option<T> {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|e| e.dyn_into::<T>().ok())
    }

    fn clear_canvas(id: &str) {
        let canvas = Self::get_element_by_id::<web_sys::HtmlCanvasElement>(id);
        if let Some(canvas) = canvas {
            let mut ctx = canvas
                .get_context("2d").ok()
                .flatten()
                .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
                .unwrap();
            ctx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        }
    }

    fn get_canvas_ctx(id: &str) -> Option<web_sys::CanvasRenderingContext2d> {
        Self::get_element_by_id::<web_sys::HtmlCanvasElement>(id)
            .and_then(|c| c.get_context("2d").ok())
            .flatten()
            .and_then(|c| c.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
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

struct Rect {
    x: f32, y: f32, width: f32, height: f32, color: Color
}

struct HitResult {
    thread_id: Option<i64>,
    thread_idx: Option<usize>,
    sample_idx: Option<usize>,
}
