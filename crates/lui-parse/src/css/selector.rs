use lui_core::{
  AttrOp, AttributeSelector, ComplexSelector, CompoundSelector, CssCombinator, CssPseudo, ParseError, PseudoSelector,
  SelectorList,
};

/// Compute the CSS specificity of a complex selector.
pub fn complex_specificity(sel: &ComplexSelector) -> (u32, u32, u32) {
  let mut a = 0u32;
  let mut b = 0u32;
  let mut c = 0u32;
  for compound in &sel.compounds {
    let (ca, cb, cc) = compound_specificity(compound);
    a += ca;
    b += cb;
    c += cc;
  }
  (a, b, c)
}

fn compound_specificity(compound: &CompoundSelector) -> (u32, u32, u32) {
  let mut a = 0u32;
  let mut b = 0u32;
  let mut c = 0u32;

  if compound.id.is_some() {
    a += 1;
  }
  b += compound.classes.len() as u32;
  b += compound.attrs.len() as u32;

  if let Some(ref tag) = compound.tag {
    if tag != "*" {
      c += 1;
    }
  }

  for pseudo in &compound.pseudos {
    match &pseudo.pseudo {
      CssPseudo::Where => {}
      CssPseudo::Is | CssPseudo::Not | CssPseudo::Has | CssPseudo::Matches => {
        if let Some(ref arg) = pseudo.arg {
          if let Ok(inner) = parse_selector_list(arg) {
            let max = inner
              .0
              .iter()
              .map(|sel| complex_specificity(sel))
              .max()
              .unwrap_or((0, 0, 0));
            a += max.0;
            b += max.1;
            c += max.2;
          }
        }
      }
      p if p.name().starts_with("::") => {
        c += 1;
      }
      _ => {
        b += 1;
      }
    }
  }

  (a, b, c)
}

pub fn parse_selector_list(input: &str) -> Result<SelectorList, ParseError> {
  let chars: Vec<char> = input.chars().collect();
  let mut pos = 0;
  let mut list = Vec::new();

  loop {
    skip_ws(&chars, &mut pos);
    if pos >= chars.len() {
      break;
    }
    let sel = parse_complex_selector(&chars, &mut pos)?;
    list.push(sel);
    skip_ws(&chars, &mut pos);
    if pos < chars.len() && chars[pos] == ',' {
      pos += 1;
    } else {
      break;
    }
  }

  if pos < chars.len() {
    return Err(ParseError {
      message: format!("unexpected '{}' at {}", chars[pos], pos),
      position: pos,
    });
  }

  Ok(SelectorList(list))
}

fn parse_complex_selector(chars: &[char], pos: &mut usize) -> Result<ComplexSelector, ParseError> {
  let mut compounds = Vec::new();
  let mut combinators = Vec::new();

  loop {
    skip_ws(chars, pos);
    let compound = parse_compound_selector(chars, pos)?;
    compounds.push(compound);

    let before_ws = *pos;
    skip_ws(chars, pos);
    if *pos >= chars.len() || chars[*pos] == ',' {
      break;
    }

    let had_ws = *pos > before_ws;
    let comb = parse_combinator(chars, pos);
    match comb {
      Some(c) => combinators.push(c),
      None if had_ws => combinators.push(CssCombinator::Descendant),
      None => break,
    }
  }

  Ok(ComplexSelector { compounds, combinators })
}

fn parse_combinator(chars: &[char], pos: &mut usize) -> Option<CssCombinator> {
  if *pos >= chars.len() {
    return None;
  }
  match chars[*pos] {
    '>' => {
      *pos += 1;
      Some(CssCombinator::Child)
    }
    '+' => {
      *pos += 1;
      Some(CssCombinator::NextSibling)
    }
    '~' => {
      *pos += 1;
      Some(CssCombinator::SubsequentSibling)
    }
    '|' if *pos + 1 < chars.len() && chars[*pos + 1] == '|' => {
      *pos += 2;
      Some(CssCombinator::Column)
    }
    _ => {
      if chars[*pos].is_ascii_whitespace() {
        Some(CssCombinator::Descendant)
      } else {
        None
      }
    }
  }
}

fn parse_compound_selector(chars: &[char], pos: &mut usize) -> Result<CompoundSelector, ParseError> {
  let mut sel = CompoundSelector::default();
  let start = *pos;

  loop {
    if *pos >= chars.len() {
      break;
    }

    match chars[*pos] {
      '&' => {
        *pos += 1;
        sel.pseudos.push(PseudoSelector {
          pseudo: CssPseudo::Ampersand,
          arg: None,
        });
        continue;
      }
      '#' => {
        *pos += 1;
        let id = parse_ident(chars, pos)?;
        sel.id = Some(id);
        continue;
      }
      '.' => {
        *pos += 1;
        let class = parse_ident(chars, pos)?;
        sel.classes.push(class);
        continue;
      }
      ':' => {
        let pseudo = parse_pseudo(chars, pos)?;
        sel.pseudos.push(pseudo);
        continue;
      }
      '[' => {
        let attr = parse_attr(chars, pos)?;
        sel.attrs.push(attr);
        continue;
      }
      '*' => {
        *pos += 1;
        sel.tag = Some("*".to_string());
        continue;
      }
      c if c.is_ascii_alphabetic() || c == '-' || c == '_' => {
        if sel.tag.is_some() {
          break;
        }
        sel.tag = Some(parse_ident(chars, pos)?);
        continue;
      }
      _ => break,
    }
  }

  if *pos == start {
    return Err(ParseError {
      message: "empty compound selector".to_string(),
      position: *pos,
    });
  }

  Ok(sel)
}

fn parse_pseudo(chars: &[char], pos: &mut usize) -> Result<PseudoSelector, ParseError> {
  let start = *pos;
  let mut name = String::new();
  name.push(':');
  *pos += 1;
  if *pos < chars.len() && chars[*pos] == ':' {
    name.push(':');
    *pos += 1;
  }
  let ident = parse_ident(chars, pos)?;
  name.push_str(&ident);

  let arg = if *pos < chars.len() && chars[*pos] == '(' {
    *pos += 1;
    let mut depth = 1;
    let arg_start = *pos;
    while *pos < chars.len() && depth > 0 {
      match chars[*pos] {
        '(' => depth += 1,
        ')' => depth -= 1,
        _ => {}
      }
      *pos += 1;
    }
    if depth != 0 {
      return Err(ParseError {
        message: "unclosed '(' in pseudo".to_string(),
        position: start,
      });
    }
    let arg_str: String = chars[arg_start..*pos - 1].iter().collect();
    name.push('(');
    name.push(')');
    Some(arg_str)
  } else {
    None
  };

  Ok(PseudoSelector {
    pseudo: CssPseudo::from_name(&name),
    arg,
  })
}

fn parse_attr(chars: &[char], pos: &mut usize) -> Result<AttributeSelector, ParseError> {
  *pos += 1;
  skip_ws(chars, pos);
  let name = parse_ident(chars, pos)?;
  skip_ws(chars, pos);

  let mut op = None;
  let mut value = None;
  let mut modifier = None;

  if *pos < chars.len() && chars[*pos] != ']' {
    let mut op_str = String::new();
    while *pos < chars.len()
      && (chars[*pos] == '~' || chars[*pos] == '|' || chars[*pos] == '^' || chars[*pos] == '$' || chars[*pos] == '*')
    {
      op_str.push(chars[*pos]);
      *pos += 1;
    }
    if *pos < chars.len() && chars[*pos] == '=' {
      op_str.push('=');
      *pos += 1;
    } else {
      op_str.clear();
    }

    op = match op_str.as_str() {
      "=" => Some(AttrOp::Eq),
      "~=" => Some(AttrOp::Includes),
      "|=" => Some(AttrOp::Hyphen),
      "^=" => Some(AttrOp::StartsWith),
      "$=" => Some(AttrOp::EndsWith),
      "*=" => Some(AttrOp::Contains),
      _ => None,
    };

    skip_ws(chars, pos);
    if *pos < chars.len() && (chars[*pos] == '"' || chars[*pos] == '\'') {
      let quote = chars[*pos];
      *pos += 1;
      let v_start = *pos;
      while *pos < chars.len() && chars[*pos] != quote {
        if chars[*pos] == '\\' {
          *pos += 1;
        }
        *pos += 1;
      }
      value = Some(chars[v_start..*pos].iter().collect());
      *pos += 1;
    } else if *pos < chars.len() && chars[*pos] != ']' {
      value = Some(parse_ident(chars, pos)?);
    }

    skip_ws(chars, pos);
    if *pos < chars.len() && (chars[*pos] == 'i' || chars[*pos] == 's' || chars[*pos] == 'I' || chars[*pos] == 'S') {
      modifier = Some(chars[*pos].to_ascii_lowercase());
      *pos += 1;
    }
  }

  skip_ws(chars, pos);
  if *pos >= chars.len() || chars[*pos] != ']' {
    return Err(ParseError {
      message: "expected ']'".to_string(),
      position: *pos,
    });
  }
  *pos += 1;

  Ok(AttributeSelector {
    name,
    op,
    value,
    modifier,
  })
}

fn parse_ident(chars: &[char], pos: &mut usize) -> Result<String, ParseError> {
  let start = *pos;
  if *pos < chars.len() && chars[*pos] == '-' {
    *pos += 1;
    if *pos < chars.len() && chars[*pos] == '-' {
      *pos += 1;
    }
  }
  while *pos < chars.len() && (chars[*pos].is_ascii_alphanumeric() || chars[*pos] == '-' || chars[*pos] == '_') {
    *pos += 1;
  }
  if *pos == start {
    if *pos < chars.len() && chars[*pos] == '-' {
      *pos += 1;
      return Ok("-".to_string());
    }
    return Err(ParseError {
      message: "expected identifier".to_string(),
      position: *pos,
    });
  }
  Ok(chars[start..*pos].iter().collect())
}

fn skip_ws(chars: &[char], pos: &mut usize) {
  while *pos < chars.len() && chars[*pos].is_ascii_whitespace() {
    *pos += 1;
  }
}
