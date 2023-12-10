// TODO: Constants
/// Only bit 7 can be written to, Bits 0-3 can be read only
const MASTER_CONTROL_LOC: u16 = 0xFF26;
const MASTER_PANNING_LOC: u16 = 0xFF25;
const MASTER_VOLUME_LOC: u16 = 0xFF24;

/// Constants for audio channel 1
pub mod C1 {
    pub const SWEEP_LOC: u16 = 0xFF10;
    /// Bits 0-5 are write only, other bits are read/write
    pub const LENGTH_LOC: u16 = 0xFF11;
    pub const VOLUME_ENVELOPE_LOC: u16 =  0xFF12;
    /// Write Only
    pub const FREQUENCY_LOW_LOC: u16 = 0xFF13;
    /// Only bit 6 can be read from, other bits can be written too
    pub const FREQUENCY_HIGH_LOC: u16 = 0xFF14;
}

/// Constants for audio channel 2
pub mod C2 {
    /// Bits 0-5 are write only, other bits are read/write
    pub const LENGTH_LOC: u16 = 0xFF16;
    pub const VOLUME_ENVELOPE_LOC: u16 =  0xFF17;
    /// Write Only
    pub const FREQUENCY_LOW_LOC: u16 = 0xFF18;
    /// Only bit 6 can be read from, other bits can be written too
    pub const FREQUENCY_HIGH_LOC: u16 = 0xFF19;
}

/// Constants for audio channel 3
pub mod C3 {
    pub const TOGGLE_LOC: u16 = 0xFF1A;
    /// Write Only
    pub const LENGTH_LOC: u16 = 0xFF1B;
    pub const VOLUME_LOC: u16 = 0xFF1C;
    /// Write Only
    pub const FREQUENCY_LOW_LOC: u16 = 0xFF1D;
    /// Only bit 6 can be read from, other bits can be written too
    pub const FREQUENCY_HIGH_LOC: u16 = 0xFF1E;
    pub const PATTERN_RAM_START: u16 = 0xFF30;
    pub const PATTERN_RAM_STOP: u16 = 0xFF3F;
}

/// Constants for audio channel 4
pub mod C4 {
    pub const LENGTH_LOC: u16 = 0xFF20;
    pub const VOLUME_ENVELOPE_LOC: u16 = 0xFF21;
    pub const FREQUENCY_LOC: u16 = 0xFF22;
    /// Only bit 7 can be read from, all can be written
    pub const CONTROL_LOC: u16 = 0xFF23;
}

const WAVE_PATTERN_RAM_SIZE: usize = 0xF;

/// Represents the state related to the audio processing unit
/// This is *not* implemented currently
pub struct APU {
    c1_sweep: u8,
    c1_length: u8,
    c1_volume_envelope: u8,
    c1_frequency_lo: u8,
    c1_frequency_hi: u8,

    c2_lenght: u8,
    c2_volume_envelope: u8,
    c2_frequency_lo: u8,
    c2_frequency_hi: u8,

    c3_toggle: u8,
    c3_length: u8,
    c3_volume: u8,
    c3_frequncy_lo: u8,
    c3_frequency_hi: u8,
    c3_pattern_ram: [u8; WAVE_PATTERN_RAM_SIZE],

    c4_length: u8,
    c4_volume_envelope: u8,
    c4_frequency: u8,
    c4_control: u8,

    master_volume: u8,
    sound_output: u8,
    sound_toggle: u8,
}

impl std::default::Default for APU {
    fn default() -> Self {
        Self {
            c1_sweep: 0,
            c1_length: 0,
            c1_volume_envelope: 0,
            c1_frequency_lo: 0,
            c1_frequency_hi: 0,

            c2_lenght: 0,
            c2_volume_envelope: 0,
            c2_frequency_lo: 0,
            c2_frequency_hi: 0,

            c3_toggle: 0,
            c3_length: 0,
            c3_volume: 0,
            c3_frequncy_lo: 0,
            c3_frequency_hi: 0,
            c3_pattern_ram: [0; WAVE_PATTERN_RAM_SIZE],

            c4_length: 0,
            c4_volume_envelope: 0,
            c4_frequency: 0,
            c4_control: 0,

            master_volume: 0,
            sound_output: 0,
            sound_toggle: 0,
        }
    }
}

impl crate::mmu::Memory for APU {
    
    fn handle_read(&self, index: u16) -> u8 {
        match index {
            MASTER_VOLUME_LOC => self.master_volume,
            MASTER_PANNING_LOC => self.sound_output,
            MASTER_CONTROL_LOC => self.sound_toggle,

            C1::SWEEP_LOC => self.c1_sweep,
            C1::LENGTH_LOC => self.c1_length,
            C1::VOLUME_ENVELOPE_LOC => self.c1_volume_envelope,
            C1::FREQUENCY_LOW_LOC => 0xFF,
            C1::FREQUENCY_HIGH_LOC => self.c1_frequency_hi & 0b0010_0000,

            C2::LENGTH_LOC => self.c2_lenght & 0b1100_0000,
            C2::VOLUME_ENVELOPE_LOC => self.c2_volume_envelope,
            C2::FREQUENCY_LOW_LOC => 0xFF,
            C2::FREQUENCY_HIGH_LOC => self.c2_frequency_hi & 0b0010_0000,

            C3::TOGGLE_LOC => self.c3_toggle,
            C3::LENGTH_LOC => 0xFF,
            C3::VOLUME_LOC => self.c3_volume,
            C3::FREQUENCY_LOW_LOC => 0xFF,
            C3::FREQUENCY_HIGH_LOC => self.c3_frequency_hi & 0b0100_0000,
            C3::PATTERN_RAM_START..=C3::PATTERN_RAM_STOP => self.c3_pattern_ram[(index - C3::PATTERN_RAM_START) as usize],
            
            C4::LENGTH_LOC => self.c4_length,
            C4::VOLUME_ENVELOPE_LOC => self.c4_volume_envelope,
            C4::FREQUENCY_LOC => self.c4_frequency,
            C4::CONTROL_LOC => self.c4_control & 0b1000_0000,

            _ => unreachable!("APU doesn't handle this memory")
        }
    }

    fn handle_write(&mut self, index: u16, val: u8) {
        match index {
            MASTER_VOLUME_LOC => self.master_volume = val,
            MASTER_PANNING_LOC => self.sound_output = val,
            MASTER_CONTROL_LOC => self.sound_toggle = (self.sound_toggle & 0b0111_1111) + (val & 0b1000_0000) ,

            C1::SWEEP_LOC => self.c1_sweep = val,
            C1::LENGTH_LOC => self.c1_length = val,
            C1::VOLUME_ENVELOPE_LOC => self.c1_volume_envelope = val,
            C1::FREQUENCY_LOW_LOC => self.c1_frequency_lo = val,
            C1::FREQUENCY_HIGH_LOC => self.c1_frequency_hi = val,

            C2::LENGTH_LOC => self.c2_lenght = val,
            C2::VOLUME_ENVELOPE_LOC => self.c2_volume_envelope = val,
            C2::FREQUENCY_LOW_LOC => self.c2_frequency_lo = val,
            C2::FREQUENCY_HIGH_LOC => self.c2_frequency_hi = val,

            C3::TOGGLE_LOC => self.c3_toggle = val,
            C3::LENGTH_LOC => self.c3_length = val,
            C3::VOLUME_LOC => self.c3_volume = val,
            C3::FREQUENCY_LOW_LOC => self.c3_frequncy_lo = val,
            C3::FREQUENCY_HIGH_LOC => self.c3_frequency_hi = val,
            C3::PATTERN_RAM_START..=C3::PATTERN_RAM_STOP => self.c3_pattern_ram[(index - C3::PATTERN_RAM_START) as usize] = val,
            
            C4::LENGTH_LOC => self.c4_length = val,
            C4::VOLUME_ENVELOPE_LOC => self.c4_volume_envelope = val,
            C4::FREQUENCY_LOC => self.c4_frequency = val,
            C4::CONTROL_LOC => self.c4_control = val,

            _ => unreachable!("APU doesn't handle this memory")
        }
    }
    
}

