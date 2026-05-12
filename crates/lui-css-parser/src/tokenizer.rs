use crate::unit::CssUnit;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Function(String),
    Number(f64),
    Percentage(f64),
    Dimension { value: f64, unit: CssUnit },
    Ident(String),
    String(String),
    Delim(char),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < chars.len() {
        if chars[pos].is_ascii_whitespace() {
            pos += 1;
            continue;
        }

        // CSS comment: /* ... */
        if chars[pos] == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
            pos += 2;
            while pos + 1 < chars.len() {
                if chars[pos] == '*' && chars[pos + 1] == '/' {
                    pos += 2;
                    break;
                }
                pos += 1;
            }
            continue;
        }

        if matches!(chars[pos], ',' | '/' | '(' | ')') {
            tokens.push(Token::Delim(chars[pos]));
            pos += 1;
            continue;
        }

        if chars[pos] == '"' || chars[pos] == '\'' {
            let quote = chars[pos];
            pos += 1;
            let mut s = String::new();
            while pos < chars.len() && chars[pos] != quote {
                if chars[pos] == '\\' {
                    pos += 1;
                    if pos < chars.len() { s.push(chars[pos]); }
                } else {
                    s.push(chars[pos]);
                }
                pos += 1;
            }
            tokens.push(Token::String(s));
            if pos < chars.len() { pos += 1; }
            continue;
        }

        let is_number_start = matches!(chars[pos], '+' | '-')
            && pos + 1 < chars.len()
            && (chars[pos + 1].is_ascii_digit() || chars[pos + 1] == '.');

        if is_number_start || chars[pos].is_ascii_digit()
            || (chars[pos] == '.' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit())
        {
            let start = pos;
            if matches!(chars[pos], '+' | '-') { pos += 1; }
            while pos < chars.len() && chars[pos].is_ascii_digit() { pos += 1; }
            if pos < chars.len() && chars[pos] == '.' {
                pos += 1;
                while pos < chars.len() && chars[pos].is_ascii_digit() { pos += 1; }
            }
            let num_str: String = chars[start..pos].iter().collect();
            let value: f64 = num_str.parse().unwrap_or(f64::NAN);

            if pos < chars.len() && chars[pos] == '%' {
                tokens.push(Token::Percentage(value));
                pos += 1;
            } else if pos < chars.len() && chars[pos].is_ascii_alphabetic() {
                let unit_start = pos;
                while pos < chars.len() && chars[pos].is_ascii_alphabetic() { pos += 1; }
                let unit_str: String = chars[unit_start..pos].iter().collect();
                tokens.push(Token::Dimension { value, unit: CssUnit::from_str(&unit_str) });
            } else {
                tokens.push(Token::Number(value));
            }
            continue;
        }

        let is_ident_start = chars[pos] == '_'
            || chars[pos].is_ascii_alphabetic()
            || (chars[pos] == '-' && pos + 1 < chars.len()
                && (chars[pos + 1].is_ascii_alphabetic() || chars[pos + 1] == '-' || chars[pos + 1] == '_'));

        if is_ident_start {
            let start = pos;
            pos += 1;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '-' || chars[pos] == '_') {
                pos += 1;
            }
            let ident: String = chars[start..pos].iter().collect();

            let mut peek = pos;
            while peek < chars.len() && chars[peek].is_ascii_whitespace() { peek += 1; }
            if peek < chars.len() && chars[peek] == '(' {
                tokens.push(Token::Function(ident));
            } else {
                tokens.push(Token::Ident(ident));
            }
            continue;
        }

        tokens.push(Token::Delim(chars[pos]));
        pos += 1;
    }

    tokens
}

