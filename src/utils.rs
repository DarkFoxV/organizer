use std::env;
use std::path::PathBuf;

pub fn get_exe_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Returns the base path for config assets depending on the build mode
pub fn get_assets_path() -> PathBuf {
    if cfg!(debug_assertions) {
        // Development mode
        PathBuf::from("./src/config/")
    } else {
        // Release mode: use path relative to the executable
        let exe_dir = get_exe_dir();
        exe_dir.join("config")
    }
}

pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}