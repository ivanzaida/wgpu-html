use std::time::Instant;

use crate::text_selection::EditCursor;

#[derive(Debug, Clone)]
pub struct FormControlState {
  pub value: String,
  pub edit_cursor: EditCursor,
  pub scroll_x: f32,
  pub caret_blink_epoch: Instant,
  pub focus_value_snapshot: String,
}

impl FormControlState {
  pub fn new(initial_value: &str) -> Self {
    let len = initial_value.len();
    Self {
      value: initial_value.to_string(),
      edit_cursor: EditCursor::collapsed(len),
      scroll_x: 0.0,
      caret_blink_epoch: Instant::now(),
      focus_value_snapshot: initial_value.to_string(),
    }
  }

  pub fn reset_blink(&mut self) {
    self.caret_blink_epoch = Instant::now();
  }

  pub fn caret_visible(&self) -> bool {
    (self.caret_blink_epoch.elapsed().as_millis() % 1000) < 500
  }

  pub fn value_changed_since_focus(&self) -> bool {
    self.value != self.focus_value_snapshot
  }

  pub fn selected_text(&self) -> Option<String> {
    if !self.edit_cursor.has_selection() {
      return None;
    }
    let (start, end) = self.edit_cursor.selection_range();
    Some(self.value[start..end].to_string())
  }
}
