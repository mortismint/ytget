use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub enum DownloadMsg {
    Progress(f32), // 0.0 to 100.0
    Log(String),
    Done,
    Error(String),
}

#[derive(PartialEq, Clone, Copy)]
pub enum OutputFormat {
    Video,
    AudioMp3,
}

fn yt_dlp_path() -> PathBuf {
    let filename = if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };

    // Look in PATH for yt-dlp
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(filename);

            if candidate.is_file() {
                return candidate;
            }
        }
    }

    // Fallback: Look in the same folder for yt-dlp as the executable
    let exe_path = std::env::current_exe().expect("couldn't find own exe path");

    let exe_dir = exe_path.parent().expect("exe has no parent directory");

    exe_dir.join(filename)
}

/* fn yt_dlp_path() -> PathBuf {
    let exe_path = std::env::current_exe().expect("couldn't find own exe path");
    let exe_dir = exe_path.parent().expect("exe has no parent directory");
    exe_dir.join("yt-dlp.exe")
} */

fn build_args(
    url: &str,
    format: OutputFormat,
    playlist: bool,
    output_dir: &Option<PathBuf>,
) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(dir) = output_dir {
        let template = dir.join("%(title)s.%(ext)s");
        args.push("-o".to_string());
        args.push(template.to_string_lossy().to_string());
    }

    match format {
        OutputFormat::AudioMp3 => {
            args.push("-x".to_string());
            args.push("--audio-format".to_string());
            args.push("mp3".to_string());
        }
        OutputFormat::Video => {
            args.push("--recode-video".to_string());
            args.push("mp4".to_string());
        }
    }

    if !playlist {
        args.push("--no-playlist".to_string());
    }

    args.push("--newline".to_string());

    args.push(url.to_string());
    args
}

fn parse_progress(line: &str) -> Option<f32> {
    if !line.starts_with("[download]") {
        return None;
    }

    for token in line.split_whitespace() {
        if let Some(percent_str) = token.strip_suffix('%') {
            return percent_str.parse::<f32>().ok();
        }
    }

    None
}

pub fn start(
    url: String,
    format: OutputFormat,
    playlist: bool,
    output_dir: Option<PathBuf>,
    tx: Sender<DownloadMsg>,
) {
    thread::spawn(move || {
        let ytdlp = yt_dlp_path();
        let args = build_args(&url, format, playlist, &output_dir);

        let child = Command::new(&ytdlp)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(DownloadMsg::Error(format!(
                    "Failed to run yt-dlp at {}: {}",
                    ytdlp.display(),
                    e
                )));
                return;
            }
        };

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let Ok(line) = line else { continue };

            if let Some(pct) = parse_progress(&line) {
                let _ = tx.send(DownloadMsg::Progress(pct));
            } else {
                let _ = tx.send(DownloadMsg::Log(line));
            }
        }

        match child.wait() {
            Ok(status) if status.success() => {
                let _ = tx.send(DownloadMsg::Done);
            }
            Ok(status) => {
                let _ = tx.send(DownloadMsg::Error(format!("yt-dlp exited with {}", status)));
            }
            Err(e) => {
                let _ = tx.send(DownloadMsg::Error(format!("wait() failed: {}", e)));
            }
        }
    });
}
