
/// Location of Divide register in memory
pub const DIVIDE_LOC: u16 = 0xFF04;
/// Location of TIMA in memory
pub const TIMA_LOC: u16 = 0xFF05;
/// Location of TMA in memory
pub const TMA_LOC: u16 = 0xFF06;
/// Location of TMC in mmemory
pub const TMC_LOC: u16 = 0xFF07;

macro_rules! timer_stuff(
    () => {
        0xFF04..=0xFF07
    }
);

#[derive(Default)]
/// Struct that handls the timer and all of its associated state
pub struct Timer {
    tima: u8,
    tma: u8,
    tmc: u8,
    /// Divide corresponds to the divide register
    divide: u8,
}

impl crate::mmu::Memory for Timer {

    fn handle_read(&self, index: u16) -> u8 {
        match index {
            DIVIDE_LOC => self.divide,
            TIMA_LOC => self.tima,
            TMA_LOC => self.tma,
            TMC_LOC => self.tmc,
            _ => unreachable!("Timer does not handle this memory")
        }
    }

    fn handle_write(&mut self, index: u16, val: u8) {
        match index {
            DIVIDE_LOC => self.divide = val,
            TIMA_LOC => self.tima = val,
            TMA_LOC => self.tma = val,
            TMC_LOC => self.tmc = val,
            _ => unreachable!("Timer does not handle this memory")
        };
    }
   
}
