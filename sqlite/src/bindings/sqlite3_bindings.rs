pub const SQLITE_INTEGER: u32 = 1;
pub const SQLITE_FLOAT: u32 = 2;
pub const SQLITE_BLOB: u32 = 4;
pub const SQLITE_NULL: u32 = 5;
pub const SQLITE_TEXT: u32 = 3;
pub const SQLITE_DESERIALIZE_READONLY: u32 = 4;
pub const SQLITE_ROW: u32 = 100;

#[repr(C)]
pub struct sqlite3 {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct sqlite3_stmt {
    _unused: [u8; 0],
}

unsafe extern "C" {
    pub fn sqlite3_close(arg1: *mut sqlite3) -> core::ffi::c_int;

    pub fn sqlite3_column_blob(
        arg1: *mut sqlite3_stmt,
        iCol: core::ffi::c_int,
    ) -> *const core::ffi::c_void;

    pub fn sqlite3_column_bytes(
        arg1: *mut sqlite3_stmt,
        iCol: core::ffi::c_int,
    ) -> core::ffi::c_int;

    pub fn sqlite3_column_count(pStmt: *mut sqlite3_stmt) -> core::ffi::c_int;
    pub fn sqlite3_column_double(arg1: *mut sqlite3_stmt, iCol: core::ffi::c_int) -> f64;
    pub fn sqlite3_column_int64(arg1: *mut sqlite3_stmt, iCol: core::ffi::c_int) -> core::ffi::c_longlong;
    pub fn sqlite3_column_type(arg1: *mut sqlite3_stmt, iCol: core::ffi::c_int) -> core::ffi::c_int;

    pub fn sqlite3_column_text(
        arg1: *mut sqlite3_stmt,
        iCol: core::ffi::c_int,
    ) -> *const core::ffi::c_uchar;

    pub fn sqlite3_deserialize(
        db: *mut sqlite3,
        zSchema: *const core::ffi::c_char,
        pData: *mut core::ffi::c_uchar,
        szDb: core::ffi::c_longlong,
        szBuf: core::ffi::c_longlong,
        mFlags: core::ffi::c_uint,
    ) -> core::ffi::c_int;

    pub fn sqlite3_initialize() -> core::ffi::c_int;
    
    pub fn sqlite3_finalize(pStmt: *mut sqlite3_stmt) -> core::ffi::c_int;

    pub fn sqlite3_open(
        filename: *const core::ffi::c_char,
        ppDb: *mut *mut sqlite3,
    ) -> core::ffi::c_int;

    pub fn sqlite3_prepare_v2(
        db: *mut sqlite3,
        zSql: *const core::ffi::c_char,
        nByte: core::ffi::c_int,
        ppStmt: *mut *mut sqlite3_stmt,
        pzTail: *mut *const core::ffi::c_char,
    ) -> core::ffi::c_int;

    pub fn sqlite3_step(arg1: *mut sqlite3_stmt) -> core::ffi::c_int;
}