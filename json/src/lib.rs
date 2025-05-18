#![no_std]

mod parser;
mod tokenize;

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String};
use alloc::vec::Vec;
use core::fmt::{write, Display, Formatter};
use crate::parser::{parse_tokens, TokenParseError};
use crate::tokenize::{tokenize, TokenizeError};

pub enum Value {
    Null,
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Null => { write!(f, "null") }
            Value::Boolean(value) => { write!(f, "{}", value) }
            Value::String(value) => { write!(f, "{}", value) }
            Value::Number(value) => { write!(f, "{}", value) }
            Value::Array(value) => { write!(f, "[array {}]", value.len()) }
            Value::Object(_) => { write!(f, "{{object Object}}") }
        }
    }
}

pub fn parse_str<S>(input: S) -> Result<Value, ParseError>
where
    S: AsRef<str>,
{
    let tokens = tokenize(input)?;
    let value = parse_tokens(&tokens, &mut 0)?;
    Ok(value)
}

pub fn parse(input: &[u8]) -> Result<Value, ParseError> {
    parse_str(str::from_utf8(input).unwrap())
}

#[derive(Debug)]
pub enum ParseError {
    TokenizeError(TokenizeError),
    ParseError(TokenParseError),
}

impl From<TokenParseError> for ParseError {
    fn from(err: TokenParseError) -> Self {
        Self::ParseError(err)
    }
}

impl From<TokenizeError> for ParseError {
    fn from(err: TokenizeError) -> Self {
        Self::TokenizeError(err)
    }
}