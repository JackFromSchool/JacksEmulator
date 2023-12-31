use crate::util::{ BitOperations, le_combine };
use crate::gpu::ColorPixel;
use std::sync::{ Arc, Mutex };

/*
 *  Memory map from nocash-s pandocs:
 *  0000-3FFF 16KB ROM Bank 00 (in cartridge, fixed at bank 00)
 *  4000-7FFF 16KB ROM Bank 01..NN (in cartridge, switchable bank number)
 *  8000-9FFF 8KB Video RAM (VRAM) (switchable bank 0-1 in CGB Mode)
 *  A000-BFFF 8KB External RAM (in cartridge, switchable bank, if any)
 *  C000-CFFF 4KB Work RAM Bank 0 (WRAM)
 *  D000-DFFF 4KB Work RAM Bank 1 (WRAM) (switchable bank 1-7 in CGB Mode)
 *  E000-FDFF Same as C000-DDFF (ECHO) (typically not used)
 *  FE00-FE9F Sprite Attribute Table (OAM)
 *  FEA0-FEFF Not Usable
 *  FF00-FF7F I/O Ports
 *  FF80-FFFE High RAM (HRAM)
 *  FFFF Interrupt Enable Register
 */

pub const ROM_START: u16 = 0x0000;
pub const ROM_END: u16 = 0x7FFF;
pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const CARTRIDGE_RAM_START: u16 = 0xA000;
pub const CARTRIDGE_RAM_END: u16 = 0xBFFF;
pub const WORK_RAM_START: u16 = 0xC000;
pub const WORK_RAM_END: u16 = 0xDFFF;
pub const ECHO_RAM_START: u16 = 0xE000;
pub const ECHO_RAM_END: u16 = 0xFDFF;
pub const OAM_START: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const UNUSABLE_START: u16 = 0xFEA0;
pub const UNUSABLE_END: u16 = 0xFEFF;
pub const IO_START: u16 = 0xFF00;
pub const IO_END: u16 = 0xFF7F;
pub const HRAM_START: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const INTERRUPT_REG: u16 = 0xFFFF;

macro_rules! timer_stuff(
    () => {
        0xFF04..=0xFF07
    }
);

macro_rules! apu_stuff(
    () => {
        0xFF24..=0xFF26 | 0xFF10..=0xFF14 | 0xFF16..= 0xFF19 | 0xFF1A..=0xFF1E | 0xFF30..=0xFF3F | 0xFF20..=0xFF23
    }
);

macro_rules! gpu_stuff(
    () => {
        0xFF40..=0xFF49 | 0xFF4A | 0xFF4B
    }
);

const WRAM_SIZE: usize = 0x2000;
const HRAM_SIZE: usize = 0x7F;

pub trait Memory {
    fn handle_write(&mut self, index: u16, val: u8);
    fn handle_read(&self, index: u16) -> u8;
}

pub struct MMU {
    gpu: crate::gpu::GPU,
    timer: crate::timer::Timer,
    pub interupt: crate::interupts::InteruptState,
    pub joypad: crate::joypad::Joypad,
    apu: crate::apu::APU,
    cartridge: crate::cartridge::Cartridge,
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    serial: char,
}

impl MMU {
    
    /// Returns an instance of MMU with no default values instantiated
    /// This *SHOULD NOT* be used without instantiating defaults
    fn empty() -> Self {
        use crate::{gpu::GPU, timer::Timer, interupts::InteruptState, joypad::Joypad, cartridge::Cartridge, apu::APU};
        Self {
            gpu: GPU::default(),
            timer: Timer::default(),
            interupt: InteruptState::default(),
            joypad: Joypad::default(),
            apu: APU::default(),
            cartridge: Cartridge::default(),
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            serial: ' ',
        }
    }
    
    /// Creates an instance of MMU with all default values initialized
    pub fn new(rom: Vec<u8>) -> Self {
        let mut mmu = Self::empty();

        mmu.cartridge.load_rom(rom);
        
        mmu.write_8(0xFF05, 0x00);
        mmu.write_8(0xFF06, 0x00);
        mmu.write_8(0xFF07, 0x00);
        mmu.write_8(0xFF10, 0x80);
        mmu.write_8(0xFF11, 0xBF);
        mmu.write_8(0xFF12, 0xF3);
        mmu.write_8(0xFF14, 0xBF);
        mmu.write_8(0xFF16, 0x3F);
        mmu.write_8(0xFF17, 0x00);
        mmu.write_8(0xFF19, 0xBF);
        mmu.write_8(0xFF1A, 0x7F);
        mmu.write_8(0xFF1B, 0xFF);
        mmu.write_8(0xFF1C, 0x9F);
        mmu.write_8(0xFF1E, 0xBF);
        mmu.write_8(0xFF20, 0xFF);
        mmu.write_8(0xFF21, 0x00);
        mmu.write_8(0xFF22, 0x00);
        mmu.write_8(0xFF23, 0xBF);
        mmu.write_8(0xFF24, 0x77);
        mmu.write_8(0xFF25, 0xF3);
        mmu.write_8(0xFF26, 0xF1);
        mmu.write_8(0xFF40, 0x91);
        mmu.write_8(0xFF42, 0x00);
        mmu.write_8(0xFF43, 0x00);
        mmu.write_8(0xFF45, 0x00);
        mmu.write_8(0xFF47, 0xFC);
        mmu.write_8(0xFF48, 0xFF);
        mmu.write_8(0xFF49, 0xFF);
        mmu.write_8(0xFF4A, 0x00);
        mmu.write_8(0xFF4B, 0x00);
        mmu.write_8(0xFFFF, 0x00);

        mmu
    }
    
    /// Wrties a u8 to the indexed point in memory
    pub fn write_8(&mut self, index: u16, value: u8) {
        use crate::interupts::IF_LOC;

        if index == 0xFF50 && self.cartridge.booting {
            self.cartridge.booting = false;
        }

        let mut io = false;
        match index {
            ROM_START..=ROM_END => self.cartridge.handle_write(index, value),
            VRAM_START..=VRAM_END => self.gpu.handle_write(index, value),
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => self.cartridge.handle_write(index, value),
            WORK_RAM_START..=WORK_RAM_END => self.wram[(index - WORK_RAM_START) as usize] = value,
            ECHO_RAM_START..=ECHO_RAM_END => self.wram[(index - ECHO_RAM_START) as usize] = value,
            OAM_START..=OAM_END => self.gpu.handle_write(index, value),
            UNUSABLE_START..=UNUSABLE_END => (),
            IO_START..=IO_END => io = true,
            HRAM_START..=HRAM_END => self.hram[(index - HRAM_START) as usize] = value,
            INTERRUPT_REG => self.interupt.handle_write(index, value) 
        }

        if !io {
            return
        }

        match index {
            0xFF01 => self.serial = value as char,
            0xFF02 => print!("{}", self.serial),
            crate::joypad::JOYPAD_REG_LOC => self.joypad.handle_write(index, value),
            IF_LOC => self.interupt.handle_write(index, value),
            timer_stuff!() => self.timer.handle_write(index, value),
            apu_stuff!() => self.apu.handle_write(index, value),
            gpu_stuff!() => self.gpu.handle_write(index, value),
            _ => { log::warn!("Tried to write to non existent register {:x}", index)},
        }

        if self.gpu.dma_transfer != 0 {
            self.dma_transfer();
        }
    }
    
    /// Writes a u16 to the indexed point in memory where the low nible is at index and the high
    /// nibble is at index+1
    pub fn write_16(&mut self, index: u16, value: u16) {
        let (ms, ls) = value.split();

        self.write_8(index, ls);
        self.write_8(index + 1, ms);
    }

    /// Returns a u8 from the passed in memory index
    pub fn read_8(&self, index: u16) -> u8 {
        use crate::interupts::IF_LOC;
        use crate::joypad::JOYPAD_REG_LOC;
        
        match index {
            ROM_START..=ROM_END => self.cartridge.handle_read(index),
            VRAM_START..=VRAM_END => self.gpu.handle_read(index),
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => self.cartridge.handle_read(index),
            WORK_RAM_START..=WORK_RAM_END => self.wram[(index - WORK_RAM_START) as usize],
            ECHO_RAM_START..=ECHO_RAM_END => self.wram[(index - ECHO_RAM_START) as usize],
            OAM_START..=OAM_END => self.gpu.handle_read(index),
            UNUSABLE_START..=UNUSABLE_END => 0xFF,
            IO_START..=IO_END => {
                match index {
                    JOYPAD_REG_LOC => self.joypad.handle_read(index),
                    IF_LOC => self.interupt.handle_read(index),
                    timer_stuff!() => self.timer.handle_read(index),
                    apu_stuff!() => self.apu.handle_read(index),
                    gpu_stuff!() => self.gpu.handle_read(index),
                    _ => {log::warn!("Reading from memory that doesn't exist: {}", index); 0xFF}
                }
            },
            HRAM_START..=HRAM_END => self.hram[(index - HRAM_START) as usize],
            INTERRUPT_REG => self.interupt.handle_read(index)
        }
    }

    /// Returns a u16 from the passed in memory index where the low nibble is at index and the
    /// high nibble is at index+1
    pub fn read_16(&self, index: u16) -> u16 {
        let ls = self.read_8(index);
        let ms = self.read_8(index.wrapping_add(1));
        le_combine(ls, ms)
    }

    pub fn tick(&mut self, ticks: u8, ignore_master: bool, shared_array: &Arc<Mutex<[[ColorPixel; 160]; 144]>>) -> crate::interupts::Interupt {
        let mut interupts = 0;
        
        interupts |= self.timer.update_time(ticks);
        if self.joypad.interupt_possible {
            interupts |= 0b0001_0000;
        }
        interupts |= self.gpu.update_graphics(ticks, shared_array);

        self.interupt.update_interupts(interupts);
        self.interupt.do_interupts(ignore_master)
    }

    pub fn enable_interupts(&mut self) {
        self.interupt.master = true;
    }

    pub fn disble_interupts(&mut self) {
        self.interupt.master = false;
    }

    pub fn dma_transfer(&mut self) {
        let adress = (self.gpu.dma_transfer as u16) * 100;
        for i in 0..0xA0 {
            self.write_8(0xFE00+i, self.read_8(adress+i));
        }
        self.gpu.dma_transfer = 0;
    }

}
