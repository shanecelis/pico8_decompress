/*
  (c) Copyright 2014-2016 Lexaloffle Games LLP
  author: joseph@lexaloffle.com

  compression used in code section of .p8.png format

  This software is provided 'as-is', without any express or implied
  warranty. In no event will the authors be held liable for any damages
  arising from the use of this software.

  Permission is granted to anyone to use this software for any purpose,
  including commercial applications, and to alter it and redistribute it
  freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not
  claim that you wrote the original software. If you use this software
  in a product, an acknowledgment in the product documentation would be
  appreciated but is not required.
  2. Altered source versions must be plainly marked as such, and must not be
  misrepresented as being the original software.
  3. This notice may not be removed or altered from any source distribution.

*/

const LITERALS: usize = 60;

const FUTURE_CODE: &str = "if(_update60)_update=function()_update60()_update60()end";
const FUTURE_CODE2: &str =
    "if(_update60)_update=function()_update60()_update_buttons()_update60()end";

const LITERAL: &str = "^\n 0123456789abcdefghijklmnopqrstuvwxyz!#%(){}[]<>+=/*:;.,~_";

#[derive(thiserror::Error, Debug)]
pub enum P8Error {
    #[error("Invalid block reference")]
    InvalidBlock,
    #[error("Decompressed length exceeds output buffer")]
    OutputExceeded,
    #[error("Unexpected end of input")]
    EndOfInput,
}

// decompresses the mini format used in .p8.png code sections
pub fn decompress(input: &[u8], output: &mut [u8]) -> Result<usize, P8Error> {
    let mut in_pos = 0;

    macro_rules! read_val {
        () => {{
            if in_pos >= input.len() {
                return Err(P8Error::EndOfInput);
            }
            let v = input[in_pos];
            in_pos += 1;
            v
        }};
    }

    // skip 4-byte header ":c:"
    for _ in 0..4 {
        read_val!();
    }

    // read uncompressed length (big endian)
    let mut len = (read_val!() as usize) << 8;
    len += read_val!() as usize;

    // skip compressed length (2 bytes, unused)
    read_val!();
    read_val!();

    if len > output.len() {
        return Err(P8Error::OutputExceeded);
    }

    let mut out_pos = 0;

    while out_pos < len {
        let val = read_val!();

        if (val as usize) < LITERALS {
            if val == 0 {
                output[out_pos] = read_val!();
            } else {
                output[out_pos] = LITERAL
                    .as_bytes()
                    .get(val as usize)
                    .copied()
                    .unwrap_or(b'?'); // fallback character
            }
            out_pos += 1;
        } else {
            let mut block_offset = (val as usize - LITERALS) * 16;
            let val2 = read_val!();
            block_offset += (val2 % 16) as usize;
            let block_length = (val2 / 16) as usize + 2;

            if block_offset == 0 || out_pos < block_offset || out_pos + block_length > output.len()
            {
                return Err(P8Error::InvalidBlock);
            }

            let (src, dst) = output.split_at_mut(out_pos);
            let (from, _to) = dst.split_at_mut(block_length);
            from.copy_from_slice(
                &src[out_pos - block_offset..out_pos - block_offset + block_length],
            );
            out_pos += block_length;
        }
    }

    // Strip FUTURE_CODE or FUTURE_CODE2 from the end if present
    let mut final_len = out_pos;

    let as_str = std::str::from_utf8(&output[..final_len]).unwrap_or("");

    if let Some(pos) = as_str.find(FUTURE_CODE) {
        if pos + FUTURE_CODE.len() == final_len {
            final_len = pos;
        }
    }
    if let Some(pos) = as_str.find(FUTURE_CODE2) {
        if pos + FUTURE_CODE2.len() == final_len {
            final_len = pos;
        }
    }

    output[final_len] = 0; // null-terminate if treating as C string

    Ok(final_len)
}
