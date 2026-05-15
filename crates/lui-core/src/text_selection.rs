#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCursor {
  pub path: Vec<usize>,
  pub char_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditCursor {
  pub cursor: usize,
  pub selection_anchor: Option<usize>,
}

impl EditCursor {
  pub fn collapsed(pos: usize) -> Self {
    Self {
      cursor: pos,
      selection_anchor: None,
    }
  }

  pub fn has_selection(&self) -> bool {
    self.selection_anchor.is_some_and(|a| a != self.cursor)
  }

  pub fn selection_range(&self) -> (usize, usize) {
    let anchor = self.selection_anchor.unwrap_or(self.cursor);
    (anchor.min(self.cursor), anchor.max(self.cursor))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSelection {
  pub anchor: TextCursor,
  pub focus: TextCursor,
}

impl TextSelection {
  pub fn is_collapsed(&self) -> bool {
    self.anchor == self.focus
  }

  pub fn ordered(&self) -> (&TextCursor, &TextCursor) {
    if cursor_less(&self.anchor, &self.focus) {
      (&self.anchor, &self.focus)
    } else {
      (&self.focus, &self.anchor)
    }
  }
}

pub fn cursor_less(a: &TextCursor, b: &TextCursor) -> bool {
  match a.path.cmp(&b.path) {
    std::cmp::Ordering::Less => true,
    std::cmp::Ordering::Greater => false,
    std::cmp::Ordering::Equal => a.char_index < b.char_index,
  }
}

#[derive(Debug, Clone, Copy)]
pub struct SelectionColors {
  pub background: [f32; 4],
  pub foreground: [f32; 4],
}

impl Default for SelectionColors {
  fn default() -> Self {
    Self {
      background: [0.23, 0.51, 0.96, 0.45],
      foreground: [1.0, 1.0, 1.0, 1.0],
    }
  }
}
