use super::compression_block_header::BlockHeader;

pub fn compress_blocks<T: Iterator<Item = u8>>(compression_blocks: T) -> CompressedBlocksIter<T> {
    CompressedBlocksIter::<T>::new(compression_blocks)
}

const BIT_MASKS: [u8; 9] = [0, 1, 3, 7, 15, 31, 63, 127, 255];

pub struct CompressedBlocksIter<T : Iterator<Item = u8>> {
    elements: T,

    block: Option<BlockHeader>,
    byte: u8,
    current_bit: u8,
    current_byte: u8
}

impl<T : Iterator<Item = u8>>  CompressedBlocksIter<T> { 
    fn new(elements: T) -> CompressedBlocksIter<T> {
        return CompressedBlocksIter {
            elements,
            block: None,
            byte: 0,
            current_bit: 8,
            current_byte: 0
        }
    }

    fn fill_byte(&mut self) -> Option<u8> {
        while let Some(unit) = self.elements.next() {
            print!("{:#010b}\t| ", unit);
            let result: Option<u8>;
            match self.block {
                None => {
                    self.block = Some(BlockHeader::from_byte(unit));
                    print!("block\t| ");
                    result = self.write_bits(unit, 8);
                },
                Some(block) => {
                    if self.current_byte == 0 {
                        print!("mask\t| ");
                        // first unit in the block is a bit mask
                        result = self.write_bits(unit.overflowing_shr((8 - block.matched_bits).into()).0, block.matched_bits);
                    }
                    else {
                        print!("value\t| ");
                        result = self.write_bits(unit, 8 - block.matched_bits)
                    }
                    // checking if this is the last byte in the block
                    // starting from 0 and including 1 block metadata byte (header is emitted when block is fetched)
                    if self.current_byte == block.bytes_length {
                        self.block = None;
                        self.current_byte = 0;
                    }
                    else {
                        self.current_byte += 1;
                    }
                }
            }

            if let Some(byte) = result {
                return Some(byte);
            }
        }

        if self.current_bit < 8 {
            // this is the last bits that do not fill full byte
            return Some(self.byte);
        }

        return None;
    }

    // singnificant bits should be at the end of the byte
    fn write_bits(&mut self, bits: u8, bits_count: u8) -> Option<u8> {
        if bits_count == 0 {
            println!("writing 0 bits");
            return None;
        }
        // if bits_count > 8 {
        //     unreachable!()
        // }
        let value = BIT_MASKS[bits_count as usize] & bits;
        print!("writing {} bits (curr_pos = {}) -> {:#010b} + {:#010b} ", bits_count, self.current_bit, self.byte, value);

        if self.current_bit == bits_count {
            self.byte += value;
            self.current_bit = 8;
            
            println!("= {:#010b}", self.byte);
            let full_byte = self.byte;
            self.byte = 0;
            return Some(full_byte);
        }

        if self.current_bit > bits_count {
            // all bits will fit into current byte
            self.byte += value << (self.current_bit - bits_count);
            self.current_bit -= bits_count;

            println!("= {:#010b}", self.byte);
            return None;
        }

        {
            let overflow_bytes_count = bits_count - self.current_bit;
            let partial_val = value >> overflow_bytes_count;
            // print!("(shifted {:#010b}) ", partial_val);
            self.byte += partial_val;
            let full_byte = self.byte;
            println!(" = {:#010b}", self.byte);
            self.byte = 0;

            // writing leftover bits to the new byte
            self.current_bit = 8 - overflow_bytes_count;
            self.byte = value << self.current_bit;

            return Some(full_byte);
        }
    }
}

impl<T : Iterator<Item = u8>> Iterator for CompressedBlocksIter<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        return self.fill_byte();
    }
}
