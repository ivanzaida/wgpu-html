use crate::{
  at_rule::AtRuleParser,
  stylesheet::{CssRule, MediaFeature, MediaQuery, MediaQueryList, MediaRule, MediaType},
};

pub struct MediaAtRuleParser;

impl MediaAtRuleParser {
  pub fn parse_media_query_list_from(input: &str) -> Option<MediaQueryList> {
    parse_media_query_list(input)
  }
}

impl AtRuleParser for MediaAtRuleParser {
  fn name(&self) -> &'static str {
    "media"
  }

  fn parse_block(&self, prelude: &str, block: &str, parse_nested: &dyn Fn(&str) -> Vec<CssRule>) -> Option<CssRule> {
    let query = parse_media_query_list(prelude)?;
    let rules = parse_nested(block);
    Some(CssRule::Media(MediaRule { query, rules }))
  }
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
  input
    .as_bytes()
    .get(word.len())
    .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'-' && *b != b'_')
}

fn strip_ascii_word_prefix<'a>(input: &'a str, word: &str) -> Option<&'a str> {
  let input = input.trim_start();
  if !starts_with_ascii_word_ci(input, word) {
    return None;
  }
  Some(&input[word.len()..])
}
