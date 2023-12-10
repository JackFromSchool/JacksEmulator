
#[derive(Default, Clone, Copy)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

const FLAG_Z: u8 = 0b1000_0000;
const FLAG_Z_INV: u8 = !FLAG_Z;
const FLAG_N: u8 = 0b0100_0000;
const FLAG_N_INV: u8 = !FLAG_N;
const FLAG_H: u8 = 0b0010_0000;
const FLAG_H_INV: u8 = !FLAG_H;
const FLAG_C: u8 = 0b0001_0000;
const FLAG_C_INV: u8 = !FLAG_C;

impl Registers {

    pub fn get_af(&self) -> u16 {
        Self::combine(self.a, self.f)
    }

    pub fn set_af(&mut self, num: u16) {
        let (a, f) = Self::split(num);
        self.a = a;
        self.f = f;
    }
    
    pub fn get_bc(&self) -> u16 {
        Self::combine(self.b, self.c)
    }

    pub fn set_bc(&mut self, num: u16) {
        let (b, c) = Self::split(num);
        self.b = b;
        self.c = c;
    }
    
    pub fn get_de(&self) -> u16 {
        Self::combine(self.d, self.e)
    }

    pub fn set_de(&mut self, num: u16) {
        let (d, e) = Self::split(num);
        self.d = d;
        self.e = e;
    }
    
    pub fn get_hl(&self) -> u16 {
        Self::combine(self.h, self.l)
    }

    pub fn set_hl(&mut self, num: u16) {
        let (h, l) = Self::split(num);
        self.h = h;
        self.l = l;
    }
    
    /// Sets the Z flag
    pub fn set_z(&mut self) {
        self.f = self.f & FLAG_Z_INV + FLAG_Z;
    }
    
    /// Unsets the Z flag
    pub fn unset_z(&mut self) {
        self.f = self.f & FLAG_Z_INV;
    }
    
    /// Sets the N flag
    pub fn set_n(&mut self) {
        self.f = self.f & FLAG_N_INV + FLAG_N;
    }
    
    /// Unsets the N flag
    pub fn unset_n(&mut self) {
        self.f = self.f & FLAG_N_INV;
    }
    
    /// Sets the H flag
    pub fn set_h(&mut self) {
        self.f = self.f & FLAG_H_INV + FLAG_H;
    }
    
    /// Unsets the H flag
    pub fn unset_h(&mut self) {
        self.f = self.f & FLAG_H_INV;
    }
    
    /// Sets the C flag
    pub fn set_c(&mut self) {
        self.f = self.f & FLAG_C_INV + FLAG_C;
    }
    
    /// Unsets the C flag
    pub fn unset_c(&mut self) {
        self.f = self.f & FLAG_C_INV;
    }
    
    /// Combines two u8's into one u16 where high is the most significant byte and low is the least
    /// significant byte
    fn combine(high: u8, low: u8) -> u16 {
        ((high as u16) << 4) + (low as u16)
    }
    
    /// Splits a u16 into two u8's where position 0 in the tuple is the most significant byte and
    /// position 1 is the least significant byte
    fn split(num: u16) -> (u8, u8) {
        (((num & 0b1111_0000) >> 4) as u8, (num & 0b0000_1111) as u8)
    }
}
