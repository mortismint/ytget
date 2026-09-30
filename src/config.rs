use std::path::{Path, PathBuf};

fn config_file_path() -> PathBuf {
    let exe_path = std::env::current_exe().expect("Failed to get current executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Failed to get executable directory");
    exe_dir.join("yetget_config.txt")
}

pub fn load_output_dir() -> Option<PathBuf> {
    let contents = std::fs::read_to_string(config_file_path()).ok()?;
    let path = PathBuf::from(contents.trim());

    if path.is_dir() { Some(path) } else { None }
}

pub fn save_output_dir(path: &Path) {
    let _ = std::fs::write(config_file_path(), path.to_string_lossy().as_bytes());
}
