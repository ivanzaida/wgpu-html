use lui_core::{TextCursor, TextSelection};
use lui_glyph::{ShapedRun, TextContext, TextStyle};
use lui_layout::LayoutBox;

use crate::paint::style;

pub fn select_word(
  cursor: &TextCursor,
  tree_root: &LayoutBox<'_>,
  text_ctx: &mut TextContext,
) -> Option<TextSelection> {
  let (run, _) = shape_at_cursor(cursor, tree_root, text_ctx)?;
  let (start, end) = word_boundaries(&run, cursor.char_index)?;
  Some(TextSelection {
    anchor: TextCursor { path: cursor.path.clone(), char_index: start },
    focus: TextCursor { path: cursor.path.clone(), char_index: end },
  })
}

pub fn select_line(
  cursor: &TextCursor,
  tree_root: &LayoutBox<'_>,
  text_ctx: &mut TextContext,
) -> Option<TextSelection> {
  let (run, _) = shape_at_cursor(cursor, tree_root, text_ctx)?;
  let (start, end) = line_boundaries(&run, cursor.char_index);
  Some(TextSelection {
    anchor: TextCursor { path: cursor.path.clone(), char_index: start },
    focus: TextCursor { path: cursor.path.clone(), char_index: end },
  })
}

pub fn selected_text(
  selection: &TextSelection,
  tree_root: &LayoutBox<'_>,
) -> Option<String> {
  if selection.is_collapsed() {
    return None;
  }
  let (start, end) = selection.ordered();
  let mut out = String::new();
  let mut path = Vec::new();
  let mut prev_parent: Option<Vec<usize>> = None;
  collect_selected_text(tree_root, &mut path, start, end, &mut prev_parent, &mut out);
  if out.is_empty() { None } else { Some(out) }
}

fn collect_selected_text(
  b: &LayoutBox<'_>,
  path: &mut Vec<usize>,
  start: &TextCursor,
  end: &TextCursor,
  prev_parent: &mut Option<Vec<usize>>,
  out: &mut String,
) {
  if path.as_slice() > end.path.as_slice() {
    return;
  }

  if b.node.element().is_text() {
    if let lui_core::HtmlElement::Text(s) = b.node.element() {
      let ws = style::css_str(b.style.white_space);
      let collapsed;
      let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
        collapsed = lui_layout::flow::collapse_whitespace(s.as_ref());
        collapsed
      } else {
        s.as_ref().to_string()
      };
      if text.is_empty() {
        return;
      }

      let cmp_start = path.as_slice().cmp(start.path.as_slice());
      let cmp_end = path.as_slice().cmp(end.path.as_slice());

      if cmp_start == std::cmp::Ordering::Less && cmp_end == std::cmp::Ordering::Greater {
        return;
      }

      let chars: Vec<char> = text.chars().collect();
      let from = if cmp_start == std::cmp::Ordering::Equal {
        start.char_index.min(chars.len())
      } else if cmp_start == std::cmp::Ordering::Greater {
        0
      } else {
        return;
      };
      let to = if cmp_end == std::cmp::Ordering::Equal {
        end.char_index.min(chars.len())
      } else {
        chars.len()
      };

      if to > from {
        let parent = &path[..path.len().saturating_sub(1)];
        if let Some(pp) = prev_parent.as_ref() {
          if pp.as_slice() != parent {
            out.push('\n');
          }
        }
        *prev_parent = Some(parent.to_vec());
        let fragment: String = chars[from..to].iter().collect();
        out.push_str(&fragment);
      }
    }
    return;
  }

  for (i, child) in b.children.iter().enumerate() {
    path.push(i);
    collect_selected_text(child, path, start, end, prev_parent, out);
    path.pop();
  }
}

pub fn first_text_cursor(tree_root: &LayoutBox<'_>) -> Option<TextCursor> {
  let mut path = Vec::new();
  first_text_cursor_inner(tree_root, &mut path)
}

fn first_text_cursor_inner(b: &LayoutBox<'_>, path: &mut Vec<usize>) -> Option<TextCursor> {
  if b.node.element().is_text() {
    if let lui_core::HtmlElement::Text(s) = b.node.element() {
      if !s.is_empty() {
        return Some(TextCursor { path: path.clone(), char_index: 0 });
      }
    }
  }
  for (i, child) in b.children.iter().enumerate() {
    path.push(i);
    if let Some(c) = first_text_cursor_inner(child, path) {
      return Some(c);
    }
    path.pop();
  }
  None
}

pub fn last_text_cursor(
  tree_root: &LayoutBox<'_>,
) -> Option<TextCursor> {
  let mut path = Vec::new();
  last_text_cursor_inner(tree_root, &mut path)
}

fn last_text_cursor_inner(b: &LayoutBox<'_>, path: &mut Vec<usize>) -> Option<TextCursor> {
  for i in (0..b.children.len()).rev() {
    path.push(i);
    if let Some(c) = last_text_cursor_inner(&b.children[i], path) {
      return Some(c);
    }
    path.pop();
  }
  if b.node.element().is_text() {
    if let lui_core::HtmlElement::Text(s) = b.node.element() {
      let ws = style::css_str(b.style.white_space);
      let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
        lui_layout::flow::collapse_whitespace(s.as_ref())
      } else {
        s.as_ref().to_string()
      };
      let char_count = text.chars().count();
      if char_count > 0 {
        return Some(TextCursor { path: path.clone(), char_index: char_count });
      }
    }
  }
  None
}

fn shape_at_cursor<'a>(
  cursor: &TextCursor,
  tree_root: &LayoutBox<'a>,
  text_ctx: &mut TextContext,
) -> Option<(ShapedRun, String)> {
  let b = box_at_path(tree_root, &cursor.path)?;
  let raw_text = match b.node.element() {
    lui_core::HtmlElement::Text(s) => s.as_ref(),
    _ => return None,
  };
  let ws = style::css_str(b.style.white_space);
  let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
    lui_layout::flow::collapse_whitespace(raw_text)
  } else {
    raw_text.to_string()
  };
  if text.is_empty() {
    return None;
  }
  let font_size = style::css_font_size(b.style.font_size);
  let line_height = match b.style.line_height {
    Some(lui_core::CssValue::Dimension { value, unit: lui_core::CssUnit::Px }) => *value as f32,
    Some(lui_core::CssValue::Number(n)) => *n as f32 * font_size,
    _ => font_size * 1.2,
  };
  let weight = match b.style.font_weight {
    Some(lui_core::CssValue::Number(n)) => (*n as u16).min(1000),
    _ => 400,
  };
  let font_family = style::css_str(b.style.font_family);
  let ts = TextStyle {
    font_size,
    line_height,
    font_family,
    weight,
    ..Default::default()
  };
  let run = text_ctx.shape(&text, &ts);
  Some((run, text))
}

fn box_at_path<'a>(root: &'a LayoutBox<'a>, path: &[usize]) -> Option<&'a LayoutBox<'a>> {
  let mut current = root;
  for &idx in path {
    current = current.children.get(idx)?;
  }
  Some(current)
}

#[derive(Clone, Copy, PartialEq)]
enum TokenKind {
  Word,
  Whitespace,
  Punctuation,
}

fn token_kind(c: char) -> TokenKind {
  if c.is_alphanumeric() || c == '_' {
    TokenKind::Word
  } else if c.is_whitespace() {
    TokenKind::Whitespace
  } else {
    TokenKind::Punctuation
  }
}

fn word_boundaries(run: &ShapedRun, char_index: usize) -> Option<(usize, usize)> {
  let chars: Vec<char> = run.text.chars().collect();
  if chars.is_empty() {
    return None;
  }
  let idx = char_index.min(chars.len().saturating_sub(1));
  let kind = token_kind(chars[idx]);

  let mut start = idx;
  while start > 0 && token_kind(chars[start - 1]) == kind {
    start -= 1;
  }
  let mut end = idx + 1;
  while end < chars.len() && token_kind(chars[end]) == kind {
    end += 1;
  }
  Some((start, end))
}

fn line_boundaries(run: &ShapedRun, char_index: usize) -> (usize, usize) {
  if run.lines.is_empty() {
    return (0, run.char_count());
  }
  let glyph_idx = run.char_to_glyph_index(char_index);
  for line in &run.lines {
    if glyph_idx >= line.glyph_start && glyph_idx < line.glyph_end {
      let start_char = run.glyph_to_char_index(line.glyph_start);
      let end_char = if line.glyph_end > 0 {
        run.glyph_to_char_index(line.glyph_end - 1) + 1
      } else {
        start_char
      };
      return (start_char, end_char);
    }
  }
  if let Some(last) = run.lines.last() {
    let start_char = run.glyph_to_char_index(last.glyph_start);
    let end_char = if last.glyph_end > 0 {
      run.glyph_to_char_index(last.glyph_end - 1) + 1
    } else {
      start_char
    };
    return (start_char, end_char);
  }
  (0, run.char_count())
}
