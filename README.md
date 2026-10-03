# ytget

A lightweight desktop YouTube downloader built with Rust. `ytget` provides a graphical interface for [yt-dlp](https://github.com/yt-dlp/yt-dlp), with options for video and audio formats, playlists, and download location.

> **Note:** `ytget` is a front end for `yt-dlp`; it does not include or install `yt-dlp` for you.

## Features

- Download from a video or playlist URL.
- Search YouTube by entering a search term, with suggestions as you type.
- Choose the original format, WebM or MP4 video, or MP3, M4A, FLAC, WAV, or Opus audio.
- Opt in to downloading a full playlist.
- Choose an output folder and see download progress in the app.
- Remembers the selected output folder between runs.

## Platform support

`ytget` currently works on Windows and Linux. macOS support has not been tested, so it is unknown at this time.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) with Cargo, to build from source.
- [yt-dlp](https://github.com/yt-dlp/yt-dlp#installation), available on your `PATH` or placed beside the `ytget` executable.
- [FFmpeg](https://ffmpeg.org/download.html) available on your `PATH` for audio extraction/conversion and any format merging that requires it. `yt-dlp` uses FFmpeg for these operations.

The app looks for `yt-dlp.exe` on Windows and `yt-dlp` on other platforms. It checks directories on `PATH` first, then the directory containing the app executable.

## Build and run

From the project directory:

```sh
cargo run
```

To build an optimized executable:

```sh
cargo build --release
```

The resulting executable is in `target/release/` (`ytget.exe` on Windows). Make sure `yt-dlp` and, where needed, FFmpeg are installed and discoverable as described above before downloading.

## Using the app

1. Enter a video or playlist URL, or type a search phrase and choose a suggestion (or download using the phrase).
2. Choose an output format. **Original** leaves format selection to `yt-dlp`.
3. Enable **Download full playlist** if you intend to download the entire playlist. Otherwise, playlist URLs are limited to the individual video where applicable.
4. Use **Browse...** to choose a save folder. If no folder is selected, `yt-dlp` uses its default output location.
5. Click **Download** and follow the progress and status shown in the window.

The selected output folder is stored in a `ytget_config.txt` file beside the application executable. The app needs permission to write that file to remember the setting.

Search suggestions are retrieved from Google’s YouTube search suggestion endpoint. A search term is passed to `yt-dlp` as a YouTube search; a URL beginning with `http://` or `https://` is passed directly to `yt-dlp`.

## Third-party software and usage

`ytget` relies on [yt-dlp](https://github.com/yt-dlp/yt-dlp) and optionally [FFmpeg](https://ffmpeg.org/). These tools are separate from this project and are subject to their own licenses and terms. Please respect the rights of content owners and the terms and applicable laws governing the services and content you access.

## License

This project is licensed under the [MIT License](LICENSE).

## AI assistance

AI tools were used to help write this README and assist with minor, repetitive coding tasks. The project is reviewed and maintained by its author.
