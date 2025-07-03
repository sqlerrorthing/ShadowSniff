#![feature(let_chains)]
#![no_std]

extern crate alloc;
pub mod sqlite;

use crate::sqlite::db::SqliteDatabase;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::Error;
use core::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Value {
    String(Arc<str>),
    Integer(i64),
    Float(f64),
    Blob(Arc<[u8]>),
    Null
}

impl Value {
    pub fn as_str(&self) -> Option<Arc<str>> {
        if let Value::String(s) = self {
            Some(s.clone())
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

    pub fn as_blob(&self) -> Option<Arc<[u8]>> {
        if let Value::Blob(b) = self {
            Some(b.clone())
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
            Value::String(value) => write!(f, "{value}"),
            Value::Integer(value) => write!(f, "{value}"),
            Value::Float(value) => write!(f, "{value}"),
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
    fn get_value(&self, key: usize) -> Option<Value>;
}

pub enum Databases {
    Sqlite,
}

impl Databases {
    pub fn read_from_bytes(&self, bytes: Vec<u8>) -> Result<impl DatabaseReader, Error> {
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

#[cfg(test)]
mod tests {
    use crate::Databases;
    use crate::{DatabaseReader, TableRecord};
    use utils::log_debug;
    use utils::path::get_current_directory;

    extern crate alloc;

    #[test]
    fn load() {
        let absolute = get_current_directory().unwrap().parent().unwrap() / "test.db";
        let Ok(file) = absolute.read_file() else {
            panic!("file {absolute} not found")
        };

        let db = Databases::Sqlite.read_from_bytes(file).unwrap();

        for record in db.read_table("Customers").unwrap() {
            log_debug!("{}", record.get_value(0).unwrap().as_integer().unwrap())
        }
    }
}