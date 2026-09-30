mod app;
mod config;
mod downloader;
mod theme;

use app::YtGetApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ytget",
        options,
        Box::new(|cc| {
            theme::apply_theme(&cc.egui_ctx);
            Ok(Box::new(YtGetApp::default()))
        }),
    )
}
