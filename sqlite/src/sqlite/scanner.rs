use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::sqlite::cursor::Cursor;
use crate::sqlite::page::{Cell, Page, PageType};
use crate::sqlite::pager::Pager;

pub(super) struct PositionedPage {
    pub page: Arc<Page>,
    pub cell: usize
}

impl PositionedPage {
    pub fn next_cell(&mut self) -> Option<&Cell> {
        let cell = self.page.get(self.cell);
        self.cell += 1;
        cell
    }
    
    pub fn next_page(&mut self) -> Option<u32> {
        if self.page.header.page_type == PageType::TableInterior && self.cell == self.page.cells.len() {
            self.cell += 1;
            self.page.header.rightmost_pointer
        } else {
            None
        }
    }
}

pub(super) struct Scanner {
    initial_page: usize,
    page_stack: Vec<PositionedPage>,
    pager: Pager
}

impl Scanner {
    pub fn new(page: usize, pager: Pager) -> Scanner {
        Scanner {
            initial_page: page,
            page_stack: Vec::new(),
            pager
        }
    }
    
    pub fn next_record(&mut self) -> Option<Cursor> {
        loop {
            match self.next_elem() {
                
            }
        }
    }
    
    fn next_elem(&mut self) -> Option<ScannerElem> {
        let Some(page) = self.current_page()? else {
            return None;
        };
        
        let Some(page) = page.next_page() {
            return Some(ScannerElem::Page(page))
        }
        
        let Some(cell) = page.next_cell() else {
            return None
        };
        
        match cell {
            Cell::TableLeaf(cell) => {
                let header = parse_record_header(&cell.payload)?;
                Some(ScannerElem::Cursor(Cursor {
                    header,
                    payload: cell.payload.clone()
                }))
            }
            Cell::TableInterior(cell) => Some(ScannerElem::Page(cell.left_child_page))
        }
    }
    
    fn current_page(&mut self) -> Option<&mut PositionedPage> {
        if self.page_stack.is_empty() {
            let page = match self.pager.read_page(self.initial_page) {
                Some(page) => page.clone(),
                None => return None
            };
            
            self.page_stack.push(PositionedPage { page, cell: 0 })
        }
        
        self.page_stack.last_mut()
    }
}

enum ScannerElem {
    Page(u32),
    Cursor(Cursor),
}