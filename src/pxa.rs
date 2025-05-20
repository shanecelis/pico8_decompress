/*
  pxa compression snippets for PICO-8 cartridge format (as of 0.2.4c)

  author: joseph@lexaloffle.com

  Copyright (c) 2020-22  Lexaloffle Games LLP
*/

use std::{fmt, cmp::min};

const PXA_MIN_BLOCK_LEN: usize = 3;
const BLOCK_LEN_CHAIN_BITS: usize = 3;
const BLOCK_DIST_BITS: usize = 5;
const TINY_LITERAL_BITS: usize = 4;

struct PxaDecompressor<'a> {
    bit: u8,
    dest_pos: usize,
    src_pos: usize,
    src_buf: &'a [u8],
    dest_buf: Vec<u8>,
    literal: [u8; 256],
    literal_pos: [u8; 256],
}

impl fmt::Debug for PxaDecompressor<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("PxaDecompressor")
            .field("bit", &self.bit)
            .field("dest_pos", &self.dest_pos)
            .field("src_pos", &self.src_pos)
            .field("src_buf", &&self.src_buf[0..=self.src_pos])
            .finish()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PxaError {
    #[error("Invalid header")]
    InvalidHeader,
    #[error("Literal overflow")]
    LiteralOverflow,
}

/// Decompress Pico8 P8 PNG compressed text data, usually Lua code.
pub fn decompress(src_buf: &[u8], max_len: Option<usize>) -> Result<Vec<u8>, PxaError> {
    PxaDecompressor::new(src_buf).decompress(max_len)
}

impl<'a> PxaDecompressor<'a> {
    fn new(src_buf: &'a [u8]) -> Self {
        let mut literal = [0; 256];
        let mut literal_pos = [0; 256];

        // Initialize literals state
        for i in 0..256 {
            literal[i] = i as u8;
            literal_pos[i] = i as u8;
        }

        PxaDecompressor {
            bit: 1,
            dest_pos: 0,
            src_pos: 0,
            src_buf,
            dest_buf: Vec::new(),
            literal,
            literal_pos,
        }
    }

    fn getbit(&mut self) -> bool {
        let ret = (self.src_buf[self.src_pos] & self.bit) != 0;
        if self.bit == 128 {
            self.bit = 1;
            self.src_pos += 1;
        } else {
            self.bit <<= 1;
        }
        ret
    }

    fn getval(&mut self, bits: usize) -> usize {
        assert!(bits <= 15, "bits were {bits}");

        let mut val = 0;
        for i in 0..bits {
            if self.getbit() {
                val |= 1 << i;
            }
        }
        val
    }

    // fn putbit(&mut self, bval: bool) {
    //     if bval {
    //         self.dest_buf[self.dest_pos] |= self.bit;
    //     } else {
    //         self.dest_buf[self.dest_pos] &= !self.bit;
    //     }
    //     if self.bit == 128 {
    //         self.bit = 1;
    //         self.dest_pos += 1;
    //         // self.byte = self.dest_buf[self.dest_pos];
    //     } else {
    //         self.bit <<= 1;
    //     }
    // }

    // fn putval(&mut self, val: usize, bits: usize) -> usize {
    //     for i in 0..bits {
    //         self.putbit((val & (1 << i)) != 0);
    //     }
    //     bits
    // }

    // fn putchain(&mut self, mut val: usize, link_bits: usize, max_bits: usize) -> usize {
    //     let max_link_val = (1 << link_bits) - 1;
    //     let mut bits_written = 0;
    //     let mut vv = max_link_val;

    //     while vv == max_link_val {
    //         vv = min(val, max_link_val);
    //         bits_written += self.putval(vv, link_bits);
    //         val -= vv;

    //         if bits_written >= max_bits {
    //             break;
    //         }
    //     }
    //     bits_written
    // }

    fn getchain(&mut self, link_bits: usize, max_bits: usize) -> usize {
        let max_link_val = (1 << link_bits) - 1;
        let mut val = 0;
        let mut vv = max_link_val;
        let mut bits_read = 0;

        while vv == max_link_val {
            vv = self.getval(link_bits);
            bits_read += link_bits;
            val += vv;
            if bits_read >= max_bits {
                // next val is implicitly 0
                break;
            }
        }
        val
    }

    fn getnum(&mut self) -> Option<usize> {
        // 1  15 bits // more frequent so put first
        // 01 10 bits
        // 00  5 bits
        let bits = (3 - self.getchain(1, 2)) * BLOCK_DIST_BITS;
        let val = self.getval(bits);

        if val == 0 && bits == 10 {
            // Raw block marker
            None
        } else {
            Some(val)
        }
    }

    pub fn decompress(&mut self, max_len: Option<usize>) -> Result<Vec<u8>, PxaError> {
        let mut header: [u8; 8] = [0; 8];
        for item in &mut header {
            *item = self.getval(8) as u8;
        }
        if header[0] != 0 || header[1] != b'p' || header[2] != b'x' || header[3] != b'a' {
            return Err(PxaError::InvalidHeader);
        }

        let raw_len = ((header[4] as usize) << 8) | header[5] as usize;
        let comp_len = ((header[6] as usize) << 8) | header[7] as usize;
        let max_len = max_len.map(|x| min(x, raw_len)).unwrap_or(raw_len);
        self.dest_buf = vec![0x00; max_len];

        while self.src_pos < comp_len && self.dest_pos < max_len {
            let block_type = self.getbit();

            if !block_type {
                let block_offset = self.getnum().map(|x| x + 1);

                if let Some(block_offset) = block_offset {

                    let mut block_len = self.getchain(BLOCK_LEN_CHAIN_BITS, 100000) + PXA_MIN_BLOCK_LEN;

                    while block_len > 0 {
                        self.dest_buf[self.dest_pos] = self.dest_buf[self.dest_pos - block_offset];
                        self.dest_pos += 1;
                        block_len -= 1;
                    }
                } else {
                    while self.dest_pos < max_len {
                        let v = self.getval(8) as u8;
                        self.dest_buf[self.dest_pos] = v;
                        if self.dest_buf[self.dest_pos] == 0 {
                            break;
                        }
                        self.dest_pos += 1;
                    }
                }
            } else {
                let mut lpos = 0;
                let mut bits = 0;
                let mut safety = 0;
                while self.getbit() && safety < 16 {
                    lpos += 1 << (TINY_LITERAL_BITS + bits);
                    bits += 1;
                    safety += 1;
                }

                bits += TINY_LITERAL_BITS;
                lpos += self.getval(bits);

                if lpos > 255 {
                    return Err(PxaError::LiteralOverflow);
                }

                let c = self.literal[lpos];

                self.dest_buf[self.dest_pos] = c;
                self.dest_pos += 1;
                // self.dest_buf[self.dest_pos] = 0;

                for i in (1..=lpos).rev() {
                    self.literal[i] = self.literal[i - 1];
                    self.literal_pos[self.literal[i] as usize] += 1;
                }
                self.literal[0] = c;
                self.literal_pos[c as usize] = 0;
            }
        }
        assert_eq!(self.dest_buf.len(), self.dest_pos);
        Ok(std::mem::take(&mut self.dest_buf))
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    const COMPRESSED_DATA: &[u8] = include_bytes!("p8png-test.p8.png");
    fn decompress_data(max_len: Option<usize>) -> Vec<u8> {
        let v = extract_bits_from_png(COMPRESSED_DATA).unwrap();
        // grab the bytes of the image.
        decompress(&v[0x4300..=0x7fff], max_len).unwrap()
    }

    // #[test]
    // fn test_decompress2() {
    //     let code_u8 = decompress_data(Some(2));
    //     let code = std::str::from_utf8(&code_u8).unwrap();
    //     let lines: Vec<_> = code.lines().collect();
    //     assert_eq!("--", lines[0]);
    // }

    #[test]
    fn test_decompress3() {
        let code_u8 = decompress_data(Some(3));
        let code = std::str::from_utf8(&code_u8).unwrap();
        let lines: Vec<_> = code.lines().collect();
        assert_eq!("-- ", lines[0]);
    }

}
