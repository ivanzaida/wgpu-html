//! CSS stylesheet parsing — selectors + rules.
//!
//! Scope: comma-separated selector lists, each entry being a chain of
//! compound selectors joined by the descendant combinator (whitespace).
//! A compound selector supports an optional tag (or universal `*`),
//! optional id, any number of classes, simple attribute selectors, plus
//! an optional set of dynamic pseudo-classes (`:hover`, `:active`,
//! `:focus`). Other combinators (`>`, `+`, `~`) and
//! unsupported pseudo-classes / pseudo-elements still drop the rule.

use std::collections::HashMap;

use wgpu_html_models::Style;

use crate::css_parser::{CssWideKeyword, parse_inline_style_decls};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType {
  All,
  Screen,
  Print,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaFeature {
  Width(f32),
  MinWidth(f32),
  MaxWidth(f32),
  Height(f32),
  MinHeight(f32),
  MaxHeight(f32),
  OrientationPortrait,
  OrientationLandscape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaQuery {
  pub not: bool,
  pub media_type: MediaType,
  pub features: Vec<MediaFeature>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaQueryList {
  /// Comma-separated media queries. The list matches when any query
  /// matches; each query's media type/features are ANDed.
  pub queries: Vec<MediaQuery>,
}

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
  /// `:focus` — accepted for UA/author styles. The runtime does
  /// not track keyboard focus yet, so it only matches if a caller
  /// explicitly supplies a focused match context.
  Focus,
  /// `:visited` — accepted so browser UA link defaults parse.
  /// The engine has no navigation history, so it only matches if
  /// style matching grows visited-link state later.
  Visited,
  /// `:root` — matches only the document root element.
  Root,
  /// `:first-child` — matches if this element is the first child of its parent.
  FirstChild,
  /// `:last-child` — matches if this element is the last child of its parent.
  LastChild,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSelector {
  pub name: String,
  pub value: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Selector {
  /// `Some("div")` for `div`, `None` for universal `*` or no tag part.
  pub tag: Option<String>,
  pub id: Option<String>,
  pub classes: Vec<String>,
  /// Simple attribute selectors: `[hidden]`, `[dir="rtl"]`,
  /// `input[type=submit]`. Operators other than exact equality are
  /// intentionally unsupported for now.
  pub attributes: Vec<AttributeSelector>,
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
    let cls = (self.classes.len() + self.attributes.len() + self.pseudo_classes.len()) as u32;
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
      || !self.attributes.is_empty()
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
  /// Active media conditions enclosing this rule. Multiple entries
  /// come from nested `@media` blocks and are ANDed by the cascade.
  pub media: Vec<MediaQueryList>,
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
  let input = strip_comments(css);
  parse_rules(&input, &mut Vec::new(), &mut rules);
  Stylesheet { rules }
}

pub fn parse_media_query_list(input: &str) -> Option<MediaQueryList> {
  let mut queries = Vec::new();
  for raw in split_top_level(input, ',') {
    let raw = raw.trim();
    if raw.is_empty() {
      continue;
    }
    queries.push(parse_media_query(raw)?);
  }
  (!queries.is_empty()).then_some(MediaQueryList { queries })
}

fn parse_media_query(input: &str) -> Option<MediaQuery> {
  let mut not = false;
  let mut media_type = MediaType::All;
  let mut saw_type = false;
  let mut features = Vec::new();
  let mut parts = split_media_and(input);

  if let Some(first) = parts.first_mut() {
    if let Some(rest) = strip_ascii_word_prefix(first, "not") {
      not = true;
      *first = rest.trim();
    } else if let Some(rest) = strip_ascii_word_prefix(first, "only") {
      *first = rest.trim();
    }
  }

  for part in parts {
    let part = part.trim();
    if part.is_empty() {
      continue;
    }
    if part.starts_with('(') && part.ends_with(')') {
      features.push(parse_media_feature(&part[1..part.len() - 1])?);
      continue;
    }
    if saw_type {
      return None;
    }
    media_type = parse_media_type(part)?;
    saw_type = true;
  }

  Some(MediaQuery {
    not,
    media_type,
    features,
  })
}

fn parse_media_type(input: &str) -> Option<MediaType> {
  if input.eq_ignore_ascii_case("all") {
    Some(MediaType::All)
  } else if input.eq_ignore_ascii_case("screen") {
    Some(MediaType::Screen)
  } else if input.eq_ignore_ascii_case("print") {
    Some(MediaType::Print)
  } else {
    None
  }
}

fn parse_media_feature(input: &str) -> Option<MediaFeature> {
  let (name, value) = input.split_once(':')?;
  let name = name.trim().to_ascii_lowercase();
  let value = value.trim();
  match name.as_str() {
    "width" => Some(MediaFeature::Width(parse_media_length_px(value)?)),
    "min-width" => Some(MediaFeature::MinWidth(parse_media_length_px(value)?)),
    "max-width" => Some(MediaFeature::MaxWidth(parse_media_length_px(value)?)),
    "height" => Some(MediaFeature::Height(parse_media_length_px(value)?)),
    "min-height" => Some(MediaFeature::MinHeight(parse_media_length_px(value)?)),
    "max-height" => Some(MediaFeature::MaxHeight(parse_media_length_px(value)?)),
    "orientation" if value.eq_ignore_ascii_case("portrait") => Some(MediaFeature::OrientationPortrait),
    "orientation" if value.eq_ignore_ascii_case("landscape") => Some(MediaFeature::OrientationLandscape),
    _ => None,
  }
}

fn parse_media_length_px(input: &str) -> Option<f32> {
  let trimmed = input.trim();
  if trimmed == "0" {
    return Some(0.0);
  }
  let value = trimmed.strip_suffix("px")?.trim().parse::<f32>().ok()?;
  value.is_finite().then_some(value)
}

fn parse_rules(input: &str, media_stack: &mut Vec<MediaQueryList>, rules: &mut Vec<Rule>) {
  let mut cursor = 0usize;
  while cursor < input.len() {
    cursor = skip_whitespace(input, cursor);
    if cursor >= input.len() {
      break;
    }
    let Some(open_rel) = input[cursor..].find('{') else {
      break;
    };
    let open = cursor + open_rel;
    let header = input[cursor..open].trim();
    let Some(close) = find_matching_brace(input, open) else {
      break;
    };
    let body = &input[open + 1..close];

    if let Some(query) = strip_ascii_word_prefix(header, "@media").and_then(|h| {
      let h = h.trim();
      (!h.is_empty()).then_some(h).and_then(parse_media_query_list)
    }) {
      media_stack.push(query);
      parse_rules(body, media_stack, rules);
      media_stack.pop();
    } else if !header.starts_with('@') {
      let selectors = parse_selector_list(header);
      let decls = parse_inline_style_decls(body);
      if !selectors.is_empty() {
        rules.push(Rule {
          selectors,
          declarations: decls.normal,
          important: decls.important,
          keywords: decls.keywords_normal,
          important_keywords: decls.keywords_important,
          media: media_stack.clone(),
        });
      }
    }

    cursor = close + 1;
  }
}

fn skip_whitespace(input: &str, mut cursor: usize) -> usize {
  while cursor < input.len() && input.as_bytes()[cursor].is_ascii_whitespace() {
    cursor += 1;
  }
  cursor
}

fn find_matching_brace(input: &str, open: usize) -> Option<usize> {
  let bytes = input.as_bytes();
  let mut depth = 0usize;
  let mut quote: Option<u8> = None;
  let mut escaped = false;
  for (i, &b) in bytes.iter().enumerate().skip(open) {
    if let Some(q) = quote {
      if escaped {
        escaped = false;
      } else if b == b'\\' {
        escaped = true;
      } else if b == q {
        quote = None;
      }
      continue;
    }
    match b {
      b'\'' | b'"' => quote = Some(b),
      b'{' => depth += 1,
      b'}' => {
        depth = depth.saturating_sub(1);
        if depth == 0 {
          return Some(i);
        }
      }
      _ => {}
    }
  }
  None
}

fn split_top_level(input: &str, delimiter: char) -> Vec<&str> {
  let mut out = Vec::new();
  let mut depth = 0usize;
  let mut start = 0usize;
  for (i, ch) in input.char_indices() {
    match ch {
      '(' => depth += 1,
      ')' => depth = depth.saturating_sub(1),
      c if c == delimiter && depth == 0 => {
        out.push(&input[start..i]);
        start = i + ch.len_utf8();
      }
      _ => {}
    }
  }
  out.push(&input[start..]);
  out
}

fn split_media_and(input: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let mut depth = 0usize;
  let mut start = 0usize;
  let bytes = input.as_bytes();
  let mut i = 0usize;
  while i < bytes.len() {
    match bytes[i] {
      b'(' => {
        depth += 1;
        i += 1;
      }
      b')' => {
        depth = depth.saturating_sub(1);
        i += 1;
      }
      _ if depth == 0 && starts_with_ascii_word_ci(&input[i..], "and") => {
        out.push(input[start..i].trim());
        i += 3;
        start = i;
      }
      _ => i += 1,
    }
  }
  out.push(input[start..].trim());
  out
}

fn starts_with_ascii_word_ci(input: &str, word: &str) -> bool {
  if input.len() < word.len() || !input[..word.len()].eq_ignore_ascii_case(word) {
    return false;
  }
  let before_ok = true;
  let after_ok = input
    .as_bytes()
    .get(word.len())
    .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'-' && *b != b'_');
  before_ok && after_ok
}

fn strip_ascii_word_prefix<'a>(input: &'a str, word: &str) -> Option<&'a str> {
  let input = input.trim_start();
  if !starts_with_ascii_word_ci(input, word) {
    return None;
  }
  Some(&input[word.len()..])
}

fn parse_selector_list(s: &str) -> Vec<Selector> {
  s.split(',').filter_map(|part| parse_selector(part.trim())).collect()
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
/// (tag/id/classes/attrs/universal/pseudo-classes).
/// Returns `None` if the compound contains anything we don't handle.
fn parse_compound(s: &str) -> Option<Selector> {
  if s.is_empty() {
    return None;
  }
  let mut sel = Selector::default();
  let s = extract_attribute_selectors(s, &mut sel)?;
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
        "focus" => {
          sel.pseudo_classes.push(PseudoClass::Focus);
          buf.clear();
        }
        "visited" => {
          sel.pseudo_classes.push(PseudoClass::Visited);
          buf.clear();
        }
        "root" => {
          sel.pseudo_classes.push(PseudoClass::Root);
          buf.clear();
        }
        "first-child" => {
          sel.pseudo_classes.push(PseudoClass::FirstChild);
          buf.clear();
        }
        "last-child" => {
          sel.pseudo_classes.push(PseudoClass::LastChild);
          buf.clear();
        }
        // Anything we don't recognize (`::before`,
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
      // combinators were already split off as whitespace):
      // drop the rule.
      _ => return None,
    }
  }
  commit(&mut buf, kind, &mut sel)?;

  if !sel.is_meaningful() {
    return None;
  }
  Some(sel)
}

fn extract_attribute_selectors(s: &str, sel: &mut Selector) -> Option<String> {
  let mut out = String::with_capacity(s.len());
  let mut rest = s;
  loop {
    let Some(open) = rest.find('[') else {
      out.push_str(rest);
      break;
    };
    out.push_str(&rest[..open]);
    let after_open = &rest[open + 1..];
    let close = after_open.find(']')?;
    parse_attribute_selector(&after_open[..close], sel)?;
    rest = &after_open[close + 1..];
  }
  Some(out)
}

fn parse_attribute_selector(raw: &str, sel: &mut Selector) -> Option<()> {
  let raw = raw.trim();
  if raw.is_empty() {
    return None;
  }
  let (name, value) = if let Some((name, value)) = raw.split_once('=') {
    let name = normalize_attr_name(name)?;
    let value = strip_attr_quotes(value.trim())?;
    (name, Some(value.to_ascii_lowercase()))
  } else {
    (normalize_attr_name(raw)?, None)
  };
  sel.attributes.push(AttributeSelector { name, value });
  Some(())
}

fn normalize_attr_name(name: &str) -> Option<String> {
  let name = name.trim();
  if name.is_empty()
    || !name
      .chars()
      .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':')
  {
    return None;
  }
  Some(name.to_ascii_lowercase())
}

fn strip_attr_quotes(value: &str) -> Option<String> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return None;
  }
  let bytes = trimmed.as_bytes();
  if bytes.len() >= 2
    && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"') || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
  {
    Some(trimmed[1..trimmed.len() - 1].to_string())
  } else if bytes
    .iter()
    .all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'-' | b'_' | b':' | b'.'))
  {
    Some(trimmed.to_string())
  } else {
    None
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
    // We accept dynamic state pseudo-classes only; structural
    // pseudo-classes and pseudo-elements still drop.
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
  fn parses_focus_and_visited_pseudo_classes() {
    let focus = parse_selector(":focus").unwrap();
    assert_eq!(focus.pseudo_classes, vec![PseudoClass::Focus]);
    let visited = parse_selector("a:visited").unwrap();
    assert_eq!(visited.tag.as_deref(), Some("a"));
    assert_eq!(visited.pseudo_classes, vec![PseudoClass::Visited]);
  }

  #[test]
  fn parses_attribute_presence_selector() {
    let s = parse_selector("abbr[title]").unwrap();
    assert_eq!(s.tag.as_deref(), Some("abbr"));
    assert_eq!(s.attributes.len(), 1);
    assert_eq!(s.attributes[0].name, "title");
    assert_eq!(s.attributes[0].value, None);
  }

  #[test]
  fn parses_attribute_equality_selector() {
    let s = parse_selector(r#"input[type="submit"]"#).unwrap();
    assert_eq!(s.tag.as_deref(), Some("input"));
    assert_eq!(s.attributes.len(), 1);
    assert_eq!(s.attributes[0].name, "type");
    assert_eq!(s.attributes[0].value.as_deref(), Some("submit"));
  }

  #[test]
  fn attribute_selector_adds_class_specificity() {
    let plain = parse_selector("input").unwrap().specificity();
    let attr = parse_selector("input[type=submit]").unwrap().specificity();
    assert!(attr > plain);
  }

  #[test]
  fn parses_pseudo_class_after_id_and_class() {
    let s = parse_selector("button#go.primary:hover:active").unwrap();
    assert_eq!(s.tag.as_deref(), Some("button"));
    assert_eq!(s.id.as_deref(), Some("go"));
    assert_eq!(s.classes, vec!["primary"]);
    assert_eq!(s.pseudo_classes, vec![PseudoClass::Hover, PseudoClass::Active]);
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
  fn parses_media_block_rules() {
    let sheet = parse_stylesheet(
      r#"
            .base { width: 10px; }
            @media screen and (max-width: 600px) {
                .base { width: 20px; }
            }
            "#,
    );
    assert_eq!(sheet.rules.len(), 2);
    assert!(sheet.rules[0].media.is_empty());
    assert_eq!(sheet.rules[1].media.len(), 1);
    assert_eq!(sheet.rules[1].media[0].queries.len(), 1);
    assert_eq!(
      sheet.rules[1].media[0].queries[0].features,
      vec![MediaFeature::MaxWidth(600.0)]
    );
  }

  #[test]
  fn nested_media_blocks_are_anded_on_rules() {
    let sheet = parse_stylesheet(
      r#"
            @media screen {
                @media (orientation: landscape) {
                    .wide { height: 10px; }
                }
            }
            "#,
    );
    assert_eq!(sheet.rules.len(), 1);
    assert_eq!(sheet.rules[0].media.len(), 2);
  }

  #[test]
  fn parses_not_media_modifier() {
    let list = parse_media_query_list("not print and (min-width: 300px)").unwrap();
    assert_eq!(list.queries.len(), 1);
    assert!(list.queries[0].not);
    assert_eq!(list.queries[0].media_type, MediaType::Print);
    assert_eq!(list.queries[0].features, vec![MediaFeature::MinWidth(300.0)]);
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
