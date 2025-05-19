use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use crate::Value;

pub trait Reader {
    fn read_table<S>(table_name: S) -> Option<Box<dyn RecordIterator>>
    where 
        S: AsRef<str>;
}

pub trait RecordIterator: Iterator<Item = Box<dyn Record>> {}

pub trait Record {
    fn get_value_by_key(&self, key: &RecordKey) -> Option<&Value>;
}

pub trait RecordExtension: Record {
    fn get_value(&self, key: impl Into<RecordKey>) -> Option<&Value> {
        self.get_value_by_key(&key.into())
    }
}

impl<T: Record + ?Sized> RecordExtension for T {}

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