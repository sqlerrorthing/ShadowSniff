use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::bail;

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Create,
    Table,
    LPar,
    RPar,
    Comma,
    SemiColon,
    Identifier(String),
}

impl Token {
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Token::Identifier(ident) => Some(ident),
            _ => None,
        }
    }
}

pub fn tokenize(input: &str) -> anyhow::Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '(' => tokens.push(Token::LPar),
            ')' => tokens.push(Token::RPar),
            ',' => tokens.push(Token::Comma),
            ';' => tokens.push(Token::SemiColon),
            c if c.is_whitespace() => continue,
            c if c.is_alphabetic() => {
                let mut ident = c.to_string().to_lowercase();
                while let Some(cc) = chars.next_if(|&cc| cc.is_alphanumeric() || cc == '_') {
                    ident.extend(cc.to_lowercase());
                }

                match ident.as_str() {
                    "create" => tokens.push(Token::Create),
                    "table" => tokens.push(Token::Table),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }
            _ => bail!("unexpected character: {}", c),
        }
    }

    Ok(tokens)
}
