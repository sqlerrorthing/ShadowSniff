use alloc::string::String;
use alloc::vec::Vec;
use core::num::ParseFloatError;

#[derive(Debug, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Null,
    False,
    True,
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenizeError {
    CharNotRecognized(char),
    ParseNumberError(ParseFloatError),
    UnclosedQuotes,
    UnfinishedLiteralValue,
}

pub fn tokenize<S>(input: S) -> Result<Vec<Token>, TokenizeError>
where 
    S: AsRef<str>,
{
    let chars: Vec<char> = input.as_ref().chars().collect();
    let mut index = 0;

    let mut tokens = Vec::new();
    while index < chars.len() {
        let Some(token) = make_token(&chars, &mut index)? else {
            break; // no more tokens
        };
        
        tokens.push(token);
        index += 1;
    }
    Ok(tokens)
}

fn make_token(chars: &Vec<char>, index: &mut usize) -> Result<Option<Token>, TokenizeError> {
    let mut ch = chars[*index];
    while ch.is_ascii_whitespace() {
        *index += 1;
        if *index >= chars.len() {
            return Ok(None);
        }
        ch = chars[*index];
    }
    
    let token = match ch {
        '[' => Token::LeftBracket,
        ']' => Token::RightBracket,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ',' => Token::Comma,
        ':' => Token::Colon,

        'n' => tokenize_null(chars, index)?,
        't' => tokenize_true(chars, index)?,
        'f' => tokenize_false(chars, index)?,

        c if c.is_ascii_digit() || c == '-' => tokenize_float(chars, index)?,

        '"' => tokenize_string(chars, index)?,

        ch => return Err(TokenizeError::CharNotRecognized(ch)),
    };

    Ok(Some(token))
}

fn tokenize_null(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "null".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1;
    Ok(Token::Null)
}

fn tokenize_true(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "true".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1;
    Ok(Token::True)
}

fn tokenize_false(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "false".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1;
    Ok(Token::False)
}

fn tokenize_string(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    debug_assert!(chars[*index] == '"');
    let mut string = String::new();
    let mut is_escaping = false;

    loop {
        *index += 1;
        if *index >= chars.len() {
            return Err(TokenizeError::UnclosedQuotes);
        }

        let ch = chars[*index];
        match ch {
            '"' if !is_escaping => break,
            '\\' => is_escaping = !is_escaping,
            _ => is_escaping = false,
        }

        string.push(ch);
    }

    Ok(Token::String(string))
}

fn tokenize_float(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    let mut unparsed_num = String::new();
    let mut has_decimal = false;
    let mut has_exponent = false;

    while *index < chars.len() {
        let ch = chars[*index];
        match ch {
            c if c.is_ascii_digit() || c == '-' => {
                unparsed_num.push(c);
            }
            '.' if !has_decimal && !has_exponent => {
                unparsed_num.push('.');
                has_decimal = true;
            }
            'e' | 'E' if !has_exponent => {
                unparsed_num.push('e');
                has_exponent = true;
                *index += 1;
                if *index < chars.len() {
                    let next_ch = chars[*index];
                    if next_ch == '+' || next_ch == '-' {
                        unparsed_num.push(next_ch);
                    } else {
                        *index -= 1;
                    }
                }
            }
            _ => break,
        }
        *index += 1;
    }

    if *index > 0 {
        *index -= 1;
    }

    match unparsed_num.parse() {
        Ok(f) => Ok(Token::Number(f)),
        Err(err) => Err(TokenizeError::ParseNumberError(err)),
    }
}