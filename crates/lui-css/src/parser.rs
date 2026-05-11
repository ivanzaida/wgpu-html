use crate::tokenizer::{tokenize, Token};
use crate::value::CssValue;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl ParseError {
    pub fn new(msg: impl Into<String>, pos: usize) -> Self {
        ParseError { message: msg.into(), position: pos }
    }
}

/// Parse a CSS value string into a `CssValue` tree.
pub fn parse_value(input: &str) -> Result<CssValue, ParseError> {
    let tokens = tokenize(input);
    let (value, pos) = parse_tokens(&tokens, 0)?;
    if pos != tokens.len() {
        return Err(ParseError::new(format!("trailing tokens at position {}", pos), pos));
    }
    Ok(value)
}

fn parse_tokens(tokens: &[Token], pos: usize) -> Result<(CssValue, usize), ParseError> {
    if pos >= tokens.len() {
        return Err(ParseError::new("unexpected end of input", pos));
    }

    match &tokens[pos] {
        Token::Function(name) => {
            let function = crate::CssFunction::from_name(name)
                .ok_or_else(|| ParseError::new(format!("unknown function: {name}"), pos))?;

            let mut p = pos + 1;
            if p >= tokens.len() || tokens[p] != Token::Delim('(') {
                return Err(ParseError::new("expected '(' after function name", p));
            }
            p += 1;

            let mut args = Vec::new();
            loop {
                if p >= tokens.len() {
                    return Err(ParseError::new("expected ')'", p));
                }
                if tokens[p] == Token::Delim(')') {
                    p += 1;
                    break;
                }
                if !args.is_empty() {
                    if tokens[p] != Token::Delim(',') {
                        return Err(ParseError::new(format!("expected ',' between args, found {:?}", tokens[p]), p));
                    }
                    p += 1;
                }
                let (arg, next) = parse_tokens(tokens, p)?;
                args.push(arg);
                p = next;
            }

            Ok((CssValue::Function { function, args }, p))
        }

        Token::Number(n) => Ok((CssValue::Number(*n), pos + 1)),

        Token::Percentage(n) => Ok((CssValue::Percentage(*n), pos + 1)),

        Token::Dimension { value, unit } => {
            Ok((CssValue::Dimension { value: *value, unit: unit.clone() }, pos + 1))
        }

        Token::String(s) => Ok((CssValue::String(s.clone()), pos + 1)),

        Token::Ident(s) => Ok((CssValue::String(s.clone()), pos + 1)),

        Token::Delim(c) => Err(ParseError::new(format!("unexpected delimiter '{c}'"), pos)),
    }
}

