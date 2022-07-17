use std::{error, fs, path, io::Read, collections::VecDeque};

pub fn compress(filepath: &str) -> Result<impl Iterator<Item = u8>, Box<dyn error::Error>> {
    let path = path::Path::new(filepath);
    let file = fs::File::open(path)?;
    return Ok(Compressor::new(file.bytes().flatten()));
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
        let mut current = Block { bytes_length: 1, bits_compressed: 0, matched_bits: 7 };

        for byte in self.buff.iter() {
            // TODO: match_bits should just return length
            let matched_bits = match_bits(prev_byte, *byte, current.matched_bits);
            let bytes_length = current.bytes_length + 1;
            let bits_compressed = matched_bits * bytes_length;

            prev_byte = *byte;

            if matched_bits == 0 {
                if current.matched_bits > 0 && current.bytes_length > 1 {
                    // if we have non zero candidates we should break on 0 match
                    break;
                }
                current.bytes_length += 1;
                continue;
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
        if self.bytes_in_block_left > 1 {
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

#[derive(Clone, Copy, Debug)]
struct Block {
    bytes_length: u8,
    bits_compressed: u8,
    matched_bits: u8
}

impl Block {
    fn get_header(&self) -> u8 {
        // first 3 bits is a but prefix length
        // rest 5 bits is an amount of bytes
        return self.matched_bits.checked_shl(5).expect("matched_bits less then 8") + self.bytes_length;
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