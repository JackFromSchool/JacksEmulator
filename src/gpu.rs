use crate::mmu::{
    VRAM_START, VRAM_END, OAM_START, OAM_END,
};

use std::sync::{ Arc, Mutex, MutexGuard};

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
const VRAM_SIZE: usize = 0x2000;

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

enum TilePixelValue {
    Zero,
    One,
    Two,
    Three,
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
type TileArray = [[u8; 8]; 8];

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
                self.draw_scan_line(shared_array);
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

    fn draw_scan_line(&mut self, shared_array: &MutexPixels) {
        let control = self.lcd_control;

        if (control & 0b0000_0001) == 0b0000_0001 {
            self.render_tiles(shared_array);
        }

        if (control & 0b0000_0010) == 0b0000_0010 {
            //self.render_sprites()
        }
    }

    fn render_tiles(&mut self, shared_array: &MutexPixels) {
        let mut locked_array = shared_array.lock().unwrap();

        let window_tile_map_start = if self.lcd_control & 0x2 == 0x2 {
            if self.lcd_control & 0x64 == 0x64 {
                Some(0x9C00)
            } else {
                Some(0x9800)
            }
        } else {
            None
        };

        let bg_tile_map_start = if self.lcd_control & 0x8 == 0x8 {
            0x9C00
        } else {
            0x9800
        };

        let tile_data_start: u16 = if self.lcd_control & 0x10 == 0x10 {
            0x8000
        } else {
            0x8800
        };
        
        let sy = self.scroll_y;
        let sx = self.scroll_x;
        let wy = self.window_y;
        let wx = self.window_x.wrapping_sub(7);
        
        let tile_y = sy.wrapping_add(self.current_scanline) / 8;
        
        for index in 0..160 {
            let tile_x = sx.wrapping_add(index) / 8;

            let current_tile: u16 = (tile_y as u16)*32 + (tile_x as u16);
            
            // TODO: Update to account for scrolling
            let tile_identifier = if wx >= sx.wrapping_add(index) && wy >= sy.wrapping_add(self.current_scanline) && window_tile_map_start.is_some() {
                self.vram[((window_tile_map_start.unwrap()+current_tile)-VRAM_START) as usize]
            } else {
                self.vram[((bg_tile_map_start+current_tile)-VRAM_START) as usize]
            };

            let tile_start = if tile_data_start == 0x8000 {
                tile_data_start+(tile_identifier as u16)*16
            } else {
                tile_data_start+(128u8.wrapping_add_signed(tile_identifier as i8) as u16)*16
            };

            let inner_tile_y = sy.wrapping_add(self.current_scanline) % 8;
            let inner_tile_x = sx.wrapping_add(index) % 8;

            let tile = self.get_tile(tile_start);
            
            let pixel = tile[inner_tile_y as usize][inner_tile_x as usize];

            let color = match pixel {
                0 => (self.bg_palatte & 0b0000_0011),
                1 => (self.bg_palatte & 0b0000_1100) >> 2,
                2 => (self.bg_palatte & 0b0011_0000) >> 4,
                3 => (self.bg_palatte & 0b1100_0000) >> 6,
                _ => unreachable!()
            };

            let color_pixel = match color {
                0 => ColorPixel { r: 0xd0, g: 0xd0, b: 0x58, a: 255 },
                1 => ColorPixel { r: 0xa0, g: 0xa8, b: 0x40, a: 255 },
                2 => ColorPixel { r: 0x70, g: 0x80, b: 0x28, a: 255 },
                3 => ColorPixel { r: 0x40, g: 0x50, b: 0x10, a: 255 },
                _ => unreachable!()
            };

            locked_array[self.current_scanline as usize][index as usize] = color_pixel;
        }

    }

    fn render_sprites(&mut self) {
        todo!()
    }

    fn get_bit_val(byte: u8, index: u32) -> u8 {
        if byte & (2u8.pow(index)) == byte & (2u8.pow(index)) {
            1
        } else {
            0
        }
    }

    fn get_tile(&self, tile_start: u16) -> TileArray {
        let mut ret_array: TileArray = [[0; 8]; 8];
        
        for i in (0..16).filter(|x| x % 2 == 0) {
            let first_byte = self.vram[((tile_start+i) - VRAM_START) as usize];
            let second_byte = self.vram[((tile_start+i) - VRAM_START) as usize];

            for j in 0..8 {
                ret_array[(i/2) as usize][7-j] = 
                    ((first_byte & 2u8.pow(j as u32)) >> j) |
                    (((second_byte & 2u8.pow(j as u32)) >> j) << 1);
            }
        }

        ret_array
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
