#![doc(html_root_url = "https://docs.rs/pico8_decompress/0.1.0")]
#![doc = include_str!("../README.md")]
#![forbid(missing_docs)]
#[cfg(feature = "png")]
use std::io::{self};
pub mod p8;
pub mod pxa;

/// Extract the two least significant bits from PNG RGBA frame data.
pub fn extract_bits(bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(bytes.len() / 4);
    let mut accum = 0;
    for (i, (byte, offset)) in bytes.iter().zip([2, 1, 0, 3].iter().cycle()).enumerate() {
        let semi_nybble_index = i % 4;
        let semi_nybble = *byte & 0b11;
        accum |= semi_nybble << (offset * 2);
        if semi_nybble_index == 3 {
            v.push(accum);
            accum = 0;
        }
    }
    v
}

/// Compression kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Compression {
    /// The latest compression scheme
    Pxa,
    /// The previous compression scheme
    P8,
    /// First and probably no compression
    Legacy,
}

/// Errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// P8 error
    #[error("p8 decompression error: {0}")]
    P8(#[from] p8::P8Error),
    /// PXA error
    #[error("pxa decompression error: {0}")]
    Pxa(#[from] pxa::PxaError),
}

fn compression_header(src_buf: &[u8]) -> Compression {
    if src_buf[0] == 0 || src_buf[1] == b'p' || src_buf[2] == b'x' || src_buf[3] == b'a' {
        Compression::Pxa
    } else if src_buf[0] == b':' || src_buf[1] == b'c' || src_buf[2] == b':' || src_buf[3] == 0 {
        Compression::P8
    } else {
        Compression::Legacy
    }
}

/// Decompress bytes using header to determine if it is Pxa or P8 compression.
pub fn decompress(src_buf: &[u8], max_len: Option<usize>) -> Result<Vec<u8>, Error> {
    match compression_header(src_buf) {
        Compression::Pxa => Ok(pxa::decompress(src_buf, max_len)?),
        Compression::P8 => {
            // No max length?
            let mut output = vec![0; 65536];
            let size = p8::decompress(src_buf, &mut output)?;
            output.truncate(size);
            Ok(output)
        }
        Compression::Legacy => todo!(),
    }
}

/// Extract two least significant bits from PNG file contents directly.
#[cfg(feature = "png")]
pub fn extract_bits_from_png(png: impl io::Read) -> io::Result<Vec<u8>> {
    let decoder = png::Decoder::new(png);
    let mut reader = decoder.read_info()?;
    // Allocate the output buffer.
    let mut buf = vec![0; reader.output_buffer_size()];
    // Read the next frame. An APNG might contain multiple frames.
    let _info = reader.next_frame(&mut buf)?;
    Ok(extract_bits(&buf))
}

#[cfg(test)]
mod tests {

    fn offset(i: usize) -> usize {
        let v = i % 4;
        v ^ ((!v & 1) << 1)
    }

    #[test]
    fn offset_works() {
        assert_eq!(offset(0), 2); // 0b00 -> 0b10
        assert_eq!(offset(1), 1); // 0b01 -> 0b01
        assert_eq!(offset(2), 0); // 0b10 -> 0b00
        assert_eq!(offset(3), 3); // 0b11 -> 0b11
        assert_eq!(offset(4), 2);
    }
}
