use std::{collections::VecDeque};
use super::compression_block_header::BlockHeader;

pub fn to_compression_blocks<T : Iterator<Item = u8>>(bytes_iter: T) -> CompressionBlockIter<T> {
    CompressionBlockIter::<T>::new(bytes_iter)
}

pub struct CompressionBlockIter<T : Iterator<Item = u8>>
{
    bytes: T,
    buff: VecDeque<u8>,
    bytes_in_block_left: u8
}

impl<T : Iterator<Item = u8>> CompressionBlockIter<T> {
    fn new(bytes: T) -> CompressionBlockIter<T> {
        CompressionBlockIter {
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

    fn find_best_block(&self) -> BlockHeader {
        let mut candidates: Vec<BlockHeader> = vec![];

        let mut prev_byte = self.buff[0];
        let mut current = BlockHeader { bytes_length: 1, matched_bits: 7 };

        for byte in self.buff.iter().skip(1) {
            let matched_bits = match_bits(prev_byte, *byte, current.matched_bits);
            let bytes_length = current.bytes_length + 1;
            let bits_compressed = matched_bits * bytes_length;

            prev_byte = *byte;

            if matched_bits == 0 {
                break;
            }

            if bits_compressed < current.get_bits_compressed() {
                candidates.push(current);
                current = BlockHeader { bytes_length, matched_bits }
            }
            else
            {
                current.bytes_length += 1;
                current.matched_bits = matched_bits;
            }
        }

        candidates.push(current);

        return *(candidates.iter().max_by_key(|x| x.get_bits_compressed()).expect("there's always should be at leas one candidate"));
    }
}

impl<T : Iterator<Item = u8>> Iterator for CompressionBlockIter<T> {
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
        // println!("\n{:?}", block);
        // pushing first byte clone to the front to serve as a block prefix data
        self.buff.push_front(self.buff[0]);
        // plus one for block prefix
        self.bytes_in_block_left = block.bytes_length + 1;

        return Some(block.get_byte());
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
        use crate::compression::to_compression_blocks::match_bits;

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
