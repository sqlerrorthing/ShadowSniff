use crate::ZipArchive;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use rand_chacha::rand_core::RngCore;
use rand_chacha::ChaCha20Rng;
use utils::random::ChaCha20RngExt;

pub(super) fn create_zip(archive: &ZipArchive) -> Vec<u8> {
    let mut zip_data = Vec::new();
    let mut central_directory = Vec::new();
    let mut offset = 0;

    for entry in &archive.entries {
        let (compression_method, mut compressed) = (
            archive.compression.method(),
            archive.compression.compress(&entry.data)
        );

        let crc = crc32(&entry.data);
        let path_bytes = entry.path.as_bytes();

        let (encryption_header, general_flag) = protect_data(crc, &mut compressed, archive.password.as_ref())
            .unwrap_or((vec![], 0));

        let compressed_size = encryption_header.len() + compressed.len();

        let local_header = create_local_header(
            crc,
            general_flag,
            compression_method,
            entry.modified,
            compressed_size,
            entry.data.len(),
            path_bytes
        );

        zip_data.extend(&local_header);
        zip_data.extend(&encryption_header);
        zip_data.extend(&compressed);

        let central_header = create_central_header(
            crc,
            general_flag,
            compression_method,
            entry.modified,
            compressed_size,
            entry.data.len(),
            path_bytes,
            offset
        );

        central_directory.extend(&central_header);
        offset += local_header.len() + compressed_size;
    }

    let central_offset = zip_data.len();
    zip_data.extend(&central_directory);

    let eocd = create_end_of_central_directory(
        archive.entries.len(),
        central_directory.len(),
        central_offset,
        archive.comment.as_ref()
    );

    zip_data.extend(eocd);

    zip_data
}

fn protect_data(
    crc: u32,
    payload: &mut Vec<u8>,
    password: Option<&String>
) -> Option<(Vec<u8>, u16)> {
    if let Some(password) = password {
        let (mut k0, mut k1, mut k2) = init_keys(password);
        let header = gen_encryption_header(crc, &mut k0, &mut k1, &mut k2);

        for byte in payload {
            let plain = *byte;
            let cipher = plain ^ decrypt_byte(k2);
            *byte = cipher;
            update_keys(plain, &mut k0, &mut k1, &mut k2);
        }

        Some((header.to_vec(), 0x01))
    } else {
        None
    }
}

macro_rules! extend {
    ($($data:expr),+ $(,)?) => {{
        let mut extended = Vec::new();

        $(
            extended.extend($data);
        )+

        extended
    }};
}

fn create_local_header(
    crc: u32,
    general_flag: u16,
    compression_method: u16,
    modified: (u16, u16),
    compressed_len: usize,
    data_len: usize,
    path: &[u8]
) -> Vec<u8> {
    extend!(
        [0x50, 0x4B, 0x03, 0x04],
        20u16.to_le_bytes(),
        general_flag.to_le_bytes(),
        compression_method.to_le_bytes(),
        modified.0.to_le_bytes(),
        modified.1.to_le_bytes(),
        crc.to_le_bytes(),
        (compressed_len as u32).to_le_bytes(),
        (data_len as u32).to_le_bytes(),
        (path.len() as u16).to_le_bytes(),
        0u16.to_le_bytes(),
        path,
    )
}

#[allow(clippy::too_many_arguments)]
fn create_central_header(
    crc: u32,
    general_flag: u16,
    compression_method: u16,
    modified: (u16, u16),
    compressed_len: usize,
    data_len: usize,
    path: &[u8],
    offset: usize
) -> Vec<u8> {
    extend!(
        [0x50, 0x4B, 0x01, 0x02],
        20u16.to_le_bytes(),
        20u16.to_le_bytes(),
        general_flag.to_le_bytes(),
        compression_method.to_le_bytes(),
        modified.0.to_le_bytes(),
        modified.1.to_le_bytes(),
        crc.to_le_bytes(),
        (compressed_len as u32).to_le_bytes(),
        (data_len as u32).to_le_bytes(),
        (path.len() as u16).to_le_bytes(),
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        [0, 0, 0, 0],
        (offset as u32).to_le_bytes(),
        path
    )
}

fn create_end_of_central_directory(
    entries_len: usize,
    central_size: usize,
    central_offset: usize,
    comment: Option<&String>
) -> Vec<u8> {
    let mut vec = extend!(
        [0x50, 0x4B, 0x05, 0x06],
        0u16.to_le_bytes(),
        0u16.to_le_bytes(),
        (entries_len as u16).to_le_bytes(),
        (entries_len as u16).to_le_bytes(),
        (central_size as u32).to_le_bytes(),
        (central_offset as u32).to_le_bytes()
    );

    if let Some(comment) = comment {
        let comment = comment.as_bytes();
        vec.extend(&(comment.len() as u16).to_le_bytes());
        vec.extend(comment);
    } else {
        vec.extend(0u16.to_le_bytes());
    }

    vec
}

fn crc32(data: &[u8]) -> u32 {
    let polynomial: u32 = 0xEDB88320;
    let mut crc: u32 = 0xFFFFFFFF;

    for &byte in data {
        let current_byte = byte as u32;
        crc ^= current_byte;
        for _ in 0..8 {
            let mask = if crc & 1 != 0 { polynomial } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }

    !crc
}

fn init_keys<S>(password: &S) -> (u32, u32, u32)ццццццц
where
    S: AsRef<str> + ?Sized
{
    let mut k0 = 0x12345678;
    let mut k1 = 0x23456789;
    let mut k2 = 0x34567890;

    for b in password.as_ref().bytes() {
        update_keys(b, &mut k0, &mut k1, &mut k2);
    }

    (k0, k1, k2)
}

fn update_keys(byte: u8, k0: &mut u32, k1: &mut u32, k2: &mut u32) {
    *k0 = crc32_byte(*k0, byte);
    *k1 = (*k1).wrapping_add(*k0 & 0xFF);
    *k1 = (*k1).wrapping_mul(134775813).wrapping_add(1);
    *k2 = crc32_byte(*k2, (*k1 >> 24) as u8);
}

fn crc32_byte(crc: u32, b: u8) -> u32 {
    let mut c = crc ^ (b as u32);
    for _ in 0..8 {
        c = if c & 1 != 0 {
            0xEDB88320 ^ (c >> 1)
        } else {
            c >> 1
        };
    }

    c
}

fn decrypt_byte(k2: u32) -> u8 {
    let temp = (k2 & 0xFFFF) | 0x0002;
    ((temp * (temp ^ 1)) >> 8) as u8
}

fn gen_encryption_header(crc: u32, k0: &mut u32, k1: &mut u32, k2: &mut u32) -> [u8; 12] {
    let mut header = [0u8; 12];
    let mut rng = ChaCha20Rng::from_nano_time();

    for i in header.iter_mut().take(11) {
        let plain = rng.next_u32() as u8;
        *i = plain ^ decrypt_byte(*k2);
        update_keys(plain, k0, k1, k2);
    }

    let final_plain = (crc >> 24) as u8;

    header[11] = final_plain ^ decrypt_byte(*k2);
    update_keys(final_plain, k0, k1, k2);

    header
}