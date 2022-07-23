use std::{env, error, process};

pub mod compression;

use compression::compress_file;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();
    let filepath = args.get(1).unwrap_or_else(|| {
        eprintln!("filepath is not provided");
        process::exit(1);
    });

    // let filepath = "assets/pic.jpg";

    let file_bytes = compress_file(filepath)?;

    for b in file_bytes {
        println!("{}", b);
    }

    return Ok(());
}