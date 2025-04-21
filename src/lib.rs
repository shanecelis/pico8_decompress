#[cfg(feature = "png")]
use std::io::{self};
mod pxa_decompress;
pub use pxa_decompress::*;
/// Extract the two least significant bits from PNG RGBA frame data.
pub fn extract_bits(bytes: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(bytes.len() / 4);
    let mut accum = 0;
    for (i, (byte, offset)) in bytes.iter().zip([2,1,0,3].iter().cycle()).enumerate() {
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
