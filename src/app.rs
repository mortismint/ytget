use crate::config;
use crate::downloader::{self, DownloadMsg, OutputFormat};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};

pub struct YtGetApp {
    url: String,
    format: OutputFormat,
    playlist: bool,
    status: String,
    progress: f32,
    downloading: bool,
    rx: Option<Receiver<DownloadMsg>>,
    output_dir: Option<PathBuf>,
}

impl Default for YtGetApp {
    fn default() -> Self {
        Self {
            url: String::new(),
            format: OutputFormat::Video,
            playlist: false,
            status: String::new(),
            progress: 0.0,
            downloading: false,
            rx: None,
            output_dir: config::load_output_dir(),
        }
    }
}

impl YtGetApp {
    fn start_download(&mut self) {
        let (tx, rx) = channel();
        self.rx = Some(rx);
        self.downloading = true;
        self.progress = 0.0;
        self.status = "Starting...".to_string();

        downloader::start(
            self.url.clone(),
            self.format,
            self.playlist,
            self.output_dir.clone(),
            tx,
        );
    }

    fn poll_messages(&mut self) {
        let Some(rx) = &self.rx else { return };

        while let Ok(msg) = rx.try_recv() {
            match msg {
                DownloadMsg::Progress(pct) => {
                    self.progress = pct / 100.0;
                    self.status = format!("Downloading... {:.1}%", pct);
                }
                DownloadMsg::Log(line) => {
                    self.status = line;
                }
                DownloadMsg::Done => {
                    self.status = "Download completed successfully.".to_string();
                    self.progress = 1.0;
                    self.downloading = false;
                }
                DownloadMsg::Error(err) => {
                    self.status = err;
                    self.downloading = false;
                }
            }
        }
    }
}

impl eframe::App for YtGetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;

            ui.heading("ytget — YouTube Downloader");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("URL:");
                ui.text_edit_singleline(&mut self.url);
            });

            ui.horizontal(|ui| {
                ui.label("Format:");
                ui.radio_value(&mut self.format, OutputFormat::Video, "Video (mp4)");
                ui.radio_value(&mut self.format, OutputFormat::AudioMp3, "Audio (mp3)");
            });

            ui.checkbox(&mut self.playlist, "Download full playlist");

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Save to:");

                let display_text = match self.output_dir.as_deref() {
                    Some(path) => path.display().to_string(),
                    None => "(default folder)".to_string(),
                };
                ui.label(display_text);

                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        config::save_output_dir(&path);
                        self.output_dir = Some(path);
                    }
                }
            });

            ui.add_enabled_ui(!self.downloading, |ui| {
                if ui.button("Download").clicked() {
                    self.start_download();
                }
            });

            ui.add_space(6.0);

            ui.add(egui::ProgressBar::new(self.progress).show_percentage());

            ui.label(&self.status);
        });

        if self.downloading {
            ctx.request_repaint();
        }
    }
}
