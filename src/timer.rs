
/// Location of Divide register in memory
pub const DIVIDE_LOC: u16 = 0xFF04;
/// Location of TIMA in memory
pub const TIMA_LOC: u16 = 0xFF05;
/// Location of TMA in memory
pub const TMA_LOC: u16 = 0xFF06;
/// Location of TMC in mmemory
pub const TMC_LOC: u16 = 0xFF07;

#[derive(Default)]
/// Struct that handls the timer and all of its associated state
pub struct Timer {
    tima: u8,
    tma: u8,
    tmc: u8,
    /// Divide corresponds to the divide register
    divide: u8,

    time_elapsed: u32,
    divide_elapsed: u32,
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
            DIVIDE_LOC => self.divide = 0,
            TIMA_LOC => self.tima = val,
            TMA_LOC => self.tma = val,
            TMC_LOC => self.tmc = val,
            _ => unreachable!("Timer does not handle this memory")
        };
    }
   
}

const HZ_4069: u32 = 1024;
const HZ_262144: u32 = 16;
const HZ_65536: u32 = 64;
const HZ_16384: u32 =  256;

impl Timer {

    pub fn update_time(&mut self, ticks: u8) -> u8 {
        let mut ret = 0;

        self.divide_elapsed += ticks as u32;
        
        if self.divide_elapsed >= 255 {
            self.divide_elapsed = 0;
            self.divide = self.divide.wrapping_add(1);
        }
        
        // If timer enabled
        if (self.tmc & 0b0000_0100) == 0b0000_0100 {
            let frequency = match self.tmc & 0b0000_0011 {
                0b0000_0000 => HZ_4069,
                0b0000_0001 => HZ_262144,
                0b0000_0010 => HZ_65536,
                0b0000_0011 => HZ_16384,
                _ => unreachable!()
            };
            
            self.time_elapsed += ticks as u32;

            while self.time_elapsed >= frequency {

                if self.tima == 255 {
                    self.tima = self.tma;
                    ret |= 0b0000_0100;
                    
                } else {
                    self.tima += 1;
                }
                
                self.time_elapsed -= frequency;
            }
        }

        ret
    }
}
