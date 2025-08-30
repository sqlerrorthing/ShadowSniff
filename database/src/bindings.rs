/*
 * This file is part of ShadowSniff (https://github.com/sqlerrorthing/ShadowSniff)
 *
 * MIT License
 *
 * Copyright (c) 2025 sqlerrorthing
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::bindings::sqlite3_bindings::{
    SQLITE_BLOB, SQLITE_DESERIALIZE_FREEONCLOSE, SQLITE_DESERIALIZE_RESIZEABLE, SQLITE_FLOAT,
    SQLITE_INTEGER, SQLITE_ROW, SQLITE_TEXT, sqlite3, sqlite3_close, sqlite3_column_blob,
    sqlite3_column_bytes, sqlite3_column_count, sqlite3_column_double, sqlite3_column_int64,
    sqlite3_column_text, sqlite3_column_type, sqlite3_deserialize, sqlite3_finalize,
    sqlite3_initialize, sqlite3_malloc, sqlite3_open, sqlite3_prepare_v2, sqlite3_step,
    sqlite3_stmt,
};
use crate::{Database, DatabaseReader, TableRecord, Value};
use alloc::ffi::CString;
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::{IntoIter, Vec};
use core::iter::FusedIterator;
use core::ptr;
use core::ptr::null_mut;
use delegate::delegate;
use obfstr::obfstr as s;

mod sqlite3_bindings;

pub struct Sqlite3BindingsDatabase {
    db: *mut sqlite3,
}

impl Drop for Sqlite3BindingsDatabase {
    fn drop(&mut self) {
        unsafe {
            sqlite3_close(self.db);
        }
    }
}

impl Database for Sqlite3BindingsDatabase {
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, i32>
    where
        Self: Sized,
    {
        unsafe {
            sqlite3_initialize();
        }

        if bytes.len() < 16 || &bytes[0..16] != b"SQLite format 3\0" {
            return Err(26); // SQLITE_NOTADB
        }

        let mut db: *mut sqlite3 = null_mut();
        let rc = unsafe { sqlite3_open(c":memory:".as_ptr(), &mut db) };

        if rc != 0 {
            return Err(rc);
        }

        let data_size = bytes.len();
        let data_ptr = unsafe { sqlite3_malloc(data_size as i32) } as *mut u8;
        if data_ptr.is_null() {
            unsafe { sqlite3_close(db) };
            return Err(7); // SQLITE_NOMEM
        }

        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), data_ptr, data_size);
        }

        let rc = unsafe {
            sqlite3_deserialize(
                db,
                c"main".as_ptr(),
                data_ptr,
                data_size as i64,
                data_size as i64,
                SQLITE_DESERIALIZE_RESIZEABLE | SQLITE_DESERIALIZE_FREEONCLOSE,
            )
        };

        if rc != 0 {
            unsafe {
                sqlite3_close(db);
            }
            return Err(rc);
        }

        Ok(Self { db })
    }
}

impl DatabaseReader for Sqlite3BindingsDatabase {
    type Iter = IntoIter<SqliteRow>;
    type Record = SqliteRow;

    fn read_table<S>(&self, table_name: S) -> Option<Self::Iter>
    where
        S: AsRef<str>,
    {
        let query = format!("{} {}", s!("SELECT * FROM"), table_name.as_ref());
        let c_query = CString::new(query).unwrap();
        let mut stmt: *mut sqlite3_stmt = null_mut();

        let rc =
            unsafe { sqlite3_prepare_v2(self.db, c_query.as_ptr(), -1, &mut stmt, null_mut()) };

        if rc != 0 || stmt.is_null() {
            return None;
        }

        let table = SqliteTable::from(stmt);
        unsafe { sqlite3_finalize(stmt) };

        Some(table.rows.into_iter())
    }
}

struct SqliteTable {
    rows: Vec<SqliteRow>,
}

impl From<*mut sqlite3_stmt> for SqliteTable {
    fn from(stmt: *mut sqlite3_stmt) -> Self {
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
                                Value::String(String::from_utf8_lossy(bytes).into())
                            }
                        }
                        SQLITE_BLOB => {
                            let ptr = sqlite3_column_blob(stmt, i as i32);
                            let len = sqlite3_column_bytes(stmt, i as i32) as usize;
                            if ptr.is_null() {
                                Value::Null
                            } else {
                                let slice = core::slice::from_raw_parts(ptr as *const u8, len);
                                Value::Blob(Arc::from(slice.to_vec()))
                            }
                        }
                        _ => Value::Null,
                    }
                };

                row.push(val);
            }

            rows.push(SqliteRow::from(row));
        }

        Self { rows }
    }
}

#[derive(Clone)]
pub struct SqliteRow(IntoIter<Value>);

impl From<Vec<Value>> for SqliteRow {
    fn from(value: Vec<Value>) -> Self {
        Self(value.into_iter())
    }
}

impl Iterator for SqliteRow {
    type Item = Value;

    delegate! {
        to self.0 {
            fn next(&mut self) -> Option<Self::Item>;
            fn size_hint(&self) -> (usize, Option<usize>);
            fn nth(&mut self, n: usize) -> Option<Self::Item>;
            fn last(self) -> Option<Self::Item>;
            fn count(self) -> usize;
        }
    }
}

impl DoubleEndedIterator for SqliteRow {
    delegate! {
        to self.0 {
            fn next_back(&mut self) -> Option<Self::Item>;
            fn nth_back(&mut self, n: usize) -> Option<Self::Item>;
        }
    }
}

impl ExactSizeIterator for SqliteRow {
    delegate! {
        to self.0 {
            fn len(&self) -> usize;
        }
    }
}

impl FusedIterator for SqliteRow {}

impl TableRecord for SqliteRow {
    fn get_value(&self, key: usize) -> Option<Value> {
        self.0.as_slice().get(key).cloned()
    }
}
