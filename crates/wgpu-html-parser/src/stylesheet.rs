//! CSS stylesheet parsing — selectors + rules.
//!
//! Scope: simple selectors only (combinations of tag, id, classes on a
//! single element), comma-separated selector lists. No combinators
//! (descendant, child, sibling). Universal `*` matches anything.

use wgpu_html_models::Style;

use crate::css_parser::parse_inline_style;

/// One simple selector: optionally a tag (or universal), optionally an id,
/// any number of classes. All conditions must hold for a match.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selector {
    /// `Some("div")` for `div`, `None` for universal `*` or no tag part.
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    /// True for selectors written as `*` (or `*.foo`, etc.). With this set
    /// the selector still matches even if no tag/id/class constraints
    /// remain after the universal.
    pub universal: bool,
}

impl Selector {
    /// Standard CSS specificity packed into a u32:
    /// (id_count << 16) | (class_count << 8) | tag_count.
    pub fn specificity(&self) -> u32 {
        let id = if self.id.is_some() { 1 } else { 0 };
        let cls = self.classes.len() as u32;
        let tag = if self.tag.is_some() { 1 } else { 0 };
        (id << 16) | (cls << 8) | tag
    }
}

/// One rule: any of the listed selectors triggers the declarations.
#[derive(Debug, Clone)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Style,
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn append(&mut self, other: Stylesheet) {
        self.rules.extend(other.rules);
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

pub fn parse_stylesheet(css: &str) -> Stylesheet {
    let mut rules = Vec::new();
    let mut input = strip_comments(css);

    loop {
        input = trim_left(&input).to_string();
        if input.is_empty() {
            break;
        }

        let Some(open) = input.find('{') else { break };
        let header = input[..open].trim().to_string();
        let after_open = &input[open + 1..];
        let Some(close) = after_open.find('}') else { break };
        let body = &after_open[..close];

        let selectors = parse_selector_list(&header);
        let declarations = parse_inline_style(body);
        if !selectors.is_empty() {
            rules.push(Rule {
                selectors,
                declarations,
            });
        }

        input = after_open[close + 1..].to_string();
    }

    Stylesheet { rules }
}

fn parse_selector_list(s: &str) -> Vec<Selector> {
    s.split(',')
        .filter_map(|part| parse_selector(part.trim()))
        .collect()
}

fn parse_selector(s: &str) -> Option<Selector> {
    if s.is_empty() {
        return None;
    }
    let mut sel = Selector::default();
    let mut buf = String::new();
    // What kind of identifier is currently in `buf`?
    #[derive(Copy, Clone)]
    enum Kind {
        Tag,
        Id,
        Class,
    }
    let mut kind = Kind::Tag;

    let commit = |buf: &mut String, kind: Kind, sel: &mut Selector| {
        if buf.is_empty() {
            return;
        }
        match kind {
            Kind::Tag => {
                if buf == "*" {
                    sel.universal = true;
                    buf.clear();
                } else {
                    sel.tag = Some(std::mem::take(buf));
                }
            }
            Kind::Id => sel.id = Some(std::mem::take(buf)),
            Kind::Class => sel.classes.push(std::mem::take(buf)),
        }
    };

    for ch in s.chars() {
        match ch {
            '#' => {
                commit(&mut buf, kind, &mut sel);
                kind = Kind::Id;
            }
            '.' => {
                commit(&mut buf, kind, &mut sel);
                kind = Kind::Class;
            }
            c if c.is_alphanumeric() || c == '-' || c == '_' || c == '*' => buf.push(c),
            // Anything else (whitespace, combinators, attribute selectors, …)
            // means we don't yet support this selector — skip the rule.
            _ => return None,
        }
    }
    commit(&mut buf, kind, &mut sel);

    if !sel.universal
        && sel.tag.is_none()
        && sel.id.is_none()
        && sel.classes.is_empty()
    {
        return None;
    }
    Some(sel)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn trim_left(s: &str) -> &str {
    s.trim_start()
}

/// Strip C-style block comments. The CSS spec only allows `/* */`.
fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tag_selector() {
        let s = parse_selector("div").unwrap();
        assert_eq!(s.tag.as_deref(), Some("div"));
        assert!(s.id.is_none());
        assert!(s.classes.is_empty());
    }

    #[test]
    fn parses_id_selector() {
        let s = parse_selector("#hero").unwrap();
        assert_eq!(s.id.as_deref(), Some("hero"));
        assert!(s.tag.is_none());
    }

    #[test]
    fn parses_class_selector() {
        let s = parse_selector(".card").unwrap();
        assert_eq!(s.classes, vec!["card"]);
    }

    #[test]
    fn parses_compound_selector() {
        let s = parse_selector("div#hero.card.big").unwrap();
        assert_eq!(s.tag.as_deref(), Some("div"));
        assert_eq!(s.id.as_deref(), Some("hero"));
        assert_eq!(s.classes, vec!["card", "big"]);
    }

    #[test]
    fn universal_keeps_tag_none() {
        let s = parse_selector("*").unwrap();
        assert!(s.tag.is_none());
    }

    #[test]
    fn rejects_combinators() {
        assert!(parse_selector("div p").is_none());
        assert!(parse_selector("div > p").is_none());
    }

    #[test]
    fn parses_simple_stylesheet() {
        let css = r#"
            #parent { width: 100px; padding: 10px; }
            .child { width: 30px; height: 30px; }
            #c1 { background-color: red; }
        "#;
        let sheet = parse_stylesheet(css);
        assert_eq!(sheet.rules.len(), 3);
        assert_eq!(sheet.rules[0].selectors[0].id.as_deref(), Some("parent"));
        assert_eq!(sheet.rules[1].selectors[0].classes, vec!["child"]);
        assert!(sheet.rules[2].declarations.background_color.is_some());
    }

    #[test]
    fn handles_comma_lists() {
        let sheet = parse_stylesheet("h1, h2, .big { color: red; }");
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selectors.len(), 3);
    }

    #[test]
    fn strips_comments() {
        let sheet = parse_stylesheet("/* hi */ .x { /* ok */ color: red; }");
        assert_eq!(sheet.rules.len(), 1);
    }

    #[test]
    fn specificity_ordering() {
        let id = parse_selector("#a").unwrap().specificity();
        let cls = parse_selector(".a").unwrap().specificity();
        let tag = parse_selector("a").unwrap().specificity();
        assert!(id > cls);
        assert!(cls > tag);
    }
}
