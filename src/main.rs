use eframe::egui;

mod gb;
mod ui;
mod config;
mod library;
mod boxart;

/// Entry point — loads config and launches the egui application.
fn main() -> eframe::Result {
    let cfg = config::Config::load();

    // Generate a simple app icon (32x32 purple "RBC" badge)
    let icon = generate_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([cfg.window_width, cfg.window_height])
            .with_title("RBC — RustBoy Color")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "RBC",
        options,
        Box::new(|_cc| Ok(Box::new(ui::App::new(cfg)))),
    )
}

/// Generate a 32x32 RGBA icon with a Game Boy-inspired design.
fn generate_icon() -> egui::IconData {
    const SIZE: usize = 32;
    let mut pixels = vec![0u8; SIZE * SIZE * 4];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = (y * SIZE + x) * 4;
            // Purple rounded rectangle background
            let in_border = x >= 2 && x < SIZE - 2 && y >= 2 && y < SIZE - 2;
            if in_border {
                // Dark purple background
                pixels[i] = 88;      // R
                pixels[i + 1] = 50;  // G
                pixels[i + 2] = 168; // B
                pixels[i + 3] = 255; // A

                // Light green "screen" area in upper portion
                if x >= 6 && x < SIZE - 6 && y >= 5 && y < 18 {
                    pixels[i] = 144;
                    pixels[i + 1] = 200;
                    pixels[i + 2] = 80;
                    pixels[i + 3] = 255;
                }
            }
        }
    }

    egui::IconData {
        rgba: pixels,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}
