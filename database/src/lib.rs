#![no_std]

extern crate alloc;
pub mod bindings;

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use filesystem::FileSystem;
use filesystem::path::Path;

#[derive(Clone)]
pub enum Value {
    String(Arc<str>),
    Integer(i64),
    Float(f64),
    Blob(Arc<[u8]>),
    Null,
}

impl Value {
    pub fn as_string(&self) -> Option<Arc<str>> {
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

/// A trait representing a database which can be created from raw bytes.
///
/// This trait extends `DatabaseReader` which provides methods to read data from the database.
///
/// # Methods
/// - `from_bytes(bytes: Vec<u8>) -> Result<Self, i32>`: Constructs the database from a vector of bytes.
///
/// # Errors
/// Returns an `Err(i32)` on failure to parse the bytes into a database.
pub trait Database: DatabaseReader {
    /// Create a database instance from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A vector of bytes representing the database content.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if the bytes could be parsed into a database,
    /// otherwise returns an `Err(i32)` error code.
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, i32>
    where
        Self: Sized;
}

/// A trait for reading data from a database.
///
/// Provides an interface to read tables and their records.
///
/// # Associated Types
/// - `Iter`: An iterator over the records in the table.
/// - `Record`: The record type, must implement `TableRecord`.
pub trait DatabaseReader {
    /// The type of iterator returned when reading a table.
    type Iter: Iterator<Item = Self::Record>;

    /// The record type stored in the database tables.
    type Record: TableRecord;

    /// Reads a table by name, returning an iterator over its records if found.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the table to read.
    ///
    /// # Returns
    ///
    /// Returns `Some(iterator)` over the records of the table if it exists,
    /// or `None` if the table could not be found.
    fn read_table<S>(&self, table_name: S) -> Option<Self::Iter>
    where
        S: AsRef<str>;
}

/// An extension trait for `Database` to provide additional constructors.
pub trait DatabaseExt: Database {
    /// Create a database instance from a file path using the given filesystem.
    ///
    /// # Arguments
    ///
    /// * `fs` - A reference to a filesystem instance that implements `FileSystem`.
    /// * `path` - The path to the database file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if the file was successfully read and parsed into a database,
    /// otherwise returns an `Err(i32)` error code.
    fn from_path<R, F, P>(fs: R, path: P) -> Result<Self, i32>
    where
        R: AsRef<F>,
        F: FileSystem,
        P: AsRef<Path>,
        Self: Sized;
}

impl<T: Database> DatabaseExt for T {
    fn from_path<R, F, P>(fs: R, path: P) -> Result<Self, i32>
    where
        R: AsRef<F>,
        F: FileSystem,
        P: AsRef<Path>,
        Self: Sized,
    {
        let data = fs.as_ref().read_file(path.as_ref()).map_err(|e| e as i32)?;
        Self::from_bytes(data)
    }
}

pub trait TableRecord {
    fn get_value(&self, key: usize) -> Option<Value>;
}
