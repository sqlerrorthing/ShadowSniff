use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use embedded_io::{Read, Seek};
use utils::io::cursor::ByteCursor;
use crate::sqlite::db::DbHeader;
use crate::sqlite::page::Page;

pub(super) const HEADER_SIZE: usize = 100;
const HEADER_PREFIX: &[u8] = b"SQLite format 3\0";
const HEADER_PAGE_SIZE_OFFSET: usize = 16;
const PAGE_MAX_SIZE: u32 = 65536;

pub(super) struct Pager<I: Read + Seek = ByteCursor> {
    input: Arc<I>,
    page_size: usize,
    pages: Arc<BTreeMap<usize, Arc<Page>>>
}

impl<I: Read + Seek> Pager<I> {
    pub fn new(input: I, page_size: usize) -> Self {
        Self {
            input: Arc::new(input),
            page_size,
            pages: Arc::default()
        }
    }
}

pub(super) fn parse_header(buffer: &[u8]) -> Option<DbHeader> {
    if !buffer.starts_with(HEADER_PREFIX) {
        return None;
    }
    
    let page_size_raw = read_be_word_at(buffer, HEADER_PAGE_SIZE_OFFSET);
    let page_size = match page_size_raw {
        1 => PAGE_MAX_SIZE,
        n if n.is_power_of_two() => n as u32,
        _ => return None
    };
    
    Some(DbHeader { page_size })
}

fn read_be_word_at(input: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(input[offset..offset + 2].try_into().unwrap())
}

fn read_be_double_at(input: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(input[offset..offset + 4].try_into().unwrap())
}