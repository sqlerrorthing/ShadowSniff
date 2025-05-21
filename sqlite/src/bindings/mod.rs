use crate::bindings::sqlite3_bindings::{sqlite3, sqlite3_close, sqlite3_column_blob, sqlite3_column_bytes, sqlite3_column_count, sqlite3_column_double, sqlite3_column_int64, sqlite3_column_text, sqlite3_column_type, sqlite3_deserialize, sqlite3_finalize, sqlite3_initialize, sqlite3_open, sqlite3_prepare_v2, sqlite3_step, sqlite3_stmt, SQLITE_BLOB, SQLITE_DESERIALIZE_RESIZEABLE, SQLITE_FLOAT, SQLITE_INTEGER, SQLITE_NULL, SQLITE_ROW, SQLITE_TEXT};
use crate::{DatabaseReader, RecordKey, Table, TableRecord, Value};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::{IntoIter, Vec};
use core::ffi::c_char;
use core::ptr::null_mut;
use obfstr::obfstr as s;
use utils::path::Path;

mod sqlite3_bindings;

pub(crate) struct Sqlite3BindingsReader {
    db: *mut sqlite3
}

impl Sqlite3BindingsReader {
    pub fn new_from_file(db_path: &Path) -> Result<Self, i32> {
        let c_path = CString::new(db_path);
        
        unsafe {
            sqlite3_initialize();
        }
        
        let mut db: *mut sqlite3 = null_mut();
        let rc = unsafe {
            sqlite3_open(c_path.as_ptr(), &mut db)
        };
        
        if rc != 0 {
            Err(rc)
        } else {
            Ok(Self { db })
        }
    }
    
    pub fn new_from_bytes(db_bytes: &[u8]) -> Result<Self, i32> {
        let mut db: *mut sqlite3 = null_mut();

        unsafe {
            sqlite3_initialize();
        }
        
        let rc = unsafe {
            sqlite3_open(":memory:\0".as_ptr() as *const i8, &mut db)
        };
        
        if rc != 0 {
            return Err(rc);
        }

        let data_size = db_bytes.len();
        let data = db_bytes.to_vec().into_boxed_slice();
        let data_ptr = Box::into_raw(data) as *mut u8;
        
        let rc = unsafe {
            sqlite3_deserialize(
                db,
                b"main\0".as_ptr() as *const i8,
                data_ptr,
                data_size as i64,
                data_size as i64,
                SQLITE_DESERIALIZE_RESIZEABLE
            )
        };
        
        if rc != 0 {
            return Err(rc);
        }
        
        Ok(Self { db })
    }
}

impl Drop for Sqlite3BindingsReader {
    fn drop(&mut self) {
        unsafe {
            sqlite3_close(self.db); 
        }
    }
}

impl DatabaseReader for Sqlite3BindingsReader {
    fn read_table<S>(&self, table_name: S) -> Option<Box<dyn Table>>
    where
        S: AsRef<str>
    {
        let query = format!("{} {}", s!("SELECT * FROM"), table_name.as_ref());
        let mut stmt: *mut sqlite3_stmt = null_mut();
        let c_query = CString::new(query.as_ref());

        let rc = unsafe {
            sqlite3_prepare_v2(self.db, c_query.as_ptr(), -1, &mut stmt, null_mut())
        };

        if rc != 0 || stmt.is_null() {
            return None;
        }
        
        let table = SqliteTable::from_stmt(stmt);
        unsafe { sqlite3_finalize(stmt) };
        
        Some(Box::new(table))
    }
}

struct SqliteRow {
    row: Vec<Value>
}

impl TableRecord for SqliteRow {
    fn get_value_by_key(&self, key: &RecordKey) -> Option<&Value> {
        match key { 
            RecordKey::Idx(id) => self.row.get(*id)
        }
    }
}

impl TableRecord for &SqliteRow {
    fn get_value_by_key(&self, key: &RecordKey) -> Option<&Value> {
        (*self).get_value_by_key(key)
    }
}

struct SqliteTable {
    initial_length: usize,
    rows: IntoIter<SqliteRow>
}

impl SqliteTable {
    fn from_stmt(stmt: *mut sqlite3_stmt) -> Self {
        let col_count = unsafe { sqlite3_column_count(stmt) } as usize;
        let mut rows = Vec::new();

        loop {
            let rc = unsafe { sqlite3_step(stmt) };
            if rc != SQLITE_ROW as i32 {
                break;
            }

            let mut row = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let val = unsafe {
                    match sqlite3_column_type(stmt, i as i32) as u32 {
                        SQLITE_INTEGER => Value::Integer(sqlite3_column_int64(stmt, i as i32)),
                        SQLITE_FLOAT => Value::Float(sqlite3_column_double(stmt, i as i32)),
                        SQLITE_TEXT => {
                            let text_ptr = sqlite3_column_text(stmt, i as i32);
                            let len = sqlite3_column_bytes(stmt, i as i32) as usize;
                            if text_ptr.is_null() {
                                Value::Null
                            } else {
                                let bytes = core::slice::from_raw_parts(text_ptr, len);
                                Value::String(String::from_utf8_lossy(bytes).into_owned())
                            }
                        },
                        SQLITE_BLOB => {
                            let ptr = sqlite3_column_blob(stmt, i as i32);
                            let len = sqlite3_column_bytes(stmt, i as i32) as usize;
                            if ptr.is_null() {
                                Value::Null
                            } else {
                                let slice = core::slice::from_raw_parts(ptr as *const u8, len);
                                Value::Blob(slice.to_vec())
                            }
                        },
                        SQLITE_NULL => Value::Null,
                        _ => Value::Null,
                    }
                };
                
                row.push(val);
            }

            rows.push(SqliteRow { row });
        }

        SqliteTable { 
            initial_length: rows.len(), 
            rows: rows.into_iter()
        }
    }
}

impl Table for SqliteTable {
    fn records_length(&self) -> usize {
        self.initial_length
    }
}

impl Iterator for SqliteTable {
    type Item = Box<dyn TableRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rows.next().map(|row| Box::new(row) as Box<dyn TableRecord>)
    }
}

pub struct CString {
    inner: Vec<u8>,
}

impl CString {
    pub fn new(s: &str) -> Self {
        let mut v = Vec::with_capacity(s.len() + 1);
        v.extend_from_slice(s.as_bytes());
        v.push(0);
        Self { inner: v }
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.inner.as_ptr() as *const c_char
    }
}