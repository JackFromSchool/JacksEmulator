use crate::mmu::{
    VRAM_START, VRAM_END, OAM_START, OAM_END,
};

use std::sync::{ Arc, Mutex };

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

const HBLANK_PERIOD: u64 = 204+172+80;
const DRAW_PERIOD: u64 = 172+80;
const OAM_PERIOD: u64 = 80;

#[derive(Default, Clone, Copy)]
pub struct ColorPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

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
    pub dma_transfer: u8,

    // Internal State
    mode: Mode,
    ticks_on_line: u64,
    
    inner_ray: [[ColorPixel; 160]; 144],
}

type MutexPixels = Arc<Mutex<[[ColorPixel; 160]; 144]>>;

impl GPU {
    
    pub fn update_graphics(&mut self, ticks: u8, shared_array: &MutexPixels) -> u8 {
        let interupt = self.set_status();
        
        if (self.lcd_control & 0b1000_0000) == 0b1000_0000 {
            self.ticks_on_line += ticks as u64;
        } else {
            return 0;
        }

        if self.ticks_on_line >= 456 {
            self.ticks_on_line = 0;
            
            if self.current_scanline < 144 {
                //self.draw_scan_line(shared_array);
            }
           
            self.current_scanline += 1;

            if self.current_scanline > 153 {
                self.current_scanline = 0;
            }

        }

        interupt
    }

    fn set_status(&mut self) -> u8 {
        let mut interupt = 0;
        let mut req_interupt = false;
        let status = self.lcd_status;

        if (self.lcd_control & 0b1000_0000) != 0b1000_0000 {
            self.current_scanline = 0;
            self.lcd_status = (status & 0b1111_1100) | 1;
            return 0;
        }
        
        let current_mode: Mode;

        if self.current_scanline >= 144 {
            
            current_mode = Mode::VBlank;
            self.lcd_status = (status & 0b1111_1100) | 0b0000_0001;
            if (self.lcd_status & 0b0001_0000) == 0b0001_0000 {
                req_interupt = true;
            }
            
        } else {
            match self.ticks_on_line {
                0..=OAM_PERIOD => {
                    current_mode = Mode::OAMScan;
                    
                    self.lcd_status = (status & 0b1111_1100) | 0b0000_0010;
                    if (self.lcd_status & 0b0010_0000) == 0b0010_0000 {
                        req_interupt = true;
                    }
                },
                81..=DRAW_PERIOD => {
                    current_mode = Mode::Draw;
                    
                    self.lcd_status = (status & 0b1111_1100) | 0b0000_0011;
                },
                _ => {
                    current_mode = Mode::HBlank;
                    
                    self.lcd_status = status & 0b1111_1100;
                    if (self.lcd_status & 0b0000_1000) == 0b0000_1000 {
                        req_interupt = true;
                    }
                },
            }
        }

        if req_interupt && self.mode != current_mode {
            interupt |= 0b0000_0010;
        }

        if self.current_scanline == self.compare {
            self.lcd_status = (status & 0b1111_1011) | 0b0000_0100;
            interupt |= 0b0000_0100;
        } else {
            self.lcd_status &= 0b1111_1011;
        }


        interupt
    }

    fn draw_scan_line(&mut self) {
        let control = self.lcd_control;

        if (control & 0b0000_0001) == 0b0000_0001 {
            self.render_tiles()
        }

        if (control & 0b0000_0010) == 0b0000_0010 {
            self.render_sprites()
        }
    }

    fn render_tiles(&mut self) {
        let control = self.lcd_control;

        let sy = self.scroll_y;
        let sx = self.scroll_x;
        let wy = self.window_y;
        let wx = self.window_x;
        
        let tile_data_start = if (control & 0b0001_0000) == 0b0001_0000 {
            0x8800 // Unsigned Look-up
        } else {
            0x8000 // Signed Look-up
        };


        let mut window = false;

        if (control & 0b0010_0000) == 0b0010_0000 {
            if wy <= self.current_scanline {
                window = true;
            }
        }

        let bg_layout_start = if !window {
            if (control & 0b0000_1000) == 0b0000_1000 {
                0x9C00
            } else {
                0x9800
            }
        } else {
            if (control & 0b0100_0000) == 0b0100_0000 {
                0x9C00
            } else {
                0x9800
            }
        };


        let y_pos = if !window {
            sy + self.current_scanline
        } else {
            self.current_scanline - wy
        };

        let tile_row = ( y_pos/8)*32;

    }

    fn render_sprites(&mut self) {
        todo!()
    }
    
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
            ticks_on_line: 0,
            inner_ray: [[ColorPixel::default(); 160]; 144],
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
            CURR_SCANLINE_LOC => self.current_scanline = 0,
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
