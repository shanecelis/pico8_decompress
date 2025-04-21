// mod p8_compress;
// pub use p8_compress::*;
// mod pxa_compress_snippets;
mod pxa_decompress;
pub use pxa_decompress::*;
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

#[cfg(test)]
mod tests {
    use super::*;
    fn offset(i: usize) -> usize {
        (i + 2) % 4
    }

    #[test]
    fn it_works() {
        assert_eq!(offset(0), 2);
        assert_eq!(offset(1), 1);
        assert_eq!(offset(2), 0);
        assert_eq!(offset(3), 3);
    }
}
