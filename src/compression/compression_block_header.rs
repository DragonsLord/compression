#[derive(Clone, Copy, Debug)]
pub struct BlockHeader {
    pub bytes_length: u8,
    pub matched_bits: u8,
}

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

    pub fn get_bits_compressed(&self) -> u8 {
      self.matched_bits * self.bytes_length
    }
}