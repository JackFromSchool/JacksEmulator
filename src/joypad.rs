pub const JOYPAD_REG_LOC: u16 = 0xFF00;

#[derive(Default)]
/// Represents the state related to the Joypad while also handling events
pub struct Joypad {
    /// Corelates to the register in memory mapped to the joypad IO
    joypad_reg: u8
}

impl crate::mmu::Memory for Joypad {
    
    fn handle_write(&mut self, index: u16, val: u8) {
        if index != JOYPAD_REG_LOC {
            unreachable!("Joypad does not manage this memory");
        } else {
            self.joypad_reg = self.joypad_reg & 0b0000_1111 + val & 0b1111_0000;
        }
    }

    fn handle_read(&self, index: u16) -> u8 {
        if index != JOYPAD_REG_LOC {
            unreachable!("Joypad does not manage this memory")
        } else {
            self.joypad_reg
        }
    }
    
}
