/// Game Boy timer. Tracks DIV (free-running counter) and TIMA (configurable timer).
pub struct Timer {
    /// Internal 16-bit counter. Upper 8 bits are exposed as DIV (0xFF04).
    pub counter: u16,
    /// Timer counter. Increments at rate set by TAC, overflows trigger interrupt.
    pub tima: u8,
    /// Timer modulo. TIMA reloads to this value on overflow.
    pub tma: u8,
    /// Timer control. Bit 2 = enable, bits 1-0 = clock select.
    pub tac: u8,
}

impl Timer {
    /// Create a new timer in its initial state.
    pub fn new() -> Self {
        Timer {
            counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    /// Advance the timer by the given number of T-cycles.
    /// Returns true if TIMA overflowed (timer interrupt should fire).
    pub fn tick(&mut self, cycles: u8) -> bool {
        let old_counter = self.counter;
        self.counter = self.counter.wrapping_add(cycles as u16);
        if self.tac & (1 << 2) == 0 {
            return false;
        }
        let threshold = self.get_cycle_increment();
        if old_counter / threshold != self.counter / threshold {
            if self.tima == 255 { // Next add would cause overflow
                self.tima = self.tma;
                return true;
            }
            self.tima += 1; // We know that it can't overflow in this case.
        }
        return false
    }

    /// Return the TIMA increment threshold in T-cycles based on TAC clock select.
    fn get_cycle_increment(&self) -> u16 {
        let clock_select = self.tac & 3;
        // Cycles returned from Cpu::step are T-Cycles and therefore we return M-Cycle * 4
        match clock_select {
            0 => 256 * 4,
            1 => 4 * 4,
            2 => 16 * 4,
            _ => 64 * 4,
        }
    }
}
