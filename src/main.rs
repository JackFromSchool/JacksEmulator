use JEmulator::dissasembler::Dissasembler;
use JEmulator::cpu::Cpu;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::new().filter_or("", "debug")).init();
    
    let d = Dissasembler::new().unwrap();
    
    let mut cpu = Cpu::from_rom(include_bytes!("../roms/drMario.gb").to_vec());

    loop {
        cpu.tick(&d);
    }
}
