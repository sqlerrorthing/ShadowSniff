#![no_std]

extern crate alloc;
mod parser;
mod tokenize;

use crate::parser::{parse_tokens, TokenParseError};
use crate::tokenize::{tokenize, TokenizeError};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};

#[cfg_attr(test, derive(Debug))]
pub enum Value {
    Null,
    Boolean(bool),
    String(String),
    Number(f64),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Value {
    pub fn as_null(&self) -> Option<()> {
        if let Value::Null = self {
            Some(())
        } else {
            None
        }
    }
    
    pub fn as_bool(&self) -> Option<&bool> {
        if let Self::Boolean(val) = self {
            Some(val)
        } else {
            None
        }
    }
    
    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(val) = self {
            Some(val)
        } else {
            None
        }
    }
    
    pub fn as_number(&self) -> Option<&f64> {
        if let Self::Number(val) = self {
            Some(val)
        } else {
            None
        }
    }
    
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Self::Array(val) = self {
            Some(val)
        } else {
            None
        }
    }
    
    pub fn as_object(&self) -> Option<&BTreeMap<String, Value>> {
        if let Self::Object(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn get(&self, key: impl Into<Key>) -> Option<&Value> {
        match (self, key.into()) {
            (Value::Object(map), Key::Str(k)) => map.get(&k),
            (Value::Array(arr), Key::Idx(i)) => arr.get(i),
            _ => None,
        }
    }
}

pub enum Key {
    Str(String),
    Idx(usize),
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        if let Ok(i) = s.parse::<usize>() {
            Key::Idx(i)
        } else {
            Key::Str(s.to_string())
        }
    }
}

impl From<usize> for Key {
    fn from(i: usize) -> Self {
        Key::Idx(i)
    }
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

#[cfg_attr(test, derive(Debug))]
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

#[cfg(test)]
mod tests {
    use super::*;

    extern crate std;
    
    #[test]
    fn test_parse_str() {
        let input = r#"
        {"id":"***","username":"***","avatar":null,"discriminator":"0","public_flags":0,"flags":0,"banner":null,"accent_color":null,"global_name":"***","avatar_decoration_data":null,"collectibles":null,"banner_color":null,"clan":null,"primary_guild":null,"mfa_enabled":false,"locale":"***","premium_type":0,"email":"***","verified":true,"phone":null,"nsfw_allowed":true,"linked_users":[],"bio":"","authenticator_types":[],"age_verification_status":1}
        "#;

        match parse_str(input) {
            Ok(result) => {
                std::dbg!(&result);
            }
            Err(err) => panic!("parse_str failed: {:?}", err),
        }
    }
}