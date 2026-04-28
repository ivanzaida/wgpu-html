//! CSS stylesheet parsing — selectors + rules.
//!
//! Scope: comma-separated selector lists, each entry being a chain of
//! compound selectors joined by the descendant combinator (whitespace).
//! A compound selector is the same simple-selector mix supported
//! before: optional tag (or universal `*`), optional id, any number
//! of classes, plus an optional set of dynamic pseudo-classes
//! (`:hover`, `:active`). Other combinators (`>`, `+`, `~`) and
//! unsupported pseudo-classes / pseudo-elements still drop the rule.

use std::collections::HashMap;

use wgpu_html_models::Style;

use crate::css_parser::{CssWideKeyword, parse_inline_style_decls};

/// One selector: a "subject" compound (tag/id/classes/universal) for
/// the element itself, plus an optional ordered list of ancestor
/// compounds that must be found somewhere up the parent chain
/// (descendant combinator).
///
/// `ancestors[0]` is the closest ancestor (the compound that appears
/// immediately to the left of the subject in the source); deeper
/// entries have to be found further up. Each ancestor compound is
/// itself a `Selector` with `ancestors` empty — the chain is flat by
/// convention.
/// Dynamic pseudo-classes that gate a compound selector against the
/// document's `InteractionState`. Currently only state-driven
/// pseudo-classes are supported; structural pseudo-classes (`:nth-`,
/// `:first-child`, …) drop the rule during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoClass {
    /// `:hover` — matches when this element is on the document's
    /// hover chain (see `InteractionState::hover_path`).
    Hover,
    /// `:active` — matches when this element is on the active
    /// (currently-pressed) chain.
    Active,
}

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
    /// Required ancestor compounds, ordered closest → furthest. Empty
    /// for the common simple-selector case. Entries are themselves
    /// `Selector`s but always have `ancestors` empty.
    pub ancestors: Vec<Selector>,
    /// Pseudo-classes the subject must satisfy. Multiple are AND'd.
    /// Each adds 1 to the class bucket of `specificity()`, matching
    /// CSS Selectors-4.
    pub pseudo_classes: Vec<PseudoClass>,
}

impl Selector {
    /// Standard CSS specificity packed into a u32:
    /// `(id_count << 16) | (class_count << 8) | tag_count`. For
    /// descendant chains, every compound contributes (so `.a .b`
    /// has specificity 2-classes, not 1). Pseudo-classes count as
    /// classes per CSS Selectors-4.
    pub fn specificity(&self) -> u32 {
        let mut total = self.compound_specificity();
        for a in &self.ancestors {
            total += a.compound_specificity();
        }
        total
    }

    /// Specificity of just this compound, ignoring any ancestors. Used
    /// internally and by `specificity()`.
    pub fn compound_specificity(&self) -> u32 {
        let id = if self.id.is_some() { 1 } else { 0 };
        let cls = (self.classes.len() + self.pseudo_classes.len()) as u32;
        let tag = if self.tag.is_some() { 1 } else { 0 };
        (id << 16) | (cls << 8) | tag
    }

    /// True iff this compound has at least one tag/id/class/universal
    /// constraint or pseudo-class. Used to reject empty compounds
    /// like `""` or stray syntax that parses to nothing.
    pub fn is_meaningful(&self) -> bool {
        self.universal
            || self.tag.is_some()
            || self.id.is_some()
            || !self.classes.is_empty()
            || !self.pseudo_classes.is_empty()
    }
}

/// One rule: any of the listed selectors triggers the declarations.
/// `declarations` holds the normal-importance properties; `important`
/// holds the ones marked `!important`. Cascade applies them in
/// separate passes per CSS-Cascade-3 §6.4.
///
/// `keywords` and `important_keywords` carry per-property CSS-wide
/// keywords (`inherit / initial / unset`) that override any matching
/// value the cascade has accumulated. Keys are CSS property names in
/// kebab-case (`color`, `font-size`, …).
#[derive(Debug, Clone, Default)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Style,
    pub important: Style,
    pub keywords: HashMap<String, CssWideKeyword>,
    pub important_keywords: HashMap<String, CssWideKeyword>,
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
        let Some(close) = after_open.find('}') else {
            break;
        };
        let body = &after_open[..close];

        let selectors = parse_selector_list(&header);
        let decls = parse_inline_style_decls(body);
        if !selectors.is_empty() {
            rules.push(Rule {
                selectors,
                declarations: decls.normal,
                important: decls.important,
                keywords: decls.keywords_normal,
                important_keywords: decls.keywords_important,
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

/// Parse a comma-separated entry into a (possibly descendant-chained)
/// selector. Returns `None` if the input contains an unsupported
/// combinator (`>`, `+`, `~`) or syntax we don't recognize.
fn parse_selector(s: &str) -> Option<Selector> {
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let mut compounds: Vec<Selector> = Vec::with_capacity(parts.len());
    for p in parts {
        compounds.push(parse_compound(p)?);
    }
    let mut subject = compounds.pop().expect("non-empty");
    // `compounds` now holds the non-subject parts in source order
    // (left → right). `ancestors[0]` is the closest ancestor (the
    // compound immediately to the subject's left), so reverse.
    compounds.reverse();
    subject.ancestors = compounds;
    Some(subject)
}

/// Parse one whitespace-free compound
/// (tag/id/classes/universal/pseudo-classes).
/// Returns `None` if the compound contains anything we don't handle.
fn parse_compound(s: &str) -> Option<Selector> {
    if s.is_empty() {
        return None;
    }
    let mut sel = Selector::default();
    let mut buf = String::new();
    #[derive(Copy, Clone)]
    enum Kind {
        Tag,
        Id,
        Class,
        Pseudo,
    }
    let mut kind = Kind::Tag;

    fn commit(buf: &mut String, kind: Kind, sel: &mut Selector) -> Option<()> {
        if buf.is_empty() {
            return Some(());
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
            Kind::Pseudo => match buf.as_str() {
                "hover" => {
                    sel.pseudo_classes.push(PseudoClass::Hover);
                    buf.clear();
                }
                "active" => {
                    sel.pseudo_classes.push(PseudoClass::Active);
                    buf.clear();
                }
                // Anything we don't recognize (`:focus`, `::before`,
                // `:nth-child`, …) drops the whole rule.
                _ => return None,
            },
        }
        Some(())
    }

    for ch in s.chars() {
        match ch {
            '#' => {
                commit(&mut buf, kind, &mut sel)?;
                kind = Kind::Id;
            }
            '.' => {
                commit(&mut buf, kind, &mut sel)?;
                kind = Kind::Class;
            }
            ':' => {
                commit(&mut buf, kind, &mut sel)?;
                kind = Kind::Pseudo;
            }
            c if c.is_alphanumeric() || c == '-' || c == '_' || c == '*' => buf.push(c),
            // Unsupported character in a single compound (other
            // combinators were already split off as whitespace;
            // attribute selectors `[a=b]` land here): drop the rule.
            _ => return None,
        }
    }
    commit(&mut buf, kind, &mut sel)?;

    if !sel.is_meaningful() {
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
    fn parses_descendant_combinator() {
        // `div p` → subject `p` with required ancestor `div`.
        let s = parse_selector("div p").unwrap();
        assert_eq!(s.tag.as_deref(), Some("p"));
        assert_eq!(s.ancestors.len(), 1);
        assert_eq!(s.ancestors[0].tag.as_deref(), Some("div"));
    }

    #[test]
    fn parses_three_level_descendant_chain() {
        // Subject `.c`, immediate ancestor `.b`, further `.a`.
        let s = parse_selector(".a .b .c").unwrap();
        assert_eq!(s.classes, vec!["c"]);
        assert_eq!(s.ancestors.len(), 2);
        assert_eq!(s.ancestors[0].classes, vec!["b"]);
        assert_eq!(s.ancestors[1].classes, vec!["a"]);
    }

    #[test]
    fn descendant_specificity_sums_compounds() {
        // `.a .b` → 2 classes worth of specificity, not 1.
        let two = parse_selector(".a .b").unwrap().specificity();
        let one = parse_selector(".b").unwrap().specificity();
        assert!(two > one);
    }

    #[test]
    fn rejects_unsupported_combinators() {
        // `>`, `+`, `~` still drop the rule.
        assert!(parse_selector("div > p").is_none());
        assert!(parse_selector("div + p").is_none());
        assert!(parse_selector("div ~ p").is_none());
    }

    #[test]
    fn rejects_unknown_pseudo_classes() {
        // We accept `:hover` / `:active` only; everything else drops.
        assert!(parse_selector("a:focus").is_none());
        assert!(parse_selector("p::before").is_none());
        assert!(parse_selector("li:nth-child").is_none());
    }

    #[test]
    fn parses_hover_pseudo_class() {
        let s = parse_selector("a:hover").unwrap();
        assert_eq!(s.tag.as_deref(), Some("a"));
        assert_eq!(s.pseudo_classes, vec![PseudoClass::Hover]);
    }

    #[test]
    fn parses_bare_hover_pseudo_class() {
        // `:hover { ... }` matches every hovered element.
        let s = parse_selector(":hover").unwrap();
        assert!(s.tag.is_none());
        assert!(s.id.is_none());
        assert_eq!(s.pseudo_classes, vec![PseudoClass::Hover]);
    }

    #[test]
    fn parses_pseudo_class_after_id_and_class() {
        let s = parse_selector("button#go.primary:hover:active").unwrap();
        assert_eq!(s.tag.as_deref(), Some("button"));
        assert_eq!(s.id.as_deref(), Some("go"));
        assert_eq!(s.classes, vec!["primary"]);
        assert_eq!(
            s.pseudo_classes,
            vec![PseudoClass::Hover, PseudoClass::Active]
        );
    }

    #[test]
    fn pseudo_class_adds_class_specificity() {
        // `a:hover` should beat plain `a` on specificity (1 class +
        // 1 tag vs 1 tag).
        let plain = parse_selector("a").unwrap().specificity();
        let hover = parse_selector("a:hover").unwrap().specificity();
        assert!(hover > plain);
        // Two pseudo-classes match a `.x.y` for specificity.
        let two_pc = parse_selector("a:hover:active").unwrap().specificity();
        let two_cls = parse_selector("a.x.y").unwrap().specificity();
        assert_eq!(two_pc, two_cls);
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
