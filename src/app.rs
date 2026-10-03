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
    suggestions: Vec<String>,
    search_rx: Option<Receiver<Vec<String>>>,
    last_input: String,
    last_requested_query: String,
    last_input_change: std::time::Instant,
}

impl Default for YtGetApp {
    fn default() -> Self {
        Self {
            url: String::new(),
            format: OutputFormat::Original,
            playlist: false,
            status: String::new(),
            progress: 0.0,
            downloading: false,
            rx: None,
            output_dir: config::load_output_dir(),
            suggestions: Vec::new(),
            search_rx: None,
            last_input: String::new(),
            last_requested_query: String::new(),
            last_input_change: std::time::Instant::now(),
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

        let input = self.url.trim().to_string();

        if input.is_empty() {
            self.status = "Please enter a URL or search term.".to_string();
            self.downloading = false;
            return;
        }

        let source = if input.starts_with("http://") || input.starts_with("https://") {
            input
        } else {
            format!("ytsearch:{}", input)
        };

        downloader::start(
            source,
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

    fn poll_suggestions(&mut self) {
        let Some(rx) = &self.search_rx else {
            return;
        };

        while let Ok(suggestions) = rx.try_recv() {
            self.suggestions = suggestions;
        }
    }

    fn update_suggestions(&mut self) {
        let query = self.url.trim().to_string();
        let is_url = query.starts_with("http") || query.starts_with("https");

        // In case of empty input dont show anything
        if query.is_empty() || is_url {
            self.suggestions.clear();
            self.search_rx = None;
            self.last_input.clear();
            self.last_requested_query.clear();
            return;
        }

        // Detect input
        if query != self.last_input {
            self.last_input = query.clone();
            self.last_input_change = std::time::Instant::now();
            self.suggestions.clear();
        }

        // Wait 300 ms after the last input change before requesting suggestions
        if self.last_input_change.elapsed().as_millis() < 300 {
            return;
        }

        // Dont request the exact same query twice
        if query == self.last_requested_query {
            return;
        }

        self.last_requested_query = query.clone();

        let (tx, rx) = channel();
        self.search_rx = Some(rx);

        std::thread::spawn(move || {
            let client = match reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_millis(500))
                .build()
            {
                Ok(client) => client,
                Err(e) => {
                    eprintln!("Failed to create client: {}", e);
                    return;
                }
            };

            let response = client
                .get("https://suggestqueries.google.com/complete/search")
                .query(&[
                    ("client", "firefox"),
                    ("ds", "yt"),
                    ("q", query.as_str()),
                    ("hl", "en"),
                ])
                .send();

            let Ok(response) = response else {
                return;
            };

            let Ok(text) = response.text() else {
                return;
            };

            let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) else {
                return;
            };

            let Some(items) = data.get(1).and_then(|v| v.as_array()) else {
                return;
            };

            let suggestions = items
                .iter()
                .filter_map(|item| item.as_str().map(String::from))
                .take(8)
                .collect::<Vec<_>>();

            let _ = tx.send(suggestions);
        });
    }
}

impl eframe::App for YtGetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();
        self.poll_suggestions();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                let width = 480.0_f32.min(ui.available_width());

                ui.horizontal(|ui| {
                    ui.add_space((ui.available_width() - width).max(0.0) / 2.0);
                    ui.vertical(|ui| {
                        ui.set_width(width);

                        ui.add_space(4.0);
                        ui.vertical_centered(|ui| {
                            ui.heading("ytget");
                            ui.label(egui::RichText::new("YouTube Downloader").weak());
                        });
                        ui.add_space(12.0);

                        let card =
                            |ui: &mut egui::Ui, add_contents: &mut dyn FnMut(&mut egui::Ui)| {
                                let width = ui.available_width();

                                egui::Frame::group(ui.style())
                                    .fill(egui::Color32::from_rgb(32, 32, 38))
                                    .rounding(10.0)
                                    .inner_margin(12.0)
                                    .show(ui, |ui| {
                                        ui.set_min_width(width - 24.0);
                                        add_contents(ui);
                                    });
                            };

                        fn field_label(ui: &mut egui::Ui, text: &str) {
                            ui.add_sized(
                                [60.0, 32.0],
                                egui::Label::new(egui::RichText::new(text).size(14.0).strong()),
                            );
                        }

                        card(ui, &mut |ui| {
                            ui.horizontal(|ui| {
                                field_label(ui, "URL or search:");
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.url)
                                        .desired_width(ui.available_width()),
                                );

                                self.update_suggestions();

                                if !self.suggestions.is_empty() {
                                    let rect = response.rect;

                                    egui::Area::new(egui::Id::new("search_suggestions"))
                                        .order(egui::Order::Foreground)
                                        .fixed_pos(egui::pos2(
                                            rect.left() - 5.0,
                                            rect.bottom() + 4.0,
                                        ))
                                        .show(ctx, |ui| {
                                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                                ui.set_width(rect.width());

                                                for suggestion in &self.suggestions {
                                                    if ui
                                                        .selectable_label(false, suggestion)
                                                        .clicked()
                                                    {
                                                        self.url = suggestion.to_string();
                                                    }
                                                }
                                            });
                                        });
                                }
                            });
                        });

                        //Format + Playlist Check + Save Folder Card
                        card(ui, &mut |ui| {
                            ui.horizontal(|ui| {
                                field_label(ui, "Format:");
                                egui::ComboBox::from_id_source("format_selector")
                                    .selected_text(match self.format {
                                        OutputFormat::Original => "Original",
                                        OutputFormat::VideoWebm => "Video (webm)",
                                        OutputFormat::VideoMp4 => "Video (mp4)",
                                        OutputFormat::AudioMp3 => "Audio (mp3)",
                                        OutputFormat::AudioM4a => "Audio (m4a)",
                                        OutputFormat::AudioFlac => "Audio (flac)",
                                        OutputFormat::AudioWav => "Audio (wav)",
                                        OutputFormat::AudioOpus => "Audio (opus)",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::Original,
                                            "Original",
                                        );
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::VideoWebm,
                                            "Video (webm)",
                                        );
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::VideoMp4,
                                            "Video (mp4)",
                                        );
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::AudioMp3,
                                            "Audio (mp3)",
                                        );
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::AudioM4a,
                                            "Audio (m4a)",
                                        );
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::AudioFlac,
                                            "Audio (flac)",
                                        );
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::AudioWav,
                                            "Audio (wav)",
                                        );
                                        ui.radio_value(
                                            &mut self.format,
                                            OutputFormat::AudioOpus,
                                            "Audio (opus)",
                                        );
                                    });
                            });

                            ui.checkbox(&mut self.playlist, "Download full playlist");

                            ui.horizontal(|ui| {
                                field_label(ui, "Save to:");
                                let display_text = match self.output_dir.as_deref() {
                                    Some(path) => path.display().to_string(),
                                    None => "(default folder)".to_string(),
                                };

                                let available = ui.available_width();

                                ui.add(
                                    egui::Button::new(display_text)
                                        .sense(egui::Sense::hover())
                                        .min_size(egui::vec2((available - 75.0).max(100.0), 28.0)),
                                );

                                if ui.button("Browse...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        config::save_output_dir(&path);
                                        self.output_dir = Some(path);
                                    }
                                }
                            });
                        });

                        ui.add_space(12.0);

                        //Download Button
                        ui.add_enabled_ui(!self.downloading, |ui| {
                            let button =
                                egui::Button::new(egui::RichText::new("Download").size(16.0))
                                    .min_size(egui::vec2(ui.available_width(), 36.0));
                            if ui.add(button).clicked() {
                                self.start_download();
                            }
                        });

                        ui.add_space(6.0);

                        if self.downloading || self.progress > 0.0 {
                            ui.add(egui::ProgressBar::new(self.progress).show_percentage());
                            ui.label(&self.status);
                        }
                    });
                });
            });
        });
    }
}
