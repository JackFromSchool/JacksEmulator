use super::MemController;

const RAM_SIZE: usize = 0x2000;

pub struct NoMbc {
    rom: Box<[u8]>,
    ram: Vec<u8>,
}

impl std::default::Default for NoMbc {
    fn default() -> Self {
        Self {
            rom: Box::new([0]),
            ram: Vec::new(),
        }
    }
}

impl MemController for NoMbc {
    
    fn read(&self, index: u16) -> u8 {
        match index {
            0x4000..=0x7FFF => self.rom[(index - 0x4000) as usize],
            0xA000..=0xBFFF => self.ram[(index - 0xA000) as usize],
            _ => unreachable!("{}", index),
        }
    }

    fn write(&mut self, index: u16, val: u8) {
        match index {
            0x4000..=0x7FFF => (),
            0xA000..=0xBFFF => self.ram[(index - 0xA000) as usize] = val,
            _ => unreachable!("Area in memory should not try to be accessed by this funtion")
        }
    }

    fn load_rom(&mut self, bytes: Vec<u8>) {
        let mut vec_rom = Vec::with_capacity(RAM_SIZE);
        
        for byte in bytes {
            vec_rom.push(byte);
        }

        if vec_rom.len() > 0x4000 {
            panic!("Rom Vector exceeds max size");
        }

        self.rom = vec_rom.as_slice().into();
    }
    
}
