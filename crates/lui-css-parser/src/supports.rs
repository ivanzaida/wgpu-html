use crate::error::ParseError;

/// A parsed `@supports` condition.
#[derive(Debug, Clone, PartialEq)]
pub enum SupportsCondition {
    Feature(SupportsFeature),
    Not(Box<SupportsCondition>),
    And(Vec<SupportsCondition>),
    Or(Vec<SupportsCondition>),
}

/// A single feature test like `(display: grid)` or `selector(.foo)`.
#[derive(Debug, Clone, PartialEq)]
pub struct SupportsFeature {
    pub name: String,
    pub value: Option<String>,
    pub is_selector: bool,
}

/// Parse the prelude of a `@supports` rule.
pub fn parse_supports_condition(input: &str) -> Result<SupportsCondition, ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    parse_condition(&chars, &mut pos)
}

fn parse_condition(chars: &[char], pos: &mut usize) -> Result<SupportsCondition, ParseError> {
    skip_ws(chars, pos);

    // `not <parens>`
    if let Some(_) = match_word(chars, pos, "not") {
        let inner = parse_parens(chars, pos)?;
        return Ok(SupportsCondition::Not(Box::new(inner)));
    }

    let first = parse_parens(chars, pos)?;
    skip_ws(chars, pos);

    // and/or chain
    if let Some(_) = match_word(chars, pos, "and") {
        let mut terms = vec![first];
        loop {
            terms.push(parse_parens(chars, pos)?);
            skip_ws(chars, pos);
            if match_word(chars, pos, "and").is_none() { break; }
        }
        return Ok(SupportsCondition::And(terms));
    }

    if let Some(_) = match_word(chars, pos, "or") {
        let mut terms = vec![first];
        loop {
            terms.push(parse_parens(chars, pos)?);
            skip_ws(chars, pos);
            if match_word(chars, pos, "or").is_none() { break; }
        }
        return Ok(SupportsCondition::Or(terms));
    }

    Ok(first)
}

fn parse_parens(chars: &[char], pos: &mut usize) -> Result<SupportsCondition, ParseError> {
    skip_ws(chars, pos);
    if *pos >= chars.len() || chars[*pos] != '(' {
        return Err(ParseError::new("expected '('", *pos));
    }
    *pos += 1;
    skip_ws(chars, pos);

    // Nested `( <condition> )`
    if *pos < chars.len() && chars[*pos] == '(' {
        let inner = parse_parens(chars, pos)?;
        skip_ws(chars, pos);
        expect(chars, pos, ')')?;
        return Ok(inner);
    }

    // `selector(...)` feature
    if let Some(_) = match_word(chars, pos, "selector") {
        if *pos < chars.len() && chars[*pos] == '(' {
            *pos += 1;
            let start = *pos;
            let mut depth = 1;
            while *pos < chars.len() && depth > 0 {
                match chars[*pos] { '(' => depth += 1, ')' => depth -= 1, _ => {} }
                *pos += 1;
            }
            let sel: String = chars[start..*pos - 1].iter().collect();
            let feat = SupportsFeature { name: "selector".into(), value: Some(sel.trim().into()), is_selector: true };
            // consume the outer ')'
            skip_ws(chars, pos);
            expect(chars, pos, ')')?;
            return Ok(SupportsCondition::Feature(feat));
        }
    }

    // `( property: value )` feature
    let name_start = *pos;
    while *pos < chars.len() && (chars[*pos].is_ascii_alphanumeric() || chars[*pos] == '-') {
        *pos += 1;
    }
    let name: String = chars[name_start..*pos].iter().collect();
    skip_ws(chars, pos);

    let value = if *pos < chars.len() && chars[*pos] == ':' {
        *pos += 1;
        skip_ws(chars, pos);
        let v_start = *pos;
        while *pos < chars.len() && chars[*pos] != ')' {
            *pos += 1;
        }
        Some(chars[v_start..*pos].iter().collect::<String>())
    } else {
        None
    };

    expect(chars, pos, ')')?;
    Ok(SupportsCondition::Feature(SupportsFeature { name, value, is_selector: false }))
}

fn match_word(chars: &[char], pos: &mut usize, word: &str) -> Option<()> {
    let saved = *pos;
    skip_ws(chars, pos);
    let start = *pos;
    while *pos < chars.len() && chars[*pos].is_ascii_alphabetic() { *pos += 1; }
    let found: String = chars[start..*pos].iter().collect();
    if found.to_lowercase() == word { Some(()) } else { *pos = saved; None }
}

fn expect(chars: &[char], pos: &mut usize, c: char) -> Result<(), ParseError> {
    skip_ws(chars, pos);
    if *pos >= chars.len() || chars[*pos] != c {
        return Err(ParseError::new(format!("expected '{c}'"), *pos));
    }
    *pos += 1;
    Ok(())
}

fn skip_ws(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_ascii_whitespace() { *pos += 1; }
}
