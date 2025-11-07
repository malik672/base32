//Partial Fork from https://github.com/andreasots/base32 @ 58909ac.


// Crockford's Base32 alphabet (https://www.crockford.com/base32.html) with lowercase
// alphabetical characters. We also don't decode permissively.
const ALPHABET: &[u8] = b"0123456789abcdefghjkmnpqrstvwxyz";

pub const fn encoded_len(len: usize) -> usize {
    let last_chunk = match len % 5 {
        0 => 0,
        1 => 2,
        2 => 4,
        3 => 5,
        4 => 7,
        _ => unreachable!(),
    };
    (len / 5) * 8 + last_chunk
}

#[cfg(target_arch = "x86_64")]
pub fn encoded_len(len: usize) -> usize {
    unsafe {
        let result: usize;
        std::arch::asm!(
            "lea rcx, [rip + 2f]",  // Load address of the lookup table
            "movabs rdx, 0x3333333333333333", // Load magic constant for division by 5
            "mov rax, {len}", // move len into rax
            "mul rdx", // mul rax by rdx, result in rdx:rax(that's len * 0x3333333333333333)
            "lea rax, [rdx + 4*rdx]", // rax = 5 * (len / 5)
            "sub {len}, rax",
            "lea rax, [8*rdx]",  // rax = 8 * (len / 5)
            "or rax, qword ptr [rcx + 8*{len}]", // rax |= table[len % 5]
            "jmp 3f", // Jump over the data table
            "2:", // Table label
            ".quad 0, 2, 4, 5, 7", // Lookup table: [0, 2, 4, 5, 7]
            "3:",  // End label
            len = inout(reg) len => _,
            out("rax") result,
            out("rcx") _,
            out("rdx") _,
            options(nostack)
        );
        result
    }
}


pub const fn encoded_buffer_len(len: usize) -> usize {
    len.div_ceil(5) * 8
}

/// Writes the base32-encoding of `data` into `out`, which should have length at
/// least `encoded_buffer_len(data.len())`. Only the first
/// `encoded_len(data.len())` bytes of `out` should be used.
#[inline]
pub fn encode_into(out: &mut [u8], data: &[u8]) {
    // Process the input in chunks of length 5 (i.e 40 bits), potentially padding
    // the last chunk with zeros for now.
    for (chunk, out_chunk) in data.chunks(5).zip(out.chunks_mut(8)) {
        let block = chunk.try_into().unwrap_or_else(|_| {
            // Zero-extend the last chunk if necessary
            let mut block = [0u8; 5];
            block[..chunk.len()].copy_from_slice(chunk);
            block
        });

        // Turn our block of 5 groups of 8 bits into 8 groups of 5 bits.
        #[inline]
        fn alphabet(index: u8) -> u8 {
            ALPHABET[index as usize]
        }
        out_chunk[0] = alphabet((block[0] & 0b1111_1000) >> 3);
        out_chunk[1] = alphabet((block[0] & 0b0000_0111) << 2 | ((block[1] & 0b1100_0000) >> 6));
        out_chunk[2] = alphabet((block[1] & 0b0011_1110) >> 1);
        out_chunk[3] = alphabet((block[1] & 0b0000_0001) << 4 | ((block[2] & 0b1111_0000) >> 4));
        out_chunk[4] = alphabet((block[2] & 0b0000_1111) << 1 | (block[3] >> 7));
        out_chunk[5] = alphabet((block[3] & 0b0111_1100) >> 2);
        out_chunk[6] = alphabet((block[3] & 0b0000_0011) << 3 | ((block[4] & 0b1110_0000) >> 5));
        out_chunk[7] = alphabet(block[4] & 0b0001_1111);
    }
}

pub fn encode(data: &[u8]) -> String {
    let mut out = vec![0; encoded_buffer_len(data.len())];
    encode_into(&mut out, data);
    // Truncate any extra zeros we added on the last block.
    out.truncate(encoded_len(data.len()));
    String::from_utf8(out).unwrap()
}

#[derive(Debug, PartialEq)]
pub struct InvalidBase32Error {
    pub character: char,
    pub position: usize,
    pub string: String,
}

pub fn decode(data: &str) -> Result<Vec<u8>, InvalidBase32Error> {
    let data_bytes = data.as_bytes();
    let out_length = data_bytes.len() * 5 / 8;
    let mut out = Vec::with_capacity(out_length.div_ceil(5) * 5);

    // Process the data in 8 byte chunks, reversing the encoding process.
    for chunk in data_bytes.chunks(8) {
        let mut indexes = [0u8; 8];
        for (i, byte) in chunk.iter().enumerate() {
            // Invert the alphabet mapping to recover `indexes`.
            let offset = match *byte {
                b'0'..=b'9' => b'0',
                b'a'..=b'h' => b'a' - 10,
                b'j'..=b'k' => b'a' - 10 + 1,
                b'm'..=b'n' => b'a' - 10 + 2,
                b'p'..=b't' => b'a' - 10 + 3,
                b'v'..=b'z' => b'a' - 10 + 4,
                _ => {
                    // Recover the index within `data_bytes`
                    let position = i + chunk.as_ptr().addr() - data_bytes.as_ptr().addr();
                    return Err(InvalidBase32Error {
                        character: data[position..].chars().next().unwrap_or_else(|| {
                            panic!("Checked characters 0..{position} in {data} were one-byte")
                        }),
                        position,
                        string: data.to_string(),
                    });
                },
            };
            indexes[i] = byte - offset;
        }
        // Regroup our block of 8 5-bit indexes into 5 output bytes.
        out.push((indexes[0] << 3) | (indexes[1] >> 2));
        out.push((indexes[1] << 6) | (indexes[2] << 1) | (indexes[3] >> 4));
        out.push((indexes[3] << 4) | (indexes[4] >> 1));
        out.push((indexes[4] << 7) | (indexes[5] << 2) | (indexes[6] >> 3));
        out.push((indexes[6] << 5) | indexes[7]);
    }

    // Truncate any extra output from our last chunk.
    out.truncate(out_length);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base32_roundtrips() {
        let data = b"hello world";
        let encoded = encode(data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_invalid_base32_error() {
        assert_eq!(
            decode("01234567ë").unwrap_err(),
            InvalidBase32Error {
                character: 'ë',
                position: 8,
                string: "01234567ë".into()
            }
        );
    }
}
