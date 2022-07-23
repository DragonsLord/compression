use std::{collections::VecDeque};
use super::{to_compression_blocks::CompressionBlockIter, compression_block_header::BlockHeader};

pub fn merge_blocks<T: Iterator<Item = u8>>(blocks: CompressionBlockIter<T>) -> BlocksMergerIter<T> {
  BlocksMergerIter::<T>::new(blocks)
}

pub struct BlocksMergerIter<T : Iterator<Item = u8>> {
    compressor: CompressionBlockIter<T>,

    current_block: Option<BlockHeader>,
    block_index: u8,

    buf: VecDeque<u8>
}

impl<T : Iterator<Item = u8>> BlocksMergerIter<T> {
    fn new(compressor: CompressionBlockIter<T>) -> BlocksMergerIter<T> {
        return BlocksMergerIter {
            compressor,
            current_block: None,
            block_index: 0,
            buf: VecDeque::new()
        }
    }

    fn get_next_block(&mut self) -> Option<u8> {
        let mut block_header = self.get_next_byte()?;
        let mut block = BlockHeader::from_byte(block_header);
        self.current_block = Some(block);
        
        if block.bytes_length == 1 {
            block.matched_bits = 0;
            while let Some(next_block) = self.fetch_next_block() {
                if next_block.bytes_length != 1 || block.bytes_length > 32 {
                    // push block header into the buffer to proccess it later
                    self.buf.push_back(next_block.get_byte());
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
            block_header = block.get_byte();
        }
        
        self.block_index = 0;
        return Some(block_header);
    }

    /* Note:
        This method do not take buffer into account.
        So if it's not empty it might work incorrectly.
        It's ok now cause it's only use in the case
        when buffer is empty */
    fn fetch_next_block(&mut self) -> Option<BlockHeader> {
        if let Some(block) = self.current_block {
            // read current block bytes into buffer
            for _ in 0..=block.bytes_length {
                let byte = self.compressor.next()?;
                self.buf.push_back(byte);
            }
        }
        let block_header = self.compressor.next()?;
        return Some(BlockHeader::from_byte(block_header));
    }

    fn get_next_byte(&mut self) -> Option<u8> {
        if self.buf.len() > 0 {
            return self.buf.pop_front();
        }
        return self.compressor.next();
    }
}

impl<T : Iterator<Item = u8>> Iterator for BlocksMergerIter<T> {
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
