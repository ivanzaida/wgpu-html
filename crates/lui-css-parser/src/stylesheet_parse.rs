use crate::{
    AtRule, CssAtRule, Declaration, ParseError, SelectorList,
    StyleRule, Stylesheet,
};
use crate::media_parse::parse_media_query_list;
use crate::parser::parse_declaration;
use crate::selector_parse::{complex_specificity, parse_selector_list};
use crate::supports_parse::parse_supports_condition;

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
            Ok(parsed_rules) => rules.extend(parsed_rules),
            Err(_) => {
                while pos < chars.len() && chars[pos] != '}' { pos += 1; }
                if pos < chars.len() { pos += 1; }
            }
        }
    }

    Ok((Stylesheet { rules, at_rules, comments }, pos))
}

fn parse_style_rule(chars: &[char], pos: &mut usize) -> Result<Vec<StyleRule>, ParseError> {
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

    parse_style_rule_from_parts(sel_str.trim(), &decl_str)
}

fn parse_style_rule_from_parts(selector_src: &str, block_src: &str) -> Result<Vec<StyleRule>, ParseError> {
    let (declaration_src, nested_blocks) = split_declarations_and_nested_rules(block_src)?;
    let selector = parse_selector_list(selector_src)?;
    let declarations = parse_declaration_block(&declaration_src)?;
    let specificity = compute_specificity(&selector);
    let mut rules = Vec::new();

    if !declarations.is_empty() {
        rules.push(StyleRule { selector, declarations, specificity });
    }

    for nested in nested_blocks {
        let nested_selector = expand_nested_selector_list(selector_src, &nested.selector)?;
        rules.extend(parse_style_rule_from_parts(&nested_selector, &nested.block)?);
    }

    Ok(rules)
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

struct NestedRuleBlock {
    selector: String,
    block: String,
}

fn split_declarations_and_nested_rules(input: &str) -> Result<(String, Vec<NestedRuleBlock>), ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut declarations = String::new();
    let mut nested = Vec::new();
    let mut segment_start = 0;
    let mut pos = 0;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while pos < chars.len() {
        match chars[pos] {
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' if bracket_depth > 0 => bracket_depth -= 1,
            '{' if paren_depth == 0 && bracket_depth == 0 => {
                let segment: String = chars[segment_start..pos].iter().collect();
                let (declaration_prefix, nested_selector) = split_before_nested_selector(&segment);
                declarations.push_str(declaration_prefix);
                if !declaration_prefix.trim().is_empty() && !declaration_prefix.trim_end().ends_with(';') {
                    declarations.push(';');
                }

                let block_start = pos + 1;
                let mut depth = 1usize;
                pos += 1;
                while pos < chars.len() && depth > 0 {
                    match chars[pos] {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {}
                    }
                    pos += 1;
                }
                if depth != 0 {
                    return Err(ParseError::new("unclosed nested rule block", block_start));
                }
                let block: String = chars[block_start..pos - 1].iter().collect();
                let nested_selector = nested_selector.trim();
                if !nested_selector.is_empty() {
                    nested.push(NestedRuleBlock {
                        selector: nested_selector.to_string(),
                        block,
                    });
                }
                segment_start = pos;
                continue;
            }
            _ => {}
        }
        pos += 1;
    }

    declarations.push_str(&chars[segment_start..].iter().collect::<String>());
    Ok((declarations, nested))
}

fn split_before_nested_selector(segment: &str) -> (&str, &str) {
    if let Some(idx) = segment.rfind(';') {
        segment.split_at(idx + 1)
    } else {
        ("", segment)
    }
}

fn expand_nested_selector_list(parent_src: &str, nested_src: &str) -> Result<String, ParseError> {
    let parents = split_selector_list_src(parent_src);
    let nested = split_selector_list_src(nested_src);
    let mut expanded = Vec::new();

    for parent in &parents {
        for child in &nested {
            let child = child.trim();
            if child.is_empty() {
                continue;
            }
            if child.contains('&') {
                expanded.push(child.replace('&', parent.trim()));
            } else if starts_with_combinator(child) {
                expanded.push(format!("{} {}", parent.trim(), child));
            } else {
                expanded.push(format!("{} {}", parent.trim(), child));
            }
        }
    }

    let expanded = expanded.join(", ");
    parse_selector_list(&expanded)?;
    Ok(expanded)
}

fn starts_with_combinator(selector: &str) -> bool {
    selector.starts_with('>') || selector.starts_with('+') || selector.starts_with('~') || selector.starts_with("||")
}

fn split_selector_list_src(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut parts = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (i, ch) in chars.iter().enumerate() {
        match *ch {
            '(' => paren_depth += 1,
            ')' if paren_depth > 0 => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' if bracket_depth > 0 => bracket_depth -= 1,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                parts.push(chars[start..i].iter().collect::<String>());
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(chars[start..].iter().collect::<String>());
    parts
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
            let (property, value) = parse_declaration(prop, val_str)?;
            decls.push(Declaration { property, value, important });
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
