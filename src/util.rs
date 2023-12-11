pub trait BitOperations {
    /// Returns the number split into the most significant and least significant bit in big endian
    /// ordering.
    /// # Tuple Values:
    /// - 0: most significant bit
    /// - 1: least significant bit
    fn split(&self) -> (u8, u8);
}

impl BitOperations for u16 {
    
    fn split(&self) -> (u8, u8) {
        let ls = (self & 0b0000_0000_1111_1111) as u8;
        let ms = ((self & 0b1111_1111_0000_0000) >> 8) as u8;
        (ms, ls)
    }
    
}

/// Combines a least significant bit with most significant in little endian encoding. Pass in
/// lsBit first and then msBit
pub fn le_combine(ls: u8, ms: u8) -> u16 {
    ((ms as u16) << 8) + (ls as u16)
}
