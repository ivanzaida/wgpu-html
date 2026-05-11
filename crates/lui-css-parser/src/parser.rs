use crate::ArcStr;
use crate::color::CssColor;
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
            // var() gets special parsing
            if name == "var" {
                return parse_var_function(tokens, pos);
            }
            // url() gets special parsing — elide the function wrapper
            if name == "url" {
                return parse_url_function(tokens, pos);
            }

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
                // Skip separators: commas (legacy) and slashes (modern alpha separator)
                if tokens[p] == Token::Delim(',') || tokens[p] == Token::Delim('/') {
                    p += 1;
                    continue;
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

        Token::String(s) => Ok((CssValue::String(s.clone().into()), pos + 1)),

        Token::Ident(s) => {
            if let Some(color) = parse_color(s) {
                Ok((CssValue::Color(color), pos + 1))
            } else {
                Ok((CssValue::String(s.clone().into()), pos + 1))
            }
        }

        Token::Delim(c) => Err(ParseError::new(format!("unexpected delimiter '{c}'"), pos)),
    }
}

fn parse_color(s: &str) -> Option<CssColor> {
    if s.starts_with('#') {
        return Some(CssColor::Hex(s.into()));
    }
    if is_named_color(s) {
        return Some(CssColor::Named(s.into()));
    }
    None
}

fn is_named_color(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(),
        "transparent" | "currentcolor" | "aliceblue" | "antiquewhite" | "aqua" | "aquamarine"
        | "azure" | "beige" | "bisque" | "black" | "blanchedalmond" | "blue" | "blueviolet"
        | "brown" | "burlywood" | "cadetblue" | "chartreuse" | "chocolate" | "coral"
        | "cornflowerblue" | "cornsilk" | "crimson" | "cyan" | "darkblue" | "darkcyan"
        | "darkgoldenrod" | "darkgray" | "darkgreen" | "darkgrey" | "darkkhaki" | "darkmagenta"
        | "darkolivegreen" | "darkorange" | "darkorchid" | "darkred" | "darksalmon"
        | "darkseagreen" | "darkslateblue" | "darkslategray" | "darkslategrey" | "darkturquoise"
        | "darkviolet" | "deeppink" | "deepskyblue" | "dimgray" | "dimgrey" | "dodgerblue"
        | "firebrick" | "floralwhite" | "forestgreen" | "fuchsia" | "gainsboro" | "ghostwhite"
        | "gold" | "goldenrod" | "gray" | "green" | "greenyellow" | "grey" | "honeydew"
        | "hotpink" | "indianred" | "indigo" | "ivory" | "khaki" | "lavender" | "lavenderblush"
        | "lawngreen" | "lemonchiffon" | "lightblue" | "lightcoral" | "lightcyan"
        | "lightgoldenrodyellow" | "lightgray" | "lightgreen" | "lightgrey" | "lightpink"
        | "lightsalmon" | "lightseagreen" | "lightskyblue" | "lightslategray" | "lightslategrey"
        | "lightsteelblue" | "lightyellow" | "lime" | "limegreen" | "linen" | "magenta"
        | "maroon" | "mediumaquamarine" | "mediumblue" | "mediumorchid" | "mediumpurple"
        | "mediumseagreen" | "mediumslateblue" | "mediumspringgreen" | "mediumturquoise"
        | "mediumvioletred" | "midnightblue" | "mintcream" | "mistyrose" | "moccasin"
        | "navajowhite" | "navy" | "oldlace" | "olive" | "olivedrab" | "orange" | "orangered"
        | "orchid" | "palegoldenrod" | "palegreen" | "paleturquoise" | "palevioletred"
        | "papayawhip" | "peachpuff" | "peru" | "pink" | "plum" | "powderblue" | "purple"
        | "rebeccapurple" | "red" | "rosybrown" | "royalblue" | "saddlebrown" | "salmon"
        | "sandybrown" | "seagreen" | "seashell" | "sienna" | "silver" | "skyblue"
        | "slateblue" | "slategray" | "slategrey" | "snow" | "springgreen" | "steelblue"
        | "tan" | "teal" | "thistle" | "tomato" | "turquoise" | "violet" | "wheat" | "white"
        | "whitesmoke" | "yellow" | "yellowgreen"
    )
}

fn parse_url_function(tokens: &[Token], pos: usize) -> Result<(CssValue, usize), ParseError> {
    let mut p = pos + 1; // skip Function token
    if p >= tokens.len() || tokens[p] != Token::Delim('(') {
        return Err(ParseError::new("expected '(' after url", p));
    }
    p += 1;

    // Consume everything until ')' as the URL string
    let url_start = p;
    while p < tokens.len() && tokens[p] != Token::Delim(')') {
        p += 1;
    }
    if p >= tokens.len() {
        return Err(ParseError::new("expected ')'", p));
    }

    let url: ArcStr = tokens[url_start..p].iter().map(|t| match t {
        Token::String(s) => s.clone(),
        Token::Ident(s) => s.clone(),
        Token::Delim(c) => c.to_string(),
        _ => String::new(),
    }).collect::<String>().trim().into();

    p += 1; // skip ')'
    Ok((CssValue::Url(url), p))
}

fn parse_var_function(tokens: &[Token], pos: usize) -> Result<(CssValue, usize), ParseError> {
    let mut p = pos + 1; // skip Function token
    if p >= tokens.len() || tokens[p] != Token::Delim('(') {
        return Err(ParseError::new("expected '(' after var", p));
    }
    p += 1;

    // Read property name (must start with --)
    if p >= tokens.len() {
        return Err(ParseError::new("expected custom property name", p));
    }
    let name = match &tokens[p] {
        Token::Ident(s) | Token::String(s) => {
            if !s.starts_with("--") {
                return Err(ParseError::new(format!("var() expects a custom property starting with --, got {s}"), p));
            }
            p += 1;
            s.clone().into()
        }
        _ => return Err(ParseError::new("expected custom property name", p)),
    };

    // Optional fallback after comma
    let fallback = if p < tokens.len() && tokens[p] == Token::Delim(',') {
        p += 1;
        let (val, next) = parse_tokens(tokens, p)?;
        p = next;
        Some(Box::new(val))
    } else {
        None
    };

    if p >= tokens.len() || tokens[p] != Token::Delim(')') {
        return Err(ParseError::new("expected ')'", p));
    }
    p += 1;

    Ok((CssValue::Var { name, fallback }, p))
}

