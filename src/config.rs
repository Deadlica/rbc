use std::fs;
use std::path::PathBuf;

/// Persistent application configuration.
pub struct Config {
    pub volume: f32,
    pub muted: bool,
    pub window_width: f32,
    pub window_height: f32,
}

impl Config {
    /// Load config from disk, or return defaults.
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(contents) = fs::read_to_string(&path) {
            Self::parse(&contents)
        } else {
            Self::default()
        }
    }

    /// Save config to disk.
    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let contents = format!(
            "volume={}\nmuted={}\nwindow_width={}\nwindow_height={}\n",
            self.volume, self.muted, self.window_width, self.window_height
        );
        fs::write(path, contents).ok();
    }

    /// Config file path.
    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rbc")
            .join("config.txt")
    }

    /// Parse config from key=value text.
    fn parse(text: &str) -> Self {
        let mut cfg = Self::default();
        for line in text.lines() {
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().unwrap_or("").trim();
            match key {
                "volume" => cfg.volume = val.parse().unwrap_or(cfg.volume),
                "muted" => cfg.muted = val == "true",
                "window_width" => cfg.window_width = val.parse().unwrap_or(cfg.window_width),
                "window_height" => cfg.window_height = val.parse().unwrap_or(cfg.window_height),
                _ => {}
            }
        }
        cfg
    }

    /// Default configuration values.
    fn default() -> Self {
        Config {
            volume: 1.0,
            muted: false,
            window_width: 640.0,
            window_height: 606.0,
        }
    }
}
