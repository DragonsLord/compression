use std::{path, fs, error, io::Read};

mod compression_block_header;
mod to_compression_blocks;
mod merge_blocks;
mod compress_blocks;

use to_compression_blocks::to_compression_blocks;
use merge_blocks::merge_blocks;
use compress_blocks::compress_blocks;

pub fn compress(filepath: &str) -> Result<impl Iterator<Item = u8>, Box<dyn error::Error>> {
  let path = path::Path::new(filepath);
  let file = fs::File::open(path)?;
  return Ok(compress_blocks(merge_blocks(to_compression_blocks(file.bytes().flatten()))));
}