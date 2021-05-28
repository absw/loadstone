#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use eframe::NativeOptions;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = loadstone_front::LoadstoneApp::default();
    eframe::run_native(Box::new(app), NativeOptions::default());
}
