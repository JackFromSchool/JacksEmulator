pub struct Mbc1 {
    current_rom: Box<[u8]>,
    current_ram: Vec<u8>,
    rom_bank: Vec<Box<[u8]>>,
    ram_bank: Vec<Vec<u8>>,
    rom_index: usize,
    ram_index: usize,
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self {
            current_rom: Box::new([0]),
            current_ram: Vec::new(),
            rom_bank: Vec::new(),
            ram_bank: Vec::new(),
            ..Default::default()
        }
    }
}

impl super::MemController for Mbc1 {

    fn write(&mut self, index: u16, val: u8) {
        todo!()
    }

    fn read(&self, index: u16) -> u8 {
        todo!()
    }

    fn load_rom(&mut self, bytes: Vec<u8>) {
        todo!()
    }
    
}
