//! CSS stylesheet parsing — selectors + rules.
//!
//! Selector parsing delegates to the full CSS Level 4 parser in
//! `wgpu_html_tree::query`, which supports all four combinators
//! (` `, `>`, `+`, `~`), attribute operators, logical pseudo-classes
//! (`:is`, `:where`, `:not`, `:has`), structural pseudo-classes
//! (`:nth-child`, …), and state pseudo-classes (`:disabled`, …).
//! Pseudo-elements (`::before`, `::after`) are supported with
//! `content` string values.

use std::collections::HashMap;

use wgpu_html_models::{ArcStr, Style};
pub use wgpu_html_tree::query::{
  AttrFilter, AttrOp, Combinator, ComplexSelector, CompoundSelector, MatchContext, PseudoClass, PseudoElement,
  SelectorList,
};

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
  pub selectors: SelectorList,
  pub declarations: Style,
  pub important: Style,
  pub keywords: HashMap<ArcStr, CssWideKeyword>,
  pub important_keywords: HashMap<ArcStr, CssWideKeyword>,
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

    if strip_ascii_word_prefix(&input[cursor..], "@charset").is_some()
      || strip_ascii_word_prefix(&input[cursor..], "@import").is_some()
    {
      if let Some(semi) = input[cursor..].find(';') {
        cursor += semi + 1;
        continue;
      }
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

/// Parse a selector list string via the full query-engine parser.
/// Delegates to `wgpu_html_tree::query::SelectorList::from`.
fn parse_selector_list(s: &str) -> SelectorList {
  SelectorList::from(s)
}

/// Parse a single complex selector. Public for test compatibility.
/// Delegates to the query engine; returns the first complex selector
/// from the list, or a default (empty) on parse failure.
pub fn parse_selector(s: &str) -> ComplexSelector {
  let list = SelectorList::from(s);
  list.selectors.first().cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a `@import` directive body (the text between `@import` and `;`).
/// Returns `(url, optional_media_query)`.
///
/// Accepted forms:
/// - `@import "url";`
/// - `@import 'url';`
/// - `@import url("url");`
/// - `@import url('url');`
/// - `@import url(url);`
/// - Any of the above followed by a media query: `@import "url" screen;`
pub fn parse_import_directive(after_import: &str) -> Option<(&str, Option<&str>)> {
  let s = after_import.trim();
  let (url, rest) = if let Some(inner) = s.strip_prefix("url(") {
    let inner = inner.trim_start();
    if let Some(inner) = inner.strip_prefix('"') {
      let end = inner.find('"')?;
      let rest = inner[end + 1..].trim_start().strip_prefix(')')?.trim();
      (&inner[..end], rest)
    } else if let Some(inner) = inner.strip_prefix('\'') {
      let end = inner.find('\'')?;
      let rest = inner[end + 1..].trim_start().strip_prefix(')')?.trim();
      (&inner[..end], rest)
    } else {
      let end = inner.find(')')?;
      (inner[..end].trim(), inner[end + 1..].trim())
    }
  } else if let Some(inner) = s.strip_prefix('"') {
    let end = inner.find('"')?;
    (&inner[..end], inner[end + 1..].trim())
  } else if let Some(inner) = s.strip_prefix('\'') {
    let end = inner.find('\'')?;
    (&inner[..end], inner[end + 1..].trim())
  } else {
    return None;
  };

  let media = if rest.is_empty() { None } else { Some(rest) };
  Some((url, media))
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
