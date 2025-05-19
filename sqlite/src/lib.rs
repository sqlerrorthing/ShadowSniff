#![no_std]

mod reader;

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use crate::reader::Reader;

pub enum Value {
    String(String),
    Number(f64),
    Blob(Vec<u8>),
    Null
}

pub struct Database<T: Reader> {
    inner: T
}

impl<T: Reader> Database<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}