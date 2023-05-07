//! jbm chart renderer.

use crate::jbm::{JbmFilter, Profile};
use crate::web::{Canvas, Document, Svg};
use crate::Result;
use chrono::{Duration, Local, NaiveDateTime, TimeZone};
use log::debug;

use crate::profile::{StackTrace, ThreadState};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct JbmChartConfig {
    pub default_margin: f32,
    pub font_size: f32,
    pub header_config: JbmHeaderConfig,
    pub sample_view_config: JbmSampleViewConfig,
    pub thread_state_color_config: JbmThreadStateColorConfig,
    pub overlay_config: JbmOverlayConfig,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct JbmHeaderConfig {
    pub border_width: f32,
    pub border_color_rgb_hex: u32,
    pub element_id: String,
    pub overlay_element_id: String,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct JbmSampleViewConfig {
    pub element_id: String,
    pub overlay_element_id: String,
    pub sample_render_height: f32,
    pub sample_width_per_hour: f32,
    pub background_rgb_hex: u32,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct JbmOverlayConfig {
    pub row_highlight_argb_hex: u32,
    pub sample_highlight_rgb_hex: u32,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct JbmThreadStateColorConfig {
    pub state_runnable_rgb_hex: u32,
    pub state_sleeping_rgb_hex: u32,
    pub state_unknown_rgb_hex: u32,
}

/// State of the current rendered chart
#[derive(Default)]
pub struct JbmChartState {
    highlighted_thread_idx: Option<usize>,
    highlighted_thread_id: Option<i64>,
    highlighted_sample_idx: Option<usize>,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct JbmSampleInfo {
    pub stack_trace: StackTrace,
    pub duration_millis: i64,
    pub thread_name: String,
    pub offcpu_start: String,
    pub offcpu_end: String,
}

#[wasm_bindgen]
pub struct JbmRenderer {
    profile: Profile,
    chart_config: JbmChartConfig,
    chart_state: JbmChartState,
    document: Document,
    header: Svg,
    header_overlay: Canvas,
    chart: Canvas,
    chart_overlay: Canvas,
    row_highlight: JsValue,
    sample_highlight: JsValue,
}

#[wasm_bindgen]
impl JbmRenderer {
    #[wasm_bindgen(constructor)]
    pub fn try_new(chart_config: JbmChartConfig) -> Result<JbmRenderer> {
        let document = Document::try_new()?;
        Ok(Self {
            profile: Profile::default(),
            chart_state: JbmChartState::default(),
            header: document.get_svg_by_id(chart_config.header_config.element_id.as_str())?,
            header_overlay: document
                .get_canvas_by_id(chart_config.header_config.overlay_element_id.as_str())?,
            chart: document
                .get_canvas_by_id(chart_config.sample_view_config.element_id.as_str())?,
            chart_overlay: document
                .get_canvas_by_id(chart_config.sample_view_config.overlay_element_id.as_str())?,
            row_highlight: JsValue::from_str(
                format!("#{:x}", chart_config.overlay_config.row_highlight_argb_hex).as_str(),
            ),
            sample_highlight: JsValue::from_str(
                format!(
                    "#{:x}",
                    chart_config.overlay_config.sample_highlight_rgb_hex
                )
                .as_str(),
            ),
            document,
            chart_config,
        })
    }

    pub fn render(&self) -> Result<()> {
        let document = &self.document;

        let chart_height = self.row_height() * self.profile.threads().len() as f32;
        debug!("start render");

        self.header.clear();
        self.chart.clear();
        self.chart_overlay
            .raw
            .set_width(self.sample_view_width() as u32);
        self.chart_overlay.raw.set_height(chart_height as u32);
        self.chart.raw.set_width(self.sample_view_width() as u32);
        self.chart.raw.set_height(chart_height as u32);
        debug!("start draw frame");

        for (i, thread) in self.profile.threads().iter().enumerate() {
            if let Some(samples) = self.profile.per_thread_samples.get(&thread.os_thread_id) {
                let y = self.row_height() * i as f32
                    + (self.row_height()
                        - self.chart_config.sample_view_config.sample_render_height)
                        / 2.0;
                for (_j, sample) in samples.iter().enumerate() {
                    let x = self.sample_view_width()
                        * (sample.timestamp - self.profile.interval.start_millis) as f32
                        / self.profile.interval.duration_millis() as f32;
                    let color = match sample.state {
                        ThreadState::Unknown => {
                            self.chart_config
                                .thread_state_color_config
                                .state_unknown_rgb_hex
                        }
                        ThreadState::Runnable => {
                            self.chart_config
                                .thread_state_color_config
                                .state_runnable_rgb_hex
                        }
                        ThreadState::Sleeping => {
                            self.chart_config
                                .thread_state_color_config
                                .state_sleeping_rgb_hex
                        }
                    };

                    self.chart
                        .ctx
                        .set_fill_style(&JsValue::from_str(format!("#{:x}", color).as_str()));
                    self.chart.ctx.fill_rect(
                        x as f64,
                        y as f64,
                        self.chart_config.sample_view_config.sample_width_per_hour as f64
                            * (sample.duration_millis as f64 / 3600000.0),
                        self.chart_config.sample_view_config.sample_render_height as f64,
                    );
                }
            }

            let text = document
                .raw
                .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")?;
            let text_node = document.raw.create_text_node(&thread.name);
            text.set_attribute(
                "x",
                (self.chart_config.default_margin * 2.0)
                    .to_string()
                    .as_str(),
            )?;
            // y is the baseline of the text.
            // so we add fontSize to the current offset.
            // also add margin to allocate the margin-top.
            text.set_attribute(
                "y",
                (self.row_height() * i as f32 + self.chart_config.font_size)
                    .to_string()
                    .as_str(),
            )?;
            text.append_child(&text_node)?;
            self.header.raw.append_child(&text)?;
        }

        let header_width = self.header.raw.get_b_box()?.width();
        self.header.set_width(header_width)?;
        self.header.set_height(chart_height)?;
        self.header_overlay.raw.set_width(header_width as u32);
        self.header_overlay.raw.set_height(chart_height as u32);

        debug!("start render border");
        // render borders based on the header width retrieved from bbox
        for i in 0..(self.profile.threads().len() as isize - 1) {
            let y = (self.row_height() + self.row_height() * i as f32).to_string();
            let line = document
                .raw
                .create_element_ns(Some("http://www.w3.org/2000/svg"), "line")?;
            line.set_attribute("x1", "0")?;
            line.set_attribute("y1", y.as_str())?;
            line.set_attribute("x2", header_width.to_string().as_str())?;
            line.set_attribute("y2", y.as_str())?;
            line.set_attribute(
                "stroke-width",
                self.chart_config
                    .header_config
                    .border_width
                    .to_string()
                    .as_str(),
            )?;
            line.set_attribute(
                "stroke",
                format!(
                    "#{:x}",
                    self.chart_config.header_config.border_color_rgb_hex
                )
                .as_str(),
            )?;
            self.header.raw.append_child(&line)?;
        }

        Ok(())
    }

    pub fn initialize(&mut self, bytes: Vec<u8>) -> Result<()> {
        self.profile.load(bytes).map_err(Self::map_js_value)?;
        Ok(())
    }

    pub fn apply_filter(&mut self, filter: JbmFilter) -> Result<()> {
        self.profile
            .apply_filter(filter)
            .map_err(Self::map_js_value)?;
        self.render()
    }

    pub fn on_chart_mouse_move(&mut self, x: f32, y: f32) {
        self.on_mouse_move(Some(x), y)
    }

    pub fn on_header_mouse_move(&mut self, _x: f32, y: f32) {
        self.on_mouse_move(None, y)
    }

    pub fn on_mouse_out(&mut self) {
        self.header_overlay.clear();
        self.chart_overlay.clear();
        self.chart_state.highlighted_thread_idx = None;
        self.chart_state.highlighted_thread_id = None;
        self.chart_state.highlighted_sample_idx = None;
    }

    pub fn on_chart_click(&self) -> Option<JbmSampleInfo> {
        match self.chart_state {
            JbmChartState {
                highlighted_thread_idx: Some(thread_idx),
                highlighted_thread_id: Some(thread_id),
                highlighted_sample_idx: Some(sample_idx),
            } => {
                let thread = self.profile.threads().get(thread_idx);
                let sample = self
                    .profile
                    .per_thread_samples
                    .get(&thread_id)
                    .and_then(|s| s.get(sample_idx));
                let stack_trace = sample
                    .and_then(|s| self.profile.stack_trace_pool.get(&s.stack_trace_key))
                    .cloned();
                match (thread, sample, stack_trace) {
                    (Some(thread), Some(sample), Some(stack_trace)) => {
                        let t = NaiveDateTime::from_timestamp(
                            sample.timestamp / 1000,
                            (sample.timestamp % 1000) as u32 * 1000000,
                        );
                        Some(JbmSampleInfo {
                            stack_trace,
                            duration_millis: sample.duration_millis,
                            thread_name: thread.name.clone(),
                            offcpu_start: Local
                                .from_utc_datetime(&t)
                                .format("%Y-%m-%d %H:%M:%S.%3f")
                                .to_string(),
                            offcpu_end: (Local.from_utc_datetime(&t)
                                + Duration::milliseconds(sample.duration_millis))
                            .format("%Y-%m-%d %H:%M:%S.%3f")
                            .to_string(),
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn on_mouse_move(&mut self, x: Option<f32>, y: f32) {
        let thread_idx = (y / self.row_height()) as usize;
        let thread_id = self
            .profile
            .threads()
            .get(thread_idx)
            .map(|t| t.os_thread_id);

        let mut highlighted_sample = None;
        if let Some(thread_id) = thread_id {
            if let (Some(samples), Some(x)) = (self.profile.per_thread_samples.get(&thread_id), x) {
                // TODO: binary search
                for (i, sample) in samples.iter().enumerate() {
                    let sample_x = self.sample_view_width()
                        * (sample.timestamp - self.profile.interval.start_millis) as f32
                        / self.profile.interval.duration_millis() as f32;
                    let mut right_bound = sample_x
                        + self.chart_config.sample_view_config.sample_width_per_hour
                            * (sample.duration_millis as f32 / 3600000.0);
                    if let Some(next_sample) = samples.get(i + 1) {
                        right_bound = self.sample_view_width()
                            * (next_sample.timestamp - self.profile.interval.start_millis) as f32
                            / self.profile.interval.duration_millis() as f32;
                    }
                    if sample_x <= x && x <= right_bound {
                        highlighted_sample = Some((
                            i,
                            sample.duration_millis,
                            sample_x,
                            thread_idx as f32 * self.row_height(),
                        ));
                        break;
                    }
                }
            }
        }

        let sample_idx = highlighted_sample.map(|s| s.0);
        if thread_id != self.chart_state.highlighted_thread_id
            || sample_idx != self.chart_state.highlighted_sample_idx
        {
            self.header_overlay.clear();
            self.chart_overlay.clear();

            self.chart_overlay.ctx.set_fill_style(&self.row_highlight);
            self.header_overlay.ctx.set_fill_style(&self.row_highlight);

            let y = (thread_idx as f32 * self.row_height()) as f64;
            let h = self.row_height() as f64;

            self.chart_overlay
                .ctx
                .fill_rect(0.0, y, self.chart_overlay.raw.width() as f64, h);
            self.header_overlay
                .ctx
                .fill_rect(0.0, y, self.header_overlay.raw.width() as f64, h);

            if let Some((_, duration, x, y)) = highlighted_sample {
                self.chart_overlay
                    .ctx
                    .set_fill_style(&self.sample_highlight);
                self.chart_overlay.ctx.fill_rect(
                    x as f64,
                    y as f64,
                    self.chart_config.sample_view_config.sample_width_per_hour as f64
                        * (duration as f64 / 3600000.0),
                    h,
                );
            }
        }
        self.chart_state.highlighted_thread_idx = thread_id.map(|_| thread_idx);
        self.chart_state.highlighted_thread_id = thread_id;
        self.chart_state.highlighted_sample_idx = sample_idx;
    }

    fn sample_view_width(&self) -> f32 {
        self.chart_config.sample_view_config.sample_width_per_hour
            * (self.profile.interval.duration_millis() as f32 / 1000.0 / 3600.0)
    }

    fn row_height(&self) -> f32 {
        self.chart_config.font_size + self.chart_config.default_margin * 2.0
    }

    fn map_js_value<T: ToString>(t: T) -> JsValue {
        JsValue::from_str(t.to_string().as_str())
    }
}
