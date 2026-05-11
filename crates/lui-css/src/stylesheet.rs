use crate::CssAtRule;
use crate::CssProperty;
use crate::error::ParseError;
use crate::media::{MediaQueryList, parse_media_query_list};
use crate::parser::parse_declaration;
use crate::selector::{SelectorList, parse_selector_list};
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

/// A parsed at-rule (e.g. `@media`, `@keyframes`, `@font-face`).
#[derive(Debug, Clone)]
pub struct AtRule {
    pub at_rule: CssAtRule,
    pub prelude: String,
    pub media: Option<MediaQueryList>,
    pub rules: Vec<StyleRule>,
    pub at_rules: Vec<AtRule>,
}

/// A full parsed stylesheet.
#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<StyleRule>,
    pub at_rules: Vec<AtRule>,
}

/// Parse a full CSS stylesheet string.
pub fn parse_stylesheet(input: &str) -> Result<Stylesheet, ParseError> {
    let (stylesheet, _) = parse_rule_list(input, false, None)?;
    Ok(stylesheet)
}

fn parse_rule_list(input: &str, inside_at_rule: bool, at_rule_name: Option<&str>) -> Result<(Stylesheet, usize), ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut rules = Vec::new();
    let mut at_rules = Vec::new();

    loop {
        skip_ws(&chars, &mut pos);
        if pos >= chars.len() { break; }

        // Inside @keyframes, rules are keyframe selectors
        if inside_at_rule && at_rule_name == Some("@keyframes") {
            if let Some(rule) = parse_keyframe_rule(&chars, &mut pos)? {
                rules.push(rule);
                continue;
            }
        }

        // At-rule
        if chars[pos] == '@' {
            pos += 1; // skip '@'
            // Read at-rule name
            let name_start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '-') {
                pos += 1;
            }
            let name = String::from("@") + &chars[name_start..pos].iter().collect::<String>();
            skip_ws(&chars, &mut pos);

            // Read prelude (everything before '{' or ';')
            let prelude_start = pos;
            while pos < chars.len() && chars[pos] != '{' && chars[pos] != ';' {
                pos += 1;
            }
            let prelude: String = chars[prelude_start..pos].iter().collect();

            if pos < chars.len() && chars[pos] == '{' {
                // Block at-rule — parse nested rules
                pos += 1;
                let block_start = pos;
                let mut depth = 1;
                while pos < chars.len() && depth > 0 {
                    match chars[pos] {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {},
                    }
                    pos += 1;
                }
                let block: String = chars[block_start..pos - 1].iter().collect();
                let (inner, _) = parse_rule_list(&block, true, Some(&name))?;

                let prelude_str = prelude.trim().to_string();
                let media = if &*name == "@media" {
                    parse_media_query_list(&prelude_str).ok()
                } else {
                    None
                };

                at_rules.push(AtRule {
                    at_rule: CssAtRule::from_name(&name),
                    prelude: prelude_str,
                    media,
                    rules: inner.rules,
                    at_rules: inner.at_rules,
                });
            } else {
                // Statement at-rule — terminated by ';'
                if pos < chars.len() && chars[pos] == ';' { pos += 1; }
                at_rules.push(AtRule {
                    at_rule: CssAtRule::from_name(&name),
                    prelude: prelude.trim().to_string(),
                    media: None,
                    rules: vec![],
                    at_rules: vec![],
                });
            }
            continue;
        }

        // Skip garbage before selectors
        if !chars[pos].is_ascii_alphabetic() && chars[pos] != '.' && chars[pos] != '#' && chars[pos] != '*' && chars[pos] != '&' && chars[pos] != ':' && chars[pos] != '[' {
            pos += 1;
            continue;
        }

        // Regular style rule
        match parse_style_rule(&chars, &mut pos) {
            Ok(rule) => rules.push(rule),
            Err(_) => {
                // Skip to next '}' to recover
                while pos < chars.len() && chars[pos] != '}' { pos += 1; }
                if pos < chars.len() { pos += 1; }
            }
        }
    }

    Ok((Stylesheet { rules, at_rules }, pos))
}

fn parse_style_rule(chars: &[char], pos: &mut usize) -> Result<StyleRule, ParseError> {
    let sel_start = *pos;
    let mut brace_depth = 0;
    loop {
        if *pos >= chars.len() {
            return Err(ParseError::new("unexpected end of input", *pos));
        }
        match chars[*pos] {
            '{' => break,
            '(' => { brace_depth += 1; *pos += 1; }
            ')' => {
                if brace_depth == 0 { return Err(ParseError::new("unexpected ')'", *pos)); }
                brace_depth -= 1;
                *pos += 1;
            }
            _ => *pos += 1,
        }
    }
    let sel_str: String = chars[sel_start..*pos].iter().collect();
    *pos += 1;

    let decl_start = *pos;
    let mut depth = 1;
    while *pos < chars.len() && depth > 0 {
        match chars[*pos] {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {},
        }
        *pos += 1;
    }
    let decl_str: String = chars[decl_start..*pos - 1].iter().collect();

    let selector = parse_selector_list(sel_str.trim())?;
    let declarations = parse_declaration_block(&decl_str)?;
    let specificity = compute_specificity(&selector);

    Ok(StyleRule { selector, declarations, specificity })
}

fn parse_keyframe_rule(chars: &[char], pos: &mut usize) -> Result<Option<StyleRule>, ParseError> {
    let sel_start = *pos;

    // Keyframe selectors: "from", "to", or percentages
    while *pos < chars.len() && chars[*pos] != '{' {
        *pos += 1;
    }
    if *pos >= chars.len() { return Ok(None); }

    let sel_str: String = chars[sel_start..*pos].iter().collect();
    if !sel_str.trim().chars().any(|c| c.is_ascii_alphabetic() || c.is_ascii_digit()) {
        return Ok(None);
    }

    *pos += 1;
    let decl_start = *pos;
    let mut depth = 1;
    while *pos < chars.len() && depth > 0 {
        match chars[*pos] {
            '{' => depth += 1,
            '}' => depth -= 1,
            _ => {},
        }
        *pos += 1;
    }
    let decl_str: String = chars[decl_start..*pos - 1].iter().collect();

    let selector = SelectorList(vec![]); // Keyframe selectors aren't real selectors — placeholder
    let declarations = parse_declaration_block(&decl_str)?;
    let specificity = (0, 0, 0);

    Ok(Some(StyleRule { selector, declarations, specificity }))
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
        }
    }
    Ok(decls)
}

fn compute_specificity(selector: &SelectorList) -> (u32, u32, u32) {
    let mut a = 0u32; let mut b = 0u32; let mut c = 0u32;
    for complex in &selector.0 {
        for compound in &complex.compounds {
            if compound.id.is_some() { a += 1; }
            b += compound.classes.len() as u32;
            b += compound.attrs.len() as u32;
            for pseudo in &compound.pseudos {
                if pseudo.pseudo.name().starts_with("::") { c += 1; }
                else { b += 1; }
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
