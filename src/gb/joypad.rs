/// Game Boy button identifiers.
#[derive(Debug)]
pub enum JoypadKey {
    Right, Left, Up, Down,
    A, B, Start, Select,
}

/// Joypad state. Tracks which buttons are pressed (active low).
pub struct Joypad {
    select: u8,
    action: u8,
    direction: u8,
}

impl Joypad {
    /// Create a new joypad with all buttons released.
    pub fn new() -> Self {
        Joypad {
            select: 0,
            action: 0x0F,
            direction: 0x0F,
        }
    }

    /// Read the joypad register (0xFF00). Returns button state for the selected group.
    pub fn read(&self) -> u8 {
        if self.select & 0x10 == 0 { return self.direction; }
        if self.select & 0x20 == 0 { return self.action; }
        0x0F
    }

    /// Write to the joypad register (0xFF00). Selects button group to read.
    pub fn write(&mut self, byte: u8) {
        self.select = byte & 0x30;
    }

    /// Release a button (set bit high).
    pub fn key_up(&mut self, key: JoypadKey) {
        match key {
            JoypadKey::Right => self.direction |= 1 << 0,
            JoypadKey::Left => self.direction |= 1 << 1,
            JoypadKey::Up => self.direction |= 1 << 2,
            JoypadKey::Down => self.direction |= 1 << 3,
            JoypadKey::A => self.action |= 1 << 0,
            JoypadKey::B => self.action |= 1 << 1,
            JoypadKey::Select => self.action |= 1 << 2,
            JoypadKey::Start => self.action |= 1 << 3,
        }
    }

    /// Press a button (clear bit low).
    pub fn key_down(&mut self, key: JoypadKey) {
        match key {
            JoypadKey::Right => self.direction &= !(1 << 0),
            JoypadKey::Left => self.direction &= !(1 << 1),
            JoypadKey::Up => self.direction &= !(1 << 2),
            JoypadKey::Down => self.direction &= !(1 << 3),
            JoypadKey::A => self.action &= !(1 << 0),
            JoypadKey::B => self.action &= !(1 << 1),
            JoypadKey::Select => self.action &= !(1 << 2),
            JoypadKey::Start => self.action &= !(1 << 3),
        }
    }

    /// Reset all buttons to released state.
    pub fn reset(&mut self) {
        self.action = 0x0F;
        self.direction = 0x0F;
    }
}
