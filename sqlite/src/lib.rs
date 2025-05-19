#![no_std]

mod reader;

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use crate::reader::{Reader, RecordIterator};

pub enum Value {
    String(String),
    Number(f64),
    Blob(Vec<u8>),
    Null
}

struct DummyReader;

impl Reader for DummyReader {
    fn read_table<S>(_table_name: S) -> Option<Box<dyn RecordIterator>>
    where
        S: AsRef<str>
    {
        None
    }
}

pub fn read_sqlite3_database(_data: &[u8]) -> impl Reader {
    DummyReader
}