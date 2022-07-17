use std::{error, fs, io::Read, collections::VecDeque, path};

pub fn compress(filepath: &str) -> Result<impl Iterator<Item = u8>, Box<dyn error::Error>> {
    let path = path::Path::new(filepath);
    let file = fs::File::open(path)?;
    return Ok(BlocksMerger::new(Compressor::new(file.bytes().flatten())));
}

struct Compressor<T : Iterator<Item = u8>>
{
    bytes: T, // Just use Read?

    buff: VecDeque<u8>,

    bytes_in_block_left: u8
}

impl<T : Iterator<Item = u8>> Compressor<T> {
    pub fn new(bytes: T) -> Compressor<T> {
        Compressor {
            bytes,
            buff: VecDeque::new(),
            bytes_in_block_left: 0
        }
    }

    fn fill_buff(&mut self) {
        while self.buff.len() < 32 {
            if let Some(value) = self.bytes.next() {
                self.buff.push_back(value);
            } else {
                break;
            }
        }
    }

    fn find_best_block(&self) -> Block {
        let mut candidates: Vec<Block> = vec![];

        let mut prev_byte = self.buff[0];
        let mut current = Block { bytes_length: 1, bits_compressed: 7, matched_bits: 7 };

        for byte in self.buff.iter().skip(1) {
            let matched_bits = match_bits(prev_byte, *byte, current.matched_bits);
            let bytes_length = current.bytes_length + 1;
            let bits_compressed = matched_bits * bytes_length;

            prev_byte = *byte;

            if matched_bits == 0 {
                break;
            }

            if bits_compressed < current.bits_compressed {
                candidates.push(current);
                current = Block {
                    bytes_length,
                    bits_compressed,
                    matched_bits
                }
            }
            else
            {
                current.bits_compressed = bits_compressed;
                current.bytes_length += 1;
                current.matched_bits = matched_bits;
            }
        }

        candidates.push(current);

        return *(candidates.iter().max_by_key(|x| x.bits_compressed).expect("there's always should be at leas one candidate"));
    }
}

impl<T : Iterator<Item = u8>> Iterator for Compressor<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes_in_block_left > 0 {
            self.bytes_in_block_left -= 1;
            return Some(self.buff.pop_front().expect("iterating over non empty queue"));
        }

        self.fill_buff();

        if self.buff.len() == 0 {
            return None;
        }

        let block = self.find_best_block();
        println!("\n{:?}", block);
        // pushing first byte clone to the front to serve as a block prefix data
        self.buff.push_front(self.buff[0]);
        // plus one for block prefix
        self.bytes_in_block_left = block.bytes_length + 1;

        return Some(block.get_header());
    }
}

struct BlocksMerger<T : Iterator<Item = u8>> {
    compressor: Compressor<T>,

    current_block: Option<Block>,
    block_index: u8,

    buf: VecDeque<u8>
}

impl<T : Iterator<Item = u8>> BlocksMerger<T> {
    fn new(compressor: Compressor<T>) -> BlocksMerger<T> {
        return BlocksMerger {
            compressor,
            current_block: None,
            block_index: 0,
            buf: VecDeque::new()
        }
    }

    fn get_next_block(&mut self) -> Option<u8> {
        let mut block_header = self.get_next_byte()?;
        let mut block = Block::from_header(block_header);
        self.current_block = Some(block);
        
        if block.bytes_length == 1 {
            block.matched_bits = 0;
            while let Some(next_block) = self.fetch_next_block() {
                if next_block.bytes_length != 1 || block.bytes_length > 32 {
                    // push block header into the buffer to proccess it later
                    self.buf.push_back(next_block.get_header());
                    break;
                }
                /* besides header byte zero block will contain 2 more bytes:
                 one for 7-bit prefix and another for 1-bit byte
                 because we are merging we need prefix byte
                 but because theay equel we can just remove the last on in the buffer */
                _ = self.buf.pop_back();
                block.bytes_length += 1;
            }

            self.current_block = Some(block);
            block_header = block.get_header();
        }
        
        self.block_index = 0;
        return Some(block_header);
    }

    /* Note:
        This method do not take buffer into account.
        So if it's not empty it might work incorrectly.
        It's ok now cause it's only use in the case
        when buffer is empty */
    fn fetch_next_block(&mut self) -> Option<Block> {
        if let Some(block) = self.current_block {
            // read current block bytes into buffer
            for _ in 0..=block.bytes_length {
                let byte = self.compressor.next()?;
                self.buf.push_back(byte);
            }
        }
        let block_header = self.compressor.next()?;
        return Some(Block::from_header(block_header));
    }

    fn get_next_byte(&mut self) -> Option<u8> {
        if self.buf.len() > 0 {
            return self.buf.pop_front();
        }
        return self.compressor.next();
    }
}

impl<T : Iterator<Item = u8>> Iterator for BlocksMerger<T> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current_block {
            None => self.get_next_block(),
            Some(block) => {
                // checking if this is the last byte in the block
                // starting from 0 and including 1 block metadata byte (header is emitted when block is fetched)
                if self.block_index == block.bytes_length {
                    self.current_block = None;
                }

                self.block_index += 1;
                return self.get_next_byte();
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Block {
    bytes_length: u8,
    matched_bits: u8,
    bits_compressed: u8 // TODO: consider computing it when it's needed
}

impl Block {
    fn get_header(&self) -> u8 {
        // first 3 bits is a but prefix length
        // rest 5 bits is an amount of bytes (minus one to exlude zero value)
        return self.matched_bits.checked_shl(5).expect("matched_bits less then 8") + (self.bytes_length - 1);
    }

    fn from_header(header: u8) -> Block {
        let bytes_length = (header & 31) + 1;
        let matched_bits = (header & 224) >> 5;
        Block {
            bytes_length,
            matched_bits, 
            bits_compressed: matched_bits * bytes_length
        }
    }
}

fn match_bits(left: u8, right: u8, length: u8) -> u8 {
    let match_result = !(left ^ right);
    let mut mask: u8 = 1 << 7;
    let mut bits_left: u8 = 8;

    while bits_left > (8 - length) {
        if match_result & mask != mask {
            break;
        }
        bits_left -= 1;
        mask >>= 1
    }

    return 8 - bits_left;
}

#[cfg(test)]
mod tests {
    mod match_bits {
        use crate::compression::match_bits;

        #[test]
        fn full_match() {
            let result = match_bits(2, 2, 8);
            assert_eq!(result, 8);
        }

        #[test]
        fn zero_match() {
            let result = match_bits(255, 0, 8);
            assert_eq!(result, 0);
        }

        #[test]
        fn partial_match() {
            let result = match_bits(73, 74, 8);
            assert_eq!(result, 6);
        }

        #[test]
        fn part_match() {
            let result = match_bits(72, 72, 3);
            assert_eq!(result, 3);
        }
    }
}

// bit and, track first bits



// mod bytes_iter {
//     use std::{fs, path::{self}, error, io::Read};

//     pub fn iter_bytes(filepath: &str) -> Result<impl Iterator<Item = u8>, Box<dyn error::Error>> {
//         let file_bytes = FileBytes::new(filepath)?;
//         return Ok(file_bytes);
//     }
    
//     struct FileBytes {
//       file: fs::File,
//       buffer: [u8; 255],
    
//       bytes_in_buffer: usize,
//       current: usize
//     }
    
//     impl FileBytes {
//       fn new(file_path: &str) -> Result<FileBytes, Box<dyn error::Error>> {
//           let path = path::Path::new(file_path);
//           let file = fs::File::open(path)?;

//           return Ok(FileBytes {
//               file,
//               buffer: [0; 255],
//               bytes_in_buffer: 0,
//               current: 0
//           });
//       }
//     }
    
//     impl Iterator for FileBytes {
//         type Item = u8;

//         fn next(&mut self) -> Option<Self::Item> {
//             if self.current >= self.bytes_in_buffer {
//                 match self.file.read(&mut self.buffer) {
//                     Ok(n) => {
//                         self.bytes_in_buffer = n;
//                         self.current = 0;
//                     },
//                     Err(e) => panic!("{:?}", e),
//                 }
//             }

//             if self.current < self.bytes_in_buffer {
//                 let value = self.buffer[self.current];
//                 self.current = self.current + 1;
//                 return Some(value);
//             }

//             return None;
//         }
//     }
// }