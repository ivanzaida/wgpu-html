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
    if s.starts_with('#') {
        return Some(CssColor::Hex(s.to_string()));
    }
    if is_named_color(s) {
        return Some(CssColor::Named(s.to_string()));
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

