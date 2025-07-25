use crate::sqlite::sql::ast::{ColumnDef, CreateTableStatement, Type};
use crate::sqlite::sql::tokenizer;
use crate::sqlite::sql::tokenizer::Token;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use anyhow::{bail, Context};

#[derive(Debug)]
struct ParserState {
    tokens: Vec<Token>,
    pos: usize,
}

impl ParserState {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_create_statement(&mut self) -> anyhow::Result<CreateTableStatement> {
        match self.peek_next_token().context("unexpected end of input")? {
            Token::Create => self.parse_create_table(),
            token => bail!("unexpected token: {token:?}"),
        }
    }

    fn parse_create_table(&mut self) -> anyhow::Result<CreateTableStatement> {
        self.expect_eq(Token::Create)?;
        self.expect_eq(Token::Table)?;
        let name = self.expect_identifier()?.to_string();
        self.expect_eq(Token::LPar)?;
        let mut columns = vec![self.parse_column_def()?];
        while self.next_token_is(Token::Comma) {
            self.advance();
            columns.push(self.parse_column_def()?);
        }
        self.expect_eq(Token::RPar)?;
        Ok(CreateTableStatement { name, columns })
    }

    fn parse_column_def(&mut self) -> anyhow::Result<ColumnDef> {
        Ok(ColumnDef {
            name: self.expect_identifier()?.to_string(),
            col_type: self.parse_type()?,
        })
    }

    fn parse_type(&mut self) -> anyhow::Result<Type> {
        let type_name = self.expect_identifier()?;
        let t = match type_name.to_lowercase().as_str() {
            "integer" => Type::Integer,
            "real" => Type::Real,
            "blob" => Type::Blob,
            "text" | "string" => Type::Text,
            _ => bail!("unsupported type: {type_name}"),
        };
        Ok(t)
    }

    fn next_token_is(&self, expected: Token) -> bool {
        self.tokens.get(self.pos) == Some(&expected)
    }

    fn expect_identifier(&mut self) -> anyhow::Result<&str> {
        self.expect_matching(|t| matches!(t, Token::Identifier(_)))
            .map(|t| t.as_identifier().unwrap())
    }

    fn expect_eq(&mut self, expected: Token) -> anyhow::Result<&Token> {
        self.expect_matching(|t| *t == expected)
    }

    fn expect_matching(&mut self, f: impl Fn(&Token) -> bool) -> anyhow::Result<&Token> {
        match self.next_token() {
            Some(token) if f(token) => Ok(token),
            Some(token) => bail!("unexpected token: {:?}", token),
            None => bail!("unexpected end of input"),
        }
    }

    fn peek_next_token(&self) -> anyhow::Result<&Token> {
        self.tokens.get(self.pos).context("unexpected end of input")
    }

    fn next_token(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}

pub fn parse_create_statement(input: &str) -> anyhow::Result<CreateTableStatement> {
    let tokens = tokenizer::tokenize(input)?;
    let mut state = ParserState::new(tokens);

    state.parse_create_statement()
}
