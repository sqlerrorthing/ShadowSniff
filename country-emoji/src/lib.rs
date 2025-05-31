#![no_std]

extern crate alloc;

use alloc::string::String;

const FLAG_MAGIC_NUMBER: u32 = 0x1F1E6 /* 🇦 */ - 'A' as u32;

pub fn internal_code_to_flag<'a>(code: &str) -> Option<String> {
    let mut flag = String::new();
    
    for ch in code.trim().to_uppercase().chars() {
        if let Some(c) = char::from_u32(ch as u32 + FLAG_MAGIC_NUMBER) {
            flag.push(c);
        } else {
            return None;
        }
    }

    Some(flag)
}