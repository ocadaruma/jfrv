pub mod execution_sample;
pub mod profile;

#[cfg(target_arch = "wasm32")]
mod web;

// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use log::info;
use serde::{Deserialize, Serialize};

use tsify::Tsify;

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

#[derive(Default, Deserialize, Serialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Serialize, Default, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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
