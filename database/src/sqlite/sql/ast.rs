use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Statement {
    Select(SelectStatement),
    CreateTable(CreateTableStatement),
}

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SelectStatement {
    pub core: SelectCore,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SelectCore {
    pub result_columns: Vec<ResultColumn>,
    pub from: SelectFrom,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResultColumn {
    Star,
    Expr(ExprResultColumn),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExprResultColumn {
    pub expr: Expr,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Column(Column),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Column {
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SelectFrom {
    Table(String),
}