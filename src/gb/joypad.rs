#[derive(Debug)]
pub enum JoypadKey {
    Right, Left, Up, Down,
    A, B, Start, Select,
}

pub struct Joypad {
    select: u8,
    action: u8,
    direction: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            select: 0,
            action: 0x0F,
            direction: 0x0F,
        }
    }

    pub fn read(&self) -> u8 {
        if self.select & 0x10 == 0 { return self.direction; }
        if self.select & 0x20 == 0 { return self.action; }
        0x0F
    }

    pub fn write(&mut self, byte: u8) {
        self.select = byte & 0x30;
    }

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


    pub fn key_down(&mut self, key: JoypadKey) {
        println!("Pressed key: {:?}", key);
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

    pub fn reset(&mut self) {
        self.action = 0x0F;
        self.direction = 0x0F;
    }
}
