use std::fs;
use std::path::PathBuf;

/// Persistent application configuration.
pub struct Config {
    pub volume: f32,
    pub muted: bool,
    pub window_width: f32,
    pub window_height: f32,
    pub window_x: Option<f32>,
    pub window_y: Option<f32>,
    pub dark_mode: bool,
    pub key_right: String,
    pub key_left: String,
    pub key_up: String,
    pub key_down: String,
    pub key_a: String,
    pub key_b: String,
    pub key_start: String,
    pub key_select: String,
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
            "volume={}\nmuted={}\nwindow_width={}\nwindow_height={}\nwindow_x={}\nwindow_y={}\ndark_mode={}\nkey_right={}\nkey_left={}\nkey_up={}\nkey_down={}\nkey_a={}\nkey_b={}\nkey_start={}\nkey_select={}\n",
            self.volume, self.muted, self.window_width, self.window_height,
            self.window_x.unwrap_or(0.0), self.window_y.unwrap_or(0.0),
            self.dark_mode,
            self.key_right, self.key_left, self.key_up, self.key_down,
            self.key_a, self.key_b, self.key_start, self.key_select
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
                "window_x" => cfg.window_x = val.parse().ok(),
                "window_y" => cfg.window_y = val.parse().ok(),
                "dark_mode" => cfg.dark_mode = val == "true",
                "key_right" => cfg.key_right = val.to_string(),
                "key_left" => cfg.key_left = val.to_string(),
                "key_up" => cfg.key_up = val.to_string(),
                "key_down" => cfg.key_down = val.to_string(),
                "key_a" => cfg.key_a = val.to_string(),
                "key_b" => cfg.key_b = val.to_string(),
                "key_start" => cfg.key_start = val.to_string(),
                "key_select" => cfg.key_select = val.to_string(),
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
            window_x: None,
            window_y: None,
            dark_mode: true,
            key_right: "ArrowRight".to_string(),
            key_left: "ArrowLeft".to_string(),
            key_up: "ArrowUp".to_string(),
            key_down: "ArrowDown".to_string(),
            key_a: "Z".to_string(),
            key_b: "X".to_string(),
            key_start: "Enter".to_string(),
            key_select: "Backspace".to_string(),
        }
    }
}

/// Convert a key name string to an egui::Key.
pub fn parse_key(name: &str) -> Option<eframe::egui::Key> {
    use eframe::egui::Key;
    match name {
        "ArrowRight" => Some(Key::ArrowRight),
        "ArrowLeft" => Some(Key::ArrowLeft),
        "ArrowUp" => Some(Key::ArrowUp),
        "ArrowDown" => Some(Key::ArrowDown),
        "Enter" => Some(Key::Enter),
        "Backspace" => Some(Key::Backspace),
        "Space" => Some(Key::Space),
        "Tab" => Some(Key::Tab),
        _ if name.len() == 1 => {
            let c = name.chars().next()?;
            Key::from_name(&c.to_uppercase().to_string())
        }
        _ => Key::from_name(name),
    }
}
