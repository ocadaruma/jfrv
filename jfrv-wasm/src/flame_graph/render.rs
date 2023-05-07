use crate::profile::FrameType;
use crate::web::{Canvas, Document};
use crate::{flame_graph, Result};
use num_format::{Locale, ToFormattedString};
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
    _config: FlameGraphConfig,
    chart: Canvas,
    highlight: HtmlElement,
    highlight_text: HtmlElement,
    status: HtmlElement,
    device_pixel_ratio: f64,
    root_index: (usize, usize),
    selected_index: Option<(usize, usize)>,
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
            _config: config,
            chart,
            highlight,
            highlight_text,
            status,
            device_pixel_ratio: window.device_pixel_ratio(),
            root_index: (0, 0),
            selected_index: None,
        })
    }

    pub fn render(&self) -> Result<()> {
        self.chart.raw.style().set_property(
            "height",
            format!("{}px", 16 * self.flame_graph.levels.len()).as_str(),
        )?;

        let chart_width = self.chart_width();
        let chart_height = self.chart_height();

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

        self.inner_render()?;

        Ok(())
    }

    pub fn onmousemove(&mut self, e: web_sys::MouseEvent) -> Result<()> {
        let level = (self.chart_height() as usize - e.offset_y() as usize) / 16;
        if level < self.flame_graph.levels.len() {
            if let Some(frame_idx) = self.find_frame(
                &self.flame_graph.levels[level],
                e.offset_x() as f64 / self.ratio() + self.root().x as f64,
            ) {
                self.selected_index = Some((level, frame_idx));
                let frame = &self.flame_graph.levels[level].frames[frame_idx];

                let highlight_left = (frame.x as isize - self.root().x as isize).max(0) as f64
                    * self.ratio()
                    + self.chart.raw.offset_left() as f64;
                let highlight_width = frame.count.min(self.root().count) as f64 * self.ratio();
                let highlight_top =
                    (self.chart_height() - (level + 1) * 16) + self.chart.raw.offset_top() as usize;
                self.highlight
                    .style()
                    .set_property("left", format!("{}px", highlight_left).as_str())?;
                self.highlight
                    .style()
                    .set_property("width", format!("{}px", highlight_width).as_str())?;
                self.highlight
                    .style()
                    .set_property("top", format!("{}px", highlight_top).as_str())?;
                self.highlight.style().set_property("display", "block")?;
                self.highlight_text.set_text_content(Some(&frame.title));

                let num = frame.count;
                let denom = self.flame_graph.levels[0].frames[0].count;
                let percentage = format!("{:.2}", 100.0 * num as f64 / denom as f64);
                let title = format!(
                    "{}\n({} samples{}, {}%)",
                    frame.title,
                    frame.count.to_formatted_string(&Locale::en),
                    frame.detail.description,
                    if num >= denom { "100" } else { &percentage }
                );
                self.chart.raw.set_title(&title);
                self.chart.raw.style().set_property("cursor", "pointer")?;
                self.status
                    .set_text_content(Some(format!("Function: {}", title).as_str()));

                return Ok(());
            }
        }
        self.onmouseout(e)?;
        Ok(())
    }

    pub fn onmouseout(&mut self, _e: web_sys::MouseEvent) -> Result<()> {
        self.highlight.style().set_property("display", "none")?;
        self.status.set_text_content(Some("\u{a0}")); // fill nbsp by default
        self.chart.raw.set_title("");
        self.chart.raw.style().set_property("cursor", "")?;
        self.selected_index = None;
        Ok(())
    }

    pub fn onclick(&mut self, _e: web_sys::MouseEvent) -> Result<()> {
        if let Some(idx) = self.selected_index {
            if idx != self.root_index {
                self.root_index = idx;
                self.inner_render()?;

                // manually fire onmousemove to update highlight after rendered with new root
                return self.onmousemove(_e);
            }
        }
        Ok(())
    }

    fn root_level(&self) -> usize {
        self.root_index.0
    }

    fn root(&self) -> &Frame {
        &self.flame_graph.levels[self.root_index.0].frames[self.root_index.1]
    }

    fn ratio(&self) -> f64 {
        self.chart_width() as f64 / self.root().count as f64
    }

    fn inner_render(&self) -> Result<()> {
        self.chart.ctx.set_fill_style(&JsValue::from_str("#ffffff"));
        self.chart.ctx.fill_rect(
            0.0,
            0.0,
            self.chart_width() as f64,
            self.chart_height() as f64,
        );

        let x0 = self.root().x as isize;
        let x1 = x0 + self.root().count as isize;
        let ratio = self.ratio();
        for (h, level) in self.flame_graph.levels.iter().enumerate() {
            let y = self.chart_height() - (h + 1) * 16;
            for frame in level.frames.iter() {
                // render the frame only when it has horizontal intersection with root frame
                if (frame.x as isize) < x1 && (frame.x + frame.count) as isize > x0 {
                    let frame_x = (frame.x as isize - x0).max(0) as f64 * ratio;
                    let frame_y = y as f64;
                    let frame_width = frame.count as f64 * ratio;
                    self.chart
                        .ctx
                        .set_fill_style(&JsValue::from_str(&frame.frame_color_hex));
                    self.chart.ctx.fill_rect(
                        frame_x,
                        frame_y,
                        // this may exceeds the canvas area when we render frames
                        // which are below root, but we don't care
                        frame.count as f64 * ratio,
                        15.0,
                    );

                    // render text only when a frame has certain width
                    if frame_width >= 21.0 {
                        let chars = (frame_width / 7.0).floor() as usize;
                        self.chart.ctx.set_fill_style(&JsValue::from_str("#000000"));
                        let title = if frame.title.len() <= chars {
                            frame.title.clone()
                        } else {
                            format!("{}..", &frame.title[0..(chars - 2)])
                        };
                        self.chart.ctx.fill_text_with_max_width(
                            &title,
                            frame_x + 3.0, // add padding
                            frame_y + 12.0,
                            frame_width - 6.0,
                        )?;
                    }

                    if h < self.root_level() {
                        self.chart
                            .ctx
                            .set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.5)"));
                        self.chart
                            .ctx
                            .fill_rect(frame_x, frame_y, frame_width, 15.0);
                    }
                }
            }
        }
        Ok(())
    }

    fn chart_width(&self) -> usize {
        self.chart.raw.offset_width() as usize
    }

    fn chart_height(&self) -> usize {
        self.chart.raw.offset_height() as usize
    }

    // perform binary search against the frames in the level
    fn find_frame(&self, level: &Level, x: f64) -> Option<usize> {
        let mut left = 0;
        let mut right = level.frames.len() - 1;

        while left <= right {
            let mid = (left + right) / 2;
            let f = &level.frames[mid];
            if f.x as f64 > x {
                right = if mid > 0 { mid - 1 } else { break };
            } else if (f.x + f.count) as f64 <= x {
                left = mid + 1;
            } else {
                return Some(mid);
            }
        }
        if let Some(frame) = level.frames.get(left) {
            if (frame.x as f64 - x) * self.ratio() < 0.5 {
                return Some(left);
            }
        }
        if let Some(frame) = level.frames.get(right) {
            if (frame.x + frame.count) as f64 * self.ratio() < 0.5 {
                return Some(right);
            }
        }

        None
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
