use crate::profile::FrameType;
use crate::web::{Canvas, Document};
use crate::{flame_graph, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, Window};

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FlameGraphConfig {
    pub chart_id: String,
    pub highlight_id: String,
    pub highlight_text_id: String,
    pub status_id: String,
    pub color_palette: HashMap<FrameType, FrameColorConfig>,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FrameColorConfig {
    pub base_hex: u32,
    pub r_mix: u32,
    pub g_mix: u32,
    pub b_mix: u32,
}

#[wasm_bindgen]
pub struct FlameGraphRenderer {
    document: Document,
    flame_graph: FlameGraph,
    config: FlameGraphConfig,
    chart: Canvas,
    highlight: HtmlElement,
    highlight_text: HtmlElement,
    status: HtmlElement,
    device_pixel_ratio: f64,
}

#[wasm_bindgen]
impl FlameGraphRenderer {
    #[wasm_bindgen(constructor)]
    pub fn try_new(
        window: Window,
        flame_graph: FlameGraph,
        config: FlameGraphConfig,
    ) -> Result<FlameGraphRenderer> {
        let document = Document {
            raw: window.document().unwrap(),
        };
        let chart = document.get_canvas_by_id(config.chart_id.as_str())?;
        let highlight = document.get_element_by_id(config.highlight_id.as_str())?;
        let highlight_text = document.get_element_by_id(config.highlight_text_id.as_str())?;
        let status = document.get_element_by_id(config.status_id.as_str())?;
        Ok(Self {
            document,
            flame_graph,
            config,
            chart,
            highlight,
            highlight_text,
            status,
            device_pixel_ratio: window.device_pixel_ratio(),
        })
    }

    pub fn render(&self) -> Result<()> {
        self.chart.raw.style().set_property(
            "height",
            format!("{}px", 16 * self.flame_graph.levels.len()).as_str(),
        )?;

        let chart_width = self.chart.raw.offset_width() as usize;
        let chart_height = self.chart.raw.offset_height() as usize;

        // fix the canvas size to current size
        self.chart
            .raw
            .style()
            .set_property("width", format!("{}px", chart_width).as_str())?;
        self.chart
            .raw
            .set_width((chart_width as f64 * self.device_pixel_ratio) as u32);
        self.chart
            .raw
            .set_height((chart_height as f64 * self.device_pixel_ratio) as u32);
        self.chart
            .ctx
            .scale(self.device_pixel_ratio, self.device_pixel_ratio)?;
        if let Some(body) = self.document.raw.body() {
            self.chart
                .ctx
                .set_font(&body.style().get_property_value("font")?);
        }

        let x0 = self.flame_graph.levels[0].frames[0].x;
        let x1 = x0 + self.flame_graph.levels[0].frames[0].count;
        let px = chart_width as f64 / self.flame_graph.levels[0].frames[0].count as f64;
        for (h, level) in self.flame_graph.levels.iter().enumerate() {
            let y = chart_height - (h + 1) * 16;
            for frame in level.frames.iter() {
                // why this condition is necessary?
                if frame.x < x1 && frame.x + frame.count > x0 {
                    self.chart
                        .ctx
                        .set_fill_style(&JsValue::from_str(&frame.frame_color_hex));
                    self.chart.ctx.fill_rect(
                        (frame.x - x0) as f64 * px,
                        y as f64,
                        frame.count as f64 * px,
                        15.0,
                    );

                    if frame.count as f64 * px >= 21.0 {
                        let chars = ((frame.count as f64 * px) / 7.0).floor() as usize;
                        self.chart.ctx.set_fill_style(&JsValue::from_str("#000000"));
                        let title = if frame.title.len() <= chars {
                            frame.title.clone()
                        } else {
                            format!("{}..", &frame.title[0..(chars - 2)])
                        };
                        self.chart.ctx.fill_text_with_max_width(
                            &title,
                            (frame.x - x0) as f64 * px + 3.0,
                            y as f64 + 12.0,
                            frame.count as f64 * px - 6.0,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FlameGraph {
    levels: Vec<Level>,
}

impl FlameGraph {
    pub fn from(
        flame_graph: &flame_graph::FlameGraph,
        color_palette: &HashMap<FrameType, FrameColorConfig>,
    ) -> Self {
        let mut result = Self::default();

        result
            .levels
            .resize_with(flame_graph.depth + 1, Level::default);
        Self::traverse_frame(
            "all",
            &flame_graph.root,
            0,
            0,
            &mut result.levels,
            color_palette,
        );

        result
    }

    fn traverse_frame(
        title: &str,
        frame: &flame_graph::Frame,
        x: usize,
        level: usize,
        levels: &mut Vec<Level>,
        color_palette: &HashMap<FrameType, FrameColorConfig>,
    ) {
        let mut result = Frame::default();
        result.detail.interpreted_count = Self::nonzero(frame.interpreted_count);
        result.detail.c1_compiled_count = Self::nonzero(frame.c1_count);
        result.detail.inlined_count = Self::nonzero(frame.inlined_count);

        if let Some(cnt) = result.detail.interpreted_count {
            result
                .detail
                .description
                .push_str(format!(", int={}", cnt).as_str());
        }
        if let Some(cnt) = result.detail.c1_compiled_count {
            result
                .detail
                .description
                .push_str(format!(", c1={}", cnt).as_str());
        }
        if let Some(cnt) = result.detail.inlined_count {
            result
                .detail
                .description
                .push_str(format!(", inln={}", cnt).as_str());
        }

        result.x = x;
        result.count = frame.total_count;
        result.frame_type = frame.calculated_type();
        result.frame_color_hex =
            format!("#{:x}", Self::get_color(result.frame_type, color_palette));
        result.title = title.to_string();
        levels[level].frames.push(result);

        let mut x = x;
        x += frame.self_count;
        for (k, v) in frame.children.iter() {
            Self::traverse_frame(k.name.as_str(), v, x, level + 1, levels, color_palette);
            x += v.total_count;
        }
    }

    fn nonzero(n: usize) -> Option<usize> {
        if n > 0 {
            Some(n)
        } else {
            None
        }
    }

    fn get_color(
        frame_type: FrameType,
        color_palette: &HashMap<FrameType, FrameColorConfig>,
    ) -> u32 {
        let color_config = color_palette.get(&frame_type).unwrap();
        let random = js_sys::Math::random();
        let color_jitter = (((color_config.r_mix as f64 * random) as u32) << 16)
            | (((color_config.g_mix as f64 * random) as u32) << 8)
            | ((color_config.b_mix as f64 * random) as u32);

        color_config.base_hex + color_jitter
    }
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub frames: Vec<Frame>,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub x: usize,
    pub count: usize,
    pub frame_type: FrameType,
    pub frame_color_hex: String,
    pub title: String,
    pub detail: FrameDetail,
}

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FrameDetail {
    interpreted_count: Option<usize>,
    c1_compiled_count: Option<usize>,
    inlined_count: Option<usize>,
    description: String,
}
