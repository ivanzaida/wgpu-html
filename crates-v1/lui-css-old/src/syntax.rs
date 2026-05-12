use crate::{
  declaration::{DeclarationBlock, Importance},
  values::ArcStr,
};

#[derive(Debug, Clone)]
pub struct RawAtRule {
  pub name: String,
  pub prelude: String,
  pub block: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RawQualifiedRule {
  pub prelude: String,
  pub block: String,
}

#[derive(Debug, Clone)]
pub enum RawRule {
  AtRule(RawAtRule),
  QualifiedRule(RawQualifiedRule),
}

pub fn parse_raw_rules(input: &str) -> Vec<RawRule> {
  let input = strip_comments(input);
  let mut rules = Vec::new();
  parse_raw_rules_inner(&input, &mut rules);
  rules
}

fn parse_raw_rules_inner(input: &str, rules: &mut Vec<RawRule>) {
  let mut cursor = 0usize;
  while cursor < input.len() {
    cursor = skip_whitespace(input, cursor);
    if cursor >= input.len() {
      break;
    }

    if input[cursor..].starts_with('@') {
      let name_start = cursor + 1;
      let name_end = advance_while(input, name_start, |b| b.is_ascii_alphanumeric() || b == b'-');
      let name = input[name_start..name_end].to_string();

      let semi_pos = input[name_end..].find(';').map(|r| name_end + r);
      let open_pos = input[name_end..].find('{').map(|r| name_end + r);

      match (semi_pos, open_pos) {
        (Some(semi), Some(open)) if semi < open => {
          let prelude = input[name_end..semi].trim().to_string();
          rules.push(RawRule::AtRule(RawAtRule {
            name,
            prelude,
            block: None,
          }));
          cursor = semi + 1;
          continue;
        }
        (_, Some(open)) => {
          let prelude = input[name_end..open].trim().to_string();
          if let Some(close) = find_matching_brace(input, open) {
            let block = input[open + 1..close].to_string();
            rules.push(RawRule::AtRule(RawAtRule {
              name,
              prelude,
              block: Some(block),
            }));
            cursor = close + 1;
            continue;
          }
          break;
        }
        (Some(semi), None) => {
          let prelude = input[name_end..semi].trim().to_string();
          rules.push(RawRule::AtRule(RawAtRule {
            name,
            prelude,
            block: None,
          }));
          cursor = semi + 1;
          continue;
        }
        (None, None) => break,
      }
    }

    let Some(open_rel) = input[cursor..].find('{') else {
      break;
    };
    let open = cursor + open_rel;
    let prelude = input[cursor..open].trim().to_string();
    let Some(close) = find_matching_brace(input, open) else {
      break;
    };
    let block = input[open + 1..close].to_string();

    if !prelude.is_empty() {
      rules.push(RawRule::QualifiedRule(RawQualifiedRule { prelude, block }));
    }

    cursor = close + 1;
  }
}

pub fn parse_raw_declarations(input: &str) -> DeclarationBlock {
  let mut block = DeclarationBlock::new();
  for decl in input.split(';') {
    let decl = decl.trim();
    if decl.is_empty() {
      continue;
    }
    if let Some((property, value)) = decl.split_once(':') {
      let raw_prop = property.trim();
      let property: ArcStr = if raw_prop.starts_with("--") {
        ArcStr::from(raw_prop)
      } else {
        ArcStr::from(raw_prop.to_ascii_lowercase().as_str())
      };
      let (value, important) = strip_important(value.trim());
      let importance = if important {
        Importance::Important
      } else {
        Importance::Normal
      };
      block.push(property, value, importance);
    }
  }
  block
}

fn strip_important(value: &str) -> (&str, bool) {
  let trimmed = value.trim_end();
  let bytes = trimmed.as_bytes();
  let mut i = bytes.len();
  while i > 0 && bytes[i - 1].is_ascii_alphabetic() {
    i -= 1;
  }
  let word = &trimmed[i..];
  if !word.eq_ignore_ascii_case("important") {
    return (trimmed, false);
  }
  let mut j = i;
  while j > 0 && bytes[j - 1].is_ascii_whitespace() {
    j -= 1;
  }
  if j == 0 || bytes[j - 1] != b'!' {
    return (trimmed, false);
  }
  (trimmed[..j - 1].trim_end(), true)
}

fn skip_whitespace(input: &str, mut cursor: usize) -> usize {
  while cursor < input.len() && input.as_bytes()[cursor].is_ascii_whitespace() {
    cursor += 1;
  }
  cursor
}

fn advance_while(input: &str, mut cursor: usize, pred: impl Fn(u8) -> bool) -> usize {
  let bytes = input.as_bytes();
  while cursor < bytes.len() && pred(bytes[cursor]) {
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
