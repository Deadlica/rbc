use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single ROM entry in the library.
#[derive(Clone)]
pub struct LibraryEntry {
    pub path: String,
    pub name: String,
    pub last_played: u64,
    pub play_time_secs: u64,
    pub art_path: Option<String>,
}

/// Manages the collection of previously played ROMs.
pub struct Library {
    pub entries: Vec<LibraryEntry>,
}

impl Library {
    /// Load library from disk, or return empty.
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(contents) = fs::read_to_string(&path) {
            Self::parse(&contents)
        } else {
            Library { entries: Vec::new() }
        }
    }

    /// Save library to disk.
    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let mut out = String::new();
        for entry in &self.entries {
            out.push_str(&format!(
                "{}|{}|{}|{}|{}\n",
                entry.path,
                entry.name,
                entry.last_played,
                entry.play_time_secs,
                entry.art_path.as_deref().unwrap_or("")
            ));
        }
        fs::write(path, out).ok();
    }

    /// Add or update a ROM entry. Sets last_played to now.
    pub fn touch(&mut self, rom_path: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let name = std::path::Path::new(rom_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| rom_path.to_string());

        if let Some(entry) = self.entries.iter_mut().find(|e| e.name == name) {
            entry.last_played = now;
            entry.path = rom_path.to_string();
        } else {
            self.entries.push(LibraryEntry {
                path: rom_path.to_string(),
                name,
                last_played: now,
                play_time_secs: 0,
                art_path: None,
            });
        }
        self.save();
    }

    /// Add play time to a ROM entry.
    pub fn add_play_time(&mut self, rom_path: &str, secs: u64) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.path == rom_path) {
            entry.play_time_secs += secs;
        }
        self.save();
    }

    /// Set custom art path for a ROM entry.
    pub fn set_art(&mut self, rom_path: &str, art_path: String) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.path == rom_path) {
            entry.art_path = Some(art_path);
        }
        self.save();
    }

    /// Remove a ROM entry from the library (does not delete save data).
    pub fn remove(&mut self, rom_path: &str) {
        self.entries.retain(|e| e.path != rom_path);
        self.save();
    }

    /// Library file path.
    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rbc")
            .join("library.txt")
    }

    /// Parse library from pipe-delimited text.
    fn parse(text: &str) -> Self {
        let entries = text.lines().filter_map(|line| {
            let parts: Vec<&str> = line.splitn(5, '|').collect();
            if parts.len() < 4 { return None; }
            Some(LibraryEntry {
                path: parts[0].to_string(),
                name: parts[1].to_string(),
                last_played: parts[2].parse().unwrap_or(0),
                play_time_secs: parts[3].parse().unwrap_or(0),
                art_path: parts.get(4).and_then(|s| {
                    if s.is_empty() { None } else { Some(s.to_string()) }
                }),
            })
        }).collect();
        Library { entries }
    }
}
