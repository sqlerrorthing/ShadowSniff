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

use crate::Value;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::tokenize::Token;

pub type ParseResult = Result<Value, TokenParseError>;

pub fn parse_tokens(tokens: &[Token], index: &mut usize) -> ParseResult {
    let token = &tokens[*index];
    if matches!(
        token,
        Token::Null | Token::False | Token::True | Token::Number(_) | Token::String(_)
    ) {
        *index += 1
    }
    match token {
        Token::Null => Ok(Value::Null),
        Token::False => Ok(Value::Boolean(false)),
        Token::True => Ok(Value::Boolean(true)),
        Token::Number(number) => Ok(Value::Number(*number)),
        Token::String(string) => parse_string(string),
        Token::LeftBracket => parse_array(tokens, index),
        Token::LeftBrace => parse_object(tokens, index),
        _ => Err(TokenParseError::ExpectedValue),
    }
}

fn parse_string(input: &str) -> ParseResult {
    let unescaped = unescape_string(input)?;
    Ok(Value::String(unescaped))
}

fn unescape_string(input: &str) -> Result<String, TokenParseError> {
    let mut output = String::new();

    let mut is_escaping = false;
    let mut chars = input.chars();
    while let Some(next_char) = chars.next() {
        if is_escaping {
            match next_char {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'b' => output.push('\u{8}'),
                'f' => output.push('\u{12}'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                'u' => {
                    let mut sum = 0;
                    for i in 0..4 {
                        let next_char = chars.next().ok_or(TokenParseError::UnfinishedEscape)?;
                        let digit = next_char
                            .to_digit(16)
                            .ok_or(TokenParseError::InvalidHexValue)?;
                        sum += (16u32).pow(3 - i) * digit;
                    }
                    let unescaped_char =
                        char::from_u32(sum).ok_or(TokenParseError::InvalidHexValue)?;
                    output.push(unescaped_char);
                }
                _ => output.push(next_char),
            }
            is_escaping = false;
        } else if next_char == '\\' {
            is_escaping = true;
        } else {
            output.push(next_char);
        }
    }
    Ok(output)
}

fn parse_array(tokens: &[Token], index: &mut usize) -> ParseResult {
    debug_assert!(tokens[*index] == Token::LeftBracket);

    let mut array: Vec<Value> = Vec::new();
    loop {
        *index += 1;
        if tokens[*index] == Token::RightBracket {
            break;
        }

        let value = parse_tokens(tokens, index)?;
        array.push(value);

        let token = &tokens[*index];
        match token {
            Token::Comma => {}
            Token::RightBracket => break,
            _ => return Err(TokenParseError::ExpectedComma),
        }
    }
    *index += 1;

    Ok(Value::Array(array))
}

fn parse_object(tokens: &[Token], index: &mut usize) -> ParseResult {
    debug_assert!(tokens[*index] == Token::LeftBrace);

    let mut map = BTreeMap::new();
    loop {
        *index += 1;
        if tokens[*index] == Token::RightBrace {
            break;
        }

        if let Token::String(s) = &tokens[*index] {
            *index += 1;
            if Token::Colon == tokens[*index] {
                *index += 1;
                let key = unescape_string(s)?;
                let value = parse_tokens(tokens, index)?;
                map.insert(key, value);
            } else {
                return Err(TokenParseError::ExpectedColon);
            }

            match &tokens[*index] {
                Token::Comma => {}
                Token::RightBrace => break,
                _ => return Err(TokenParseError::ExpectedComma),
            }
        } else {
            return Err(TokenParseError::ExpectedProperty);
        }
    }
    *index += 1;

    Ok(Value::Object(map))
}

#[derive(Debug, PartialEq)]
pub enum TokenParseError {
    UnclosedBracket,
    UnclosedBrace,

    UnfinishedEscape,
    InvalidHexValue,
    InvalidCodePointValue,

    ExpectedColon,
    ExpectedComma,
    ExpectedValue,
    ExpectedProperty,

    NeedsComma,
    TrailingComma,
}
