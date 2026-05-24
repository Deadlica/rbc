# RBC — RustBoy Color

A Game Boy Color emulator written in Rust.

[Documentation](https://deadlica.github.io/rbc/)

## Usage

```
cargo run --release -- <rom.gb>
```

Place `cgb_boot.bin` in the working directory for the Nintendo logo animation on startup.

## Features

- Full SM83 CPU (all opcodes, CB prefix, interrupts, HALT, EI delay)
- PPU with background, window, and sprite rendering
- CGB color palettes, VRAM banking, tile attributes
- MBC1, MBC3 (with RTC), and MBC5 cartridge support
- 4-channel audio (pulse, wave, noise) with stereo output
- Joypad input (arrow keys, Z/X/Enter/Backspace)
- Save files (.sav) persisted to disk
- Double speed mode
- HDMA (VRAM DMA)
- WRAM banking
- PPU mode transitions and STAT interrupt
- Boot ROM support
- Frame timing synced via audio output

## Controls

| Key       | Button |
|-----------|--------|
| Arrows    | D-pad  |
| Z         | A      |
| X         | B      |
| Enter     | Start  |
| Backspace | Select |

## Building

```
cargo build --release
```

Requires `libasound2-dev` on Linux for audio support.
