use lui_models::{
  ArcStr, Style,
  common::{
    css_enums::*,
    CssColor, CssContent, CssImage, CssLength, CssMathExpr, CssNumericFunction, GridLine,
    GridTrackSize,
  },
};
pub fn parse_css_length(value: &str) -> Option<CssLength> {
  let v = value.trim();
  if v.is_empty() {
    return None;
  }
  if v.eq_ignore_ascii_case("auto") {
    return Some(CssLength::Auto);
  }
  if v == "0" {
    return Some(CssLength::Zero);
  }
  if let Some(inner) = strip_func(v, "calc") {
    if let Some(expr) = parse_css_math_expr(inner) {
      return Some(CssLength::Calc(Box::new(expr)));
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "min") {
    let args: Vec<CssLength> = split_top_level_commas(inner)
      .into_iter()
      .filter_map(parse_css_length)
      .collect();
    if !args.is_empty() {
      return Some(CssLength::Min(args));
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "max") {
    let args: Vec<CssLength> = split_top_level_commas(inner)
      .into_iter()
      .filter_map(parse_css_length)
      .collect();
    if !args.is_empty() {
      return Some(CssLength::Max(args));
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "clamp") {
    let args: Vec<CssLength> = split_top_level_commas(inner)
      .into_iter()
      .filter_map(parse_css_length)
      .collect();
    if args.len() == 3 {
      return Some(CssLength::Clamp {
        min: Box::new(args[0].clone()),
        preferred: Box::new(args[1].clone()),
        max: Box::new(args[2].clone()),
      });
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "fit-content") {
    return parse_css_length(inner).or_else(|| Some(CssLength::Raw(ArcStr::from(v))));
  }
  if is_numeric_function_value(v) {
    if let Some(expr) = parse_css_math_expr(v) {
      return Some(CssLength::Calc(Box::new(expr)));
    }
  }
  if let Some(s) = v.strip_suffix("px") {
    return s.trim().parse::<f32>().ok().map(CssLength::Px);
  }
  if let Some(s) = v.strip_suffix('%') {
    return s.trim().parse::<f32>().ok().map(CssLength::Percent);
  }
  if let Some(s) = v.strip_suffix("rem") {
    return s.trim().parse::<f32>().ok().map(CssLength::Rem);
  }
  if let Some(s) = v.strip_suffix("em") {
    return s.trim().parse::<f32>().ok().map(CssLength::Em);
  }
  if let Some(s) = v.strip_suffix("vw") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vw);
  }
  if let Some(s) = v.strip_suffix("vh") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vh);
  }
  if let Some(s) = v.strip_suffix("vmin") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vmin);
  }
  if let Some(s) = v.strip_suffix("vmax") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vmax);
  }
  // Bare number (treat as raw)
  Some(CssLength::Raw(ArcStr::from(v)))
}

fn is_numeric_function_value(v: &str) -> bool {
  let Some(open) = v.find('(') else {
    return false;
  };
  let name = v[..open].trim();
  numeric_function_from_name(name).is_some() && v.ends_with(')')
}

fn parse_css_math_expr(input: &str) -> Option<CssMathExpr> {
  let mut parser = MathParser::new(input);
  let expr = parser.parse_sum()?;
  parser.skip_ws();
  if parser.is_eof() { Some(expr) } else { None }
}

struct MathParser<'a> {
  input: &'a str,
  pos: usize,
}

impl<'a> MathParser<'a> {
  fn new(input: &'a str) -> Self {
    Self { input, pos: 0 }
  }

  fn parse_sum(&mut self) -> Option<CssMathExpr> {
    let mut lhs = self.parse_product()?;
    loop {
      self.skip_ws();
      if self.consume_char('+') {
        let rhs = self.parse_product()?;
        lhs = CssMathExpr::Add(Box::new(lhs), Box::new(rhs));
      } else if self.consume_char('-') {
        let rhs = self.parse_product()?;
        lhs = CssMathExpr::Sub(Box::new(lhs), Box::new(rhs));
      } else {
        return Some(lhs);
      }
    }
  }

  fn parse_product(&mut self) -> Option<CssMathExpr> {
    let mut lhs = self.parse_unary()?;
    loop {
      self.skip_ws();
      if self.consume_char('*') {
        let rhs = self.parse_unary()?;
        lhs = CssMathExpr::Mul(Box::new(lhs), Box::new(rhs));
      } else if self.consume_char('/') {
        let rhs = self.parse_unary()?;
        lhs = CssMathExpr::Div(Box::new(lhs), Box::new(rhs));
      } else {
        return Some(lhs);
      }
    }
  }

  fn parse_unary(&mut self) -> Option<CssMathExpr> {
    self.skip_ws();
    if self.consume_char('+') {
      return self.parse_unary();
    }
    if self.consume_char('-') {
      let rhs = self.parse_unary()?;
      return Some(CssMathExpr::Sub(Box::new(CssMathExpr::Number(0.0)), Box::new(rhs)));
    }
    self.parse_primary()
  }

  fn parse_primary(&mut self) -> Option<CssMathExpr> {
    self.skip_ws();
    if self.consume_char('(') {
      let inner = self.parse_sum()?;
      self.skip_ws();
      return self.consume_char(')').then_some(inner);
    }

    let start = self.pos;
    let ch = self.peek_char()?;
    if ch.is_ascii_alphabetic() || ch == '_' {
      let name = self.consume_ident();
      self.skip_ws();
      if self.consume_char('(') {
        let args_start = self.pos;
        let mut depth = 1i32;
        while let Some(c) = self.next_char() {
          match c {
            '(' => depth += 1,
            ')' => {
              depth -= 1;
              if depth == 0 {
                let args = &self.input[args_start..self.pos - 1];
                return self.parse_function(&name, args);
              }
            }
            _ => {}
          }
        }
        return None;
      }
      self.pos = start;
    }

    self.parse_numeric_or_length()
  }

  fn parse_function(&self, name: &str, args: &str) -> Option<CssMathExpr> {
    let fn_kind = numeric_function_from_name(name)?;
    let parsed: Vec<CssMathExpr> = split_top_level_commas(args)
      .into_iter()
      .map(parse_css_math_expr)
      .collect::<Option<Vec<_>>>()?;
    Some(CssMathExpr::Function(fn_kind, parsed))
  }

  fn parse_numeric_or_length(&mut self) -> Option<CssMathExpr> {
    let start = self.pos;
    let _number = self.consume_number_text()?;
    let unit_start = self.pos;
    while let Some(c) = self.peek_char() {
      if c.is_ascii_alphabetic() || c == '%' {
        self.next_char();
      } else {
        break;
      }
    }
    let token = &self.input[start..self.pos];
    if self.pos > unit_start {
      return parse_css_length(token).map(CssMathExpr::Length);
    }
    token.parse::<f32>().ok().map(CssMathExpr::Number)
  }

  fn consume_number_text(&mut self) -> Option<&'a str> {
    let start = self.pos;
    if matches!(self.peek_char(), Some('+') | Some('-')) {
      self.next_char();
    }
    let mut saw_digit = false;
    while let Some(c) = self.peek_char() {
      if c.is_ascii_digit() {
        saw_digit = true;
        self.next_char();
      } else {
        break;
      }
    }
    if self.consume_char('.') {
      while let Some(c) = self.peek_char() {
        if c.is_ascii_digit() {
          saw_digit = true;
          self.next_char();
        } else {
          break;
        }
      }
    }
    if !saw_digit {
      self.pos = start;
      return None;
    }
    if matches!(self.peek_char(), Some('e') | Some('E')) {
      let exp_start = self.pos;
      self.next_char();
      if matches!(self.peek_char(), Some('+') | Some('-')) {
        self.next_char();
      }
      let mut saw_exp_digit = false;
      while let Some(c) = self.peek_char() {
        if c.is_ascii_digit() {
          saw_exp_digit = true;
          self.next_char();
        } else {
          break;
        }
      }
      if !saw_exp_digit {
        self.pos = exp_start;
      }
    }
    Some(&self.input[start..self.pos])
  }

  fn consume_ident(&mut self) -> String {
    let start = self.pos;
    while let Some(c) = self.peek_char() {
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        self.next_char();
      } else {
        break;
      }
    }
    self.input[start..self.pos].to_string()
  }

  fn skip_ws(&mut self) {
    while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
      self.next_char();
    }
  }

  fn consume_char(&mut self, expected: char) -> bool {
    if self.peek_char() == Some(expected) {
      self.next_char();
      true
    } else {
      false
    }
  }

  fn peek_char(&self) -> Option<char> {
    self.input[self.pos..].chars().next()
  }

  fn next_char(&mut self) -> Option<char> {
    let c = self.peek_char()?;
    self.pos += c.len_utf8();
    Some(c)
  }

  fn is_eof(&self) -> bool {
    self.pos >= self.input.len()
  }
}

fn numeric_function_from_name(name: &str) -> Option<CssNumericFunction> {
  match name.to_ascii_lowercase().as_str() {
    "sin" => Some(CssNumericFunction::Sin),
    "cos" => Some(CssNumericFunction::Cos),
    "tan" => Some(CssNumericFunction::Tan),
    "asin" => Some(CssNumericFunction::Asin),
    "acos" => Some(CssNumericFunction::Acos),
    "atan" => Some(CssNumericFunction::Atan),
    "atan2" => Some(CssNumericFunction::Atan2),
    "pow" => Some(CssNumericFunction::Pow),
    "sqrt" => Some(CssNumericFunction::Sqrt),
    "hypot" => Some(CssNumericFunction::Hypot),
    "log" => Some(CssNumericFunction::Log),
    "exp" => Some(CssNumericFunction::Exp),
    "abs" => Some(CssNumericFunction::Abs),
    "sign" => Some(CssNumericFunction::Sign),
    "mod" => Some(CssNumericFunction::Mod),
    "rem" => Some(CssNumericFunction::Rem),
    "round" => Some(CssNumericFunction::Round),
    _ => None,
  }
}

/// Parse a CSS color value.
pub fn parse_css_color(value: &str) -> Option<CssColor> {
  let v = value.trim();
  if v.eq_ignore_ascii_case("transparent") {
    return Some(CssColor::Transparent);
  }
  if v.eq_ignore_ascii_case("currentcolor") || v.eq_ignore_ascii_case("currentColor") {
    return Some(CssColor::CurrentColor);
  }
  if v.starts_with('#') {
    return Some(CssColor::Hex(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "rgba").or_else(|| strip_func(v, "rgb")) {
    let parts = split_color_function_args(inner);
    if parts.len() >= 3 {
      let r = parse_color_component(parts[0]);
      let g = parse_color_component(parts[1]);
      let b = parse_color_component(parts[2]);
      if let Some(alpha) = parts.get(3).map(|s| parse_alpha_component(s)) {
        return Some(CssColor::Rgba(r, g, b, alpha));
      }
      return Some(CssColor::Rgb(r, g, b));
    }
  }
  if let Some(inner) = strip_func(v, "hsla").or_else(|| strip_func(v, "hsl")) {
    let parts = split_color_function_args(inner);
    if parts.len() >= 3 {
      let h = parse_hue_component(parts[0]);
      let s = parts[1].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
      let l = parts[2].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
      if let Some(alpha) = parts.get(3).map(|s| parse_alpha_component(s)) {
        return Some(CssColor::Hsla(h, s, l, alpha));
      }
      return Some(CssColor::Hsl(h, s, l));
    }
  }
  if is_preserved_color_function(v) {
    return Some(CssColor::Function(ArcStr::from(v)));
  }
  // Treat as named color
  Some(CssColor::Named(ArcStr::from(v)))
}

pub fn parse_css_image(value: &str) -> Option<CssImage> {
  let v = value.trim();
  if v.is_empty() || v.eq_ignore_ascii_case("none") {
    return None;
  }
  if let Some(url) = parse_css_url(v) {
    return Some(CssImage::Url(url));
  }
  if looks_like_function(v) {
    return Some(CssImage::Function(ArcStr::from(v)));
  }
  None
}

pub fn parse_css_url(value: &str) -> Option<ArcStr> {
  let inner = strip_function(value, "url")?;
  let inner = inner.trim();
  if inner.is_empty() {
    return None;
  }
  let unquoted =
    if (inner.starts_with('"') && inner.ends_with('"')) || (inner.starts_with('\'') && inner.ends_with('\'')) {
      if inner.len() < 2 {
        return None;
      }
      &inner[1..inner.len() - 1]
    } else {
      inner
    };
  let trimmed = unquoted.trim();
  if trimmed.is_empty() {
    None
  } else {
    Some(ArcStr::from(trimmed))
  }
}

fn looks_like_function(value: &str) -> bool {
  let trimmed = value.trim();
  let Some(open) = trimmed.find('(') else {
    return false;
  };
  trimmed.ends_with(')') && trimmed[..open].chars().all(|c| c.is_ascii_alphabetic() || c == '-')
}

pub(crate) fn strip_func<'a>(value: &'a str, func_name: &str) -> Option<&'a str> {
  let v = value.trim();
  if v.len() > func_name.len() + 2
    && v[..func_name.len()].eq_ignore_ascii_case(func_name)
    && v.as_bytes()[func_name.len()] == b'('
    && v.ends_with(')')
  {
    Some(&v[func_name.len() + 1..v.len() - 1])
  } else {
    None
  }
}

fn parse_color_component(s: &str) -> u8 {
  let s = s.trim();
  if let Some(pct) = s.strip_suffix('%') {
    let pct_val: f32 = pct.parse().unwrap_or(0.0);
    (pct_val * 2.55).round().clamp(0.0, 255.0) as u8
  } else {
    s.parse::<f32>().unwrap_or(0.0).round().clamp(0.0, 255.0) as u8
  }
}

fn parse_alpha_component(s: &str) -> f32 {
  let s = s.trim();
  if let Some(pct) = s.strip_suffix('%') {
    pct.parse::<f32>().unwrap_or(100.0) / 100.0
  } else {
    s.parse::<f32>().unwrap_or(1.0)
  }
  .clamp(0.0, 1.0)
}

fn parse_hue_component(s: &str) -> f32 {
  let s = s.trim();
  if let Some(v) = s.strip_suffix("deg") {
    v.trim().parse::<f32>().unwrap_or(0.0)
  } else if let Some(v) = s.strip_suffix("rad") {
    v.trim().parse::<f32>().unwrap_or(0.0).to_degrees()
  } else if let Some(v) = s.strip_suffix("turn") {
    v.trim().parse::<f32>().unwrap_or(0.0) * 360.0
  } else {
    s.parse::<f32>().unwrap_or(0.0)
  }
}

fn split_color_function_args(inner: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let mut start: Option<usize> = None;
  for (i, ch) in inner.char_indices() {
    if ch == ',' || ch == '/' || ch.is_whitespace() {
      if let Some(s) = start.take() {
        out.push(inner[s..i].trim());
      }
    } else if start.is_none() {
      start = Some(i);
    }
  }
  if let Some(s) = start {
    out.push(inner[s..].trim());
  }
  out.into_iter().filter(|s| !s.is_empty()).collect()
}

pub(crate) fn is_preserved_color_function(v: &str) -> bool {
  [
    "color",
    "color-mix",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
    "light-dark",
  ]
  .iter()
  .any(|name| strip_func(v, name).is_some())
}

pub(crate) fn parse_display(value: &str) -> Option<Display> {
  value.parse().ok()
}

pub(crate) fn parse_position(value: &str) -> Option<Position> {
  value.parse().ok()
}

pub(crate) fn parse_background_clip(value: &str) -> Option<BackgroundClip> {
  value.parse().ok()
}

pub(crate) fn parse_background_repeat(value: &str) -> Option<BackgroundRepeat> {
  value.parse().ok()
}

pub(crate) fn parse_border_style(value: &str) -> Option<BorderStyle> {
  value.parse().ok()
}

pub(crate) fn parse_font_weight(value: &str) -> Option<FontWeight> {
  value.parse().ok()
}

pub(crate) fn parse_font_style(value: &str) -> Option<FontStyle> {
  value.parse().ok()
}

pub(crate) fn parse_text_align(value: &str) -> Option<TextAlign> {
  value.parse().ok()
}

pub(crate) fn parse_text_transform(value: &str) -> Option<TextTransform> {
  value.parse().ok()
}

pub(crate) fn parse_white_space(value: &str) -> Option<WhiteSpace> {
  value.parse().ok()
}

pub(crate) fn parse_overflow(value: &str) -> Option<Overflow> {
  value.parse().ok()
}

pub(crate) fn parse_resize(value: &str) -> Option<Resize> {
  value.parse().ok()
}

pub(crate) fn parse_scrollbar_color(value: &str) -> Option<ScrollbarColor> {
  let v = value.trim();
  if v.eq_ignore_ascii_case("auto") {
    return Some(ScrollbarColor::Auto);
  }
  let mut parts = v.split_whitespace();
  let thumb = parts.next().and_then(parse_css_color)?;
  let track = parts.next().and_then(parse_css_color)?;
  if parts.next().is_some() {
    return None;
  }
  Some(ScrollbarColor::Custom { thumb, track })
}

pub(crate) fn parse_scrollbar_width(value: &str) -> Option<ScrollbarWidth> {
  value.parse().ok()
}

pub(crate) fn apply_overflow_shorthand(value: &str, style: &mut Style) {
  let mut parts = value.split_whitespace();
  let Some(first) = parts.next().and_then(parse_overflow) else {
    return;
  };
  let second = match parts.next() {
    Some(value) => match parse_overflow(value) {
      Some(parsed) => parsed,
      None => return,
    },
    None => first,
  };
  if parts.next().is_some() {
    return;
  }

  style.overflow = Some(first);
  style.overflow_x = Some(first);
  style.overflow_y = Some(second);
}

pub(crate) fn parse_visibility(value: &str) -> Option<Visibility> {
  value.parse().ok()
}

pub(crate) fn parse_flex_direction(value: &str) -> Option<FlexDirection> {
  value.parse().ok()
}

pub(crate) fn parse_flex_wrap(value: &str) -> Option<FlexWrap> {
  value.parse().ok()
}

pub(crate) fn parse_justify_content(value: &str) -> Option<JustifyContent> {
  value.parse().ok()
}

pub(crate) fn parse_align_items(value: &str) -> Option<AlignItems> {
  value.parse().ok()
}

pub(crate) fn parse_align_content(value: &str) -> Option<AlignContent> {
  value.parse().ok()
}

pub(crate) fn parse_align_self(value: &str) -> Option<AlignSelf> {
  value.parse().ok()
}

pub(crate) fn parse_justify_items(value: &str) -> Option<JustifyItems> {
  value.parse().ok()
}

pub(crate) fn parse_justify_self(value: &str) -> Option<JustifySelf> {
  value.parse().ok()
}

/// `grid-auto-flow: row | column | row dense | column dense | dense`.
/// `dense` packing isn't honoured at layout time yet; we accept the
/// keyword for cascade fidelity.
pub(crate) fn parse_grid_auto_flow(value: &str) -> Option<GridAutoFlow> {
  value.parse().ok()
}

/// Parse a single grid track size token: `auto`, `<flex>` (`1fr`), or
/// any `CssLength`. Returns `None` for unrecognized input.
pub(crate) fn parse_grid_track_size(value: &str) -> Option<GridTrackSize> {
  let trimmed = value.trim();
  if trimmed.eq_ignore_ascii_case("auto") {
    return Some(GridTrackSize::Auto);
  }
  if let Some(stripped) = strip_suffix_ci(trimmed, "fr") {
    if let Ok(n) = stripped.trim().parse::<f32>() {
      if n.is_finite() && n >= 0.0 {
        return Some(GridTrackSize::Fr(n));
      }
    }
  }
  parse_css_length(trimmed).map(GridTrackSize::Length)
}

/// Parse `grid-template-columns` / `grid-template-rows` as a flat list
/// of typed track sizes. Expands `repeat(<int>, <list>)` inline; leaves
/// `repeat(auto-fill, ...)` / `repeat(auto-fit, ...)` as a single `Auto`
/// track for now (still parses but doesn't auto-fit). Skips
/// `minmax()` / `fit-content()` (returns the inner length when
/// recognizable, otherwise `Auto`).
pub(crate) fn parse_grid_track_list(value: &str) -> Vec<GridTrackSize> {
  let trimmed = value.trim();
  if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
    return Vec::new();
  }
  let tokens = split_track_tokens(trimmed);
  let mut out: Vec<GridTrackSize> = Vec::new();
  for tok in tokens {
    if let Some(rest) = strip_function(&tok, "repeat") {
      // `repeat(<count>, <track-list>)`. Top-level comma split.
      let parts: Vec<&str> = split_top_level_commas(&rest);
      if parts.len() >= 2 {
        let count_tok = parts[0].trim();
        let inner = parts[1..].join(",");
        if let Ok(n) = count_tok.parse::<u32>() {
          let inner_list = parse_grid_track_list(&inner);
          for _ in 0..n {
            out.extend(inner_list.iter().cloned());
          }
          continue;
        }
        // `auto-fill` / `auto-fit` — single Auto placeholder
        // for now. Track-count resolution is a future job.
        if count_tok.eq_ignore_ascii_case("auto-fill") || count_tok.eq_ignore_ascii_case("auto-fit") {
          out.push(GridTrackSize::Auto);
          continue;
        }
      }
      continue;
    }
    if let Some(rest) = strip_function(&tok, "minmax") {
      // `minmax(<min>, <max>)` — for v1 we use the max as the
      // track size. Real two-bound clamping is deferred.
      let parts: Vec<&str> = split_top_level_commas(&rest);
      if let Some(max_tok) = parts.get(1) {
        if let Some(s) = parse_grid_track_size(max_tok.trim()) {
          out.push(s);
          continue;
        }
      }
      out.push(GridTrackSize::Auto);
      continue;
    }
    if let Some(rest) = strip_function(&tok, "fit-content") {
      // `fit-content(<length>)` — degrade to the inner length
      // for v1; the ceiling-by-content semantics are deferred.
      if let Some(s) = parse_grid_track_size(rest.trim()) {
        out.push(s);
        continue;
      }
      out.push(GridTrackSize::Auto);
      continue;
    }
    // Plain tokens.
    if let Some(size) = parse_grid_track_size(&tok) {
      out.push(size);
    }
  }
  out
}

/// Tokenize a track list into whitespace-separated entries while
/// keeping `repeat(...)`, `minmax(...)`, and `fit-content(...)` calls
/// intact.
fn split_track_tokens(s: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut buf = String::new();
  let mut depth: i32 = 0;
  for ch in s.chars() {
    match ch {
      '(' => {
        depth += 1;
        buf.push(ch);
      }
      ')' => {
        depth -= 1;
        buf.push(ch);
      }
      c if c.is_whitespace() && depth == 0 => {
        if !buf.is_empty() {
          out.push(std::mem::take(&mut buf));
        }
      }
      c => buf.push(c),
    }
  }
  if !buf.is_empty() {
    out.push(buf);
  }
  out
}

/// Split a string on commas at parenthesis depth 0. Used inside
/// `repeat(…)` / `minmax(…)` argument lists.
pub(crate) fn split_top_level_commas(s: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let bytes = s.as_bytes();
  let mut depth: i32 = 0;
  let mut start = 0;
  for (i, &b) in bytes.iter().enumerate() {
    match b {
      b'(' => depth += 1,
      b')' => depth -= 1,
      b',' if depth == 0 => {
        out.push(&s[start..i]);
        start = i + 1;
      }
      _ => {}
    }
  }
  out.push(&s[start..]);
  out
}

/// If `s` looks like `<name>(<inside>)`, return `<inside>` (trimmed).
/// Case-insensitive on the function name. Returns `None` otherwise.
pub(crate) fn strip_function(s: &str, name: &str) -> Option<String> {
  let trimmed = s.trim();
  let lower = trimmed.to_ascii_lowercase();
  let prefix = format!("{name}(");
  if !lower.starts_with(&prefix) || !trimmed.ends_with(')') {
    return None;
  }
  let inside = &trimmed[prefix.len()..trimmed.len() - 1];
  Some(inside.to_string())
}

/// Strip a case-insensitive suffix; returns the prefix when matched.
fn strip_suffix_ci<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
  if s.len() < suffix.len() {
    return None;
  }
  let cut = s.len() - suffix.len();
  if s[cut..].eq_ignore_ascii_case(suffix) {
    Some(&s[..cut])
  } else {
    None
  }
}

/// Parse one end of a `grid-row` / `grid-column` placement.
/// Recognized: `auto`, a positive integer line number, `span <n>`.
/// Negative line numbers and named lines fall through to `None`.
pub(crate) fn parse_grid_line(value: &str) -> Option<GridLine> {
  value.parse().ok()
}

#[derive(Copy, Clone)]
pub(crate) enum GridAxis {
  Column,
  Row,
}

/// Expand `grid-column` / `grid-row` shorthand into the start / end
/// longhands. Accepts:
/// - `<line>` → start=line, end=auto
/// - `<line> / <line>` → start, end
/// - `span <n> / <line>` (and the reverse), etc.
pub(crate) fn apply_grid_axis_shorthand(value: &str, style: &mut Style, axis: GridAxis) {
  // Round-trip the raw value for cascade introspection.
  match axis {
    GridAxis::Column => style.grid_column = Some(ArcStr::from(value)),
    GridAxis::Row => style.grid_row = Some(ArcStr::from(value)),
  }
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return;
  }
  let parts: Vec<&str> = trimmed.split('/').map(|p| p.trim()).collect();
  let (start_tok, end_tok) = match parts.len() {
    1 => (parts[0], "auto"),
    _ => (parts[0], parts[1]),
  };
  let start = parse_grid_line(start_tok).unwrap_or(GridLine::Auto);
  let end = parse_grid_line(end_tok).unwrap_or(GridLine::Auto);
  match axis {
    GridAxis::Column => {
      style.grid_column_start = Some(start);
      style.grid_column_end = Some(end);
    }
    GridAxis::Row => {
      style.grid_row_start = Some(start);
      style.grid_row_end = Some(end);
    }
  }
}

/// Expand the `flex` shorthand into the three longhands per CSS-Flex-1
/// §7.2 (`flex` shorthand).
///
/// Recognized forms:
/// - `none`    → 0 0 auto
/// - `auto`    → 1 1 auto
/// - `initial` → 0 1 auto
/// - `<number>`            → grow=<n>, shrink=1, basis=0%
/// - `<basis>`             → grow=1, shrink=1, basis=<basis>
/// - `<grow> <shrink>`     → grow, shrink, basis=0%
/// - `<grow> <basis>`      → grow, shrink=1, basis
/// - `<grow> <shrink> <basis>` (full form)
///
/// Token classification:
/// - A bare positive number (`1`, `0.5`) is a flex factor.
/// - Anything else (`100px`, `30%`, `auto`) is treated as basis.
pub(crate) fn apply_flex_shorthand(value: &str, style: &mut Style) {
  style.flex = Some(ArcStr::from(value));
  let trimmed = value.trim();
  let lower = trimmed.to_ascii_lowercase();
  match lower.as_str() {
    "none" => {
      style.flex_grow = Some(0.0);
      style.flex_shrink = Some(0.0);
      style.flex_basis = Some(CssLength::Auto);
      return;
    }
    "auto" => {
      style.flex_grow = Some(1.0);
      style.flex_shrink = Some(1.0);
      style.flex_basis = Some(CssLength::Auto);
      return;
    }
    "initial" => {
      style.flex_grow = Some(0.0);
      style.flex_shrink = Some(1.0);
      style.flex_basis = Some(CssLength::Auto);
      return;
    }
    _ => {}
  }

  let tokens: Vec<&str> = trimmed.split_whitespace().collect();
  let is_number = |t: &str| t.parse::<f32>().is_ok();
  let mut grow: Option<f32> = None;
  let mut shrink: Option<f32> = None;
  let mut basis: Option<CssLength> = None;

  match tokens.len() {
    0 => return,
    1 => {
      let t = tokens[0];
      if is_number(t) {
        grow = t.parse().ok();
        shrink = Some(1.0);
        basis = Some(CssLength::Percent(0.0));
      } else if let Some(b) = parse_css_length(t) {
        grow = Some(1.0);
        shrink = Some(1.0);
        basis = Some(b);
      }
    }
    2 => {
      let (a, b) = (tokens[0], tokens[1]);
      if is_number(a) && is_number(b) {
        grow = a.parse().ok();
        shrink = b.parse().ok();
        basis = Some(CssLength::Percent(0.0));
      } else if is_number(a) {
        grow = a.parse().ok();
        shrink = Some(1.0);
        basis = parse_css_length(b);
      }
    }
    _ => {
      // Three (or more — extra ignored) tokens: grow shrink basis.
      grow = tokens[0].parse().ok();
      shrink = tokens[1].parse().ok();
      basis = parse_css_length(tokens[2]);
    }
  }

  if let Some(g) = grow {
    style.flex_grow = Some(g);
  }
  if let Some(s) = shrink {
    style.flex_shrink = Some(s);
  }
  if let Some(b) = basis {
    style.flex_basis = Some(b);
  }
}

pub(crate) fn parse_cursor(value: &str) -> Option<Cursor> {
  value.parse().ok()
}

pub(crate) fn parse_pointer_events(value: &str) -> Option<PointerEvents> {
  value.parse().ok()
}

pub(crate) fn parse_user_select(value: &str) -> Option<UserSelect> {
  value.parse().ok()
}

pub(crate) fn parse_box_sizing(value: &str) -> Option<BoxSizing> {
  value.parse().ok()
}

pub(crate) fn parse_list_style_type(value: &str) -> Option<ListStyleType> {
  match value.trim().to_ascii_lowercase().as_str() {
    "disc" => Some(ListStyleType::Disc),
    "circle" => Some(ListStyleType::Circle),
    "square" => Some(ListStyleType::Square),
    "decimal" => Some(ListStyleType::Decimal),
    "decimal-leading-zero" => Some(ListStyleType::DecimalLeadingZero),
    "lower-alpha" | "lower-latin" => Some(ListStyleType::LowerAlpha),
    "upper-alpha" | "upper-latin" => Some(ListStyleType::UpperAlpha),
    "lower-roman" => Some(ListStyleType::LowerRoman),
    "upper-roman" => Some(ListStyleType::UpperRoman),
    "none" => Some(ListStyleType::None),
    _ => None,
  }
}

pub(crate) fn parse_list_style_position(value: &str) -> Option<ListStylePosition> {
  match value.trim().to_ascii_lowercase().as_str() {
    "inside" => Some(ListStylePosition::Inside),
    "outside" => Some(ListStylePosition::Outside),
    _ => None,
  }
}

pub(crate) fn parse_css_content(value: &str) -> Option<CssContent> {
  let v = value.trim();
  if v.eq_ignore_ascii_case("none") {
    return Some(CssContent::None);
  }
  if v.eq_ignore_ascii_case("normal") {
    return Some(CssContent::Normal);
  }
  if (v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')) {
    let inner = &v[1..v.len() - 1];
    return Some(CssContent::String(ArcStr::from(inner)));
  }
  None
}