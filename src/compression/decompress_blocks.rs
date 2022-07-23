use super::compression_block_header::BlockHeader;

pub fn decompress_blocks<T: Iterator<Item = u8>>(compression_blocks: T) -> DecompressedBlocksIter<T> {
    DecompressedBlocksIter::<T>::new(compression_blocks)
}

const BIT_MASKS: [u8; 9] = [0b00000000, 0b00000001, 0b00000011, 0b00000111, 0b00001111, 0b00011111, 0b00111111, 0b01111111, 0b11111111];

pub struct DecompressedBlocksIter<T : Iterator<Item = u8>> {
    compressed_bytes: T,

    current_block: Option<BlockHeader>,
    mask: Option<u8>,
    block_bytes_read: u8,
    
    byte_buf: u8,
    current_bit: u8,
}

impl<T : Iterator<Item = u8>>  DecompressedBlocksIter<T> { 
    fn new(compressed_bytes: T) -> DecompressedBlocksIter<T> {
        return DecompressedBlocksIter {
            compressed_bytes,
            current_block: None,
            mask: None,
            block_bytes_read: 0,
            current_bit: 0,
            byte_buf: 0,
        }
    }

    fn get_next_byte(&mut self) -> Option<u8> {
        loop {
            if let Some(block_header) = self.current_block
            {
                if let Some(mask) = self.mask {
                    let left_bits = self.read_bits(8 - block_header.matched_bits).expect("Broken data: expected to have more bytes in the block");
    
                    self.block_bytes_read += 1;
    
                    if self.block_bytes_read == block_header.bytes_length {
                        self.current_block = None;
                        self.mask = None;
                        self.block_bytes_read = 0;
                    }
                    
                    println!("byte: {:#010b}", mask + left_bits);
                    return Some(mask + left_bits);
                }
                else 
                {
                    // read mask
                    let mask = self.read_bits(block_header.matched_bits).expect("Broken data: could not read mask - no bytes left");
                    // shift mask to the right;
                    self.mask = Some(mask << (8 - block_header.matched_bits));
                    println!("mask: {:#010b}", self.mask.unwrap())
                }
            }
            else
            {
                // read block header
                let block_header = self.read_bits(8)?; // here's None is possible as there could be no more blocks left
                self.current_block = Some(BlockHeader::from_byte(block_header));
                if self.current_block.unwrap().matched_bits == 0 {
                    // no need to read mask for zero block
                    self.mask = Some(0);
                }
                println!("block: {:?}", self.current_block)
            }
        }
    }

    // returns byte with significant bits shifted to the left
    fn read_bits(&mut self, bits_amount: u8) -> Option<u8> {
        let mut result: u8 = 0;
        let mut read_amount = bits_amount;
        loop {
            if self.current_bit == 0 {
                // Note: None should possible only when reading block header
                self.byte_buf = self.compressed_bytes.next()?;
                self.current_bit = 8;
            }
    
            if self.current_bit >= read_amount {
    
                // curr_bit = 6
                // read_amount = 4
                // byte = 10111010 -> 00001110
    
                result += (self.byte_buf & BIT_MASKS[self.current_bit as usize]) >> (self.current_bit - read_amount);
                self.current_bit -= read_amount;
                return Some(result);
            }
    
            // self.current_bit < read_amount
            {
    
                // curr_bit = 2
                // read_amount = 5
                // byte = 10101011 -> 00011000
    
                read_amount = read_amount - self.current_bit;
                result = (self.byte_buf & BIT_MASKS[self.current_bit as usize]) << read_amount;
                self.current_bit = 0;
            }
        }
    }
}

impl<T : Iterator<Item = u8>> Iterator for DecompressedBlocksIter<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_next_byte()
    }
}
