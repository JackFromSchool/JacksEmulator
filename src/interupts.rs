
const VBLANK_LOC: usize = 0x40;
const LCD_LOC: usize = 0x48;
const TIMER_LOC: usize = 0x50;
const JOYPAD_LOC: usize = 0x60;

const IE_LOC: u16 = 0xFFFF;
pub const IF_LOC: u16 = 0xFF0F;

#[derive(Default)]
/// Represents the state related to interupts
pub struct InteruptState {
    /// # Interupt Enabled Register
    ie: u8,
    /// # Interupt Request Register
    if_r: u8,
    /// # Master Interupt Enable Switch
    /// If this is true interupts are Enabled
    pub master: bool,
}

impl crate::mmu::Memory for InteruptState {
    
    fn handle_read(&self, index: u16) -> u8 {
        match index {
            IE_LOC => self.ie,
            IF_LOC => self.if_r,
            _ => unreachable!("InteruptState does not handle this memory")
        }    
    }

    fn handle_write(&mut self, index: u16, val: u8) {
        match index {
            IE_LOC => self.ie = val,
            IF_LOC => self.if_r = val,
            _ => unreachable!("InteruptState does not handle this memory")
        }
    }
    
}
