use eframe::egui;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::boxart;
use crate::config::Config;
use crate::library::Library;
use crate::gb::Gb;
use crate::gb::ppu::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::gb::joypad::JoypadKey;

/// Emulation speed multiplier.
#[derive(PartialEq, Clone, Copy)]
enum Speed {
    Normal,
    Double,
    Quad,
    Unlimited,
}

/// Top-level application state for the egui frontend.
pub struct App {
    gb: Option<Gb>,
    texture: Option<egui::TextureHandle>,
    paused: bool,
    speed: Speed,
    rom_path: Option<String>,
    save_path: Option<String>,
    volume: f32,
    muted: bool,
    library: Library,
    config: Config,
    session_start: Option<Instant>,
    toast: Option<(String, Instant)>,
    art_cache: HashMap<String, Option<egui::TextureHandle>>,
}

impl App {
    /// Create a new App with config loaded.
    pub fn new(config: Config) -> Self {
        let volume = config.volume;
        let muted = config.muted;
        App {
            gb: None,
            texture: None,
            paused: false,
            speed: Speed::Normal,
            rom_path: None,
            save_path: None,
            volume,
            muted,
            library: Library::load(),
            config,
            session_start: None,
            toast: None,
            art_cache: HashMap::new(),
        }
    }

    /// Load a ROM file and initialize the emulator.
    fn load_rom(&mut self, path: String) {
        // Save play time for previous game
        self.flush_play_time();

        let rom = fs::read(&path).expect("Failed to read ROM");
        let save_path = path.replace(".gbc", ".sav").replace(".gb", ".sav");

        let mut gb = Gb::new();
        gb.load_rom(rom);
        gb.load_save(&save_path);
        gb.set_volume(self.volume);
        gb.set_muted(self.muted);

        let boot_path = "cgb_boot.bin";
        if Path::new(boot_path).exists() {
            let boot = fs::read(boot_path).expect("Failed to read boot ROM");
            gb.load_boot_rom(boot);
        }

        self.gb = Some(gb);
        self.rom_path = Some(path.clone());
        self.save_path = Some(save_path);
        self.paused = false;
        self.session_start = Some(Instant::now());

        self.library.touch(&path);
    }

    /// Flush accumulated play time to library.
    fn flush_play_time(&mut self) {
        if let (Some(start), Some(path)) = (self.session_start.take(), &self.rom_path.clone()) {
            let secs = start.elapsed().as_secs();
            if secs > 0 {
                self.library.add_play_time(path, secs);
            }
        }
    }

    /// Run emulation for one frame.
    fn run_frame(&mut self) {
        let gb = match &mut self.gb {
            Some(gb) => gb,
            None => return,
        };

        let fast = self.speed != Speed::Normal;
        gb.set_skip_throttle(fast);

        // At 1x: run until one frame completes (audio throttle paces us).
        // At 2x/4x/Unlimited: run multiple frames, drop excess audio samples.
        let frames_to_run: u32 = match self.speed {
            Speed::Normal => 1,
            Speed::Double => 2,
            Speed::Quad => 4,
            Speed::Unlimited => 16,
        };

        for _ in 0..frames_to_run {
            loop {
                gb.step();
                if gb.frame_ready() { break; }
            }
        }
    }

    /// Poll egui input and update joypad.
    fn poll_input(&mut self, ctx: &egui::Context) {
        let gb = match &mut self.gb {
            Some(gb) => gb,
            None => return,
        };

        gb.reset_joypad();
        ctx.input(|i| {
            if i.key_down(egui::Key::ArrowRight) { gb.key_down(JoypadKey::Right); }
            if i.key_down(egui::Key::ArrowLeft) { gb.key_down(JoypadKey::Left); }
            if i.key_down(egui::Key::ArrowUp) { gb.key_down(JoypadKey::Up); }
            if i.key_down(egui::Key::ArrowDown) { gb.key_down(JoypadKey::Down); }
            if i.key_down(egui::Key::Z) { gb.key_down(JoypadKey::A); }
            if i.key_down(egui::Key::X) { gb.key_down(JoypadKey::B); }
            if i.key_down(egui::Key::Enter) { gb.key_down(JoypadKey::Start); }
            if i.key_down(egui::Key::Backspace) { gb.key_down(JoypadKey::Select); }
        });
    }

    /// Check for save state hotkeys.
    fn check_hotkeys(&mut self, ctx: &egui::Context) {
        let mut save_slot = None;
        let mut load_slot = None;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F1) { save_slot = Some(1); }
            if i.key_pressed(egui::Key::F2) { save_slot = Some(2); }
            if i.key_pressed(egui::Key::F3) { save_slot = Some(3); }
            if i.key_pressed(egui::Key::F4) { save_slot = Some(4); }
            if i.key_pressed(egui::Key::F5) { load_slot = Some(1); }
            if i.key_pressed(egui::Key::F6) { load_slot = Some(2); }
            if i.key_pressed(egui::Key::F7) { load_slot = Some(3); }
            if i.key_pressed(egui::Key::F8) { load_slot = Some(4); }
        });
        if let Some(slot) = save_slot { self.save_state(slot); }
        if let Some(slot) = load_slot { self.load_state(slot); }
    }

    /// Get the save state file path for a given slot.
    fn state_path(&self, slot: u8) -> Option<std::path::PathBuf> {
        let rom_path = self.rom_path.as_ref()?;
        let stem = Path::new(rom_path).file_stem()?.to_string_lossy();
        let dir = dirs::config_dir()?.join("rbc").join("states");
        Some(dir.join(format!("{stem}.slot{slot}.state")))
    }

    /// Save emulator state to a slot.
    fn save_state(&mut self, slot: u8) {
        if let (Some(gb), Some(path)) = (&self.gb, self.state_path(slot)) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(path, gb.save_state()).ok();
            self.toast = Some((format!("State saved to slot {slot}"), Instant::now()));
        }
    }

    /// Load emulator state from a slot.
    fn load_state(&mut self, slot: u8) {
        if let Some(path) = self.state_path(slot) {
            if let Ok(data) = fs::read(path) {
                if let Some(gb) = &mut self.gb {
                    gb.load_state(&data);
                    self.toast = Some((format!("State loaded from slot {slot}"), Instant::now()));
                }
            }
        }
    }

    /// Render the menu bar.
    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Game Boy ROMs", &["gb", "gbc"])
                            .pick_file()
                        {
                            self.load_rom(path.to_string_lossy().to_string());
                        }
                        ui.close_menu();
                    }
                    if self.gb.is_some() {
                        if ui.button("Close ROM").clicked() {
                            self.close_rom();
                            ui.close_menu();
                        }
                        if ui.button("Set Box Art...").clicked() {
                            if let Some(img_path) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg"])
                                .pick_file()
                            {
                                if let Some(rom_path) = &self.rom_path {
                                    let name = Path::new(rom_path).file_stem()
                                        .map(|s| s.to_string_lossy().to_string())
                                        .unwrap_or_default();
                                    boxart::set_custom_art(&name, &img_path.to_string_lossy());
                                    self.art_cache.remove(&name);
                                }
                            }
                            ui.close_menu();
                        }
                    }
                    if ui.button("Quit").clicked() {
                        self.shutdown();
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Emulation", |ui| {
                    if ui.button(if self.paused { "Resume" } else { "Pause" }).clicked() {
                        self.paused = !self.paused;
                        ui.close_menu();
                    }
                    if ui.button("Reset").clicked() {
                        if let Some(path) = self.rom_path.clone() {
                            self.load_rom(path);
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label("Speed:");
                    ui.radio_value(&mut self.speed, Speed::Normal, "1x");
                    ui.radio_value(&mut self.speed, Speed::Double, "2x");
                    ui.radio_value(&mut self.speed, Speed::Quad, "4x");
                    ui.radio_value(&mut self.speed, Speed::Unlimited, "Unlimited");
                });
                ui.menu_button("Audio", |ui| {
                    if ui.checkbox(&mut self.muted, "Mute").changed() {
                        if let Some(gb) = &mut self.gb {
                            gb.set_muted(self.muted);
                        }
                    }
                    ui.label("Volume:");
                    if ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0)).changed() {
                        if let Some(gb) = &mut self.gb {
                            gb.set_volume(self.volume);
                        }
                    }
                });
                if self.gb.is_some() {
                    ui.menu_button("Save State", |ui| {
                        for slot in 1..=4 {
                            if ui.button(format!("Save Slot {slot}")).clicked() {
                                self.save_state(slot);
                                ui.close_menu();
                            }
                        }
                        ui.separator();
                        for slot in 1..=4 {
                            if ui.button(format!("Load Slot {slot}")).clicked() {
                                self.load_state(slot);
                                ui.close_menu();
                            }
                        }
                    });
                }
            });
        });
    }

    /// Render the game viewport.
    fn viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(gb) = &self.gb {
                let framebuffer = gb.framebuffer();
                let pixels: Vec<egui::Color32> = framebuffer.iter().map(|&p| {
                    let r = ((p >> 16) & 0xFF) as u8;
                    let g = ((p >> 8) & 0xFF) as u8;
                    let b = (p & 0xFF) as u8;
                    egui::Color32::from_rgb(r, g, b)
                }).collect();

                let image = egui::ColorImage {
                    size: [SCREEN_WIDTH, SCREEN_HEIGHT],
                    pixels,
                };

                let texture = self.texture.get_or_insert_with(|| {
                    ctx.load_texture("gb_screen", image.clone(), egui::TextureOptions::NEAREST)
                });
                texture.set(image, egui::TextureOptions::NEAREST);

                let available = ui.available_size();
                let aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
                let size = if available.x / available.y > aspect {
                    egui::vec2(available.y * aspect, available.y)
                } else {
                    egui::vec2(available.x, available.x / aspect)
                };

                ui.centered_and_justified(|ui| {
                    ui.image(egui::load::SizedTexture::new(texture.id(), size));
                });
            } else {
                self.library_view(ui);
            }
        });
    }

    /// Render the library home screen.
    fn library_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Library");
        ui.separator();

        if self.library.entries.is_empty() {
            ui.label("No games played yet. Use File > Open ROM to get started.");
            return;
        }

        let mut launch_path = None;
        let mut remove_path = None;
        let entries: Vec<_> = self.library.entries.clone();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &entries {
                ui.horizontal(|ui| {
                    // Box art thumbnail
                    let art_size = egui::vec2(48.0, 48.0);
                    if let Some(tex) = self.get_art_texture(ui.ctx(), &entry.name) {
                        ui.image(egui::load::SizedTexture::new(tex.id(), art_size));
                    } else {
                        ui.allocate_space(art_size);
                    }

                    if ui.button("▶").clicked() {
                        launch_path = Some(entry.path.clone());
                    }
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&entry.name).strong());
                        let hours = entry.play_time_secs / 3600;
                        let mins = (entry.play_time_secs % 3600) / 60;
                        ui.label(format!("{}  •  {hours}h {mins}m", Self::format_timestamp(entry.last_played)));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("x").clicked() {
                            remove_path = Some(entry.path.clone());
                        }
                    });
                });
                ui.separator();
            }
        });

        if let Some(path) = remove_path {
            self.library.remove(&path);
        }
        if let Some(path) = launch_path {
            self.load_rom(path);
        }
    }

    /// Get or load a box art texture for a game.
    fn get_art_texture(&mut self, ctx: &egui::Context, game_name: &str) -> Option<egui::TextureHandle> {
        if !self.art_cache.contains_key(game_name) {
            let tex = boxart::get_art(game_name).and_then(|path| {
                let data = fs::read(&path).ok()?;
                let img = image::load_from_memory(&data).ok()?.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let pixels = img.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                Some(ctx.load_texture(game_name, color_image, egui::TextureOptions::LINEAR))
            });
            self.art_cache.insert(game_name.to_string(), tex);
        }
        self.art_cache.get(game_name).cloned().flatten()
    }

    /// Format a unix timestamp as a human-readable date.
    fn format_timestamp(ts: u64) -> String {
        if ts == 0 { return "Never".to_string(); }
        let secs_per_day = 86400;
        let days_since_epoch = ts / secs_per_day;
        // Simple date calculation (good enough for display)
        let mut y = 1970i64;
        let mut remaining = days_since_epoch as i64;
        loop {
            let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
            if remaining < days_in_year { break; }
            remaining -= days_in_year;
            y += 1;
        }
        let months = [31, if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut m = 0;
        for days in months {
            if remaining < days { break; }
            remaining -= days;
            m += 1;
        }
        format!("{y}-{:02}-{:02}", m + 1, remaining + 1)
    }

    /// Close the current ROM and return to library.
    fn close_rom(&mut self) {
        if let (Some(gb), Some(save_path)) = (&self.gb, &self.save_path) {
            gb.save_game(save_path);
        }
        self.flush_play_time();
        self.gb = None;
        self.rom_path = None;
        self.save_path = None;
        self.texture = None;
    }

    /// Save everything before exit.
    fn shutdown(&mut self) {
        if let (Some(gb), Some(save_path)) = (&self.gb, &self.save_path) {
            gb.save_game(save_path);
        }
        self.flush_play_time();
        self.config.volume = self.volume;
        self.config.muted = self.muted;
        self.config.save();
    }

    /// Render a temporary toast notification.
    fn render_toast(&mut self, ctx: &egui::Context) {
        const TOAST_DURATION_SECS: f32 = 1.5;
        let toast = match &self.toast {
            Some((msg, time)) if time.elapsed().as_secs_f32() < TOAST_DURATION_SECS => msg.clone(),
            Some(_) => { self.toast = None; return; }
            None => return,
        };

        egui::Area::new(egui::Id::new("toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -20.0])
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .fill(egui::Color32::from_rgba_premultiplied(40, 40, 40, 220))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(toast).color(egui::Color32::WHITE).size(16.0));
                    });
            });
    }
}

impl eframe::App for App {
    /// Main frame update — runs menu, emulation, and rendering.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Track window size for config persistence
        let size = ctx.input(|i| i.screen_rect().size());
        self.config.window_width = size.x;
        self.config.window_height = size.y;

        self.menu_bar(ctx);

        if self.gb.is_some() && !self.paused {
            self.poll_input(ctx);
            self.run_frame();
        }

        self.check_hotkeys(ctx);
        self.viewport(ctx);
        self.render_toast(ctx);

        if self.gb.is_some() && !self.paused {
            match self.speed {
                Speed::Unlimited => ctx.request_repaint(),
                _ => ctx.request_repaint_after(std::time::Duration::from_micros(16_667)),
            }
        }
    }

    /// Called when the window is closed — persists all state.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.shutdown();
    }
}
