mod gb;

fn main() {
    let mut cpu = gb::cpu::Cpu::new();
    loop {
        cpu.step();
    }
}
