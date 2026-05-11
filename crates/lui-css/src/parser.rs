use crate::color::{CssColor, NamedColor};
use crate::css_property::CssProperty;
use crate::error::ParseError;
use crate::tokenizer::{tokenize, Token};
use crate::value::CssValue;

/// Parse a full CSS declaration `property: value`. Unknown properties and functions are
/// represented as their respective `Unknown(_)` variants, never as errors.
pub fn parse_declaration(property: &str, value: &str) -> Result<(CssProperty, CssValue), ParseError> {
    let prop = CssProperty::from_name(property);
    let val = parse_value(value)?;
    Ok((prop, val))
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
            let function = crate::CssFunction::from_name(name);

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

        Token::Ident(s) => {
            if let Some(color) = parse_color(s) {
                Ok((CssValue::Color(color), pos + 1))
            } else {
                Ok((CssValue::String(s.clone()), pos + 1))
            }
        }

        Token::Delim(c) => Err(ParseError::new(format!("unexpected delimiter '{c}'"), pos)),
    }
}

fn parse_color(s: &str) -> Option<CssColor> {
    if let Some(hex) = parse_hex_color(s) { return Some(hex); }
    NamedColor::from_name(s).map(CssColor::Named)
}

fn parse_hex_color(s: &str) -> Option<CssColor> {
    let s = s.strip_prefix('#')?;
    if s.chars().any(|c| !c.is_ascii_hexdigit()) { return None; }
    match s.len() {
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            Some(CssColor::Hex { r, g, b, a: None })
        }
        4 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&s[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&s[2..3], 16).ok()? * 17;
            let a = u8::from_str_radix(&s[3..4], 16).ok()? * 17;
            Some(CssColor::Hex { r, g, b, a: Some(a) })
        }
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some(CssColor::Hex { r, g, b, a: None })
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some(CssColor::Hex { r, g, b, a: Some(a) })
        }
        _ => None,
    }
}

