#![forbid(unsafe_code)]
#![allow(dead_code)]
#![feature(bool_to_option)]
#![feature(stmt_expr_attributes)]
#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::LoadstoneApp;

// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = LoadstoneApp::default();
    eframe::start_web(canvas_id, Box::new(app))
}
