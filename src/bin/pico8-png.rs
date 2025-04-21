use std::{
    io::{self, Write},
    process,
    fs::File,
    env
};
use pico8_pxa::*;

use png;

fn main() -> io::Result<()> {
    let args = env::args();
    let Some(arg) = args.skip(1).next() else {
        // usage(std::io::stderr())?;
        process::exit(2);
    };
    let v = extract_bits_from_png(File::open(arg)?)?;
    // Grab the bytes of the image.
    let mut out = io::stdout();
    // let mut code = vec![];
    let code = decompress(&v[0x4300..=0x7fff], None).unwrap();
    // eprintln!("size {}", code.len());
    out.write(&code);
    // out.write(&v[..]);
    Ok(())
}
