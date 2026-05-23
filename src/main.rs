use std::env;
use std::fs;

mod gb;

fn main() {
    let rom_path = env::args().nth(1).expect("Usage: rbc <rom.gb>");
    let rom: Vec<u8>= fs::read(rom_path).expect("Failed to read ROM");
    
    let mut gb = gb::Gb::new();
    gb.load_rom(rom);
    gb.run();
}
