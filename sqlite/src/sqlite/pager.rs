use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicPtr, Ordering};
use embedded_io::{Read, Seek, SeekFrom};
use utils::io::cursor::ByteCursor;
use crate::sqlite::db::DbHeader;
use crate::sqlite::page::{Cell, Page, PageHeader, PageType, TableInteriorCell, TableLeafCell};

pub(super) const HEADER_SIZE: usize = 100;
const HEADER_PREFIX: &[u8] = b"SQLite format 3\0";
const HEADER_PAGE_SIZE_OFFSET: usize = 16;
const PAGE_MAX_SIZE: u32 = 65536;

const PAGE_LEAF_TABLE_ID: u8 = 0x0d;
const PAGE_INTERIOR_TABLE_ID: u8 = 0x05;

const PAGE_FIRST_FREEBLOCK_OFFSET: usize = 1;
const PAGE_CELL_COUNT_OFFSET: usize = 3;
const PAGE_CELL_CONTENT_OFFSET: usize = 5;
const PAGE_FRAGMENTED_BYTES_COUNT_OFFSET: usize = 7;
const PAGE_RIGHTMOST_POINTER_OFFSET: usize = 8;

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

    pub fn read_page(&mut self, index: usize) -> Option<Arc<Page>> {
        if let Some(page) = self.pages.get(&n) {
            return Some(page.clone());
        }

        let page = self.load_page(index)?;
        let mut new_map = (*self.pages).clone();
        new_map.insert(index, page.clone());
        self.pages = Arc::new(new_map);

        Some(page)
    }

    fn load_page(&mut self, index: usize) -> Option<Arc<Page>> {
        let offset = index.saturating_sub(1) * self.page_size;

        self.input.seek(SeekFrom::Start(offset as u64)).ok()?;

        let mut buffer = vec![0; self.page_size];
        self.input.read_exact(&mut buffer).ok()?;

        Some(Arc::new(parse_page(&buffer, index)?))
    }
}

fn parse_page(buffer: &[u8], index: usize) -> Option<Page> {
    let ptr_offset = if index == 1 { HEADER_SIZE as u16 } else { 0 };
    let content_buffer = &buffer[ptr_offset as usize..];
    let header = parse_page_header(content_buffer)?;
    let cell_pointers = parse_cell_pointers(
        &content_buffer[header.byte_size()..],
        header.cell_count as usize,
        ptr_offset
    );

    let cells_parsing_fn = match header.page_type {
        PageType::TableLeaf => parse_page_leaf_cell,
        PageType::TableInterior => parse_page_interior_cell,
    };

    let cells = parse_cells(content_buffer, &cell_pointers, cells_parsing_fn)?;

    Some(Page {
        header,
        cell_pointers,
        cells
    })
}

fn parse_cells(
    buffer: &[u8],
    cell_pointers: &[u16],
    parse_fn: impl Fn(&[u8]) -> Option<Cell>,
) -> Option<Vec<Cell>> {
    cell_pointers
        .iter()
        .map(|&ptr| parse_fn(&buffer[ptr as usize..]))
        .collect()
}

fn parse_page_leaf_cell(mut buffer: &[u8]) -> Option<Cell> {
    let (n, size) = read_varint_at(buffer, 0);
    buffer = &buffer[n as usize..];

    let (n, row_id) = read_varint_at(buffer, 0);
    buffer = &buffer[n as usize..];

    let payload = buffer[..size as usize].to_vec();

    Some(TableLeafCell {
        size,
        row_id,
        payload,
    }.into())
}

fn parse_page_interior_cell(mut buffer: &[u8]) -> Option<Cell> {
    let left_child_page = read_be_double_at(buffer, 0);
    buffer = &buffer[4..];

    let (_, key) = read_varint_at(buffer, 0);

    Some(TableInteriorCell {
        left_child_page,
        key,
    }.into())
}

pub fn read_varint_at(buffer: &[u8], mut offset: usize) -> (u8, i64) {
    let mut size = 0;
    let mut result = 0;

    while size < 8 && buffer[offset] >= 0b1000_0000 {
        result |= ((buffer[offset] as i64) & 0b0111_1111) << (7 * size);
        offset += 1;
        size += 1;
    }

    result |= (buffer[offset] as i64) << (7 * size);

    (size + 1, result)
}

fn parse_cell_pointers(buffer: &[u8], num: usize, ptr_offset: u16) -> Vec<u16> {
    let mut pointers = Vec::with_capacity(num);
    for i in 0..num {
        pointers.push(read_be_word_at(buffer, 2 * i) - ptr_offset)
    }

    pointers
}

fn parse_page_header(buffer: &[u8]) -> Option<PageHeader> {
    let (page_type, rightmost_ptr) = match buffer[0] {
        PAGE_LEAF_TABLE_ID => (PageType::TableLeaf, false),
        PAGE_INTERIOR_TABLE_ID => (PageType::TableInterior, true),
        _ => return None
    };

    let first_free_block = read_be_word_at(buffer, PAGE_FIRST_FREEBLOCK_OFFSET);
    let cell_count = read_be_word_at(buffer, PAGE_CELL_COUNT_OFFSET);
    let cell_content_offset = match read_be_word_at(buffer, PAGE_CELL_CONTENT_OFFSET) {
        0 => 65536,
        n => n as u32,
    };
    let fragmented_bytes_count = buffer[PAGE_FRAGMENTED_BYTES_COUNT_OFFSET];

    let rightmost_pointer = if rightmost_ptr {
        Some(read_be_double_at(buffer, PAGE_RIGHTMOST_POINTER_OFFSET))
    } else {
        None
    };

    Some(PageHeader {
        page_type,
        first_free_block,
        cell_count,
        cell_content_offset,
        fragmented_bytes_count,
        rightmost_pointer,
    })
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