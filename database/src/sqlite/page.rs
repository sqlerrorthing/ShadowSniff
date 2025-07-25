use alloc::vec::Vec;
use anyhow::bail;

#[derive(Copy, Clone)]
pub struct DbHeader {
    pub page_size: u32,
    pub page_reserved_size: u8,
}

impl PageHeader {
    pub fn byte_size(&self) -> usize {
        if self.rightmost_pointer.is_some() {
            12
        } else {
            8
        }
    }

    pub fn local_and_overflow_size(
        &self,
        db_header: &DbHeader,
        payload_size: usize,
    ) -> anyhow::Result<(usize, Option<usize>)> {
        let local = self.local_payload_size(db_header, payload_size)?;
        if local == payload_size {
            Ok((local, None))
        } else {
            Ok((local, Some(payload_size.saturating_sub(local))))
        }
    }

    fn local_payload_size(
        &self,
        db_header: &DbHeader,
        payload_size: usize,
    ) -> anyhow::Result<usize> {
        match self.page_type {
            PageType::TableInterior => bail!("no payload size for interior pages"),
            PageType::TableLeaf => {
                let usable = db_header.usable_page_size();
                let max_size = usable - 35;
                if payload_size <= max_size {
                    return Ok(payload_size);
                }
                let min_size = ((usable - 12) * 32 / 255) - 23;
                let k = min_size + ((payload_size - min_size) % (usable - 4));
                let size = if k <= max_size { k } else { min_size };
                Ok(size)
            }
        }
    }
}

impl DbHeader {
    pub fn usable_page_size(&self) -> usize {
        self.page_size as usize - (self.page_reserved_size as usize)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum PageType {
    TableLeaf,
    TableInterior,
}

#[derive(Copy, Clone)]
pub struct PageHeader {
    pub page_type: PageType,
    pub cell_count: u16,
    pub rightmost_pointer: Option<u32>,
}

#[derive(Clone)]
pub struct Page {
    pub header: PageHeader,
    pub cells: Vec<Cell>,
}

impl Page {
    pub fn get(&self, n: usize) -> Option<&Cell> {
        self.cells.get(n)
    }
}

#[derive(Clone)]
pub struct TableLeafCell {
    pub payload: Vec<u8>,
    pub first_overflow: Option<usize>,
}

#[derive(Clone)]
pub struct TableInteriorCell {
    pub left_child_page: u32,
}

#[derive(Clone)]
pub enum Cell {
    TableLeaf(TableLeafCell),
    TableInterior(TableInteriorCell),
}

impl From<TableLeafCell> for Cell {
    fn from(cell: TableLeafCell) -> Self {
        Cell::TableLeaf(cell)
    }
}

impl From<TableInteriorCell> for Cell {
    fn from(cell: TableInteriorCell) -> Self {
        Cell::TableInterior(cell)
    }
}

#[derive(Clone)]
pub struct OverflowPage {
    pub next: Option<usize>,
    pub payload: Vec<u8>,
}