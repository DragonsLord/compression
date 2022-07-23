use std::{path, fs, error, io, iter::Flatten};

mod compression_block_header;
mod to_compression_blocks;
mod merge_blocks;
mod compress_blocks;

use to_compression_blocks::to_compression_blocks;
use merge_blocks::merge_blocks;
use compress_blocks::compress_blocks;

use self::{compress_blocks::CompressedBlocksIter, merge_blocks::BlocksMergerIter};

pub trait IteratorExt : Iterator<Item = u8> + Sized {
    fn compress(self) -> CompressedBlocksIter<BlocksMergerIter<Self>>
       where Self: Iterator<Item = u8>
    {
      compress_blocks(merge_blocks(to_compression_blocks(self)))
    }
}

impl<T> IteratorExt for T where T : Iterator<Item = u8> {}

pub trait CompressionExt : io::Read + Sized {
  fn compress(self) -> CompressedBlocksIter<BlocksMergerIter<Flatten<io::Bytes<Self>>>> {
    self.bytes().flatten().compress()
  }
}

impl<T> CompressionExt for T where T : io::Read {}

pub fn compress_file(filepath: &str) -> Result<impl Iterator<Item = u8>, Box<dyn error::Error>> {
  let path = path::Path::new(filepath);
  let file = fs::File::open(path)?;
  return Ok(file.compress());
}