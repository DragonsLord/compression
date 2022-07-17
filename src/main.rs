use std::{env, error, process};

use compression::compress;

pub mod compression;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filepath = args.get(1).unwrap_or_else(|| {
        eprintln!("filepath is not provided");
        process::exit(1);
    });

    let file_bytes = compress(filepath)?;

    for b in file_bytes {
        print!(" {}", b);
    }

    return Ok(());
}