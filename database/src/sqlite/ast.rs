use alloc::sync::Arc;

pub(super) struct ColumnDef {
    pub name: Arc<str>,
    pub col_type: Type,
}

#[derive(Clone, Eq, PartialEq)]
pub enum Type {
    Integer,
    Real,
    Text,
    Blob,
}