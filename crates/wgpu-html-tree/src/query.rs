//! DOM-style query helpers: `querySelector` / `querySelectorAll`.
//!
//! Implements Phase 1 of `query_selector_support_reference.md`:
//!
//! - Universal `*`, type `E`, id `#id`, class `.class`
//! - Attribute selectors with all six operators
//!   (`[a]`, `[a=v]`, `[a~=v]`, `[a|=v]`, `[a^=v]`, `[a$=v]`, `[a*=v]`)
//! - Case flags after the value: `[a=v i]` (ASCII case-insensitive),
//!   `[a=v s]` (case-sensitive — the default)
//! - Compound selectors (`a.btn[disabled]`)
//! - Selector lists (`A, B, C`)
//! - All four combinators: descendant ` `, child `>`, next-sibling
//!   `+`, subsequent-sibling `~`
//!
//! Pseudo-classes (`:is()`, `:not()`, `:nth-child()`, `:hover`, …)
//! and pseudo-elements (`::before`) are *not* parsed yet and trigger
//! the parser's lenient "no match" sentinel via `From<&str>`.
//!
//! HTML matching notes:
//! - Attribute names are lower-cased on ingest and compared
//!   case-insensitively (HTML attribute name semantics).
//! - Attribute values default to case-sensitive comparison; the
//!   `i` flag forces ASCII case-insensitive comparison.
//! - Type selectors are ASCII case-insensitive (`Div` and `div`
//!   match the same element).
//! - Sibling combinators (`+`, `~`) consider element siblings only —
//!   raw `Text` children are skipped when walking back through
//!   siblings, matching the CSS spec's "element sibling" notion.

use crate::{Element, Node, Tree};

// ── Public types ────────────────────────────────────────────────────────────

/// A single bracketed attribute filter, e.g. `[type="password" i]`.
#[derive(Debug, Clone)]
struct AttrFilter {
    /// Lower-case attribute name. `Element::attr` lower-cases its
    /// argument too, so we compare apples to apples.
    name: String,
    op: AttrOp,
    /// Empty for `[attr]` (presence). For everything else this is
    /// the comparison value, kept verbatim.
    value: String,
    /// Set by the trailing `i` flag.
    case_insensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttrOp {
    /// `[attr]`
    Exists,
    /// `[attr=v]`
    Equals,
    /// `[attr~=v]` — whitespace-separated token contains `v`.
    Includes,
    /// `[attr|=v]` — equals `v` or starts with `v-`.
    DashMatch,
    /// `[attr^=v]` — starts with `v`.
    Prefix,
    /// `[attr$=v]` — ends with `v`.
    Suffix,
    /// `[attr*=v]` — contains substring `v`.
    Substring,
}

/// One simple compound selector: tag + id + classes + attribute
/// filters (all AND-combined). No combinators, no lists.
///
/// Build with [`CompoundSelector::parse`] (strict — surfaces parse
/// errors) or via [`From<&str>`] / `Into<CompoundSelector>` (lenient
/// — malformed selectors collapse to a "matches nothing" sentinel).
#[derive(Debug, Default, Clone)]
pub struct CompoundSelector {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<AttrFilter>,
    /// `true` reduces this compound to "matches nothing", used by
    /// the lenient string conversions to signal a parse error.
    never_matches: bool,
}

/// Combinator between two compounds in a complex selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// ` ` — `B` has an ancestor matching `A`.
    Descendant,
    /// `>` — `B`'s parent matches `A`.
    Child,
    /// `+` — `B`'s immediately-preceding element sibling matches `A`.
    NextSibling,
    /// `~` — some earlier element sibling of `B` matches `A`.
    SubsequentSibling,
}

/// `A B C` / `A > B`, etc. The *subject* (the element actually
/// returned by `querySelector`) is the rightmost compound.
///
/// Invariant: `combinators.len() == compounds.len() - 1`.
#[derive(Debug, Clone)]
pub struct ComplexSelector {
    compounds: Vec<CompoundSelector>,
    combinators: Vec<Combinator>,
}

impl Default for ComplexSelector {
    fn default() -> Self {
        Self {
            compounds: vec![CompoundSelector::default()],
            combinators: Vec::new(),
        }
    }
}

/// Top-level CSS selector list (`A, B, C`). An element matches the
/// list if it matches **any** member. This is what
/// `Tree::query_selector` and friends accept (via
/// `impl Into<SelectorList>`); `&str`, owned `CompoundSelector`,
/// `&CompoundSelector`, owned `SelectorList`, and `&SelectorList`
/// all convert in.
#[derive(Debug, Clone, Default)]
pub struct SelectorList {
    selectors: Vec<ComplexSelector>,
}

// ── Parsing ─────────────────────────────────────────────────────────────────

impl SelectorList {
    /// Strict parser. Errors include a snippet pointing at the
    /// first unrecognised character. `From<&str>` collapses errors
    /// into an empty list (matches nothing).
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut selectors = Vec::new();
        for part in split_top_level_commas(input) {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                return Err("empty selector in list".into());
            }
            selectors.push(ComplexSelector::parse(trimmed)?);
        }
        if selectors.is_empty() {
            return Err("empty selector".into());
        }
        Ok(SelectorList { selectors })
    }
}

impl ComplexSelector {
    /// Parse a single complex selector — no top-level commas.
    pub fn parse(input: &str) -> Result<Self, String> {
        let s = input.trim();
        if s.is_empty() {
            return Err("empty selector".into());
        }
        let bytes = s.as_bytes();
        let mut i = 0;
        let mut compounds = Vec::new();
        let mut combinators = Vec::new();

        loop {
            // Skip leading whitespace before the next compound.
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                if compounds.is_empty() {
                    return Err("empty selector".into());
                }
                // Trailing combinator like `a > ` is ill-formed.
                return Err("selector ends with a combinator".into());
            }

            let (compound, consumed) = parse_compound(&s[i..])?;
            compounds.push(compound);
            i += consumed;

            if i >= bytes.len() {
                break;
            }

            // Look for a combinator. Track whether at least one
            // whitespace separated this compound from the next
            // token; if so and no explicit combinator follows,
            // it's a descendant combinator.
            let saw_ws = bytes[i].is_ascii_whitespace();
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }

            let comb = match bytes[i] {
                b'>' => {
                    i += 1;
                    Combinator::Child
                }
                b'+' => {
                    i += 1;
                    Combinator::NextSibling
                }
                b'~' => {
                    i += 1;
                    Combinator::SubsequentSibling
                }
                _ if saw_ws => Combinator::Descendant,
                c => return Err(format!("unexpected `{}` after compound selector", c as char)),
            };
            combinators.push(comb);
        }

        Ok(ComplexSelector {
            compounds,
            combinators,
        })
    }
}

/// Walk `s` and return slices split on top-level `,`s — those not
/// inside `[...]` or `(...)`. Quotes inside `[]` swallow `,`s too.
fn split_top_level_commas(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut bracket_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut quote: Option<u8> = None;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
            }
            None => match c {
                b'"' | b'\'' => quote = Some(c),
                b'[' => bracket_depth += 1,
                b']' => bracket_depth -= 1,
                b'(' => paren_depth += 1,
                b')' => paren_depth -= 1,
                b',' if bracket_depth == 0 && paren_depth == 0 => {
                    out.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            },
        }
        i += 1;
    }
    out.push(&s[start..]);
    out
}

/// Parse a compound selector starting at `s[0]`. Stops at the first
/// whitespace or combinator-introducing character (`>`, `+`, `~`,
/// `,`). Returns the parsed compound and the number of bytes
/// consumed.
fn parse_compound(s: &str) -> Result<(CompoundSelector, usize), String> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut out = CompoundSelector::default();

    // Optional leading tag name or `*`.
    if i < bytes.len()
        && bytes[i] != b'#'
        && bytes[i] != b'.'
        && bytes[i] != b'['
        && !is_compound_terminator(bytes[i])
    {
        let start = i;
        if bytes[i] == b'*' {
            i += 1;
        } else if is_ident_start(bytes[i]) {
            while i < bytes.len() && is_ident_char(bytes[i]) {
                i += 1;
            }
        } else {
            return Err(format!("unexpected character `{}`", bytes[i] as char));
        }
        let raw = &s[start..i];
        if raw != "*" {
            out.tag = Some(raw.to_ascii_lowercase());
        }
    }

    // Suffixes: `#id`, `.class`, `[attr…]`. Stops at compound
    // terminators (whitespace, combinator chars, comma, end).
    while i < bytes.len() && !is_compound_terminator(bytes[i]) {
        match bytes[i] {
            b'#' => {
                i += 1;
                let start = i;
                while i < bytes.len() && is_ident_char(bytes[i]) {
                    i += 1;
                }
                if i == start {
                    return Err("expected identifier after `#`".into());
                }
                if out.id.is_some() {
                    return Err("multiple `#id` in selector".into());
                }
                out.id = Some(s[start..i].to_owned());
            }
            b'.' => {
                i += 1;
                let start = i;
                while i < bytes.len() && is_ident_char(bytes[i]) {
                    i += 1;
                }
                if i == start {
                    return Err("expected identifier after `.`".into());
                }
                out.classes.push(s[start..i].to_owned());
            }
            b'[' => {
                let (filter, consumed) = parse_attr_filter(&s[i..])?;
                out.attrs.push(filter);
                i += consumed;
            }
            c => return Err(format!("unsupported selector character `{}`", c as char)),
        }
    }

    if i == 0 {
        return Err("empty compound selector".into());
    }

    Ok((out, i))
}

fn is_compound_terminator(b: u8) -> bool {
    b.is_ascii_whitespace() || b == b'>' || b == b'+' || b == b'~' || b == b','
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'-' || b == b'_'
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// Parse `[attr]`, `[attr=v]`, `[attr~=v]`, `[attr|=v]`, `[attr^=v]`,
/// `[attr$=v]`, `[attr*=v]`, with optional trailing ` i` / ` s`
/// case flag. Returns the filter and the number of bytes consumed
/// (including the closing `]`).
fn parse_attr_filter(s: &str) -> Result<(AttrFilter, usize), String> {
    let bytes = s.as_bytes();
    debug_assert_eq!(bytes[0], b'[');
    let mut i = 1;

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // Attribute name.
    let name_start = i;
    while i < bytes.len() && is_ident_char(bytes[i]) {
        i += 1;
    }
    if i == name_start {
        return Err("expected attribute name after `[`".into());
    }
    let name = s[name_start..i].to_ascii_lowercase();

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // Either close bracket (presence) or operator.
    if i >= bytes.len() {
        return Err("unterminated `[` attribute selector".into());
    }
    if bytes[i] == b']' {
        return Ok((
            AttrFilter {
                name,
                op: AttrOp::Exists,
                value: String::new(),
                case_insensitive: false,
            },
            i + 1,
        ));
    }

    let op = match bytes[i] {
        b'=' => {
            i += 1;
            AttrOp::Equals
        }
        b'~' if matches!(bytes.get(i + 1), Some(b'=')) => {
            i += 2;
            AttrOp::Includes
        }
        b'|' if matches!(bytes.get(i + 1), Some(b'=')) => {
            i += 2;
            AttrOp::DashMatch
        }
        b'^' if matches!(bytes.get(i + 1), Some(b'=')) => {
            i += 2;
            AttrOp::Prefix
        }
        b'$' if matches!(bytes.get(i + 1), Some(b'=')) => {
            i += 2;
            AttrOp::Suffix
        }
        b'*' if matches!(bytes.get(i + 1), Some(b'=')) => {
            i += 2;
            AttrOp::Substring
        }
        c => {
            return Err(format!(
                "unexpected `{}` inside attribute selector — expected an operator or `]`",
                c as char
            ));
        }
    };

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return Err("expected attribute value".into());
    }

    let value = match bytes[i] {
        b'"' | b'\'' => {
            let quote = bytes[i];
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            if i >= bytes.len() {
                return Err("unterminated quoted attribute value".into());
            }
            let v = s[start..i].to_owned();
            i += 1; // skip closing quote
            v
        }
        _ => {
            let start = i;
            while i < bytes.len() && is_ident_char(bytes[i]) {
                i += 1;
            }
            if i == start {
                return Err("expected attribute value or quoted string after operator".into());
            }
            s[start..i].to_owned()
        }
    };

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // Optional `i` / `s` case flag.
    let mut case_insensitive = false;
    if i < bytes.len() && (bytes[i] == b'i' || bytes[i] == b'I') {
        case_insensitive = true;
        i += 1;
    } else if i < bytes.len() && (bytes[i] == b's' || bytes[i] == b'S') {
        // Explicit case-sensitive flag; this is the default but we
        // still consume it so `[a=v s]` parses.
        i += 1;
    }
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    if i >= bytes.len() || bytes[i] != b']' {
        return Err("expected `]` to close attribute selector".into());
    }

    Ok((
        AttrFilter {
            name,
            op,
            value,
            case_insensitive,
        },
        i + 1,
    ))
}

// ── Conversion impls (lenient string parsing, etc.) ─────────────────────────

impl From<&str> for SelectorList {
    fn from(s: &str) -> Self {
        SelectorList::parse(s).unwrap_or_default()
    }
}

impl From<String> for SelectorList {
    fn from(s: String) -> Self {
        SelectorList::from(s.as_str())
    }
}

impl From<&String> for SelectorList {
    fn from(s: &String) -> Self {
        SelectorList::from(s.as_str())
    }
}

impl From<&SelectorList> for SelectorList {
    fn from(s: &SelectorList) -> Self {
        s.clone()
    }
}

impl From<CompoundSelector> for SelectorList {
    fn from(c: CompoundSelector) -> Self {
        SelectorList {
            selectors: vec![ComplexSelector {
                compounds: vec![c],
                combinators: Vec::new(),
            }],
        }
    }
}

impl From<&CompoundSelector> for SelectorList {
    fn from(c: &CompoundSelector) -> Self {
        SelectorList::from(c.clone())
    }
}

impl From<ComplexSelector> for SelectorList {
    fn from(c: ComplexSelector) -> Self {
        SelectorList { selectors: vec![c] }
    }
}

impl From<&ComplexSelector> for SelectorList {
    fn from(c: &ComplexSelector) -> Self {
        SelectorList::from(c.clone())
    }
}

// CompoundSelector still has a strict-parse constructor for callers
// that want to test a single element.
impl CompoundSelector {
    /// Strict parse of a single compound selector — no combinators,
    /// no lists. Errors include a snippet pointing at the bad
    /// character.
    pub fn parse(input: &str) -> Result<Self, String> {
        let s = input.trim();
        if s.is_empty() {
            return Err("empty selector".into());
        }
        let (compound, consumed) = parse_compound(s)?;
        if consumed != s.len() {
            return Err(format!(
                "trailing input after compound selector: `{}`",
                &s[consumed..]
            ));
        }
        Ok(compound)
    }

    /// Test a single [`Element`] against this compound (no
    /// combinators, no list). Cheap (no allocations).
    pub fn matches(&self, el: &Element) -> bool {
        if self.never_matches {
            return false;
        }
        if matches!(el, Element::Text(_)) {
            return false;
        }
        if let Some(tag) = &self.tag
            && !el.tag_name().eq_ignore_ascii_case(tag)
        {
            return false;
        }
        if let Some(id) = &self.id
            && el.id() != Some(id.as_str())
        {
            return false;
        }
        if !self.classes.is_empty() {
            let class_attr = el.class().unwrap_or("");
            for needed in &self.classes {
                if !class_attr.split_ascii_whitespace().any(|c| c == needed) {
                    return false;
                }
            }
        }
        for filter in &self.attrs {
            if !filter.matches_element(el) {
                return false;
            }
        }
        true
    }
}

impl From<&str> for CompoundSelector {
    fn from(s: &str) -> Self {
        Self::parse(s).unwrap_or(CompoundSelector {
            never_matches: true,
            ..CompoundSelector::default()
        })
    }
}

impl From<&CompoundSelector> for CompoundSelector {
    fn from(s: &CompoundSelector) -> Self {
        s.clone()
    }
}

impl From<String> for CompoundSelector {
    fn from(s: String) -> Self {
        CompoundSelector::from(s.as_str())
    }
}

impl From<&String> for CompoundSelector {
    fn from(s: &String) -> Self {
        CompoundSelector::from(s.as_str())
    }
}

// ── Matching ────────────────────────────────────────────────────────────────

impl AttrFilter {
    fn matches_element(&self, el: &Element) -> bool {
        let attr_value = el.attr(&self.name);
        match (self.op, attr_value) {
            (AttrOp::Exists, Some(_)) => true,
            (AttrOp::Exists, None) => false,
            (_, None) => false,
            (AttrOp::Equals, Some(v)) => self.cmp_eq(&v, &self.value),
            (AttrOp::Includes, Some(v)) => v
                .split_ascii_whitespace()
                .any(|tok| self.cmp_eq(tok, &self.value)),
            (AttrOp::DashMatch, Some(v)) => {
                if self.cmp_eq(&v, &self.value) {
                    return true;
                }
                let needle_len = self.value.len();
                if v.len() <= needle_len {
                    return false;
                }
                if !v.is_char_boundary(needle_len) || v.as_bytes()[needle_len] != b'-' {
                    return false;
                }
                self.cmp_eq(&v[..needle_len], &self.value)
            }
            (AttrOp::Prefix, Some(v)) => {
                self.value.is_empty() || self.starts_with_cmp(&v, &self.value)
            }
            (AttrOp::Suffix, Some(v)) => {
                self.value.is_empty() || self.ends_with_cmp(&v, &self.value)
            }
            (AttrOp::Substring, Some(v)) => {
                self.value.is_empty() || self.contains_cmp(&v, &self.value)
            }
        }
    }

    fn cmp_eq(&self, a: &str, b: &str) -> bool {
        if self.case_insensitive {
            a.eq_ignore_ascii_case(b)
        } else {
            a == b
        }
    }
    fn starts_with_cmp(&self, hay: &str, needle: &str) -> bool {
        if self.case_insensitive {
            hay.len() >= needle.len() && hay[..needle.len()].eq_ignore_ascii_case(needle)
        } else {
            hay.starts_with(needle)
        }
    }
    fn ends_with_cmp(&self, hay: &str, needle: &str) -> bool {
        if self.case_insensitive {
            hay.len() >= needle.len()
                && hay[hay.len() - needle.len()..].eq_ignore_ascii_case(needle)
        } else {
            hay.ends_with(needle)
        }
    }
    fn contains_cmp(&self, hay: &str, needle: &str) -> bool {
        if self.case_insensitive {
            ascii_icase_contains(hay, needle)
        } else {
            hay.contains(needle)
        }
    }
}

fn ascii_icase_contains(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let h = hay.as_bytes();
    let n = needle.as_bytes();
    if h.len() < n.len() {
        return false;
    }
    'outer: for i in 0..=h.len() - n.len() {
        for j in 0..n.len() {
            if !h[i + j].eq_ignore_ascii_case(&n[j]) {
                continue 'outer;
            }
        }
        return true;
    }
    false
}

/// Walk `root.children`-by-index from `root` to a descendant.
/// Empty path returns `root`.
fn node_at_path<'a>(root: &'a Node, path: &[usize]) -> Option<&'a Node> {
    let mut cur = root;
    for &i in path {
        cur = cur.children.get(i)?;
    }
    Some(cur)
}

/// Return the index of the closest preceding *element* sibling
/// (skipping `Element::Text` children), or `None` if there is none.
fn previous_element_sibling(parent: &Node, idx: usize) -> Option<usize> {
    (0..idx)
        .rev()
        .find(|&j| !matches!(parent.children[j].element, Element::Text(_)))
}

impl ComplexSelector {
    /// Test whether the candidate at `path` (relative to `root`)
    /// satisfies this complex selector. Walks combinators
    /// right-to-left.
    fn matches(&self, root: &Node, path: &[usize]) -> bool {
        let n = self.compounds.len();
        if n == 0 {
            return false;
        }
        // Subject must match the rightmost compound.
        let Some(subject) = node_at_path(root, path) else {
            return false;
        };
        if !self.compounds[n - 1].matches(&subject.element) {
            return false;
        }
        if n == 1 {
            return true;
        }

        // Walk the combinators right-to-left, narrowing
        // `current_path` toward an ancestor / sibling that satisfies
        // the previous compound.
        let mut current: Vec<usize> = path.to_vec();
        for k in (0..n - 1).rev() {
            let comb = self.combinators[k];
            let prev = &self.compounds[k];
            match comb {
                Combinator::Descendant => {
                    let mut found = false;
                    while !current.is_empty() {
                        current.pop();
                        let Some(node) = node_at_path(root, &current) else {
                            break;
                        };
                        if prev.matches(&node.element) {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return false;
                    }
                }
                Combinator::Child => {
                    if current.is_empty() {
                        return false;
                    }
                    current.pop();
                    let Some(parent) = node_at_path(root, &current) else {
                        return false;
                    };
                    if !prev.matches(&parent.element) {
                        return false;
                    }
                }
                Combinator::NextSibling => {
                    let Some(idx) = current.last().copied() else {
                        return false;
                    };
                    let parent_path = current[..current.len() - 1].to_vec();
                    let Some(parent) = node_at_path(root, &parent_path) else {
                        return false;
                    };
                    let Some(sib) = previous_element_sibling(parent, idx) else {
                        return false;
                    };
                    if !prev.matches(&parent.children[sib].element) {
                        return false;
                    }
                    *current.last_mut().unwrap() = sib;
                }
                Combinator::SubsequentSibling => {
                    let Some(idx) = current.last().copied() else {
                        return false;
                    };
                    let parent_path = current[..current.len() - 1].to_vec();
                    let Some(parent) = node_at_path(root, &parent_path) else {
                        return false;
                    };
                    let mut found = None;
                    let mut j = idx;
                    while let Some(prev_j) = previous_element_sibling(parent, j) {
                        if prev.matches(&parent.children[prev_j].element) {
                            found = Some(prev_j);
                            break;
                        }
                        j = prev_j;
                    }
                    let Some(sib) = found else { return false };
                    *current.last_mut().unwrap() = sib;
                }
            }
        }
        true
    }
}

impl SelectorList {
    fn matches(&self, root: &Node, path: &[usize]) -> bool {
        self.selectors.iter().any(|sel| sel.matches(root, path))
    }
}

// ── Tree traversal helpers (path collection) ────────────────────────────────

fn collect_matching_paths(root: &Node, sel: &SelectorList) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut path = Vec::new();
    walk(root, sel, &mut path, &mut out);
    out
}

fn walk(root: &Node, sel: &SelectorList, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
    if sel.matches(root, path) {
        out.push(path.clone());
    }
    let Some(node) = node_at_path(root, path) else {
        return;
    };
    let n = node.children.len();
    for i in 0..n {
        path.push(i);
        walk(root, sel, path, out);
        path.pop();
    }
}

fn first_match_path(root: &Node, sel: &SelectorList) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    if rec_first(root, sel, &mut path) {
        Some(path)
    } else {
        None
    }
}

fn rec_first(root: &Node, sel: &SelectorList, path: &mut Vec<usize>) -> bool {
    if sel.matches(root, path) {
        return true;
    }
    let Some(node) = node_at_path(root, path) else {
        return false;
    };
    let n = node.children.len();
    for i in 0..n {
        path.push(i);
        if rec_first(root, sel, path) {
            return true;
        }
        path.pop();
    }
    false
}

// ── Public node / tree methods ──────────────────────────────────────────────

impl Node {
    /// Like [`Node::query_selector`] but returns the matched node's
    /// child-index path (relative to `self`) instead of an `&mut`
    /// borrow. Useful when you want an owning, `Send`-friendly
    /// handle (e.g. to pass to `screenshot_node_to`) or want to look
    /// up the same element repeatedly without holding an exclusive
    /// borrow.
    pub fn query_selector_path(&self, sel: impl Into<SelectorList>) -> Option<Vec<usize>> {
        first_match_path(self, &sel.into())
    }

    /// Like [`Node::query_selector_all`] but returns the
    /// child-index paths of every match, in document order.
    pub fn query_selector_all_paths(&self, sel: impl Into<SelectorList>) -> Vec<Vec<usize>> {
        collect_matching_paths(self, &sel.into())
    }

    /// Return the first descendant (or `self`) that matches the CSS
    /// selector list `sel`, in document order. Returns `None` if no
    /// element matches *or* if `sel` is malformed (use
    /// [`SelectorList::parse`] / [`CompoundSelector::parse`] up
    /// front to validate).
    ///
    /// `sel` accepts any `Into<SelectorList>`: `&str`, `String`,
    /// owned or borrowed [`CompoundSelector`] / [`ComplexSelector`]
    /// / [`SelectorList`]. Re-using a parsed selector across many
    /// calls skips the parse step.
    ///
    /// Supported syntax: see the module-level docs.
    pub fn query_selector(&mut self, sel: impl Into<SelectorList>) -> Option<&mut Node> {
        let sel = sel.into();
        let path = first_match_path(self, &sel)?;
        self.at_path_mut(&path)
    }

    /// Return every descendant (and `self`) matching `sel`, in
    /// document order. Empty `Vec` if nothing matches or `sel` is
    /// malformed.
    ///
    /// Soundness: like [`Node::ancestry_at_path_mut`], returned
    /// `&mut` refs may alias when one match is an ancestor of
    /// another. Two of them must never be dereferenced concurrently.
    pub fn query_selector_all(&mut self, sel: impl Into<SelectorList>) -> Vec<&mut Node> {
        let sel = sel.into();
        let paths = collect_matching_paths(self, &sel);
        let mut out: Vec<&mut Node> = Vec::with_capacity(paths.len());
        // SAFETY: every pointer is derived from `self`'s exclusive
        // borrow; concurrent dereference is forbidden by the
        // documented contract above.
        unsafe {
            let root: *mut Node = self as *mut Node;
            for path in &paths {
                let mut cursor: *mut Node = root;
                let mut ok = true;
                for &i in path {
                    let children: *mut Vec<Node> = &raw mut (*cursor).children;
                    if i >= (*children).len() {
                        ok = false;
                        break;
                    }
                    cursor = (*children).as_mut_ptr().add(i);
                }
                if ok {
                    out.push(&mut *cursor);
                }
            }
        }
        out
    }
}

impl Tree {
    /// Path of the first element in the document tree that matches
    /// `sel`. See [`Node::query_selector_path`].
    pub fn query_selector_path(&self, sel: impl Into<SelectorList>) -> Option<Vec<usize>> {
        self.root.as_ref()?.query_selector_path(sel)
    }

    /// Paths of every element in the document tree that matches
    /// `sel`, in document order. See [`Node::query_selector_all_paths`].
    pub fn query_selector_all_paths(&self, sel: impl Into<SelectorList>) -> Vec<Vec<usize>> {
        match self.root.as_ref() {
            Some(root) => root.query_selector_all_paths(sel),
            None => Vec::new(),
        }
    }

    /// Return the first element in the document tree that matches
    /// the CSS selector `sel`. Returns `None` if the tree is empty,
    /// nothing matches, or `sel` is malformed.
    pub fn query_selector(&mut self, sel: impl Into<SelectorList>) -> Option<&mut Node> {
        self.root.as_mut()?.query_selector(sel)
    }

    /// Every element in the document tree matching `sel`, in
    /// document order. See [`Node::query_selector_all`] for the
    /// aliasing contract.
    pub fn query_selector_all(&mut self, sel: impl Into<SelectorList>) -> Vec<&mut Node> {
        match self.root.as_mut() {
            Some(root) => root.query_selector_all(sel),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu_html_models as m;

    fn div(id: Option<&str>, class: Option<&str>) -> m::Div {
        m::Div {
            id: id.map(str::to_owned),
            class: class.map(str::to_owned),
            ..m::Div::default()
        }
    }

    fn span(id: Option<&str>, class: Option<&str>) -> m::Span {
        m::Span {
            id: id.map(str::to_owned),
            class: class.map(str::to_owned),
            ..m::Span::default()
        }
    }

    fn sample() -> Tree {
        // <body>
        //   <div id="outer" class="box hero">
        //     <span class="label">hi</span>
        //     <div class="box"><span class="label">two</span></div>
        //   </div>
        //   <span id="solo" class="label primary"/>
        // </body>
        let body = Node::new(m::Body::default()).with_children(vec![
            Node::new(div(Some("outer"), Some("box hero"))).with_children(vec![
                Node::new(span(None, Some("label"))).with_children(vec![Node::new("hi")]),
                Node::new(div(None, Some("box"))).with_children(vec![
                    Node::new(span(None, Some("label"))).with_children(vec![Node::new("two")]),
                ]),
            ]),
            Node::new(span(Some("solo"), Some("label primary"))),
        ]);
        Tree::new(body)
    }

    // ── compound parsing ────────────────────────────────────────────

    #[test]
    fn parse_compound_keeps_old_grammar() {
        let s = CompoundSelector::parse("div.box#outer.hero").unwrap();
        assert_eq!(s.tag.as_deref(), Some("div"));
        assert_eq!(s.id.as_deref(), Some("outer"));
        assert_eq!(s.classes, vec!["box".to_string(), "hero".to_string()]);

        let s = CompoundSelector::parse("*.label").unwrap();
        assert!(s.tag.is_none());
        assert_eq!(s.classes, vec!["label".to_string()]);

        // Compound parser doesn't accept combinators / lists.
        assert!(CompoundSelector::parse("div span").is_err());
        assert!(CompoundSelector::parse("a, b").is_err());
        assert!(CompoundSelector::parse("a > b").is_err());
    }

    #[test]
    fn parse_attribute_operators() {
        let list = SelectorList::parse("[a][b=v][c~=v][d|=en][e^=p][f$=q][g*=r]").unwrap();
        let cs = &list.selectors[0].compounds[0];
        let ops: Vec<_> = cs.attrs.iter().map(|f| f.op).collect();
        assert_eq!(
            ops,
            vec![
                AttrOp::Exists,
                AttrOp::Equals,
                AttrOp::Includes,
                AttrOp::DashMatch,
                AttrOp::Prefix,
                AttrOp::Suffix,
                AttrOp::Substring,
            ]
        );
    }

    #[test]
    fn parse_attribute_case_flags() {
        let list = SelectorList::parse("[type=PASSWORD i]").unwrap();
        let f = &list.selectors[0].compounds[0].attrs[0];
        assert_eq!(f.value, "PASSWORD");
        assert!(f.case_insensitive);

        // `s` flag is allowed but is the default.
        let list = SelectorList::parse("[type=password s]").unwrap();
        let f = &list.selectors[0].compounds[0].attrs[0];
        assert!(!f.case_insensitive);
    }

    // ── attribute matchers ──────────────────────────────────────────

    #[test]
    fn attribute_op_includes_matches_class_token() {
        let body = Node::new(m::Body::default()).with_children(vec![
            Node::new(div(None, Some("foo bar baz"))),
            Node::new(div(None, Some("foobar"))),
        ]);
        let mut tree = Tree::new(body);
        // `class~="bar"` matches only the first (token list).
        let hits = tree.query_selector_all_paths("[class~=\"bar\"]");
        assert_eq!(hits, vec![vec![0]]);
    }

    #[test]
    fn attribute_op_dashmatch_for_lang() {
        let mut e = m::Div::default();
        e.lang = Some("en-US".to_owned());
        let mut e2 = m::Div::default();
        e2.lang = Some("english".to_owned());
        let body = Node::new(m::Body::default())
            .with_children(vec![Node::new(e), Node::new(e2)]);
        let mut tree = Tree::new(body);
        // `[lang|=en]` matches "en" itself and "en-*", but not "english".
        let hits = tree.query_selector_all_paths("[lang|=en]");
        assert_eq!(hits, vec![vec![0]]);
    }

    #[test]
    fn attribute_op_prefix_suffix_substring() {
        let mut a1 = m::A::default();
        a1.href = Some("https://example.com/path".to_owned());
        let mut a2 = m::A::default();
        a2.href = Some("/local".to_owned());
        let mut a3 = m::A::default();
        a3.href = Some("https://example.com/file.pdf".to_owned());
        let body = Node::new(m::Body::default())
            .with_children(vec![Node::new(a1), Node::new(a2), Node::new(a3)]);
        let mut tree = Tree::new(body);

        // ^= — starts with.
        assert_eq!(
            tree.query_selector_all_paths("a[href^=\"https://\"]"),
            vec![vec![0], vec![2]]
        );
        // $= — ends with.
        assert_eq!(
            tree.query_selector_all_paths("a[href$=\".pdf\"]"),
            vec![vec![2]]
        );
        // *= — substring contains.
        assert_eq!(
            tree.query_selector_all_paths("a[href*=\"example\"]"),
            vec![vec![0], vec![2]]
        );
    }

    #[test]
    fn attribute_case_insensitive_flag() {
        use m::common::html_enums::InputType;
        let body = Node::new(m::Body::default()).with_children(vec![Node::new(m::Input {
            id: Some("pw".to_owned()),
            r#type: Some(InputType::Password),
            ..m::Input::default()
        })]);
        let mut tree = Tree::new(body);
        // Default is case-sensitive: PASSWORD doesn't match.
        assert!(tree.query_selector("input[type=PASSWORD]").is_none());
        // `i` flag forces case-insensitive comparison.
        assert!(tree.query_selector("input[type=PASSWORD i]").is_some());
    }

    // ── combinators ─────────────────────────────────────────────────

    #[test]
    fn descendant_combinator() {
        let mut tree = sample();
        // body div span — every span inside any div inside body.
        // Inner spans (under div#outer and under div.box) match.
        // The solo span (direct child of body, not under div) doesn't.
        let hits = tree.query_selector_all_paths("body div span");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn child_combinator() {
        let mut tree = sample();
        // div > span — span direct child of any div.
        // Outer div's first child is a span: ✓.
        // Inner div's first child is a span: ✓.
        // Solo span is body's child, not div's: ✗.
        let hits = tree.query_selector_all_paths("div > span");
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn next_sibling_combinator() {
        let mut tree = sample();
        // span + div — div whose immediately preceding element
        // sibling is a span. The inner div sits after the inner
        // span, so it matches.
        let hits = tree.query_selector_all_paths("span + div");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn subsequent_sibling_combinator() {
        let mut tree = sample();
        // span ~ div — div with an earlier span sibling. The inner
        // div has a span sibling immediately before it.
        let hits = tree.query_selector_all_paths("span ~ div");
        assert_eq!(hits.len(), 1);
    }

    // ── selector lists ──────────────────────────────────────────────

    #[test]
    fn selector_list_unions_matches() {
        let mut tree = sample();
        // #outer or #solo — two ids at different depths.
        let hits = tree.query_selector_all_paths("#outer, #solo");
        assert_eq!(hits.len(), 2);

        // Mix tag + attr in a list.
        let hits = tree.query_selector_all_paths("div, [class~=primary]");
        // 2 divs + 1 element with class token "primary" (the solo span)
        assert_eq!(hits.len(), 3);
    }

    // ── unchanged contracts ─────────────────────────────────────────

    #[test]
    fn query_selector_by_id() {
        let mut tree = sample();
        let n = tree.query_selector("#outer").unwrap();
        assert_eq!(n.element.tag_name(), "div");
        assert_eq!(n.element.id(), Some("outer"));
        assert!(tree.query_selector("#missing").is_none());
    }

    #[test]
    fn query_selector_by_tag() {
        let mut tree = sample();
        let first = tree.query_selector("div").unwrap();
        assert_eq!(first.element.id(), Some("outer"));
    }

    #[test]
    fn query_selector_by_class_compound() {
        let mut tree = sample();
        let n = tree.query_selector("span.primary").unwrap();
        assert_eq!(n.element.id(), Some("solo"));
        assert!(tree.query_selector("span.box").is_none());
    }

    #[test]
    fn universal_selector_includes_root_self() {
        let mut tree = sample();
        let all = tree.query_selector_all("*");
        // body + outer div + label span + inner div + label span + solo span = 6
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn empty_tree_is_safe() {
        let mut tree = Tree::default();
        assert!(tree.query_selector("div").is_none());
        assert!(tree.query_selector_all("div").is_empty());
        assert!(tree.query_selector_all("a, b, c").is_empty());
    }

    #[test]
    fn invalid_selector_yields_no_match() {
        let mut tree = sample();
        // Pseudo-classes still aren't parsed; they collapse to no match.
        assert!(tree.query_selector(":hover").is_none());
        assert!(tree.query_selector_all(":nth-child(1)").is_empty());
    }

    #[test]
    fn pre_parsed_selector_reuses_across_calls() {
        let sel = SelectorList::parse(".label, #solo").unwrap();
        let mut tree = sample();
        // 3 spans carry the `label` class. `#solo` is one of them,
        // so the union is still 3 — browsers de-duplicate matches in
        // a selector list, and so do we (each element is visited
        // once during the walk).
        assert_eq!(tree.query_selector_all(&sel).len(), 3);
        // Owned — consumes.
        assert_eq!(tree.query_selector_all(sel).len(), 3);
    }

    #[test]
    fn compound_selector_into_list() {
        let cs = CompoundSelector::parse("span.label").unwrap();
        let mut tree = sample();
        // CompoundSelector → SelectorList conversion.
        assert_eq!(tree.query_selector_all(cs).len(), 3);
    }

    #[test]
    fn input_type_password_user_case() {
        use m::common::html_enums::InputType;
        let body = Node::new(m::Body::default()).with_children(vec![
            Node::new(m::Input {
                id: Some("user".to_owned()),
                r#type: Some(InputType::Text),
                ..m::Input::default()
            }),
            Node::new(m::Input {
                id: Some("pass".to_owned()),
                r#type: Some(InputType::Password),
                ..m::Input::default()
            }),
        ]);
        let mut tree = Tree::new(body);
        let hit = tree.query_selector("input[type=\"password\"]").unwrap();
        assert_eq!(hit.element.id(), Some("pass"));
    }
}
