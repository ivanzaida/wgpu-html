use crate::error::ParseError;
use crate::parser::parse_declaration;
use crate::selector::{SelectorList, parse_selector_list};
use crate::CssProperty;
use crate::value::CssValue;

/// One parsed CSS rule: a selector list with its declarations and computed specificity.
#[derive(Debug, Clone)]
pub struct StyleRule {
    pub selector: SelectorList,
    pub declarations: Vec<Declaration>,
    pub specificity: (u32, u32, u32),
}

/// A single `property: value` pair (potentially `!important`).
#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: CssProperty,
    pub value: CssValue,
    pub important: bool,
}

/// A full parsed stylesheet.
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
}

/// Parse a full CSS stylesheet string.
pub fn parse_stylesheet(input: &str) -> Result<Stylesheet, ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut rules = Vec::new();

    loop {
        skip_ws(&chars, &mut pos);
        if pos >= chars.len() { break; }

        // Skip @-rules for now
        if chars[pos] == '@' {
            skip_at_rule(&chars, &mut pos);
            continue;
        }

        // Parse the selector part (everything before '{')
        let sel_start = pos;
        let mut brace_depth = 0;
        loop {
            if pos >= chars.len() {
                return Err(ParseError::new("unexpected end of input", pos));
            }
            match chars[pos] {
                '{' => break,
                '(' => { brace_depth += 1; pos += 1; }
                ')' => {
                    if brace_depth == 0 {
                        return Err(ParseError::new("unexpected ')'", pos));
                    }
                    brace_depth -= 1;
                    pos += 1;
                }
            _ => pos += 1,
            }
        }
        let sel_str: String = chars[sel_start..pos].iter().collect();
        pos += 1; // skip '{'

        // Parse declaration block
        let decl_start = pos;
        let mut depth = 1;
        while pos < chars.len() && depth > 0 {
            match chars[pos] {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {},
            }
            pos += 1;
        }
        let decl_str: String = chars[decl_start..pos - 1].iter().collect();

        let selector = parse_selector_list(sel_str.trim())?;
        let declarations = parse_declaration_block(&decl_str)?;
        let specificity = compute_specificity(&selector);

        rules.push(StyleRule { selector, declarations, specificity });
    }

    Ok(Stylesheet { rules })
}

fn parse_declaration_block(input: &str) -> Result<Vec<Declaration>, ParseError> {
    let mut decls = Vec::new();
    for part in input.split(';') {
        let part = part.trim();
        if part.is_empty() { continue; }
        if let Some((prop, val)) = part.split_once(':') {
            let prop = prop.trim();
            let val_str = val.trim();
            let important = val_str.ends_with("!important");
            let val_str = if important {
                val_str.strip_suffix("!important").unwrap().trim()
            } else {
                val_str
            };
            let (property, value) = parse_declaration(prop, val_str)?;
            decls.push(Declaration { property, value, important });
        } else {
            return Err(ParseError::new(format!("invalid declaration: {part}"), 0));
        }
    }
    Ok(decls)
}

fn compute_specificity(selector: &SelectorList) -> (u32, u32, u32) {
    let mut a = 0u32;
    let mut b = 0u32;
    let mut c = 0u32;

    for complex in &selector.0 {
        for compound in &complex.compounds {
            if compound.id.is_some() { a += 1; }
            b += compound.classes.len() as u32;
            b += compound.attrs.len() as u32;
            for pseudo in &compound.pseudos {
                if pseudo.pseudo.name().starts_with("::") {
                    c += 1; // pseudo-element
                } else {
                    b += 1; // pseudo-class
                }
            }
            if let Some(ref tag) = compound.tag {
                if tag != "*" { c += 1; }
            }
        }
    }

    (a, b, c)
}

fn skip_ws(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_ascii_whitespace() { *pos += 1; }
}

fn skip_at_rule(chars: &[char], pos: &mut usize) {
    *pos += 1; // skip '@'
    while *pos < chars.len() && !chars[*pos].is_ascii_whitespace() && chars[*pos] != '{' {
        *pos += 1;
    }
    // If it's a block at-rule like @media { }, skip the block
    if *pos < chars.len() && chars[*pos] == '{' {
        *pos += 1;
        let mut depth = 1;
        while *pos < chars.len() && depth > 0 {
            match chars[*pos] {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {},
            }
            *pos += 1;
        }
    } else {
        // Statement at-rule like @import "..."; skip to next ';' or block end
        while *pos < chars.len() && chars[*pos] != ';' && chars[*pos] != '}' {
            *pos += 1;
        }
        if *pos < chars.len() && chars[*pos] == ';' { *pos += 1; }
    }
}
