use std::{fmt::{Debug}};

#[derive(Clone, Copy)]
pub struct BlockHeader {
    pub bytes_length: u8,
    pub matched_bits: u8,
}

const HEADER_SIZE: i32 = 8;

impl BlockHeader {
    pub fn get_byte(&self) -> u8 {
        // first 3 bits is a bit prefix length
        // rest 5 bits is an amount of bytes (minus one to exlude zero value)
        return self.matched_bits.checked_shl(5).expect("matched_bits less then 8") + (self.bytes_length - 1);
    }

    pub fn from_byte(header: u8) -> BlockHeader {
        let bytes_length = (header & 31) + 1;
        let matched_bits = (header & 224) >> 5;
        BlockHeader {
            bytes_length,
            matched_bits,
        }
    }

    pub fn get_bits_compressed(&self) -> i32 {
        Self::calc_compressed_bits(self.bytes_length, self.matched_bits)
    }

    pub fn calc_compressed_bits(bytes_length: u8, matched_bits: u8) -> i32 {
        matched_bits as i32 * (bytes_length as i32 - 1) - HEADER_SIZE
    }
}

impl Default for BlockHeader {
    fn default() -> Self {
        Self { bytes_length: 0, matched_bits: 8 }
    }
}

impl Debug for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("{{ matched = {}, len = {}, compressed = {} }}", self.matched_bits, self.bytes_length, self.get_bits_compressed());
        f.write_str(&string)
    }
}