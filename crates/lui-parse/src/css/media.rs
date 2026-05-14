use lui_core::{MediaCondition, MediaFeature, MediaModifier, MediaQuery, MediaQueryList, ParseError};

use crate::css::parser::parse_value;

pub fn parse_media_query_list(input: &str) -> Result<MediaQueryList, ParseError> {
  let chars: Vec<char> = input.chars().collect();
  let mut pos = 0;
  let mut queries = Vec::new();

  loop {
    skip_ws(&chars, &mut pos);
    if pos >= chars.len() {
      break;
    }
    queries.push(parse_media_query(&chars, &mut pos)?);
    skip_ws(&chars, &mut pos);
    if pos < chars.len() && chars[pos] == ',' {
      pos += 1;
    }
  }

  Ok(MediaQueryList(queries))
}

fn parse_media_query(chars: &[char], pos: &mut usize) -> Result<MediaQuery, ParseError> {
  let modifier = parse_modifier(chars, pos);
  skip_ws(chars, pos);

  let media_type = if *pos < chars.len() && chars[*pos].is_ascii_alphabetic() {
    let start = *pos;
    while *pos < chars.len() && chars[*pos].is_ascii_alphabetic() {
      *pos += 1;
    }
    Some(chars[start..*pos].iter().collect())
  } else {
    None
  };

  let mut conditions = Vec::new();
  skip_ws(chars, pos);
  while *pos < chars.len() && (chars[*pos] == 'a' || chars[*pos] == '(') {
    if *pos < chars.len() && chars[*pos] == 'a' {
      let word: String = chars[*pos..].iter().take(3).collect();
      if word.to_lowercase() == "and" {
        *pos += 3;
        skip_ws(chars, pos);
      }
    }
    if *pos < chars.len() && chars[*pos] == '(' {
      let feature = parse_media_feature(chars, pos)?;
      conditions.push(MediaCondition::Feature(feature));
      skip_ws(chars, pos);
    }
  }

  Ok(MediaQuery {
    modifier,
    media_type,
    conditions,
  })
}

fn parse_modifier(chars: &[char], pos: &mut usize) -> Option<MediaModifier> {
  if *pos + 3 < chars.len() {
    let word: String = chars[*pos..*pos + 3].iter().collect();
    match word.to_lowercase().as_str() {
      "not" => {
        *pos += 3;
        return Some(MediaModifier::Not);
      }
      "onl" if *pos + 4 < chars.len() && chars[*pos + 3] == 'y' => {
        *pos += 4;
        return Some(MediaModifier::Only);
      }
      _ => {}
    }
  }
  None
}

fn parse_media_feature(chars: &[char], pos: &mut usize) -> Result<MediaFeature, ParseError> {
  *pos += 1;
  skip_ws(chars, pos);

  let name_start = *pos;
  while *pos < chars.len() && (chars[*pos].is_ascii_alphanumeric() || chars[*pos] == '-') {
    *pos += 1;
  }
  let name: String = chars[name_start..*pos].iter().collect();
  skip_ws(chars, pos);

  let value = if *pos < chars.len() && chars[*pos] == ':' {
    *pos += 1;
    skip_ws(chars, pos);
    let v_start = *pos;
    while *pos < chars.len() && chars[*pos] != ')' {
      *pos += 1;
    }
    let val_str: String = chars[v_start..*pos].iter().collect();
    Some(parse_value(val_str.trim())?)
  } else {
    None
  };

  if *pos < chars.len() && chars[*pos] == ')' {
    *pos += 1;
  }
  Ok(MediaFeature { name, value })
}

fn skip_ws(chars: &[char], pos: &mut usize) {
  while *pos < chars.len() && chars[*pos].is_ascii_whitespace() {
    *pos += 1;
  }
}
