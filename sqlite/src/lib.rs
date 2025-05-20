#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Blob(Vec<u8>),
    Null
}

pub trait DatabaseReader {
    fn read_table<S>(&self, table_name: S) -> Option<Box<dyn RecordIterator>>
    where
        S: AsRef<str>;
}

pub trait RecordIterator: Iterator<Item = Box<dyn TableRecord>> {}

pub trait TableRecord {
    fn get_value_by_key(&self, key: &RecordKey) -> Option<&Value>;
}

pub trait TableRecordExtension: TableRecord {
    fn get_value(&self, key: impl Into<RecordKey>) -> Option<&Value> {
        self.get_value_by_key(&key.into())
    }
}

impl<T: TableRecord + ?Sized> TableRecordExtension for T {}

pub enum RecordKey {
    Str(String),
    Idx(usize)
}

impl From<&str> for RecordKey {
    fn from(value: &str) -> Self {
        RecordKey::Str(value.to_owned())
    }
}

impl From<usize> for RecordKey {
    fn from(value: usize) -> Self {
        RecordKey::Idx(value)
    }
}

struct DummyDatabaseReader;

impl DatabaseReader for DummyDatabaseReader {
    fn read_table<S>(&self, _table_name: S) -> Option<Box<dyn RecordIterator>>
    where
        S: AsRef<str>
    {
        None
    }
}

pub fn read_sqlite3_database(_data: Vec<u8>) -> Option<impl DatabaseReader> {
    Some(DummyDatabaseReader)
}