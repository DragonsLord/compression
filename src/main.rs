use std::{env, error::{self, Error}, process, fs, path, io::Write};

pub mod compression;

use compression::{CompressionExt};


fn main() -> Result<(), Box<dyn error::Error>> {
    let compress_mode = "--compress".to_owned();
    let args: Vec<String> = env::args().collect();
    let filepath = args.get(1).unwrap_or_else(|| {
        eprintln!("filepath is not provided");
        process::exit(1);
    });
    let mode = args.get(2).unwrap_or_else(|| { &compress_mode });

    // let filepath = "assets/pic.jpg";
    let path = path::Path::new(filepath);
    let file = fs::File::open(path)?;

    if mode == "--decompress" {
        let decompressed_bytes = file.decompress();

        let out_filepath = format!("{}.decompressed", filepath);
        let out_path = path::Path::new(&out_filepath);
        let mut out_file = fs::File::create(out_path)?;
    
        write_file(&mut out_file, decompressed_bytes)?;
    }
    else {
        let compressed_bytes = file.compress();

        let out_filepath = format!("{}.compressed", filepath);
        let out_path = path::Path::new(&out_filepath);
        let mut out_file = fs::File::create(out_path)?;
    
        write_file(&mut out_file, compressed_bytes)?;
    }

    

    return Ok(());
}

fn write_file<W: Write ,I: Iterator<Item = u8>>(writer: &mut W, bytes: I) -> Result<(), Box<dyn Error>>
{
    const SIZE: usize = 1024;

    let mut buffer = [0u8; SIZE];
    let mut index = 0;

    for i in bytes {
        buffer[index] = i;

        index += 1;
        if index == SIZE {
            writer.write_all(&buffer)?;
            index = 0;
        }
    }

    writer.write_all(&buffer[..index])?;

    Ok(())
}