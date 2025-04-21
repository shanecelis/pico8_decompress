use std::{
    io::{self, Write},
    process,
    fs::File,
    env
};
use pico8_png::*;

use png;

fn main() -> io::Result<()> {
    let args = env::args();
    let Some(arg) = args.skip(1).next() else {
        // usage(std::io::stderr())?;
        process::exit(2);
    };
    // The decoder is a build for reader and can be used to set various decoding options
    // via `Transformations`. The default output transformation is `Transformations::IDENTITY`.
    let decoder = png::Decoder::new(File::open(arg)?);
    let mut reader = decoder.read_info()?;
    // Allocate the output buffer.
    let mut buf = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let info = reader.next_frame(&mut buf)?;
    eprintln!("info {info:?} buf size {}", buf.len());
    let v = extract_bits(&buf);
    // Grab the bytes of the image.
    let mut out = io::stdout();
    // let mut code = vec![];
    let code = decompress(&v[0x4300..=0x7fff]).unwrap();
    eprintln!("size {}", code.len());
    out.write(&code);
    // out.write(&v[..]);
    Ok(())
}
