mod app;
mod config;
mod downloader;

use app::YtGetApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ytget",
        options,
        Box::new(|_cc| Ok(Box::new(YtGetApp::default()))),
    )
}
