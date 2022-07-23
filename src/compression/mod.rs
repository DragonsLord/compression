use std::{path, fs, error, io, iter::Flatten};

mod compression_block_header;
mod to_compression_blocks;
mod merge_blocks;
mod compress_blocks;
mod decompress_blocks;

use to_compression_blocks::to_compression_blocks;
use merge_blocks::{merge_blocks, BlocksMergerIter};
use compress_blocks::{compress_blocks, CompressedBlocksIter};
use decompress_blocks::{decompress_blocks, DecompressedBlocksIter};

pub trait IteratorExt : Iterator<Item = u8> + Sized {
  fn compress(self) -> CompressedBlocksIter<BlocksMergerIter<Self>>
    where Self: Iterator<Item = u8>
  {
    compress_blocks(merge_blocks(to_compression_blocks(self)))
  }

  fn decompress(self) -> DecompressedBlocksIter<Self>
    where Self: Iterator<Item = u8>
  {
    decompress_blocks(self)
  }
}

impl<T> IteratorExt for T where T : Iterator<Item = u8> {}

pub trait CompressionExt : io::Read + Sized {
  fn compress(self) -> CompressedBlocksIter<BlocksMergerIter<Flatten<io::Bytes<Self>>>> {
    self.bytes().flatten().compress()
  }

  fn decompress(self) -> DecompressedBlocksIter<Flatten<io::Bytes<Self>>> {
    self.bytes().flatten().decompress()
  }
}

impl<T> CompressionExt for T where T : io::Read {}

pub fn compress_file(filepath: &str) -> Result<impl Iterator<Item = u8>, Box<dyn error::Error>> {
  let path = path::Path::new(filepath);
  let file = fs::File::open(path)?;
  return Ok(file.compress());
}


#[cfg(test)]
mod tests {
    mod compress_decompress {
        use crate::compression::{CompressionExt, IteratorExt};

        #[test]
        fn simple_array() {
          let data: [u8; 10] = [1, 1, 2, 2, 3, 3, 4, 4, 5, 5];
          let result: Vec<u8> = data.compress().decompress().collect();
          assert_eq!(result, data);
        }

        #[test]
        fn zero_blocks() {
          let data: [u8; 4] = [0b00000000,0b11111111,0b00000000,0b11111111];
          let result: Vec<u8> = data.compress().decompress().collect();
          assert_eq!(result, data);
        }
    }
}
