use base32::{encode_into, encoded_buffer_len, encoded_len};

fn main() {
    let data = b"hello world";
    let mut out = vec![0u8; encoded_buffer_len(data.len())];
    encode_into(&mut out, data);

    let encoded_str = String::from_utf8_lossy(&out[..encoded_len(data.len())]);
    println!("Encoded: {}", encoded_str);
}
