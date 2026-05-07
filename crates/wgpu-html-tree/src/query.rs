//! DOM-style query helpers: `querySelector` / `querySelectorAll`.
//!
//! Full CSS selector support including:
//! - Universal `*`, type `E`, id `#id`, class `.class`
//! - Attribute selectors with all six operators and case flags
//! - Compound selectors, selector lists, all four combinators
//! - Pseudo-classes: logical (`:is`, `:where`, `:not`, `:has`), structural (`:first-child`, `:last-child`,
//!   `:only-child`, `:empty`, `:root`, `:scope`, `:nth-child`, `:nth-last-child`, `:first-of-type`, `:last-of-type`,
//!   `:nth-of-type`), state (`:disabled`, `:enabled`, `:checked`, `:required`, `:optional`, `:read-only`,
//!   `:read-write`, `:placeholder-shown`), interaction (`:hover`, `:focus`, `:active`, `:focus-within`), and `:lang()`,
//!   `:dir()`
//! - Pseudo-elements (`::before`, `::after`, `::first-line`, `::first-letter`) — parser accepts, matcher returns no
//!   hits
//! - Namespace prefixes (`svg|circle`, `*|*`, `|tag`)
//! - CSS escape sequences (`\XX` hex, `\.` literal)

use crate::{Element, InteractionState, Node, Tree};

// ── Public types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AttrFilter {
  pub name: String,
  pub op: AttrOp,
  pub value: String,
  pub case_insensitive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrOp {
  Exists,
  Equals,
  Includes,
  DashMatch,
  Prefix,
  Suffix,
  Substring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NthFormula {
  pub a: i32,
  pub b: i32,
}

impl NthFormula {
  fn matches(&self, index_1based: i32) -> bool {
    if self.a == 0 {
      return index_1based == self.b;
    }
    let diff = index_1based - self.b;
    if self.a > 0 {
      diff >= 0 && diff % self.a == 0
    } else {
      diff <= 0 && diff % self.a == 0
    }
  }
}

#[derive(Debug, Clone)]
pub enum PseudoClass {
  Not(SelectorList),
  Is(SelectorList),
  Where(SelectorList),
  Has(Vec<HasSelector>),
  FirstChild,
  LastChild,
  OnlyChild,
  Empty,
  Root,
  Scope,
  NthChild(NthFormula, Option<SelectorList>),
  NthLastChild(NthFormula),
  FirstOfType,
  LastOfType,
  NthOfType(NthFormula),
  Disabled,
  Enabled,
  Checked,
  Required,
  Optional,
  ReadOnly,
  ReadWrite,
  PlaceholderShown,
  Hover,
  Focus,
  Active,
  FocusWithin,
  Lang(String),
  Dir(String),
}

#[derive(Debug, Clone)]
pub struct HasSelector {
  pub leading_combinator: Combinator,
  pub compounds: Vec<CompoundSelector>,
  pub combinators: Vec<Combinator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PseudoElement {
  Before,
  After,
  FirstLine,
  FirstLetter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Namespace {
  Named(String),
  Any,
  Default,
}

#[derive(Debug, Default, Clone)]
pub struct CompoundSelector {
  pub namespace: Option<Namespace>,
  pub tag: Option<String>,
  pub id: Option<String>,
  pub classes: Vec<String>,
  pub attrs: Vec<AttrFilter>,
  pub pseudo_classes: Vec<PseudoClass>,
  pub pseudo_element: Option<PseudoElement>,
  pub never_matches: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
  Descendant,
  Child,
  NextSibling,
  SubsequentSibling,
}

#[derive(Debug, Clone)]
pub struct ComplexSelector {
  pub compounds: Vec<CompoundSelector>,
  pub combinators: Vec<Combinator>,
}

impl Default for ComplexSelector {
  fn default() -> Self {
    Self {
      compounds: vec![CompoundSelector::default()],
      combinators: Vec::new(),
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct SelectorList {
  pub selectors: Vec<ComplexSelector>,
}

/// Context threaded through the matcher for interaction-state access.
pub struct MatchContext<'a> {
  pub interaction: Option<&'a InteractionState>,
}

// ── CSS escape helpers ──────────────────────────────────────────────────────

/// Consume a CSS escape sequence starting at `bytes[pos]` (which must be `\`).
/// Returns the decoded character(s) appended to `out` and the new position.
fn consume_css_escape(s: &str, pos: usize, out: &mut String) -> Result<usize, String> {
  let bytes = s.as_bytes();
  debug_assert_eq!(bytes[pos], b'\\');
  let mut i = pos + 1;
  if i >= bytes.len() {
    return Err("trailing backslash".into());
  }
  // Hex escape: 1-6 hex digits + optional trailing whitespace.
  if bytes[i].is_ascii_hexdigit() {
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_hexdigit() && (i - start) < 6 {
      i += 1;
    }
    // Consume one optional trailing whitespace.
    if i < bytes.len() && bytes[i].is_ascii_whitespace() {
      i += 1;
    }
    // Take only the hex digits (before any whitespace).
    let digit_end = s[start..i]
      .find(|c: char| !c.is_ascii_hexdigit())
      .map_or(i, |p| start + p);
    let hex_str = &s[start..digit_end];
    let cp = u32::from_str_radix(hex_str, 16).map_err(|_| format!("bad hex escape `{}`", hex_str))?;
    if let Some(ch) = char::from_u32(cp) {
      out.push(ch);
    } else {
      out.push(char::REPLACEMENT_CHARACTER);
    }
    // Consume one trailing whitespace after hex digits.
    let mut j = digit_end;
    if j < bytes.len() && bytes[j].is_ascii_whitespace() {
      j += 1;
    }
    return Ok(j);
  }
  // Literal escape: any non-newline character.
  if bytes[i] == b'\n' || bytes[i] == b'\r' {
    return Err("newline in escape".into());
  }
  // Handle multi-byte UTF-8 correctly.
  let ch = s[i..].chars().next().unwrap();
  out.push(ch);
  Ok(i + ch.len_utf8())
}

/// Consume a CSS identifier (with escape support) starting at `s[pos]`.
/// Returns (decoded_ident, new_pos).
fn consume_css_ident(s: &str, pos: usize) -> Result<(String, usize), String> {
  let bytes = s.as_bytes();
  let mut i = pos;
  let mut out = String::new();

  // Optional leading `--` (custom ident) or single `-`.
  if i < bytes.len() && bytes[i] == b'-' {
    out.push('-');
    i += 1;
    if i < bytes.len() && bytes[i] == b'-' {
      out.push('-');
      i += 1;
      // After `--`, continue to ident chars.
    }
  }

  // First real ident char (must be name-start or escape).
  if i < bytes.len() && bytes[i] == b'\\' {
    i = consume_css_escape(s, i, &mut out)?;
  } else if i < bytes.len() && is_ident_start(bytes[i]) {
    out.push(bytes[i] as char);
    i += 1;
  } else if out.is_empty() {
    return Err("expected identifier".into());
  }

  // Remaining ident chars.
  while i < bytes.len() {
    if bytes[i] == b'\\' {
      i = consume_css_escape(s, i, &mut out)?;
    } else if is_ident_char(bytes[i]) {
      out.push(bytes[i] as char);
      i += 1;
    } else {
      break;
    }
  }

  if out.is_empty() {
    return Err("expected identifier".into());
  }
  Ok((out, i))
}

fn is_ident_start(b: u8) -> bool {
  b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_char(b: u8) -> bool {
  b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn is_compound_terminator(b: u8) -> bool {
  b.is_ascii_whitespace() || b == b'>' || b == b'+' || b == b'~' || b == b','
}

// ── Top-level parsing ───────────────────────────────────────────────────────

impl SelectorList {
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
      while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
      }
      if i >= bytes.len() {
        if compounds.is_empty() {
          return Err("empty selector".into());
        }
        return Err("selector ends with a combinator".into());
      }

      let (compound, consumed) = parse_compound(&s[i..])?;
      compounds.push(compound);
      i += consumed;

      if i >= bytes.len() {
        break;
      }

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
        c => {
          return Err(format!("unexpected `{}` after compound selector", c as char));
        }
      };
      combinators.push(comb);
    }

    Ok(ComplexSelector { compounds, combinators })
  }
}

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

// ── Compound parsing ────────────────────────────────────────────────────────

fn parse_compound(s: &str) -> Result<(CompoundSelector, usize), String> {
  let bytes = s.as_bytes();
  let mut i = 0;
  let mut out = CompoundSelector::default();

  // Check for namespace prefix (ns|tag, *|tag, |tag).
  // We need to look ahead for `|` that is NOT part of `|=`.
  if let Some(ns_end) = detect_namespace_prefix(s) {
    let ns_part = &s[..ns_end];
    i = ns_end + 1; // skip the `|`
    if ns_part == "*" {
      out.namespace = Some(Namespace::Any);
    } else if ns_part.is_empty() {
      out.namespace = Some(Namespace::Default);
    } else {
      out.namespace = Some(Namespace::Named(ns_part.to_ascii_lowercase()));
    }
  }

  // Optional tag name or `*`.
  if i < bytes.len()
    && bytes[i] != b'#'
    && bytes[i] != b'.'
    && bytes[i] != b'['
    && bytes[i] != b':'
    && !is_compound_terminator(bytes[i])
  {
    if bytes[i] == b'*' {
      i += 1;
      // `*` means no tag filter.
    } else if bytes[i] == b'\\' || is_ident_start(bytes[i]) || bytes[i] == b'-' {
      let (ident, new_i) = consume_css_ident(&s, i)?;
      out.tag = Some(ident.to_ascii_lowercase());
      i = new_i;
    } else {
      return Err(format!("unexpected character `{}`", bytes[i] as char));
    }
  }

  // Suffixes: #id, .class, [attr…], :pseudo-class, ::pseudo-element.
  while i < bytes.len() && !is_compound_terminator(bytes[i]) {
    match bytes[i] {
      b'#' => {
        i += 1;
        let (ident, new_i) = consume_css_ident(s, i)?;
        if out.id.is_some() {
          return Err("multiple `#id` in selector".into());
        }
        out.id = Some(ident);
        i = new_i;
      }
      b'.' => {
        i += 1;
        let (ident, new_i) = consume_css_ident(s, i)?;
        out.classes.push(ident);
        i = new_i;
      }
      b'[' => {
        let (filter, consumed) = parse_attr_filter(&s[i..])?;
        out.attrs.push(filter);
        i += consumed;
      }
      b':' => {
        if i + 1 < bytes.len() && bytes[i + 1] == b':' {
          // Pseudo-element (::name).
          i += 2;
          let (name, new_i) = consume_css_ident(s, i)?;
          let pe = parse_pseudo_element_name(&name)?;
          out.pseudo_element = Some(pe);
          i = new_i;
        } else {
          // Pseudo-class or legacy pseudo-element.
          i += 1;
          let (pc, new_i) = parse_pseudo_class(s, i)?;
          match pc {
            PseudoOrElement::Pseudo(p) => out.pseudo_classes.push(p),
            PseudoOrElement::Element(pe) => out.pseudo_element = Some(pe),
          }
          i = new_i;
        }
      }
      c => return Err(format!("unsupported selector character `{}`", c as char)),
    }
  }

  if i == 0 {
    return Err("empty compound selector".into());
  }

  Ok((out, i))
}

/// Detect a namespace prefix. Returns the byte offset of `|` if one is
/// found that is NOT part of `|=` and appears before any other selector
/// syntax. Returns `None` if no namespace prefix.
fn detect_namespace_prefix(s: &str) -> Option<usize> {
  let bytes = s.as_bytes();
  // Look for pattern: (ident | '*' | empty) '|' (not '=')
  let mut i = 0;
  // Skip optional ident or '*'.
  if i < bytes.len() && bytes[i] == b'*' {
    i += 1;
  } else {
    while i < bytes.len() && is_ident_char(bytes[i]) {
      i += 1;
    }
  }
  if i < bytes.len() && bytes[i] == b'|' {
    // Make sure the next char is not `=` (that would be `|=` attr op).
    if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
      return None;
    }
    return Some(i);
  }
  None
}

fn parse_pseudo_element_name(name: &str) -> Result<PseudoElement, String> {
  match name.to_ascii_lowercase().as_str() {
    "before" => Ok(PseudoElement::Before),
    "after" => Ok(PseudoElement::After),
    "first-line" => Ok(PseudoElement::FirstLine),
    "first-letter" => Ok(PseudoElement::FirstLetter),
    _ => Err(format!("unknown pseudo-element `::{}`", name)),
  }
}

enum PseudoOrElement {
  Pseudo(PseudoClass),
  Element(PseudoElement),
}

/// Parse a pseudo-class (or legacy pseudo-element) starting after the
/// initial `:`. Returns the parsed item and new byte position.
fn parse_pseudo_class(s: &str, pos: usize) -> Result<(PseudoOrElement, usize), String> {
  let (name, mut i) = consume_css_ident(s, pos)?;
  let lower = name.to_ascii_lowercase();

  // Legacy single-colon pseudo-elements.
  match lower.as_str() {
    "before" => return Ok((PseudoOrElement::Element(PseudoElement::Before), i)),
    "after" => return Ok((PseudoOrElement::Element(PseudoElement::After), i)),
    "first-line" => return Ok((PseudoOrElement::Element(PseudoElement::FirstLine), i)),
    "first-letter" => return Ok((PseudoOrElement::Element(PseudoElement::FirstLetter), i)),
    _ => {}
  }

  // Simple (no-argument) pseudo-classes.
  let simple = match lower.as_str() {
    "first-child" => Some(PseudoClass::FirstChild),
    "last-child" => Some(PseudoClass::LastChild),
    "only-child" => Some(PseudoClass::OnlyChild),
    "empty" => Some(PseudoClass::Empty),
    "root" => Some(PseudoClass::Root),
    "scope" => Some(PseudoClass::Scope),
    "first-of-type" => Some(PseudoClass::FirstOfType),
    "last-of-type" => Some(PseudoClass::LastOfType),
    "disabled" => Some(PseudoClass::Disabled),
    "enabled" => Some(PseudoClass::Enabled),
    "checked" => Some(PseudoClass::Checked),
    "required" => Some(PseudoClass::Required),
    "optional" => Some(PseudoClass::Optional),
    "read-only" => Some(PseudoClass::ReadOnly),
    "read-write" => Some(PseudoClass::ReadWrite),
    "placeholder-shown" => Some(PseudoClass::PlaceholderShown),
    "hover" => Some(PseudoClass::Hover),
    "focus" => Some(PseudoClass::Focus),
    "active" => Some(PseudoClass::Active),
    "focus-within" => Some(PseudoClass::FocusWithin),
    _ => None,
  };
  if let Some(pc) = simple {
    return Ok((PseudoOrElement::Pseudo(pc), i));
  }

  // Functional pseudo-classes (require parentheses).
  let bytes = s.as_bytes();
  if i >= bytes.len() || bytes[i] != b'(' {
    return Err(format!("unknown pseudo-class `:{}`", name));
  }
  i += 1; // skip '('

  let pc = match lower.as_str() {
    "not" | "is" | "where" => {
      let (inner, new_i) = consume_balanced_parens(s, i)?;
      i = new_i;
      let inner = inner.trim();
      if inner.is_empty() {
        return Err(format!(":{}() with empty argument", lower));
      }
      let sel = SelectorList::parse(inner)?;
      match lower.as_str() {
        "not" => PseudoClass::Not(sel),
        "is" => PseudoClass::Is(sel),
        "where" => PseudoClass::Where(sel),
        _ => unreachable!(),
      }
    }
    "has" => {
      let (inner, new_i) = consume_balanced_parens(s, i)?;
      i = new_i;
      let inner = inner.trim();
      if inner.is_empty() {
        return Err(":has() with empty argument".into());
      }
      let selectors = parse_has_argument(inner)?;
      PseudoClass::Has(selectors)
    }
    "nth-child" => {
      let (inner, new_i) = consume_balanced_parens(s, i)?;
      i = new_i;
      let inner = inner.trim();
      if inner.is_empty() {
        return Err(":nth-child() with empty argument".into());
      }
      let (formula, of_sel) = parse_nth_with_of(inner)?;
      PseudoClass::NthChild(formula, of_sel)
    }
    "nth-last-child" => {
      let (inner, new_i) = consume_balanced_parens(s, i)?;
      i = new_i;
      let inner = inner.trim();
      if inner.is_empty() {
        return Err(":nth-last-child() with empty argument".into());
      }
      let formula = parse_nth_formula(inner)?;
      PseudoClass::NthLastChild(formula)
    }
    "nth-of-type" => {
      let (inner, new_i) = consume_balanced_parens(s, i)?;
      i = new_i;
      let inner = inner.trim();
      if inner.is_empty() {
        return Err(":nth-of-type() with empty argument".into());
      }
      let formula = parse_nth_formula(inner)?;
      PseudoClass::NthOfType(formula)
    }
    "lang" => {
      let (inner, new_i) = consume_balanced_parens(s, i)?;
      i = new_i;
      let inner = inner.trim();
      PseudoClass::Lang(inner.to_owned())
    }
    "dir" => {
      let (inner, new_i) = consume_balanced_parens(s, i)?;
      i = new_i;
      let inner = inner.trim();
      PseudoClass::Dir(inner.to_ascii_lowercase())
    }
    _ => return Err(format!("unknown pseudo-class `:{}`", name)),
  };

  Ok((PseudoOrElement::Pseudo(pc), i))
}

/// Consume balanced parentheses content. `pos` is right after the opening `(`.
/// Returns (content_inside_parens, position_after_closing_paren).
fn consume_balanced_parens(s: &str, pos: usize) -> Result<(&str, usize), String> {
  let bytes = s.as_bytes();
  let mut depth = 1i32;
  let mut i = pos;
  let mut quote: Option<u8> = None;
  while i < bytes.len() && depth > 0 {
    match quote {
      Some(q) => {
        if bytes[i] == q {
          quote = None;
        }
      }
      None => match bytes[i] {
        b'"' | b'\'' => quote = Some(bytes[i]),
        b'(' => depth += 1,
        b')' => depth -= 1,
        _ => {}
      },
    }
    if depth > 0 {
      i += 1;
    }
  }
  if depth != 0 {
    return Err("unbalanced parentheses".into());
  }
  let content = &s[pos..i];
  Ok((content, i + 1))
}

/// Parse the argument of `:has()` — a comma-separated list of relative selectors.
fn parse_has_argument(s: &str) -> Result<Vec<HasSelector>, String> {
  let mut out = Vec::new();
  for part in split_top_level_commas(s) {
    let t = part.trim();
    if t.is_empty() {
      return Err("empty relative selector in :has()".into());
    }
    out.push(parse_relative_selector(t)?);
  }
  Ok(out)
}

/// Parse a single relative selector (for inside `:has()`).
fn parse_relative_selector(s: &str) -> Result<HasSelector, String> {
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() && bytes[i].is_ascii_whitespace() {
    i += 1;
  }
  // Check for leading combinator.
  let leading = match bytes.get(i) {
    Some(b'>') => {
      i += 1;
      Combinator::Child
    }
    Some(b'+') => {
      i += 1;
      Combinator::NextSibling
    }
    Some(b'~') => {
      i += 1;
      Combinator::SubsequentSibling
    }
    _ => Combinator::Descendant,
  };
  let rest = s[i..].trim();
  if rest.is_empty() {
    return Err("empty relative selector after combinator".into());
  }
  let complex = ComplexSelector::parse(rest)?;
  Ok(HasSelector {
    leading_combinator: leading,
    compounds: complex.compounds,
    combinators: complex.combinators,
  })
}

// ── An+B formula parsing ────────────────────────────────────────────────────

fn parse_nth_with_of(s: &str) -> Result<(NthFormula, Option<SelectorList>), String> {
  // Check for "X of S" syntax.
  // Find " of " that's not inside parens.
  let lower = s.to_ascii_lowercase();
  if let Some(of_pos) = find_of_keyword(&lower) {
    let formula_part = s[..of_pos].trim();
    let sel_part = s[of_pos + 3..].trim(); // skip " of"
    let formula = parse_nth_formula(formula_part)?;
    let sel = SelectorList::parse(sel_part)?;
    Ok((formula, Some(sel)))
  } else {
    let formula = parse_nth_formula(s)?;
    Ok((formula, None))
  }
}

fn find_of_keyword(s: &str) -> Option<usize> {
  let bytes = s.as_bytes();
  let mut i = 0;
  let mut paren_depth = 0i32;
  while i < bytes.len() {
    match bytes[i] {
      b'(' => paren_depth += 1,
      b')' => paren_depth -= 1,
      _ if paren_depth == 0 => {
        if i > 0 && bytes[i].is_ascii_whitespace() && i + 3 <= bytes.len() {
          let rest = &s[i..];
          let trimmed = rest.trim_start();
          if trimmed.len() >= 2
            && trimmed.as_bytes()[0].to_ascii_lowercase() == b'o'
            && trimmed.as_bytes()[1].to_ascii_lowercase() == b'f'
            && (trimmed.len() == 2 || trimmed.as_bytes()[2].is_ascii_whitespace())
          {
            return Some(i);
          }
        }
      }
      _ => {}
    }
    i += 1;
  }
  None
}

fn parse_nth_formula(s: &str) -> Result<NthFormula, String> {
  let s = s.trim();
  if s.is_empty() {
    return Err("empty nth formula".into());
  }
  let lower = s.to_ascii_lowercase();
  match lower.as_str() {
    "odd" => return Ok(NthFormula { a: 2, b: 1 }),
    "even" => return Ok(NthFormula { a: 2, b: 0 }),
    _ => {}
  }

  // Try An+B parsing.
  let bytes = lower.as_bytes();
  let mut i = 0;

  // Optional sign for A.
  let mut a_neg = false;
  if i < bytes.len() && bytes[i] == b'-' {
    a_neg = true;
    i += 1;
  } else if i < bytes.len() && bytes[i] == b'+' {
    i += 1;
  }

  // Check if we have 'n'.
  let num_start = i;
  while i < bytes.len() && bytes[i].is_ascii_digit() {
    i += 1;
  }
  let has_digits = i > num_start;

  if i < bytes.len() && bytes[i] == b'n' {
    // An+B form.
    let a = if has_digits {
      let v: i32 = lower[num_start..i].parse().map_err(|_| "bad nth coefficient")?;
      if a_neg { -v } else { v }
    } else if a_neg {
      -1
    } else {
      1
    };
    i += 1; // skip 'n'

    // Skip whitespace.
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
      i += 1;
    }

    if i >= bytes.len() {
      return Ok(NthFormula { a, b: 0 });
    }

    let b_neg = match bytes[i] {
      b'+' => {
        i += 1;
        false
      }
      b'-' => {
        i += 1;
        true
      }
      _ => return Err(format!("unexpected `{}` in nth formula", bytes[i] as char)),
    };

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
      i += 1;
    }

    let b_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
      i += 1;
    }
    if i == b_start {
      return Err("expected number after +/- in nth formula".into());
    }
    let b: i32 = lower[b_start..i].parse().map_err(|_| "bad nth offset")?;
    let b = if b_neg { -b } else { b };

    if i != bytes.len() {
      return Err(format!("trailing content in nth formula: `{}`", &lower[i..]));
    }
    Ok(NthFormula { a, b })
  } else if has_digits && i == bytes.len() {
    // Plain integer B.
    let b: i32 = lower[num_start..i].parse().map_err(|_| "bad nth number")?;
    let b = if a_neg { -b } else { b };
    Ok(NthFormula { a: 0, b })
  } else {
    Err(format!("invalid nth formula `{}`", s))
  }
}

// ── Attribute filter parsing ────────────────────────────────────────────────

fn parse_attr_filter(s: &str) -> Result<(AttrFilter, usize), String> {
  let bytes = s.as_bytes();
  debug_assert_eq!(bytes[0], b'[');
  let mut i = 1;

  while i < bytes.len() && bytes[i].is_ascii_whitespace() {
    i += 1;
  }

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
      i += 1;
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

  let mut case_insensitive = false;
  if i < bytes.len() && (bytes[i] == b'i' || bytes[i] == b'I') {
    case_insensitive = true;
    i += 1;
  } else if i < bytes.len() && (bytes[i] == b's' || bytes[i] == b'S') {
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

impl std::ops::Index<usize> for SelectorList {
  type Output = ComplexSelector;
  fn index(&self, idx: usize) -> &ComplexSelector {
    &self.selectors[idx]
  }
}

// ── Conversion impls ────────────────────────────────────────────────────────

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

impl CompoundSelector {
  pub fn parse(input: &str) -> Result<Self, String> {
    let s = input.trim();
    if s.is_empty() {
      return Err("empty selector".into());
    }
    let (compound, consumed) = parse_compound(s)?;
    if consumed != s.len() {
      return Err(format!("trailing input after compound selector: `{}`", &s[consumed..]));
    }
    Ok(compound)
  }

  /// Test a single [`Element`] against this compound (basic selectors only,
  /// no pseudo-class context). Cheap (no allocations).
  pub fn matches(&self, el: &Element) -> bool {
    self.matches_basic(el)
  }

  fn matches_basic(&self, el: &Element) -> bool {
    if self.never_matches {
      return false;
    }
    if matches!(el, Element::Text(_)) {
      return false;
    }
    if let Some(tag) = &self.tag {
      if !el.tag_name().eq_ignore_ascii_case(tag) {
        return false;
      }
    }
    if let Some(id) = &self.id {
      if el.id() != Some(id.as_str()) {
        return false;
      }
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
    // Namespace: Named(...) never matches in our namespaceless model.
    if let Some(Namespace::Named(_)) = &self.namespace {
      return false;
    }
    true
  }

  /// Full match with pseudo-class context. Used by the cascade and
  /// `querySelector`. Skips pseudo-elements.
  pub fn matches_in_tree(&self, root: &Node, path: &[usize], ctx: &MatchContext) -> bool {
    if self.pseudo_element.is_some() {
      return false;
    }
    let Some(node) = node_at_path(root, path) else {
      return false;
    };
    if !self.matches_basic(&node.element) {
      return false;
    }
    for pc in &self.pseudo_classes {
      if !match_pseudo_class(pc, root, path, node, ctx) {
        return false;
      }
    }
    true
  }

  /// Match ignoring the pseudo-element field. Used by the cascade to
  /// check whether a `::before`/`::after` rule's subject compound
  /// matches the originating element.
  pub fn matches_in_tree_as_pseudo_origin(&self, root: &Node, path: &[usize], ctx: &MatchContext) -> bool {
    let Some(node) = node_at_path(root, path) else {
      return false;
    };
    if !self.matches_basic(&node.element) {
      return false;
    }
    for pc in &self.pseudo_classes {
      if !match_pseudo_class(pc, root, path, node, ctx) {
        return false;
      }
    }
    true
  }

  /// CSS specificity of this compound (ignoring ancestors/combinators).
  /// Format: `(id_count << 16) | (class_count << 8) | tag_count`.
  pub fn specificity(&self) -> u32 {
    let id = if self.id.is_some() { 1 } else { 0 };
    let cls = (self.classes.len() + self.attrs.len() + self.pseudo_classes.len()) as u32;
    // :where() contributes zero specificity; its inner compounds are excluded.
    let cls = cls
      - self
        .pseudo_classes
        .iter()
        .filter(|pc| matches!(pc, PseudoClass::Where(_)))
        .count() as u32;
    let tag = if self.tag.is_some() { 1 } else { 0 };
    (id << 16) | (cls << 8) | tag
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

// ── Attribute matching ──────────────────────────────────────────────────────

impl AttrFilter {
  fn matches_element(&self, el: &Element) -> bool {
    let attr_value = el.attr(&self.name);
    match (self.op, attr_value) {
      (AttrOp::Exists, Some(_)) => true,
      (AttrOp::Exists, None) => false,
      (_, None) => false,
      (AttrOp::Equals, Some(v)) => self.cmp_eq(&v, &self.value),
      (AttrOp::Includes, Some(v)) => v.split_ascii_whitespace().any(|tok| self.cmp_eq(tok, &self.value)),
      (AttrOp::DashMatch, Some(v)) => {
        if self.cmp_eq(&v, &self.value) {
          return true;
        }
        let nl = self.value.len();
        v.len() > nl && v.is_char_boundary(nl) && v.as_bytes()[nl] == b'-' && self.cmp_eq(&v[..nl], &self.value)
      }
      (AttrOp::Prefix, Some(v)) => !self.value.is_empty() && self.starts_with_cmp(&v, &self.value),
      (AttrOp::Suffix, Some(v)) => !self.value.is_empty() && self.ends_with_cmp(&v, &self.value),
      (AttrOp::Substring, Some(v)) => !self.value.is_empty() && self.contains_cmp(&v, &self.value),
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
      hay.len() >= needle.len() && hay[hay.len() - needle.len()..].eq_ignore_ascii_case(needle)
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

// ── Pseudo-class matching ───────────────────────────────────────────────────

fn match_pseudo_class(pc: &PseudoClass, root: &Node, path: &[usize], node: &Node, ctx: &MatchContext) -> bool {
  match pc {
    // ── Logical ──
    PseudoClass::Not(sel) => !sel.matches_in_tree(root, path, ctx),
    PseudoClass::Is(sel) | PseudoClass::Where(sel) => sel.matches_in_tree(root, path, ctx),
    PseudoClass::Has(has_sels) => match_has(root, path, has_sels, ctx),

    // ── Structural ──
    PseudoClass::Root => path.is_empty(),
    PseudoClass::Scope => path.is_empty(),
    PseudoClass::Empty => node.children.is_empty(),
    PseudoClass::FirstChild => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      first_element_child_index(parent) == Some(my_idx)
    }
    PseudoClass::LastChild => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      last_element_child_index(parent) == Some(my_idx)
    }
    PseudoClass::OnlyChild => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      first_element_child_index(parent) == Some(my_idx) && last_element_child_index(parent) == Some(my_idx)
    }
    PseudoClass::NthChild(formula, of_sel) => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      let pos = match of_sel {
        None => element_position_1based(parent, my_idx),
        Some(sel) => {
          let parent_path = &path[..path.len() - 1];
          element_position_1based_of(root, parent, parent_path, my_idx, sel, ctx)
        }
      };
      pos.is_some_and(|p| formula.matches(p))
    }
    PseudoClass::NthLastChild(formula) => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      if let Some(pos) = element_position_from_end_1based(parent, my_idx) {
        formula.matches(pos)
      } else {
        false
      }
    }
    PseudoClass::FirstOfType => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      let tag = node.element.tag_name();
      first_of_type_index(parent, tag) == Some(my_idx)
    }
    PseudoClass::LastOfType => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      let tag = node.element.tag_name();
      last_of_type_index(parent, tag) == Some(my_idx)
    }
    PseudoClass::NthOfType(formula) => {
      if path.is_empty() {
        return false;
      }
      let parent = node_at_path(root, &path[..path.len() - 1]).unwrap();
      let my_idx = *path.last().unwrap();
      let tag = node.element.tag_name();
      if let Some(pos) = nth_of_type_position(parent, tag, my_idx) {
        formula.matches(pos)
      } else {
        false
      }
    }

    // ── State ──
    PseudoClass::Disabled => el_is_disabled(&node.element),
    PseudoClass::Enabled => el_is_enableable(&node.element) && !el_is_disabled(&node.element),
    PseudoClass::Checked => el_is_checked(&node.element),
    PseudoClass::Required => el_is_required(&node.element),
    PseudoClass::Optional => el_is_optionable(&node.element) && !el_is_required(&node.element),
    PseudoClass::ReadOnly => el_is_read_only(node),
    PseudoClass::ReadWrite => el_is_read_write(node),
    PseudoClass::PlaceholderShown => el_placeholder_shown(node),

    // ── Interaction ──
    PseudoClass::Hover => {
      let Some(ia) = ctx.interaction else {
        return false;
      };
      let Some(hp) = &ia.hover_path else {
        return false;
      };
      hp.len() >= path.len() && &hp[..path.len()] == path
    }
    PseudoClass::Focus => {
      let Some(ia) = ctx.interaction else {
        return false;
      };
      ia.focus_path.as_ref().is_some_and(|fp| fp == path)
    }
    PseudoClass::Active => {
      let Some(ia) = ctx.interaction else {
        return false;
      };
      let Some(ap) = &ia.active_path else {
        return false;
      };
      ap.len() >= path.len() && &ap[..path.len()] == path
    }
    PseudoClass::FocusWithin => {
      let Some(ia) = ctx.interaction else {
        return false;
      };
      let Some(fp) = &ia.focus_path else {
        return false;
      };
      // Match if focus_path starts with `path` (i.e. focus is inside).
      fp.len() >= path.len() && &fp[..path.len()] == path
    }

    // ── Lang / Dir ──
    PseudoClass::Lang(lang_arg) => match_lang(root, path, lang_arg),
    PseudoClass::Dir(dir_arg) => match_dir(root, path, dir_arg),
  }
}

// ── Structural helpers ──────────────────────────────────────────────────────

fn first_element_child_index(parent: &Node) -> Option<usize> {
  parent
    .children
    .iter()
    .position(|c| !matches!(c.element, Element::Text(_)))
}

fn last_element_child_index(parent: &Node) -> Option<usize> {
  parent
    .children
    .iter()
    .rposition(|c| !matches!(c.element, Element::Text(_)))
}

fn element_position_1based(parent: &Node, idx: usize) -> Option<i32> {
  if matches!(parent.children.get(idx)?.element, Element::Text(_)) {
    return None;
  }
  let mut pos = 0i32;
  for (i, c) in parent.children.iter().enumerate() {
    if matches!(c.element, Element::Text(_)) {
      continue;
    }
    pos += 1;
    if i == idx {
      return Some(pos);
    }
  }
  None
}

fn element_position_1based_of(
  root: &Node,
  parent: &Node,
  parent_path: &[usize],
  idx: usize,
  sel: &SelectorList,
  ctx: &MatchContext,
) -> Option<i32> {
  if matches!(parent.children.get(idx)?.element, Element::Text(_)) {
    return None;
  }
  let mut pos = 0i32;
  for (i, c) in parent.children.iter().enumerate() {
    if matches!(c.element, Element::Text(_)) {
      continue;
    }
    // Check if this child matches the `of` selector.
    let child_path: Vec<usize> = parent_path.iter().copied().chain(std::iter::once(i)).collect();
    if sel.matches_in_tree(root, &child_path, ctx) {
      pos += 1;
      if i == idx {
        return Some(pos);
      }
    }
  }
  None
}

fn element_position_from_end_1based(parent: &Node, idx: usize) -> Option<i32> {
  if matches!(parent.children.get(idx)?.element, Element::Text(_)) {
    return None;
  }
  let mut pos = 0i32;
  for (i, c) in parent.children.iter().enumerate().rev() {
    if matches!(c.element, Element::Text(_)) {
      continue;
    }
    pos += 1;
    if i == idx {
      return Some(pos);
    }
  }
  None
}

fn first_of_type_index(parent: &Node, tag: &str) -> Option<usize> {
  parent
    .children
    .iter()
    .position(|c| c.element.tag_name().eq_ignore_ascii_case(tag))
}

fn last_of_type_index(parent: &Node, tag: &str) -> Option<usize> {
  parent
    .children
    .iter()
    .rposition(|c| c.element.tag_name().eq_ignore_ascii_case(tag))
}

fn nth_of_type_position(parent: &Node, tag: &str, idx: usize) -> Option<i32> {
  let mut pos = 0i32;
  for (i, c) in parent.children.iter().enumerate() {
    if c.element.tag_name().eq_ignore_ascii_case(tag) {
      pos += 1;
      if i == idx {
        return Some(pos);
      }
    }
  }
  None
}

// ── State helpers ───────────────────────────────────────────────────────────

fn el_is_disabled(el: &Element) -> bool {
  match el {
    Element::Input(e) => e.disabled == Some(true),
    Element::Textarea(e) => e.disabled == Some(true),
    Element::Select(e) => e.disabled == Some(true),
    Element::Button(e) => e.disabled == Some(true),
    Element::Optgroup(e) => e.disabled == Some(true),
    Element::OptionElement(e) => e.disabled == Some(true),
    Element::Fieldset(e) => e.disabled == Some(true),
    _ => false,
  }
}

fn el_is_enableable(el: &Element) -> bool {
  matches!(
    el,
    Element::Input(_)
      | Element::Textarea(_)
      | Element::Select(_)
      | Element::Button(_)
      | Element::Optgroup(_)
      | Element::OptionElement(_)
      | Element::Fieldset(_)
  )
}

fn el_is_checked(el: &Element) -> bool {
  match el {
    Element::Input(e) => e.checked == Some(true),
    Element::OptionElement(e) => e.selected == Some(true),
    _ => false,
  }
}

fn el_is_required(el: &Element) -> bool {
  match el {
    Element::Input(e) => e.required == Some(true),
    Element::Textarea(e) => e.required == Some(true),
    Element::Select(e) => e.required == Some(true),
    _ => false,
  }
}

fn el_is_optionable(el: &Element) -> bool {
  matches!(el, Element::Input(_) | Element::Textarea(_) | Element::Select(_))
}

fn el_is_read_only(node: &Node) -> bool {
  match &node.element {
    Element::Input(e) => e.readonly == Some(true),
    Element::Textarea(e) => e.readonly == Some(true),
    _ => false,
  }
}

fn el_is_read_write(node: &Node) -> bool {
  match &node.element {
    Element::Input(e) => e.readonly != Some(true) && e.disabled != Some(true),
    Element::Textarea(e) => e.readonly != Some(true) && e.disabled != Some(true),
    _ => false,
  }
}

fn el_placeholder_shown(node: &Node) -> bool {
  match &node.element {
    Element::Input(e) => {
      e.placeholder.is_some() && (e.value.is_none() || e.value.as_ref().is_some_and(|v| v.is_empty()))
    }
    Element::Textarea(e) => {
      e.placeholder.is_some()
        && !node
          .children
          .iter()
          .any(|c| matches!(&c.element, Element::Text(t) if !t.is_empty()))
    }
    _ => false,
  }
}

// ── Lang / Dir matching ─────────────────────────────────────────────────────

fn match_lang(root: &Node, path: &[usize], lang_arg: &str) -> bool {
  // Walk from current node up to root looking for a `lang` attribute.
  let mut p = path.to_vec();
  loop {
    if let Some(n) = node_at_path(root, &p) {
      if let Some(lang_val) = n.element.attr("lang") {
        return lang_dash_matches(&lang_val, lang_arg);
      }
    }
    if p.is_empty() {
      break;
    }
    p.pop();
  }
  false
}

fn lang_dash_matches(lang: &str, prefix: &str) -> bool {
  if lang.eq_ignore_ascii_case(prefix) {
    return true;
  }
  if lang.len() > prefix.len()
    && lang.as_bytes()[prefix.len()] == b'-'
    && lang[..prefix.len()].eq_ignore_ascii_case(prefix)
  {
    return true;
  }
  false
}

fn match_dir(root: &Node, path: &[usize], dir_arg: &str) -> bool {
  // Check own dir attribute; fall back to ltr.
  if let Some(n) = node_at_path(root, path) {
    if let Some(dir_val) = n.element.attr("dir") {
      return dir_val.eq_ignore_ascii_case(dir_arg);
    }
  }
  // Default direction is ltr.
  dir_arg.eq_ignore_ascii_case("ltr")
}

// ── :has() matching ─────────────────────────────────────────────────────────

fn match_has(root: &Node, subject_path: &[usize], has_sels: &[HasSelector], ctx: &MatchContext) -> bool {
  for hs in has_sels {
    if match_one_has(root, subject_path, hs, ctx) {
      return true;
    }
  }
  false
}

fn match_one_has(root: &Node, subject_path: &[usize], hs: &HasSelector, ctx: &MatchContext) -> bool {
  match hs.leading_combinator {
    Combinator::Descendant => {
      // Any descendant of subject matches the inner complex selector.
      let subject = node_at_path(root, subject_path).unwrap();
      has_walk_descendants(root, subject_path, subject, hs, ctx)
    }
    Combinator::Child => {
      // Direct children of subject.
      let subject = node_at_path(root, subject_path).unwrap();
      for ci in 0..subject.children.len() {
        if matches!(subject.children[ci].element, Element::Text(_)) {
          continue;
        }
        let child_path: Vec<usize> = subject_path.iter().copied().chain(std::iter::once(ci)).collect();
        if has_match_from(root, &child_path, hs, ctx) {
          return true;
        }
      }
      false
    }
    Combinator::NextSibling => {
      if subject_path.is_empty() {
        return false;
      }
      let idx = *subject_path.last().unwrap();
      let parent_path = &subject_path[..subject_path.len() - 1];
      let parent = node_at_path(root, parent_path).unwrap();
      if let Some(next) = next_element_sibling(parent, idx) {
        let sib_path: Vec<usize> = parent_path.iter().copied().chain(std::iter::once(next)).collect();
        has_match_from(root, &sib_path, hs, ctx)
      } else {
        false
      }
    }
    Combinator::SubsequentSibling => {
      if subject_path.is_empty() {
        return false;
      }
      let idx = *subject_path.last().unwrap();
      let parent_path = &subject_path[..subject_path.len() - 1];
      let parent = node_at_path(root, parent_path).unwrap();
      for si in (idx + 1)..parent.children.len() {
        if matches!(parent.children[si].element, Element::Text(_)) {
          continue;
        }
        let sib_path: Vec<usize> = parent_path.iter().copied().chain(std::iter::once(si)).collect();
        if has_match_from(root, &sib_path, hs, ctx) {
          return true;
        }
      }
      false
    }
  }
}

/// Check if the node at `candidate_path` matches the has-selector's inner complex
/// (compounds + combinators, where the candidate is treated as the first compound).
fn has_match_from(root: &Node, candidate_path: &[usize], hs: &HasSelector, ctx: &MatchContext) -> bool {
  // The first compound must match the candidate.
  if hs.compounds.is_empty() {
    return false;
  }
  if !hs.compounds[0].matches_in_tree(root, candidate_path, ctx) {
    return false;
  }
  if hs.compounds.len() == 1 {
    return true;
  }
  // For remaining compounds, do a forward search.
  has_match_rest(root, candidate_path, hs, 0, ctx)
}

fn has_match_rest(
  root: &Node,
  current_path: &[usize],
  hs: &HasSelector,
  compound_idx: usize,
  ctx: &MatchContext,
) -> bool {
  if compound_idx + 1 >= hs.compounds.len() {
    return true;
  }
  let comb = hs.combinators[compound_idx];
  let next_compound = &hs.compounds[compound_idx + 1];
  match comb {
    Combinator::Descendant => {
      let node = node_at_path(root, current_path).unwrap();
      has_forward_walk(root, current_path, node, next_compound, hs, compound_idx + 1, ctx)
    }
    Combinator::Child => {
      let node = node_at_path(root, current_path).unwrap();
      for ci in 0..node.children.len() {
        if matches!(node.children[ci].element, Element::Text(_)) {
          continue;
        }
        let cp: Vec<usize> = current_path.iter().copied().chain(std::iter::once(ci)).collect();
        if next_compound.matches_in_tree(root, &cp, ctx) && has_match_rest(root, &cp, hs, compound_idx + 1, ctx) {
          return true;
        }
      }
      false
    }
    Combinator::NextSibling => {
      if current_path.is_empty() {
        return false;
      }
      let idx = *current_path.last().unwrap();
      let pp = &current_path[..current_path.len() - 1];
      let parent = node_at_path(root, pp).unwrap();
      if let Some(next) = next_element_sibling(parent, idx) {
        let sp: Vec<usize> = pp.iter().copied().chain(std::iter::once(next)).collect();
        next_compound.matches_in_tree(root, &sp, ctx) && has_match_rest(root, &sp, hs, compound_idx + 1, ctx)
      } else {
        false
      }
    }
    Combinator::SubsequentSibling => {
      if current_path.is_empty() {
        return false;
      }
      let idx = *current_path.last().unwrap();
      let pp = &current_path[..current_path.len() - 1];
      let parent = node_at_path(root, pp).unwrap();
      for si in (idx + 1)..parent.children.len() {
        if matches!(parent.children[si].element, Element::Text(_)) {
          continue;
        }
        let sp: Vec<usize> = pp.iter().copied().chain(std::iter::once(si)).collect();
        if next_compound.matches_in_tree(root, &sp, ctx) && has_match_rest(root, &sp, hs, compound_idx + 1, ctx) {
          return true;
        }
      }
      false
    }
  }
}

fn has_forward_walk(
  root: &Node,
  _parent_path: &[usize],
  parent: &Node,
  compound: &CompoundSelector,
  hs: &HasSelector,
  cidx: usize,
  ctx: &MatchContext,
) -> bool {
  // Walk all descendants of parent looking for a match.
  fn walk(
    root: &Node,
    path: &mut Vec<usize>,
    parent: &Node,
    compound: &CompoundSelector,
    hs: &HasSelector,
    cidx: usize,
    ctx: &MatchContext,
  ) -> bool {
    for ci in 0..parent.children.len() {
      if matches!(parent.children[ci].element, Element::Text(_)) {
        continue;
      }
      path.push(ci);
      if compound.matches_in_tree(root, path, ctx) && has_match_rest(root, path, hs, cidx, ctx) {
        path.pop();
        return true;
      }
      if walk(root, path, &parent.children[ci], compound, hs, cidx, ctx) {
        path.pop();
        return true;
      }
      path.pop();
    }
    false
  }
  let mut path = _parent_path.to_vec();
  walk(root, &mut path, parent, compound, hs, cidx, ctx)
}

fn has_walk_descendants(
  root: &Node,
  subject_path: &[usize],
  subject: &Node,
  hs: &HasSelector,
  ctx: &MatchContext,
) -> bool {
  fn walk(root: &Node, path: &mut Vec<usize>, node: &Node, hs: &HasSelector, ctx: &MatchContext) -> bool {
    for ci in 0..node.children.len() {
      if matches!(node.children[ci].element, Element::Text(_)) {
        continue;
      }
      path.push(ci);
      if has_match_from(root, path, hs, ctx) {
        path.pop();
        return true;
      }
      if walk(root, path, &node.children[ci], hs, ctx) {
        path.pop();
        return true;
      }
      path.pop();
    }
    false
  }
  let mut path = subject_path.to_vec();
  walk(root, &mut path, subject, hs, ctx)
}

// ── Tree traversal ──────────────────────────────────────────────────────────

fn node_at_path<'a>(root: &'a Node, path: &[usize]) -> Option<&'a Node> {
  let mut cur = root;
  for &i in path {
    cur = cur.children.get(i)?;
  }
  Some(cur)
}

fn previous_element_sibling(parent: &Node, idx: usize) -> Option<usize> {
  (0..idx)
    .rev()
    .find(|&j| !matches!(parent.children[j].element, Element::Text(_)))
}

fn next_element_sibling(parent: &Node, idx: usize) -> Option<usize> {
  ((idx + 1)..parent.children.len()).find(|&j| !matches!(parent.children[j].element, Element::Text(_)))
}

// ── Complex / SelectorList matching ─────────────────────────────────────────

impl ComplexSelector {
  /// The subject (rightmost) compound — the element the selector targets.
  pub fn subject(&self) -> &CompoundSelector {
    self
      .compounds
      .last()
      .expect("ComplexSelector has at least one compound")
  }

  /// All ancestor compounds (everything except the subject), in source order
  /// (closest ancestor first, matching the old `Selector::ancestors` convention).
  pub fn ancestor_compounds(&self) -> &[CompoundSelector] {
    let n = self.compounds.len();
    if n <= 1 { &[] } else { &self.compounds[..n - 1] }
  }

  /// CSS specificity: sum of all compound specificities in the chain.
  pub fn specificity(&self) -> u32 {
    self.compounds.iter().map(|c| c.specificity()).sum()
  }

  pub fn matches_in_tree(&self, root: &Node, path: &[usize], ctx: &MatchContext) -> bool {
    let n = self.compounds.len();
    if n == 0 {
      return false;
    }
    if !self.compounds[n - 1].matches_in_tree(root, path, ctx) {
      return false;
    }
    if n == 1 {
      return true;
    }

    let mut current: Vec<usize> = path.to_vec();
    for k in (0..n - 1).rev() {
      let comb = self.combinators[k];
      let prev = &self.compounds[k];
      match comb {
        Combinator::Descendant => {
          let mut found = false;
          while !current.is_empty() {
            current.pop();
            if prev.matches_in_tree(root, &current, ctx) {
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
          if !prev.matches_in_tree(root, &current, ctx) {
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
          current.truncate(current.len() - 1);
          current.push(sib);
          if !prev.matches_in_tree(root, &current, ctx) {
            return false;
          }
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
            let mut test_path = parent_path.clone();
            test_path.push(prev_j);
            if prev.matches_in_tree(root, &test_path, ctx) {
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

  /// Match this selector as a pseudo-element rule against the
  /// originating element at `path`. The subject compound's
  /// `pseudo_element` field is ignored; the caller has already
  /// filtered by pseudo-element type.
  pub fn matches_pseudo_in_tree(&self, root: &Node, path: &[usize], ctx: &MatchContext) -> bool {
    let n = self.compounds.len();
    if n == 0 {
      return false;
    }
    if !self.compounds[n - 1].matches_in_tree_as_pseudo_origin(root, path, ctx) {
      return false;
    }
    if n == 1 {
      return true;
    }

    let mut current: Vec<usize> = path.to_vec();
    for k in (0..n - 1).rev() {
      let comb = self.combinators[k];
      let prev = &self.compounds[k];
      match comb {
        Combinator::Descendant => {
          let mut found = false;
          while !current.is_empty() {
            current.pop();
            if prev.matches_in_tree(root, &current, ctx) {
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
          if !prev.matches_in_tree(root, &current, ctx) {
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
          current.truncate(current.len() - 1);
          current.push(sib);
          if !prev.matches_in_tree(root, &current, ctx) {
            return false;
          }
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
            let mut test_path = parent_path.clone();
            test_path.push(prev_j);
            if prev.matches_in_tree(root, &test_path, ctx) {
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
  /// Highest specificity among all selectors in the list.
  pub fn max_specificity(&self) -> u32 {
    self.selectors.iter().map(|s| s.specificity()).max().unwrap_or(0)
  }

  pub fn is_empty(&self) -> bool {
    self.selectors.is_empty()
  }

  pub fn iter(&self) -> impl Iterator<Item = &ComplexSelector> {
    self.selectors.iter()
  }

  pub fn len(&self) -> usize {
    self.selectors.len()
  }

  pub fn matches_in_tree(&self, root: &Node, path: &[usize], ctx: &MatchContext) -> bool {
    self.selectors.iter().any(|sel| sel.matches_in_tree(root, path, ctx))
  }
}

// ── Path collection ─────────────────────────────────────────────────────────

fn collect_matching_paths(root: &Node, sel: &SelectorList, ctx: &MatchContext) -> Vec<Vec<usize>> {
  let mut out = Vec::new();
  let mut path = Vec::new();
  walk_collect(root, sel, &mut path, &mut out, ctx);
  out
}

fn walk_collect(root: &Node, sel: &SelectorList, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>, ctx: &MatchContext) {
  if sel.matches_in_tree(root, path, ctx) {
    out.push(path.clone());
  }
  let Some(node) = node_at_path(root, path) else {
    return;
  };
  let n = node.children.len();
  for i in 0..n {
    path.push(i);
    walk_collect(root, sel, path, out, ctx);
    path.pop();
  }
}

fn first_match_path(root: &Node, sel: &SelectorList, ctx: &MatchContext) -> Option<Vec<usize>> {
  let mut path = Vec::new();
  if rec_first(root, sel, &mut path, ctx) {
    Some(path)
  } else {
    None
  }
}

fn rec_first(root: &Node, sel: &SelectorList, path: &mut Vec<usize>, ctx: &MatchContext) -> bool {
  if sel.matches_in_tree(root, path, ctx) {
    return true;
  }
  let Some(node) = node_at_path(root, path) else {
    return false;
  };
  let n = node.children.len();
  for i in 0..n {
    path.push(i);
    if rec_first(root, sel, path, ctx) {
      return true;
    }
    path.pop();
  }
  false
}

// ── Public Node / Tree methods ──────────────────────────────────────────────

impl Node {
  pub fn query_selector_path(&self, sel: impl Into<SelectorList>) -> Option<Vec<usize>> {
    let ctx = MatchContext { interaction: None };
    first_match_path(self, &sel.into(), &ctx)
  }

  pub fn query_selector_all_paths(&self, sel: impl Into<SelectorList>) -> Vec<Vec<usize>> {
    let ctx = MatchContext { interaction: None };
    collect_matching_paths(self, &sel.into(), &ctx)
  }

  pub fn query_selector(&mut self, sel: impl Into<SelectorList>) -> Option<&mut Node> {
    let ctx = MatchContext { interaction: None };
    let sel = sel.into();
    let path = first_match_path(self, &sel, &ctx)?;
    self.at_path_mut(&path)
  }

  pub fn query_selector_all(&mut self, sel: impl Into<SelectorList>) -> Vec<&mut Node> {
    let ctx = MatchContext { interaction: None };
    let sel = sel.into();
    let paths = collect_matching_paths(self, &sel, &ctx);
    let mut out: Vec<&mut Node> = Vec::with_capacity(paths.len());
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
  pub fn query_selector_path(&self, sel: impl Into<SelectorList>) -> Option<Vec<usize>> {
    let ctx = MatchContext {
      interaction: Some(&self.interaction),
    };
    first_match_path(self.root.as_ref()?, &sel.into(), &ctx)
  }

  pub fn query_selector_all_paths(&self, sel: impl Into<SelectorList>) -> Vec<Vec<usize>> {
    match self.root.as_ref() {
      Some(root) => {
        let ctx = MatchContext {
          interaction: Some(&self.interaction),
        };
        collect_matching_paths(root, &sel.into(), &ctx)
      }
      None => Vec::new(),
    }
  }

  pub fn query_selector(&mut self, sel: impl Into<SelectorList>) -> Option<&mut Node> {
    let ctx = MatchContext {
      interaction: Some(&self.interaction),
    };
    let sel = sel.into();
    let path = first_match_path(self.root.as_ref()?, &sel, &ctx)?;
    self.root.as_mut()?.at_path_mut(&path)
  }

  pub fn query_selector_all(&mut self, sel: impl Into<SelectorList>) -> Vec<&mut Node> {
    let sel = sel.into();
    // Collect paths using a shared borrow first.
    let paths = match self.root.as_ref() {
      Some(root) => {
        let ctx = MatchContext {
          interaction: Some(&self.interaction),
        };
        collect_matching_paths(root, &sel, &ctx)
      }
      None => return Vec::new(),
    };
    // Now resolve paths to &mut references.
    let Some(root) = self.root.as_mut() else {
      return Vec::new();
    };
    let mut out: Vec<&mut Node> = Vec::with_capacity(paths.len());
    unsafe {
      let root_ptr: *mut Node = root as *mut Node;
      for path in &paths {
        let mut cursor: *mut Node = root_ptr;
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
