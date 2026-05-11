use crate::{
  at_rule::AtRuleParser,
  stylesheet::{CssRule, Keyframe, KeyframeSelector, KeyframesRule},
  syntax::parse_raw_declarations,
  values::ArcStr,
};

pub struct KeyframesAtRuleParser;

impl AtRuleParser for KeyframesAtRuleParser {
  fn name(&self) -> &'static str {
    "keyframes"
  }

  fn parse_block(&self, prelude: &str, block: &str, _parse_nested: &dyn Fn(&str) -> Vec<CssRule>) -> Option<CssRule> {
    let name = parse_keyframes_name(prelude)?;
    let keyframes = parse_keyframe_blocks(block);
    Some(CssRule::Keyframes(KeyframesRule { name, keyframes }))
  }
}

fn parse_keyframes_name(prelude: &str) -> Option<ArcStr> {
  let name = prelude.trim();
  let name = if (name.starts_with('"') && name.ends_with('"')) || (name.starts_with('\'') && name.ends_with('\'')) {
    &name[1..name.len() - 1]
  } else {
    name
  };
  (!name.is_empty()).then(|| ArcStr::from(name))
}

fn parse_keyframe_blocks(input: &str) -> Vec<Keyframe> {
  let mut keyframes = Vec::new();
  let mut cursor = 0;
  let bytes = input.as_bytes();

  while cursor < bytes.len() {
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
      cursor += 1;
    }
    if cursor >= bytes.len() {
      break;
    }

    let Some(open_rel) = input[cursor..].find('{') else {
      break;
    };
    let open = cursor + open_rel;
    let selector_text = input[cursor..open].trim();

    let Some(close) = find_matching_brace(input, open) else {
      break;
    };
    let body = &input[open + 1..close];

    let selectors = parse_keyframe_selectors(selector_text);
    if !selectors.is_empty() {
      keyframes.push(Keyframe {
        selectors,
        declarations: parse_raw_declarations(body),
      });
    }

    cursor = close + 1;
  }

  keyframes
}

fn parse_keyframe_selectors(text: &str) -> Vec<KeyframeSelector> {
  text
    .split(',')
    .filter_map(|s| {
      let s = s.trim();
      if s.eq_ignore_ascii_case("from") {
        Some(KeyframeSelector::From)
      } else if s.eq_ignore_ascii_case("to") {
        Some(KeyframeSelector::To)
      } else if let Some(pct) = s.strip_suffix('%') {
        pct.trim().parse::<f32>().ok().map(KeyframeSelector::Percentage)
      } else {
        None
      }
    })
    .collect()
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
