mod app;
mod downloader;

use app::YtGetApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ytget",
        options,
        Box::new(|_cc| Ok(Box::new(YtGetApp::default()))),
    )
}