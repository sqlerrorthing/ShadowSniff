use crate::sqlite::cursor::{Cursor, Scanner};
use crate::sqlite::pager::{parse_header, Pager, HEADER_SIZE};
use crate::sqlite::sql;
use crate::sqlite::sql::ast::ColumnDef;
use crate::{DatabaseReader, TableRecord, Value};
use alloc::borrow::ToOwned;
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::{anyhow, Context};
use embedded_io::Read;
use half_io::VecReader;
use utils::log_debug;

pub(super) struct TableMetadata {
    pub name: Arc<str>,
    pub columns: Vec<ColumnDef>,
    pub first_page: usize
}

pub struct SqliteDatabase {
    tables_metadata: Vec<TableMetadata>,
    pager: Pager
}

impl TableMetadata {
    fn from_cursor(mut cursor: Cursor) -> anyhow::Result<Option<Self>> {
        let type_value = cursor
            .field(0)?
            .context("missing type field")
            .context("invalid type field")?;

        if type_value.as_str() != Some("table") {
            return Ok(None);
        }

        let create_stmt = cursor
            .field(4)?
            .context("missing create statement")
            .context("invalid create statement")?
            .as_str()
            .context("table create statement should be a string")?
            .to_owned();

        log_debug!("{create_stmt}\n");
        let create = sql::parse_create_statement(&create_stmt)?;

        let first_page = cursor
            .field(3)?
            .context("missing table first page")?
            .as_int()
            .context("table first page should be an integer")? as usize;

        Ok(Some(TableMetadata {
            name: create.name.into(),
            columns: create.columns,
            first_page,
        }))
    }
}

pub struct DummyRecord;

impl TableRecord for DummyRecord {
    fn get_value(&self, key: usize) -> Option<&Value> {
        None
    }
}

pub struct DummyIter;

impl Iterator for DummyIter {
    type Item = DummyRecord;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl DatabaseReader for SqliteDatabase {
    type Iter = DummyIter;
    type Record = DummyRecord;

    fn read_table<S>(&self, table_name: S) -> Option<Self::Iter>
    where
        S: AsRef<str>
    {
        todo!()
    }
}

impl SqliteDatabase {
    pub fn scanner(&self, page: usize) -> Scanner {
        Scanner::new(page, self.pager.clone())
    }

    fn collect_tables_metadata(pager: Pager) -> anyhow::Result<Vec<TableMetadata>> {
        let mut metadata = Vec::new();
        let mut scanner = Scanner::new(1, pager);

        while let Some(record) = scanner.next_record()? {
            if let Some(m) = TableMetadata::from_cursor(record)? {
                metadata.push(m);
            }
        }

        Ok(metadata)
    }
}

impl TryFrom<Vec<u8>> for SqliteDatabase {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mut value = VecReader::new(value);

        let mut header_buffer = [0; HEADER_SIZE];
        value.read_exact(&mut header_buffer)
            .map_err(|_| anyhow!("Failed to read SQLite header"))?;

        let header = parse_header(&header_buffer).map_err(|_| anyhow!("Failed to parse SQLite header"))?;

        let pager = Pager::new(header, value);

        let tables_metadata = Self::collect_tables_metadata(pager.clone())?;

        Ok(SqliteDatabase {
            pager,
            tables_metadata
        })
    }
}