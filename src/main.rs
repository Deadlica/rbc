use std::env;
use std::fs;

mod gb;

fn main() {
    let rom_path = env::args().nth(1).expect("Usage: rbc <rom.gb>");
    let save_path = rom_path.replace(".gbc", ".sav").replace(".gb", ".sav");
    let rom: Vec<u8>= fs::read(rom_path).expect("Failed to read ROM");
    
    let mut gb = gb::Gb::new();
    gb.load_rom(rom);
    gb.load_save(&save_path);
    gb.run();
    gb.save_game(&save_path);
}
