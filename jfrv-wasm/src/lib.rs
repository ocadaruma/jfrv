pub mod execution_sample;
pub mod jbm;
pub mod profile;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use tsify::Tsify;

#[cfg(target_arch = "wasm32")]
type Result<T> = std::result::Result<T, JsValue>;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug)
        .map_err(|e| JsError::new(e.to_string().as_str()))?;
    info!("jfrv wasm loaded");
    Ok(())
}

#[derive(Default, Deserialize, Serialize)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(rename_all = "camelCase")]
pub struct TimeInterval {
    pub start_millis: i64,
    pub end_millis: i64,
}

impl TimeInterval {
    pub fn new(start_millis: i64, end_millis: i64) -> Self {
        Self {
            start_millis,
            end_millis,
        }
    }

    pub fn duration_millis(&self) -> i64 {
        self.end_millis - self.start_millis
    }
}
