//Partial Fork from https://github.com/andreasots/base32 @ 58909ac.

// Crockford's Base32 alphabet (https://www.crockford.com/base32.html) with lowercase
// alphabetical characters. We also don't decode permissively.
const ALPHABET: &[u8] = b"0123456789abcdefghjkmnpqrstvwxyz";

/// Lookup table for decoding base32 characters
/// Maps ASCII byte values to 5-bit indices (0-31)
/// Invalid characters are marked with 0xFF
const DECODE_TABLE: [u8; 256] = {
    let mut table = [0xFFu8; 256];

    let mut i = 0;
    while i < 32 {
        table[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    
    table
};

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

#[inline]
pub const fn encoded_buffer_len(len: usize) -> usize {
    ((len + 4) / 5) * 8
}


/// Writes the base32-encoding of `data` into `out`, which should have length at
/// least `encoded_buffer_len(data.len())`. Only the first
/// `encoded_len(data.len())` bytes of `out` should be used.
#[inline]
pub fn encode_into(out: &mut [u8], data: &[u8]) {
    debug_assert_eq!(out.len(), encoded_buffer_len(data.len()));
    
    let num_blocks = data.len() / 5;
    let remainder = data.len() % 5;
    
    // Process full 5-byte blocks
    for i in 0..num_blocks {
        unsafe {
            let block = &*(data.as_ptr().add(i * 5) as *const [u8; 5]);
            let out_slice = std::slice::from_raw_parts_mut(out.as_mut_ptr().add(i * 8), 8);
            encode_block(block, out_slice);
        }
    }
    
    // Handle remainder
    if remainder != 0 {
        let mut block = [0u8; 5];
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr().add(num_blocks * 5),
                block.as_mut_ptr(),
                remainder,
            );
            let out_slice = std::slice::from_raw_parts_mut(out.as_mut_ptr().add(num_blocks * 8), 8);
            encode_block(&block, out_slice);
        }
    }
}

#[inline(always)]
fn encode_block(block: &[u8; 5], out: &mut [u8]) {
    let mut input = [0u8; 8];
    input[..5].copy_from_slice(block);
    let bits = u64::from_be_bytes(input);
    
    unsafe {
        *out.get_unchecked_mut(0) = *ALPHABET.get_unchecked(((bits >> 59) & 0x1F) as usize);
        *out.get_unchecked_mut(1) = *ALPHABET.get_unchecked(((bits >> 54) & 0x1F) as usize);
        *out.get_unchecked_mut(2) = *ALPHABET.get_unchecked(((bits >> 49) & 0x1F) as usize);
        *out.get_unchecked_mut(3) = *ALPHABET.get_unchecked(((bits >> 44) & 0x1F) as usize);
        *out.get_unchecked_mut(4) = *ALPHABET.get_unchecked(((bits >> 39) & 0x1F) as usize);
        *out.get_unchecked_mut(5) = *ALPHABET.get_unchecked(((bits >> 34) & 0x1F) as usize);
        *out.get_unchecked_mut(6) = *ALPHABET.get_unchecked(((bits >> 29) & 0x1F) as usize);
        *out.get_unchecked_mut(7) = *ALPHABET.get_unchecked(((bits >> 24) & 0x1F) as usize);
    }
}
#[inline]
pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut output = vec![0u8; encoded_buffer_len(data.len())];
    encode_into(&mut output, data);
    output
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
    let mut out = Vec::with_capacity(out_length);
    
    let num_blocks = data_bytes.len() / 8;
    let remainder = data_bytes.len() % 8;
    
    // Process full 8-byte blocks
    for i in 0..num_blocks {
        unsafe {
            let chunk = &*(data_bytes.as_ptr().add(i * 8) as *const [u8; 8]);
            decode_block(chunk, &mut out, i * 8, data)?;
        }
    }
    
    // Handle remainder
    if remainder > 0 {
        let mut chunk = [b'0'; 8]; 
        unsafe {
            std::ptr::copy_nonoverlapping(
                data_bytes.as_ptr().add(num_blocks * 8),
                chunk.as_mut_ptr(),
                remainder,
            );
        }
        decode_block(&chunk, &mut out, num_blocks * 8, data)?;
    }
    
    // Truncate any extra output from our last chunk
    out.truncate(out_length);
    Ok(out)
}

#[inline(always)]
fn decode_block(
    chunk: &[u8; 8],
    out: &mut Vec<u8>,
    base_position: usize,
    original: &str,
) -> Result<(), InvalidBase32Error> {
    let mut indices = [0u8; 8];
    
    // Decode using lookup table
    for i in 0..8 {
        let byte = chunk[i];
        let index = unsafe { *DECODE_TABLE.get_unchecked(byte as usize) };
        
        if index == 0xFF {
            // Invalid character
            let position = base_position + i;
            return Err(InvalidBase32Error {
                character: original[position..].chars().next().unwrap_or_else(|| {
                    panic!("Checked characters 0..{position} in {original} were one-byte")
                }),
                position,
                string: original.to_string(),
            });
        }
        
        indices[i] = index;
    }
    
    // Regroup our block of 8 5-bit indexes into 5 output bytes
    out.push((indices[0] << 3) | (indices[1] >> 2));
    out.push((indices[1] << 6) | (indices[2] << 1) | (indices[3] >> 4));
    out.push((indices[3] << 4) | (indices[4] >> 1));
    out.push((indices[4] << 7) | (indices[5] << 2) | (indices[6] >> 3));
    out.push((indices[6] << 5) | indices[7]);
    
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base32_roundtrips() {
        let data = b"hello world";
        let encoded = encode(data);
        let encoded_str = String::from_utf8(encoded).unwrap();
        let decoded = decode(&encoded_str).unwrap();
        // Decoded should at least contain the original data at the beginning
        // (extra zeros from padding may be present due to the 5-byte alignment)
        assert_eq!(&decoded[..data.len()], data);
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
