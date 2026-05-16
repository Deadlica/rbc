use minifb::{Window, WindowOptions};
use super::ppu::{SCREEN_WIDTH, SCREEN_HEIGHT};

const SCALE: usize = 4;

pub struct Display {
    window: Window,
}

impl Display {
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

    pub fn update(&mut self, framebuffer: &[u32]) {
        self.window.update_with_buffer(framebuffer, SCREEN_WIDTH, SCREEN_HEIGHT).unwrap();
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }
}
