use crate::{DatabaseReader, TableRecord, Value};

pub struct SqliteDatabase {
    
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

impl TryFrom<&[u8]> for SqliteDatabase {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}