//! Pure text-editing operations for form controls.
//!
//! Every function takes the current value + cursor and returns the
//! new value + cursor. No side effects; no tree access. This keeps
//! the logic unit-testable without a full `Tree`.

use crate::EditCursor;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Byte offset of the start of the previous char boundary (or 0).
fn prev_char(s: &str, pos: usize) -> usize {
  let pos = pos.min(s.len());
  if pos == 0 {
    return 0;
  }
  let mut i = pos - 1;
  while i > 0 && !s.is_char_boundary(i) {
    i -= 1;
  }
  i
}

/// Byte offset of the start of the next char boundary (or len).
fn next_char(s: &str, pos: usize) -> usize {
  let pos = pos.min(s.len());
  if pos >= s.len() {
    return s.len();
  }
  let mut i = pos + 1;
  while i < s.len() && !s.is_char_boundary(i) {
    i += 1;
  }
  i
}

// ── Insertions ───────────────────────────────────────────────────────────────

/// Insert `text` at the cursor, replacing any selection.
pub fn insert_text(value: &str, cursor: &EditCursor, text: &str) -> (String, EditCursor) {
  let (start, end) = cursor.selection_range();
  let start = start.min(value.len());
  let end = end.min(value.len());
  let mut result = String::with_capacity(value.len() + text.len());
  result.push_str(&value[..start]);
  result.push_str(text);
  result.push_str(&value[end..]);
  let new_pos = start + text.len();
  (result, EditCursor::collapsed(new_pos))
}

/// Insert a line break (`\n`). For `<textarea>` only.
pub fn insert_line_break(value: &str, cursor: &EditCursor) -> (String, EditCursor) {
  insert_text(value, cursor, "\n")
}

// ── Deletions ────────────────────────────────────────────────────────────────

/// Delete the selected text without inserting anything.
pub fn delete_selection(value: &str, cursor: &EditCursor) -> (String, EditCursor) {
  if !cursor.has_selection() {
    return (value.to_string(), cursor.clone());
  }
  let (start, end) = cursor.selection_range();
  let start = start.min(value.len());
  let end = end.min(value.len());
  let mut result = String::with_capacity(value.len() - (end - start));
  result.push_str(&value[..start]);
  result.push_str(&value[end..]);
  (result, EditCursor::collapsed(start))
}

/// Delete one character backward (Backspace).
pub fn delete_backward(value: &str, cursor: &EditCursor) -> (String, EditCursor) {
  if cursor.has_selection() {
    return delete_selection(value, cursor);
  }
  let pos = cursor.cursor.min(value.len());
  if pos == 0 {
    return (value.to_string(), cursor.clone());
  }
  let prev = prev_char(value, pos);
  let mut result = String::with_capacity(value.len() - (pos - prev));
  result.push_str(&value[..prev]);
  result.push_str(&value[pos..]);
  (result, EditCursor::collapsed(prev))
}

/// Delete one character forward (Delete key).
pub fn delete_forward(value: &str, cursor: &EditCursor) -> (String, EditCursor) {
  if cursor.has_selection() {
    return delete_selection(value, cursor);
  }
  let pos = cursor.cursor.min(value.len());
  if pos >= value.len() {
    return (value.to_string(), cursor.clone());
  }
  let next = next_char(value, pos);
  let mut result = String::with_capacity(value.len() - (next - pos));
  result.push_str(&value[..pos]);
  result.push_str(&value[next..]);
  (result, EditCursor::collapsed(pos))
}

// ── Cursor movement ──────────────────────────────────────────────────────────

/// Move cursor left by one character.
pub fn move_left(value: &str, cursor: &EditCursor, extend_selection: bool) -> EditCursor {
  if !extend_selection && cursor.has_selection() {
    let (start, _) = cursor.selection_range();
    return EditCursor::collapsed(start);
  }
  let pos = cursor.cursor.min(value.len());
  let new_pos = prev_char(value, pos);
  if extend_selection {
    EditCursor {
      cursor: new_pos,
      selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
    }
  } else {
    EditCursor::collapsed(new_pos)
  }
}

/// Move cursor right by one character.
pub fn move_right(value: &str, cursor: &EditCursor, extend_selection: bool) -> EditCursor {
  if !extend_selection && cursor.has_selection() {
    let (_, end) = cursor.selection_range();
    return EditCursor::collapsed(end);
  }
  let pos = cursor.cursor.min(value.len());
  let new_pos = next_char(value, pos);
  if extend_selection {
    EditCursor {
      cursor: new_pos,
      selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
    }
  } else {
    EditCursor::collapsed(new_pos)
  }
}

/// Move cursor to start of value (Home).
pub fn move_home(value: &str, cursor: &EditCursor, extend_selection: bool) -> EditCursor {
  let pos = cursor.cursor.min(value.len());
  // For multi-line: move to start of current line.
  let line_start = value[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
  if extend_selection {
    EditCursor {
      cursor: line_start,
      selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
    }
  } else {
    EditCursor::collapsed(line_start)
  }
}

/// Move cursor to end of value (End).
pub fn move_end(value: &str, cursor: &EditCursor, extend_selection: bool) -> EditCursor {
  let pos = cursor.cursor.min(value.len());
  // For multi-line: move to end of current line.
  let line_end = value[pos..].find('\n').map(|i| pos + i).unwrap_or(value.len());
  if extend_selection {
    EditCursor {
      cursor: line_end,
      selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
    }
  } else {
    EditCursor::collapsed(line_end)
  }
}

/// Select all text (Ctrl+A).
pub fn select_all(value: &str) -> EditCursor {
  EditCursor {
    cursor: value.len(),
    selection_anchor: Some(0),
  }
}

// ── Multi-line (textarea) ────────────────────────────────────────────────────

/// Move cursor up one line. Scans for `\n` boundaries.
pub fn move_up(value: &str, cursor: &EditCursor, extend_selection: bool) -> EditCursor {
  let pos = cursor.cursor.min(value.len());
  let line_start = value[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
  if line_start == 0 {
    // Already on first line — move to start.
    let new_pos = 0;
    return if extend_selection {
      EditCursor {
        cursor: new_pos,
        selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
      }
    } else {
      EditCursor::collapsed(new_pos)
    };
  }
  let col = pos - line_start;
  let prev_line_end = line_start - 1; // the \n
  let prev_line_start = value[..prev_line_end].rfind('\n').map(|i| i + 1).unwrap_or(0);
  let prev_line_len = prev_line_end - prev_line_start;
  let new_pos = prev_line_start + col.min(prev_line_len);
  if extend_selection {
    EditCursor {
      cursor: new_pos,
      selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
    }
  } else {
    EditCursor::collapsed(new_pos)
  }
}

/// Move cursor down one line. Scans for `\n` boundaries.
pub fn move_down(value: &str, cursor: &EditCursor, extend_selection: bool) -> EditCursor {
  let pos = cursor.cursor.min(value.len());
  let line_start = value[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
  let col = pos - line_start;
  let Some(newline_pos) = value[pos..].find('\n') else {
    // Already on last line — move to end.
    let new_pos = value.len();
    return if extend_selection {
      EditCursor {
        cursor: new_pos,
        selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
      }
    } else {
      EditCursor::collapsed(new_pos)
    };
  };
  let next_line_start = pos + newline_pos + 1;
  let next_line_end = value[next_line_start..]
    .find('\n')
    .map(|i| next_line_start + i)
    .unwrap_or(value.len());
  let next_line_len = next_line_end - next_line_start;
  let new_pos = next_line_start + col.min(next_line_len);
  if extend_selection {
    EditCursor {
      cursor: new_pos,
      selection_anchor: Some(cursor.selection_anchor.unwrap_or(pos)),
    }
  } else {
    EditCursor::collapsed(new_pos)
  }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn insert_at_end() {
    let (v, c) = insert_text("abc", &EditCursor::collapsed(3), "d");
    assert_eq!(v, "abcd");
    assert_eq!(c.cursor, 4);
    assert!(!c.has_selection());
  }

  #[test]
  fn insert_at_start() {
    let (v, c) = insert_text("abc", &EditCursor::collapsed(0), "x");
    assert_eq!(v, "xabc");
    assert_eq!(c.cursor, 1);
  }

  #[test]
  fn insert_at_middle() {
    let (v, c) = insert_text("ac", &EditCursor::collapsed(1), "b");
    assert_eq!(v, "abc");
    assert_eq!(c.cursor, 2);
  }

  #[test]
  fn insert_replaces_selection() {
    let cursor = EditCursor {
      cursor: 3,
      selection_anchor: Some(1),
    };
    let (v, c) = insert_text("abcd", &cursor, "XY");
    assert_eq!(v, "aXYd");
    assert_eq!(c.cursor, 3);
    assert!(!c.has_selection());
  }

  #[test]
  fn backspace_at_start_is_noop() {
    let (v, c) = delete_backward("abc", &EditCursor::collapsed(0));
    assert_eq!(v, "abc");
    assert_eq!(c.cursor, 0);
  }

  #[test]
  fn backspace_deletes_one_char() {
    let (v, c) = delete_backward("abc", &EditCursor::collapsed(2));
    assert_eq!(v, "ac");
    assert_eq!(c.cursor, 1);
  }

  #[test]
  fn backspace_deletes_selection() {
    let cursor = EditCursor {
      cursor: 3,
      selection_anchor: Some(1),
    };
    let (v, c) = delete_backward("abcd", &cursor);
    assert_eq!(v, "ad");
    assert_eq!(c.cursor, 1);
  }

  #[test]
  fn delete_forward_at_end_is_noop() {
    let (v, c) = delete_forward("abc", &EditCursor::collapsed(3));
    assert_eq!(v, "abc");
    assert_eq!(c.cursor, 3);
  }

  #[test]
  fn delete_forward_deletes_one_char() {
    let (v, c) = delete_forward("abc", &EditCursor::collapsed(1));
    assert_eq!(v, "ac");
    assert_eq!(c.cursor, 1);
  }

  #[test]
  fn move_left_collapses_selection() {
    let cursor = EditCursor {
      cursor: 3,
      selection_anchor: Some(1),
    };
    let c = move_left("abcd", &cursor, false);
    assert_eq!(c.cursor, 1);
    assert!(!c.has_selection());
  }

  #[test]
  fn move_left_with_shift_extends_selection() {
    let c = move_left("abc", &EditCursor::collapsed(2), true);
    assert_eq!(c.cursor, 1);
    assert_eq!(c.selection_anchor, Some(2));
  }

  #[test]
  fn move_right_collapses_selection() {
    let cursor = EditCursor {
      cursor: 1,
      selection_anchor: Some(3),
    };
    let c = move_right("abcd", &cursor, false);
    assert_eq!(c.cursor, 3);
    assert!(!c.has_selection());
  }

  #[test]
  fn home_goes_to_line_start() {
    let c = move_home("abc\ndef", &EditCursor::collapsed(5), false);
    assert_eq!(c.cursor, 4); // start of "def"
  }

  #[test]
  fn end_goes_to_line_end() {
    let c = move_end("abc\ndef", &EditCursor::collapsed(1), false);
    assert_eq!(c.cursor, 3); // end of "abc"
  }

  #[test]
  fn select_all_selects_everything() {
    let c = select_all("hello");
    assert_eq!(c.cursor, 5);
    assert_eq!(c.selection_anchor, Some(0));
  }

  #[test]
  fn move_up_on_first_line_goes_to_start() {
    let c = move_up("abc\ndef", &EditCursor::collapsed(2), false);
    assert_eq!(c.cursor, 0);
  }

  #[test]
  fn move_up_preserves_column() {
    let c = move_up("abc\ndef", &EditCursor::collapsed(5), false);
    assert_eq!(c.cursor, 1); // col 1 of "abc"
  }

  #[test]
  fn move_up_clamps_column() {
    let c = move_up("ab\ncdef", &EditCursor::collapsed(7), false);
    assert_eq!(c.cursor, 2); // col 2 of "ab" (clamped from col 4)
  }

  #[test]
  fn move_down_on_last_line_goes_to_end() {
    let c = move_down("abc\ndef", &EditCursor::collapsed(5), false);
    assert_eq!(c.cursor, 7);
  }

  #[test]
  fn move_down_preserves_column() {
    let c = move_down("abc\ndef", &EditCursor::collapsed(1), false);
    assert_eq!(c.cursor, 5); // col 1 of "def"
  }

  #[test]
  fn insert_line_break_adds_newline() {
    let (v, c) = insert_line_break("abc", &EditCursor::collapsed(1));
    assert_eq!(v, "a\nbc");
    assert_eq!(c.cursor, 2);
  }

  #[test]
  fn multibyte_backspace() {
    // "aéb" — é is 2 bytes (U+00E9)
    let (v, c) = delete_backward("aéb", &EditCursor::collapsed(3));
    assert_eq!(v, "ab");
    assert_eq!(c.cursor, 1);
  }

  #[test]
  fn multibyte_move_right() {
    let c = move_right("aéb", &EditCursor::collapsed(1), false);
    assert_eq!(c.cursor, 3); // skip the 2-byte é
  }
}
