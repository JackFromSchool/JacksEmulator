use JEmulator::dissasembler::Dissasembler;
use JEmulator::cpu::Cpu;

use std::time::Instant;

const MAX_CYCLES: u64 = 69905;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::new().filter_or("", "info")).init();
    
    let d = Dissasembler::new().unwrap();
    
    let mut cpu = Cpu::from_rom(include_bytes!("../roms/drMario.gb").to_vec());

    loop {
        
        let start = Instant::now();
        let mut cycles = 0;

        while cycles < MAX_CYCLES {
            let tick_cycles = cpu.tick(&d);
            cycles += tick_cycles as u64;

            cpu.mmu.tick(tick_cycles);
        }

        'here: loop {
            if start.elapsed().as_nanos() >= 16_666_667 {
                break 'here;
            }
        }
        
    }
    
}
