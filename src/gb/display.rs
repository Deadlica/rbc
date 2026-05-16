use minifb::{Window, WindowOptions};
use super::ppu::{SCREEN_WIDTH, SCREEN_HEIGHT};

const SCALE: usize = 4;

/// Handles the emulator window and presents framebuffer data to the screen.
pub struct Display {
    window: Window,
}

impl Display {
    /// Create a new display window scaled up from the native GB resolution.
    pub fn new() -> Self {
        let window = Window::new(
            "RBC",
            SCREEN_WIDTH * SCALE,
            SCREEN_HEIGHT * SCALE,
            WindowOptions::default(),
        ).expect("Faield to create window.");
        Display {
            window
        }
    }

    /// Push a completed framebuffer to the window.
    pub fn update(&mut self, framebuffer: &[u32]) {
        self.window.update_with_buffer(framebuffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    }

    /// Returns true if the window is still open (user hasn't closed it).
    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }
}
