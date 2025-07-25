use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: Type,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Type {
    Integer,
    Real,
    Text,
    Blob,
}
