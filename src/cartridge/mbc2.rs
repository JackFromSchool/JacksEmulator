
pub struct Mbc2 {
    
}

impl Default for Mbc2 {
    fn default() -> Self {
       Self {} 
    }
}

impl super::MemController for Mbc2 {

    fn read(&self, index: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, index: u16, val: u8) {
        todo!()
    }

    fn load_rom(&mut self, bytes: Vec<u8>) {
        todo!()
    }
    
}
