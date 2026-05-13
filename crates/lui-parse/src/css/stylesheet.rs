use lui_core::{
    AtRule, CssAtRule, Declaration, ParseError, SelectorList,
    StyleRule, Stylesheet,
};
use crate::css::media::parse_media_query_list;
use crate::css::parser::parse_declaration;
use crate::css::selector::{complex_specificity, parse_selector_list};
use crate::css::supports::parse_supports_condition;

pub fn parse_stylesheet(input: &str) -> Result<Stylesheet, ParseError> {
    let (stylesheet, _) = parse_rule_list(input, false, None)?;
    Ok(stylesheet)
}

fn parse_rule_list(input: &str, inside_at_rule: bool, at_rule_name: Option<&str>) -> Result<(Stylesheet, usize), ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = 0;
    let mut rules = Vec::new();
    let mut at_rules = Vec::new();
    let mut comments = Vec::new();

    loop {
        skip_ws_and_comments(&chars, &mut pos, &mut comments);
        if pos >= chars.len() { break; }

        if inside_at_rule && at_rule_name == Some("@keyframes") {
            if let Some(rule) = parse_keyframe_rule(&chars, &mut pos)? {
                rules.push(rule);
                continue;
            }
        }

        if chars[pos] == '@' {
            pos += 1;
            let name_start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '-') {
                pos += 1;
            }
            let name = String::from("@") + &chars[name_start..pos].iter().collect::<String>();
            skip_ws_and_comments(&chars, &mut pos, &mut comments);

            let prelude_start = pos;
            while pos < chars.len() && chars[pos] != '{' && chars[pos] != ';' {
                pos += 1;
            }
            let prelude: String = chars[prelude_start..pos].iter().collect();

            if pos < chars.len() && chars[pos] == '{' {
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
                let supports = if &*name == "@supports" {
                    parse_supports_condition(&prelude_str).ok()
                } else {
                    None
                };

                at_rules.push(AtRule {
                    at_rule: CssAtRule::from_name(&name),
                    prelude: prelude_str,
                    media,
                    supports,
                    rules: inner.rules,
                    at_rules: inner.at_rules,
                    comments: inner.comments,
                });
            } else {
                if pos < chars.len() && chars[pos] == ';' { pos += 1; }
                at_rules.push(AtRule {
                    at_rule: CssAtRule::from_name(&name),
                    prelude: prelude.trim().to_string(),
                    media: None,
                    supports: None,
                    rules: vec![],
                    at_rules: vec![],
                    comments: vec![],
                });
            }
            continue;
        }

        if !chars[pos].is_ascii_alphabetic() && chars[pos] != '.' && chars[pos] != '#' && chars[pos] != '*' && chars[pos] != '&' && chars[pos] != ':' && chars[pos] != '[' {
            pos += 1;
            continue;
        }

        match parse_style_rule(&chars, &mut pos) {
            Ok(rule) => rules.push(rule),
            Err(_) => {
                while pos < chars.len() && chars[pos] != '}' { pos += 1; }
                if pos < chars.len() { pos += 1; }
            }
        }
    }

    Ok((Stylesheet { rules, at_rules, comments }, pos))
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

    let selector = SelectorList(vec![]);
    let declarations = parse_declaration_block(&decl_str)?;
    let specificity = (0, 0, 0);

    Ok(Some(StyleRule { selector, declarations, specificity }))
}

pub fn parse_declaration_block(input: &str) -> Result<Vec<Declaration>, ParseError> {
    let mut decls = Vec::new();
    for part in input.split(';') {
        let part = part.trim();
        if part.is_empty() { continue; }
        if let Some((prop, val)) = part.split_once(':') {
            let prop = prop.trim();
            let val_str = strip_comments(val.trim());
            if val_str.is_empty() { continue; }
            let important = val_str.ends_with("!important");
            let val_str = if important {
                val_str.strip_suffix("!important").unwrap().trim()
            } else {
                &val_str
            };
            match parse_declaration(prop, val_str) {
                Ok((property, value)) => decls.push(Declaration { property, value, important }),
                Err(_) => {
                    let property = lui_core::CssProperty::from_name(prop);
                    let value = lui_core::CssValue::String(lui_core::ArcStr::from(val_str));
                    decls.push(Declaration { property, value, important });
                }
            }
        }
    }
    Ok(decls)
}

fn strip_comments(input: &str) -> String {
    if !input.contains("/*") { return input.to_string(); }
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' { i += 2; break; }
                i += 1;
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

fn compute_specificity(selector: &SelectorList) -> (u32, u32, u32) {
    selector.0.iter()
        .map(|complex| complex_specificity(complex))
        .max()
        .unwrap_or((0, 0, 0))
}

fn skip_ws_and_comments(chars: &[char], pos: &mut usize, comments: &mut Vec<String>) {
    loop {
        while *pos < chars.len() && chars[*pos].is_ascii_whitespace() { *pos += 1; }
        if *pos + 1 < chars.len() && chars[*pos] == '/' && chars[*pos + 1] == '*' {
            let comment_start = *pos;
            *pos += 2;
            while *pos + 1 < chars.len() {
                if chars[*pos] == '*' && chars[*pos + 1] == '/' {
                    *pos += 2;
                    break;
                }
                *pos += 1;
            }
            let comment: String = chars[comment_start + 2..*pos - 2].iter().collect();
            comments.push(comment);
        } else {
            break;
        }
    }
}
