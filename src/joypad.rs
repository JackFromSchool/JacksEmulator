pub const JOYPAD_REG_LOC: u16 = 0xFF00;

/// Represents the state related to the Joypad while also handling events
pub struct Joypad {
    /// Corelates to the register in memory mapped to the joypad IO
    joypad_reg: u8,

    direction_byte: u8,
    button_byte: u8,
    pub interupt_possible: bool,
}

pub enum ButtonEvent {
    None,
    Start,
    Select,
    A,
    B,
    Up,
    Down,
    Left,
    Right,
}

impl ButtonEvent {

    pub fn is_button(&self) -> bool {
        match self {
            Self::A | Self::B | Self::Select | Self::Start => true,
            _ => false,
        }
    }

    pub fn is_direction(&self) -> bool {
        match self {
            Self::Right | Self::Left | Self::Up | Self::Down => true,
            _ => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }
    
}

const RIGHT_A: u8 = 0b0000_0001;
const LEFT_B: u8 = 0b0000_0010;
const UP_SELECT: u8 = 0b0000_0100;
const DOWN_START: u8 = 0b0000_1000;

const BUTTON: u8 = 0b0010_0000;
const DIRECTION: u8 = 0b0001_0000;

pub struct ButtonEventWrapper {
    pub event: ButtonEvent,
    pub new_state: winit::event::ElementState,
}

impl Default for Joypad {
    fn default() -> Self {
        Self {
            joypad_reg: 0,
            direction_byte: 0xF,
            button_byte: 0xF,
            interupt_possible: false,
        }
    }
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
            if self.joypad_reg & 0x10 == 0 {
                (self.joypad_reg & 0xF0) + (self.direction_byte & 0xF)
            } else if self.joypad_reg & 0x20 == 0 {
                (self.joypad_reg & 0xF0) + (self.button_byte & 0xF)
            } else {
                0xFF
            }
        }
    }
    
}

impl Joypad {
    
    pub fn update_state(&mut self, wrapper: ButtonEventWrapper) {
        // true means the button is NOT PRESSED
        
        let curr_state = match wrapper.event {
            ButtonEvent::Down => self.direction_byte & DOWN_START,
            ButtonEvent::Start => self.button_byte & DOWN_START,
            ButtonEvent::Up => self.direction_byte & UP_SELECT,
            ButtonEvent::Select => self.button_byte & UP_SELECT,
            ButtonEvent::Left => self.direction_byte & LEFT_B,
            ButtonEvent::B => self.button_byte & LEFT_B,
            ButtonEvent::Right => self.direction_byte & RIGHT_A,
            ButtonEvent::A => self.direction_byte & RIGHT_A,
            ButtonEvent::None => unreachable!()
        };

        let base = match wrapper.event {
            ButtonEvent::A | ButtonEvent::Right => RIGHT_A,
            ButtonEvent::Left | ButtonEvent::B => LEFT_B,
            ButtonEvent::Select | ButtonEvent::Up => UP_SELECT,
            ButtonEvent::Down | ButtonEvent::Start => DOWN_START,
            ButtonEvent::None => unreachable!()
        };

        let new_state = match wrapper.new_state {
            winit::event::ElementState::Pressed => false,
            winit::event::ElementState::Released => true, 
        };

        if curr_state > 0 && !new_state {
            if self.joypad_reg & BUTTON == 0 && wrapper.event.is_button() {
                self.interupt_possible = true;
            } else if self.joypad_reg & DIRECTION == 0 && wrapper.event.is_direction() {
                self.interupt_possible = true;
            }
        }
        
        if new_state {
            if wrapper.event.is_button() {
                self.button_byte |= base & 0xF;
            } else {
                self.direction_byte |= base & 0xF;
            }
        } else {
            if wrapper.event.is_button() {
                self.button_byte &= !base;
            } else {
                self.direction_byte &= !base;
            }
        }
    }
    
}
