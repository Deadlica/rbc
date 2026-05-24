use std::env;
use std::fs;
use std::path::Path;

mod gb;

fn main() {
    let rom_path = env::args().nth(1).expect("Usage: rbc <rom.gb>");
    let save_path = rom_path.replace(".gbc", ".sav").replace(".gb", ".sav");
    let rom: Vec<u8> = fs::read(&rom_path).expect("Failed to read ROM");
    
    let mut gb = gb::Gb::new();
    gb.load_rom(rom);
    gb.load_save(&save_path);

    // Load boot ROM if present next to the executable or in current dir
    let boot_path = "cgb_boot.bin";
    if Path::new(boot_path).exists() {
        let boot = fs::read(boot_path).expect("Failed to read boot ROM");
        gb.load_boot_rom(boot);
    }

    gb.run();
    gb.save_game(&save_path);
}
