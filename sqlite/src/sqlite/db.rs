use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use embedded_io::{Read, Seek};
use crate::sqlite::pager;
use crate::sqlite::pager::{Pager, HEADER_SIZE};
use crate::{DatabaseReader, RecordIterator, Type};

pub(crate) struct DbHeader {
    pub page_size: u32
}

pub(crate) struct ColumnDefinition {
    name: String,
    col_type: Type
}

pub(crate) struct TableMetadata {
    name: String,
    columns: Vec<ColumnDefinition>,
    first_page: usize
}

pub(crate) struct SqliteDatabase {
    header: DbHeader,
    table_metadata: TableMetadata,
    pager: Pager
}

impl SqliteDatabase {
    pub fn from_cursor<I>(mut cursor: I) -> Option<Self>
    where 
        I: Read + Seek
    {
        let mut header_buffer = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buffer).ok()?;
        
        let header = pager::parse_header(&header_buffer)?;
        
        let pager = Pager::new(cursor, header.page_size as usize);
        
        todo!("https://github.com/geoffreycopin/rqlite/blob/master/src/db.rs#L72")
    }
}

impl DatabaseReader for SqliteDatabase {
    fn read_table<S>(&self, table_name: S) -> Option<Box<dyn RecordIterator>>
    where
        S: AsRef<str>
    {
        None
    }
}