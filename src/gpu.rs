
use crate::mmu::{
    VRAM_START, VRAM_END, OAM_START, OAM_END,
};

/// Location in memory where the current scanline is stored (read-only)
const CURR_SCANLINE_LOC: u16 = 0xFF44;
/// Location in memory wher the concidence data is stored
const COMPARE_LOC: u16 = 0xFF45;
/// Location in memory where the lcd status is stored, (bits 0-2 write only)
const LCD_STATUS_LOC: u16 = 0xFF41;
/// Location in memory of the LCD Control Register
const LCD_CONTROL_LOC: u16 = 0xFF40;
/// Location in memory of the scroll y register
const SCROLL_Y_LOC: u16 = 0xFF42;
/// Location in memory of the scroll x register
const SCROLL_X_LOC: u16 = 0xFF43;
/// Location in memory of the window y register
const WINDOW_Y_LOC: u16 = 0xFF4A;
/// Location in memory of the window x register
const WINDOW_X_LOC: u16 = 0xFF4B;
/// Location in memory of the BGP
const BGP_LOC: u16 = 0xFF47;
/// Location in memory of OBP1 
const OBP1_LOC: u16 = 0xFF48;
/// Location in memory of OBP2
const OBP2_LOC: u16 = 0xFF49;
/// Locatio in memory of the DMA start/address register
const DMA_TRANSFER_LOC: u16 = 0xFF46;

const SPRITE_TABLE_SIZE: usize = 0xA0;
const VRAM_SIZE: usize = 0x1FFF;

#[derive(PartialEq, Eq)]
enum Mode {
    None,
    OAMScan,
    Draw,
    HBlank,
    VBlank,
}

pub struct GPU {
    // Memory Related State
    /// The VRAM data that holds all the sprites
    vram: [u8; VRAM_SIZE],
    /// The RAM data for sprite attributes
    sprite_ram: [u8; SPRITE_TABLE_SIZE],
    // Memory Registers
    /// The RAM data for the current scanline
    current_scanline: u8,
    /// The RAM data checked for the coincidence flag
    compare: u8,
    /// The RAM data for the LCD status
    lcd_status: u8,
    /// The RAM data for the LCD control register
    lcd_control: u8,
    /// The RAM data for the Y position to start drawing the viewing area from
    scroll_y: u8,
    /// The RAM data for the X position to start drawing the viewing area from
    scroll_x: u8,
    /// The RAM data for the Y position of the window
    window_y: u8,
    /// The RAM data for the X position of the window
    window_x: u8,
    /// The RAM data for the palette data register
    bg_palatte: u8,
    /// The RAM data for the object palette data register 0
    obj_palette0: u8,
    /// The RAM data for the object palette data register 1
    obj_palette1: u8,
    /// The RAM data for the DMA transfer register
    dma_transfer: u8,

    // Internal State
    mode: Mode,
}

impl Default for GPU {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            sprite_ram: [0; SPRITE_TABLE_SIZE],
            mode: Mode::None,
            current_scanline: 0,
            lcd_control: 0,
            scroll_y: 0,
            scroll_x: 0,
            window_y: 0,
            window_x: 0,
            bg_palatte: 0,
            obj_palette0: 0,
            obj_palette1: 0,
            dma_transfer: 0,
            compare: 0,
            lcd_status: 0,
        }
    }
}

impl crate::mmu::Memory for GPU {
    
    fn handle_read(&self, index: u16) -> u8 {
        match index {
            VRAM_START..=VRAM_END => {
                if self.mode != Mode::Draw {
                    self.vram[(index - VRAM_START) as usize]
                } else {
                    0xFF
                }
            },
            OAM_START..=OAM_END => {
                if self.mode == Mode::VBlank || self.mode == Mode::HBlank {
                    self.sprite_ram[(index - OAM_START) as usize]
                } else {
                    0xFF
                }
            },
            CURR_SCANLINE_LOC => self.current_scanline,
            COMPARE_LOC => self.compare,
            LCD_STATUS_LOC => self.lcd_status,
            LCD_CONTROL_LOC => self.lcd_control,
            SCROLL_Y_LOC => self.scroll_y,
            SCROLL_X_LOC => self.scroll_x,
            WINDOW_Y_LOC => self.window_y,
            WINDOW_X_LOC => self.window_x,
            BGP_LOC => self.bg_palatte,
            OBP1_LOC => self.obj_palette0,
            OBP2_LOC => self.obj_palette1,
            DMA_TRANSFER_LOC => self.dma_transfer,
            _ => unreachable!("Accessing memory that is not handled by gpu")
        }
    }

    fn handle_write(&mut self, index: u16, val: u8) {
        match index {
            VRAM_START..=VRAM_END => {
                if self.mode != Mode::Draw {
                    self.vram[(index - VRAM_START) as usize] = val;
                }
            },
            OAM_START..=OAM_END => {
                if self.mode == Mode::VBlank || self.mode == Mode::HBlank {
                    self.sprite_ram[(index - OAM_START) as usize] = val;
                }
            },
            CURR_SCANLINE_LOC => self.current_scanline = val,
            COMPARE_LOC => self.compare = val,
            LCD_STATUS_LOC => self.lcd_status = val,
            LCD_CONTROL_LOC => self.lcd_control = val,
            SCROLL_Y_LOC => self.scroll_y = val,
            SCROLL_X_LOC => self.scroll_x = val,
            WINDOW_Y_LOC => self.window_y = val,
            WINDOW_X_LOC => self.window_x = val,
            BGP_LOC => self.bg_palatte = val,
            OBP1_LOC => self.obj_palette0 = val,
            OBP2_LOC => self.obj_palette1 = val,
            DMA_TRANSFER_LOC => self.dma_transfer = val,
            _ => unreachable!("Accessing memory that is not handled by gpu")
        }
    }
    
}


