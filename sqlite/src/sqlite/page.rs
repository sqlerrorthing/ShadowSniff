use alloc::vec::Vec;

#[derive(PartialEq)]
pub(super) enum PageType {
    TableLeaf,
    TableInterior
}

pub(super) struct TableLeafCell {
    size: i64,
    row_id: i64,
    pub(crate) payload: Vec<u8>
}

pub(super) struct TableInteriorCell {
    pub(crate) left_child_page: u32,
    key: i64
}

pub(super) enum Cell {
    TableLeaf(TableLeafCell),
    TableInterior(TableInteriorCell)
}

pub(super) struct PageHeader {
    pub(crate) page_type: PageType,
    first_free_block: u16,
    cell_count: u16,
    cell_content_offset: u32,
    fragmented_bytes_count: u8,
    pub(crate) rightmost_pointer: Option<u32>
}

pub(super) struct Page {
    pub(crate) header: PageHeader,
    cell_pointers: Vec<u16>,
    pub(crate) cells: Vec<Cell>
}

impl Page {
    pub(crate) fn get(&self, index: usize) -> Option<&Cell> {
        self.cells.get(index)
    }
}

impl From<TableLeafCell> for Cell {
    fn from(value: TableLeafCell) -> Self {
        Cell::TableLeaf(value)
    }
}

impl From<TableInteriorCell> for Cell {
    fn from(value: TableInteriorCell) -> Self {
        Cell::TableInterior(value)
    }
}