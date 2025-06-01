use alloc::vec::Vec;

fn base64_char_value(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub fn base64_decode_string(input: &str) -> Option<Vec<u8>> {
    base64_decode(input.as_bytes())
}

pub fn base64_decode(input: &[u8]) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits_collected = 0;

    for &byte in input.iter().filter(|&&c| c != b'=') {
        let val = base64_char_value(byte)?;
        buf = (buf << 6) | (val as u32);
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            output.push((buf >> bits_collected) as u8);
            buf &= (1 << bits_collected) - 1;
        }
    }

    Some(output)
}

pub fn base64_encode(input: &[u8]) -> Vec<u8> {
    const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::with_capacity((input.len() + 2) / 3 * 4);

    let mut i = 0;
    while i < input.len() {
        let b0 = input[i];
        let b1 = if i + 1 < input.len() { input[i + 1] } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] } else { 0 };

        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);

        output.push(BASE64_CHARS[((triple >> 18) & 0x3F) as usize]);
        output.push(BASE64_CHARS[((triple >> 12) & 0x3F) as usize]);

        if i + 1 < input.len() {
            output.push(BASE64_CHARS[((triple >> 6) & 0x3F) as usize]);
        } else {
            output.push(b'=');
        }

        if i + 2 < input.len() {
            output.push(BASE64_CHARS[(triple & 0x3F) as usize]);
        } else {
            output.push(b'=');
        }

        i += 3;
    }

    output
}