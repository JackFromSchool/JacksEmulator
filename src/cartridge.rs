mod mbc1;
mod mbc2;
mod default;

const ROM_BANK_SIZE: usize = 0x4000;

const ROM_BANK_MODE_LOC: usize = 0x147;
const RAM_AMOUNT_LOC: usize = 0x148;

pub trait MemController {
    fn read(&self, index: u16) -> u8;
    fn write(&mut self, index: u16, val: u8);
    fn load_rom(&mut self, bytes: Vec<u8>);
}

/// Handles cartridge related state
pub struct Cartridge {
    // Memory Related State
    /// Fixed ROM Memory; No Banking
    fixed_rom: [u8; ROM_BANK_SIZE],
    boot_rom: [u8; 0x100],
    /// ROM Bank Controler
    controller: Box<dyn MemController>,

    pub booting: bool,
}

impl std::default::Default for Cartridge {
    fn default() -> Self {
        let bytes = include_bytes!("../roms/dmg_boot.bin");
        let mut boot_rom = [0; 0x100];
        for (i, byte) in bytes.iter().enumerate() {
            boot_rom[i] = *byte;
        }
        
        Self {
            fixed_rom: [0; ROM_BANK_SIZE],
            boot_rom,
            controller: Box::new(default::NoMbc::default()),
            booting: true,
        }
    }
    
}

impl Cartridge {
    
    // TODO: Implement ROM banking
    /// Loads ROM data onto the cartridge rom and memory controller
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (i, byte) in rom.iter().enumerate() {
            if i < ROM_BANK_SIZE {
                self.fixed_rom[i] = *byte;
            } else {
                break;
            }
        }

        match self.fixed_rom[ROM_BANK_MODE_LOC] {
            0..=1 => self.controller = Box::new(self::default::NoMbc::default()),
            1..=3 => self.controller = Box::new(self::mbc1::Mbc1::default()),
            5..=6 => self.controller = Box::new(self::mbc2::Mbc2::default()),
            _ => unreachable!(),
        }
        
        self.controller.load_rom(
            rom.iter().enumerate().filter(|(i, _)| *i >= ROM_BANK_SIZE).map(|(_, x)| *x).collect()
        );
    }
    
}

impl crate::mmu::Memory for Cartridge {
    
    fn handle_write(&mut self, index: u16, val: u8) {
        if (0..ROM_BANK_SIZE).contains(&(index as usize)) {
            ()
        } else {
            self.controller.write(index, val);
        }
    }
    
    fn handle_read(&self, index: u16) -> u8 {
        if (0..0x100).contains(&index) && self.booting {
            return self.boot_rom[index as usize];
        }
        
        if (0..ROM_BANK_SIZE).contains(&(index as usize)) {
            self.fixed_rom[index as usize]
        } else {
            self.controller.read(index)
        }
    }
    
}
