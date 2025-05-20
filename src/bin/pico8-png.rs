use std::{
    io::{self, Write},
    process,
    fs::File,
    env
};
use pico8_decompress::*;

fn main() -> io::Result<()> {
    let mut args = env::args();
    let Some(arg) = args.nth(1) else {
        // usage(std::io::stderr())?;
        process::exit(2);
    };
    let v = extract_bits_from_png(File::open(arg)?)?;
    // Grab the bytes of the image.
    let mut out = io::stdout();
    // let mut code = vec![];
    let code = pxa::decompress(&v[0x4300..=0x7fff], None).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    // eprintln!("size {}", code.len());
    let _ = out.write(&code)?;
    // out.write(&v[..]);
    Ok(())
}
