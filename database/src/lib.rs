#![no_std]

extern crate alloc;
pub mod sqlite;

use crate::sqlite::db::SqliteDatabase;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Error;
use core::fmt::{Display, Formatter};

pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Blob(Vec<u8>),
    Null
}

impl Value {
    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let Value::Integer(i) = self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn as_blob(&self) -> Option<&Vec<u8>> {
        if let Value::Blob(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_null(&self) -> Option<()> {
        if let Value::Null = self {
            Some(())
        } else {
            None
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::String(value) => write!(f, "{}", value),
            Value::Integer(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Blob(value) => write!(f, "{}", String::from_utf8_lossy(value)),
            Value::Null => write!(f, "null"),
        }
    }
}

pub trait DatabaseReader {
    type Iter: Iterator<Item = Self::Record>;
    type Record: TableRecord;

    fn read_table<S>(&self, table_name: S) -> Option<Self::Iter>
    where
        S: AsRef<str>;
}

pub trait TableRecord {
    fn get_value(&self, key: usize) -> Option<&Value>;
}

pub enum Databases {
    Sqlite
}

impl Databases {
    pub fn read_from_bytes(&self, bytes: &[u8]) -> Result<impl DatabaseReader, Error> {
        match self {
            Databases::Sqlite => SqliteDatabase::try_from(bytes)
        }
    }
}

impl AsRef<Databases> for Databases {
    fn as_ref(&self) -> &Databases {
        self
    }
}