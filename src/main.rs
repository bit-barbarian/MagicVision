#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod constants;
mod recognition;
mod types;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MagicVision",
        native_options,
        Box::new(|_cc| Ok(Box::new(magicvision::gui::app::Application::default()))),
    )
}
