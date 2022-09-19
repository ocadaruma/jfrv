//! Contains utility to access browser elements

use crate::Result;
use wasm_bindgen::{JsCast, JsValue};

pub struct Document {
    pub raw: web_sys::Document,
}

impl Document {
    pub fn try_new() -> Result<Self> {
        web_sys::window()
            .and_then(|w| w.document())
            .map(|raw| Self { raw })
            .ok_or_else(|| JsValue::from_str("Failed to get document"))
    }

    pub fn get_canvas_by_id(&self, id: &str) -> Result<Canvas> {
        self.get_element_by_id::<web_sys::HtmlCanvasElement>(id)
            .and_then(Canvas::try_new)
    }

    pub fn get_raw_canvas_by_id(&self, id: &str) -> Result<web_sys::HtmlCanvasElement> {
        self.get_element_by_id::<web_sys::HtmlCanvasElement>(id)
    }

    pub fn get_svg_by_id(&self, id: &str) -> Result<Svg> {
        self.get_element_by_id::<web_sys::SvgGraphicsElement>(id)
            .map(|raw| Svg { raw })
    }

    fn get_element_by_id<T: JsCast>(&self, id: &str) -> Result<T> {
        self.raw
            .get_element_by_id(id)
            .and_then(|e| e.dyn_into::<T>().ok())
            .ok_or_else(|| JsValue::from_str(format!("Element with id {} not found", id).as_str()))
    }
}

pub struct Canvas {
    pub raw: web_sys::HtmlCanvasElement,
    pub ctx: web_sys::CanvasRenderingContext2d,
}

impl Canvas {
    fn try_new(raw: web_sys::HtmlCanvasElement) -> Result<Self> {
        Ok(Self {
            ctx: raw
                .get_context("2d")
                .and_then(|c| c.ok_or_else(|| JsValue::from_str("Failed to get context 2d")))
                .map(|o| o.dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap())?,
            raw,
        })
    }

    pub fn clear(&self) {
        self.ctx
            .clear_rect(0.0, 0.0, self.raw.width() as f64, self.raw.height() as f64)
    }
}

pub struct Svg {
    pub raw: web_sys::SvgGraphicsElement,
}

impl Svg {
    pub fn clear(&self) {
        self.raw.set_inner_html("")
    }

    pub fn set_width(&self, width: f32) -> Result<()> {
        self.raw
            .set_attribute("width", width.to_string().as_str())
            .map(drop)
    }

    pub fn set_height(&self, height: f32) -> Result<()> {
        self.raw
            .set_attribute("height", height.to_string().as_str())
            .map(drop)
    }
}
