use alloc::vec::Vec;

pub enum RecordFieldType {
    Null,
    I8,
    I16,
    I24,
    I32,
    I48,
    I64,
    Float,
    Zero,
    One,
    String(usize),
    Blob(usize)
}

pub(super) struct RecordField {
    pub offset: usize,
    pub field_type: RecordFieldType
}

pub(super) struct RecordHeader {
    pub fields: Vec<RecordField>,
}

pub(super) struct Cursor {
    header: RecordHeader
}