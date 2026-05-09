pub fn is_leap_year(y: i32) -> bool {
  (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i32, m: u8) -> u8 {
  match m {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 => if is_leap_year(y) { 29 } else { 28 },
    _ => 30,
  }
}

/// Day of week for a given date (0=Monday .. 6=Sunday).
/// Uses Tomohiko Sakamoto's algorithm.
pub fn day_of_week(y: i32, m: u8, d: u8) -> u8 {
  let t = [0i32, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
  let y = if m < 3 { y - 1 } else { y };
  let dow = (y + y / 4 - y / 100 + y / 400 + t[(m - 1) as usize] + d as i32) % 7;
  // Result: 0=Sunday, 1=Monday, ..., 6=Saturday
  // Convert to 0=Monday, ..., 6=Sunday
  ((dow + 6) % 7) as u8
}

/// Parse "YYYY-MM-DD" into (year, month, day). Returns None on invalid input.
pub fn parse_date(s: &str) -> Option<(i32, u8, u8)> {
  let s = s.trim();
  let mut parts = s.splitn(3, '-');
  let y: i32 = parts.next()?.parse().ok()?;
  let m: u8 = parts.next()?.parse().ok()?;
  let d: u8 = parts.next()?.parse().ok()?;
  if m < 1 || m > 12 || d < 1 || d > days_in_month(y, m) {
    return None;
  }
  Some((y, m, d))
}

/// Parse "YYYY-MM-DDThh:mm" or "YYYY-MM-DDThh:mm:ss" into (year, month, day, hour, minute).
pub fn parse_datetime_local(s: &str) -> Option<(i32, u8, u8, u8, u8)> {
  let s = s.trim();
  let (date_part, time_part) = s.split_once('T')?;
  let (y, m, d) = parse_date(date_part)?;
  let mut time_parts = time_part.splitn(3, ':');
  let hour: u8 = time_parts.next()?.parse().ok()?;
  let minute: u8 = time_parts.next()?.parse().ok()?;
  if hour > 23 || minute > 59 { return None; }
  Some((y, m, d, hour, minute))
}

pub fn format_date(y: i32, m: u8, d: u8) -> String {
  format!("{y:04}-{m:02}-{d:02}")
}

pub fn format_datetime_local(y: i32, m: u8, d: u8, hour: u8, min: u8) -> String {
  format!("{y:04}-{m:02}-{d:02}T{hour:02}:{min:02}")
}

/// Describes a segment in a date pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateSegmentKind {
  Day,
  Month,
  Year,
  Hour,
  Minute,
  Separator,
}

/// A segment within the formatted date string.
#[derive(Debug, Clone, Copy)]
pub struct DateSegment {
  pub kind: DateSegmentKind,
  pub byte_start: usize,
  pub byte_len: usize,
}

/// Parse a date pattern into segments.
/// E.g. `"dd/mm/yyyy"` → [Day(0,2), Sep(2,1), Month(3,2), Sep(5,1), Year(6,4)]
pub fn parse_pattern_segments(pattern: &str) -> Vec<DateSegment> {
  let mut segs = Vec::new();
  let bytes = pattern.as_bytes();
  let mut i = 0;
  let mut pos = 0;
  while i < bytes.len() {
    if i + 4 <= bytes.len() && &bytes[i..i + 4] == b"yyyy" {
      segs.push(DateSegment { kind: DateSegmentKind::Year, byte_start: pos, byte_len: 4 });
      pos += 4; i += 4;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"dd" {
      segs.push(DateSegment { kind: DateSegmentKind::Day, byte_start: pos, byte_len: 2 });
      pos += 2; i += 2;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"mm" {
      segs.push(DateSegment { kind: DateSegmentKind::Month, byte_start: pos, byte_len: 2 });
      pos += 2; i += 2;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"HH" {
      segs.push(DateSegment { kind: DateSegmentKind::Hour, byte_start: pos, byte_len: 2 });
      pos += 2; i += 2;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"MM" {
      segs.push(DateSegment { kind: DateSegmentKind::Minute, byte_start: pos, byte_len: 2 });
      pos += 2; i += 2;
    } else {
      segs.push(DateSegment { kind: DateSegmentKind::Separator, byte_start: pos, byte_len: 1 });
      pos += 1; i += 1;
    }
  }
  segs
}

/// Parse a locale-formatted date string using the given pattern.
/// Returns (year, month, day) or None.
pub fn parse_formatted_date(text: &str, pattern: &str) -> Option<(i32, u8, u8)> {
  let segs = parse_pattern_segments(pattern);
  let mut y: Option<i32> = None;
  let mut m: Option<u8> = None;
  let mut d: Option<u8> = None;
  for seg in &segs {
    if seg.kind == DateSegmentKind::Separator { continue; }
    let s = text.get(seg.byte_start..seg.byte_start + seg.byte_len)?;
    match seg.kind {
      DateSegmentKind::Year => y = s.parse().ok(),
      DateSegmentKind::Month => m = s.parse().ok(),
      DateSegmentKind::Day => d = s.parse().ok(),
      _ => {}
    }
  }
  let (y, m, d) = (y?, m?, d?);
  if m < 1 || m > 12 || d < 1 || d > days_in_month(y, m) { return None; }
  Some((y, m, d))
}

/// Parse a locale-formatted datetime string using the given pattern
/// (e.g. `"mm/dd/yyyy HH:MM"`). Returns (year, month, day, hour, minute).
pub fn parse_formatted_datetime(text: &str, pattern: &str) -> Option<(i32, u8, u8, u8, u8)> {
  let segs = parse_pattern_segments(pattern);
  let mut y: Option<i32> = None;
  let mut m: Option<u8> = None;
  let mut d: Option<u8> = None;
  let mut hour: Option<u8> = None;
  let mut minute: Option<u8> = None;
  for seg in &segs {
    if seg.kind == DateSegmentKind::Separator { continue; }
    let s = text.get(seg.byte_start..seg.byte_start + seg.byte_len)?;
    match seg.kind {
      DateSegmentKind::Year => y = s.parse().ok(),
      DateSegmentKind::Month => m = s.parse().ok(),
      DateSegmentKind::Day => d = s.parse().ok(),
      DateSegmentKind::Hour => hour = s.parse().ok(),
      DateSegmentKind::Minute => minute = s.parse().ok(),
      _ => {}
    }
  }
  let (y, m, d) = (y?, m?, d?);
  let hour = hour?;
  let minute = minute?;
  if m < 1 || m > 12 || d < 1 || d > days_in_month(y, m) { return None; }
  if hour > 23 || minute > 59 { return None; }
  Some((y, m, d, hour, minute))
}

/// Format a datetime using a pattern string (e.g. `"mm/dd/yyyy HH:MM"`).
pub fn format_datetime_pattern(pattern: &str, y: i32, m: u8, d: u8, hour: u8, min: u8) -> String {
  let mut out = String::with_capacity(pattern.len() + 4);
  let bytes = pattern.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    if i + 4 <= bytes.len() && &bytes[i..i + 4] == b"yyyy" {
      out.push_str(&format!("{y:04}"));
      i += 4;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"dd" {
      out.push_str(&format!("{d:02}"));
      i += 2;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"mm" {
      out.push_str(&format!("{m:02}"));
      i += 2;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"HH" {
      out.push_str(&format!("{hour:02}"));
      i += 2;
    } else if i + 2 <= bytes.len() && &bytes[i..i + 2] == b"MM" {
      out.push_str(&format!("{min:02}"));
      i += 2;
    } else {
      out.push(bytes[i] as char);
      i += 1;
    }
  }
  out
}

/// Find the index of the editable segment containing `byte_pos`.
/// Returns `None` if `byte_pos` is on a separator.
pub fn segment_at(segments: &[DateSegment], byte_pos: usize) -> Option<usize> {
  for (i, seg) in segments.iter().enumerate() {
    if seg.kind == DateSegmentKind::Separator { continue; }
    if byte_pos >= seg.byte_start && byte_pos < seg.byte_start + seg.byte_len {
      return Some(i);
    }
  }
  None
}

/// Return the editable segment indices (excluding separators) in order.
pub fn editable_segment_indices(segments: &[DateSegment]) -> Vec<usize> {
  segments.iter().enumerate()
    .filter(|(_, s)| s.kind != DateSegmentKind::Separator)
    .map(|(i, _)| i)
    .collect()
}

/// Move cursor left, skipping over separators.
/// Returns the new byte position.
pub fn cursor_left(segments: &[DateSegment], byte_pos: usize) -> usize {
  if byte_pos == 0 { return 0; }
  let new_pos = byte_pos - 1;
  for seg in segments {
    if seg.kind == DateSegmentKind::Separator
      && new_pos >= seg.byte_start
      && new_pos < seg.byte_start + seg.byte_len
    {
      return seg.byte_start.saturating_sub(1);
    }
  }
  new_pos
}

/// Move cursor right, skipping over separators.
/// `text_len` is the total length of the formatted string.
pub fn cursor_right(segments: &[DateSegment], byte_pos: usize, text_len: usize) -> usize {
  let new_pos = byte_pos + 1;
  if new_pos >= text_len { return text_len; }
  for seg in segments {
    if seg.kind == DateSegmentKind::Separator
      && new_pos >= seg.byte_start
      && new_pos < seg.byte_start + seg.byte_len
    {
      return seg.byte_start + seg.byte_len;
    }
  }
  new_pos
}

/// Find the editable segment index at or just before `byte_pos`.
fn find_current_editable(segments: &[DateSegment], byte_pos: usize) -> Option<usize> {
  let editable = editable_segment_indices(segments);
  // Exact match (cursor inside segment).
  if let Some(pos) = editable.iter().position(|&i| {
    let s = &segments[i];
    byte_pos >= s.byte_start && byte_pos < s.byte_start + s.byte_len
  }) {
    return Some(pos);
  }
  // Cursor at end of a segment or on separator — find the last segment that ends at or before byte_pos.
  let mut best = None;
  for (ei, &si) in editable.iter().enumerate() {
    let s = &segments[si];
    if s.byte_start + s.byte_len <= byte_pos {
      best = Some(ei);
    }
  }
  best
}

/// Jump to the next editable segment. Returns (byte_start, byte_end) of the
/// next segment, or the current one if already at the last.
pub fn next_segment(segments: &[DateSegment], byte_pos: usize) -> (usize, usize) {
  let editable = editable_segment_indices(segments);
  if editable.is_empty() { return (0, 0); }
  let cur = find_current_editable(segments, byte_pos);
  let next_idx = match cur {
    Some(c) if c + 1 < editable.len() => editable[c + 1],
    _ => *editable.last().unwrap(),
  };
  let s = &segments[next_idx];
  (s.byte_start, s.byte_start + s.byte_len)
}

/// Jump to the previous editable segment.
pub fn prev_segment(segments: &[DateSegment], byte_pos: usize) -> (usize, usize) {
  let editable = editable_segment_indices(segments);
  if editable.is_empty() { return (0, 0); }
  let cur = find_current_editable(segments, byte_pos);
  let prev_idx = match cur {
    Some(c) if c > 0 => editable[c - 1],
    _ => editable[0],
  };
  let s = &segments[prev_idx];
  (s.byte_start, s.byte_start + s.byte_len)
}

/// Clamp a byte position to the nearest editable position.
/// If on a separator, snap to the start of the next editable segment
/// (or end of previous if at the end).
pub fn clamp_to_editable(segments: &[DateSegment], byte_pos: usize) -> usize {
  for seg in segments {
    if seg.kind != DateSegmentKind::Separator
      && byte_pos >= seg.byte_start
      && byte_pos < seg.byte_start + seg.byte_len
    {
      return byte_pos;
    }
  }
  // On a separator — find nearest editable segment after this position.
  for seg in segments {
    if seg.kind != DateSegmentKind::Separator && seg.byte_start >= byte_pos {
      return seg.byte_start;
    }
  }
  // Past all segments — snap to end of last editable.
  for seg in segments.iter().rev() {
    if seg.kind != DateSegmentKind::Separator {
      return seg.byte_start + seg.byte_len;
    }
  }
  0
}

/// Check if a byte position is on a separator.
pub fn is_separator(segments: &[DateSegment], byte_pos: usize) -> bool {
  segments.iter().any(|s| {
    s.kind == DateSegmentKind::Separator
      && byte_pos >= s.byte_start
      && byte_pos < s.byte_start + s.byte_len
  })
}

/// Result of inserting a character into a date display string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateEditResult {
  pub text: String,
  pub cursor: usize,
  pub consumed: bool,
}

/// Insert a digit character into the date display string using overwrite
/// mode within the current segment. Non-digit input is rejected.
/// When a segment is full, auto-advances to the next segment.
pub fn date_overwrite_char(
  text: &str,
  cursor_pos: usize,
  ch: char,
  segments: &[DateSegment],
) -> DateEditResult {
  if !ch.is_ascii_digit() {
    return DateEditResult { text: text.to_string(), cursor: cursor_pos, consumed: false };
  }

  let mut pos = clamp_to_editable(segments, cursor_pos);
  let seg_idx = match segment_at(segments, pos) {
    Some(i) => i,
    None => {
      let editable = editable_segment_indices(segments);
      if let Some(&first) = editable.first() {
        pos = segments[first].byte_start;
        first
      } else {
        return DateEditResult { text: text.to_string(), cursor: cursor_pos, consumed: false };
      }
    }
  };

  let seg = &segments[seg_idx];
  // Replace the character at `pos` in the text.
  let mut chars: Vec<u8> = text.bytes().collect();
  while chars.len() < seg.byte_start + seg.byte_len {
    chars.push(b'0');
  }
  if pos < chars.len() {
    chars[pos] = ch as u8;
  }
  let new_text = String::from_utf8(chars).unwrap_or_else(|_| text.to_string());

  // Advance cursor: if at end of segment, jump to next editable segment.
  let next_pos = pos + 1;
  let new_cursor = if next_pos >= seg.byte_start + seg.byte_len {
    // Auto-advance to next segment
    let editable = editable_segment_indices(segments);
    let cur_e = editable.iter().position(|&i| i == seg_idx);
    match cur_e {
      Some(c) if c + 1 < editable.len() => segments[editable[c + 1]].byte_start,
      _ => next_pos.min(new_text.len()),
    }
  } else {
    next_pos
  };

  DateEditResult { text: new_text, cursor: new_cursor, consumed: true }
}

/// Backspace: find the nearest editable position before cursor,
/// replace that character with '0', move cursor there.
pub fn date_backspace(
  text: &str,
  cursor_pos: usize,
  segments: &[DateSegment],
) -> DateEditResult {
  if cursor_pos == 0 {
    return DateEditResult { text: text.to_string(), cursor: 0, consumed: false };
  }
  // Walk backwards to find the nearest editable character position.
  let mut target = cursor_pos - 1;
  loop {
    if segment_at(segments, target).is_some() {
      let mut chars: Vec<u8> = text.bytes().collect();
      if target < chars.len() {
        chars[target] = b'0';
      }
      let new_text = String::from_utf8(chars).unwrap_or_else(|_| text.to_string());
      return DateEditResult { text: new_text, cursor: target, consumed: true };
    }
    if target == 0 { break; }
    target -= 1;
  }
  DateEditResult { text: text.to_string(), cursor: cursor_pos, consumed: false }
}

/// Validate a segment value against its allowed range.
pub fn validate_segment(kind: DateSegmentKind, value: &str, year: i32, month: u8) -> bool {
  let n: u32 = match value.parse() {
    Ok(v) => v,
    Err(_) => return false,
  };
  match kind {
    DateSegmentKind::Month => (1..=12).contains(&n),
    DateSegmentKind::Day => {
      let max = if month >= 1 && month <= 12 { days_in_month(year, month) as u32 } else { 31 };
      (1..=max).contains(&n)
    }
    DateSegmentKind::Year => n >= 1 && n <= 9999,
    DateSegmentKind::Hour => n <= 23,
    DateSegmentKind::Minute => n <= 59,
    DateSegmentKind::Separator => true,
  }
}

/// Validate the entire formatted date string against its pattern.
pub fn validate_formatted(text: &str, segments: &[DateSegment]) -> bool {
  if text.len() < segments.last().map(|s| s.byte_start + s.byte_len).unwrap_or(0) {
    return false;
  }
  let mut year: i32 = 0;
  let mut month: u8 = 0;
  // First pass: extract year and month for day validation.
  for seg in segments {
    if seg.kind == DateSegmentKind::Separator { continue; }
    let Some(s) = text.get(seg.byte_start..seg.byte_start + seg.byte_len) else { return false };
    match seg.kind {
      DateSegmentKind::Year => year = s.parse().unwrap_or(0),
      DateSegmentKind::Month => month = s.parse().unwrap_or(0),
      _ => {}
    }
  }
  // Second pass: validate each segment.
  for seg in segments {
    if seg.kind == DateSegmentKind::Separator { continue; }
    let Some(s) = text.get(seg.byte_start..seg.byte_start + seg.byte_len) else { return false };
    if !validate_segment(seg.kind, s, year, month) {
      return false;
    }
  }
  true
}

/// Get the date/datetime pattern for the currently focused date input.
pub fn focused_date_pattern_from_tree(tree: &crate::Tree) -> String {
  use wgpu_html_models::common::html_enums::InputType;
  let is_datetime = tree.interaction.focus_path.as_deref()
    .and_then(|p| tree.root.as_ref()?.at_path(p))
    .map(|n| matches!(&n.element, crate::Element::Input(inp) if matches!(inp.r#type, Some(InputType::DatetimeLocal))))
    .unwrap_or(false);
  if is_datetime { tree.locale.datetime_pattern().to_string() } else { tree.locale.date_pattern().to_string() }
}

pub fn prev_month(y: i32, m: u8) -> (i32, u8) {
  if m <= 1 { (y - 1, 12) } else { (y, m - 1) }
}

pub fn next_month(y: i32, m: u8) -> (i32, u8) {
  if m >= 12 { (y + 1, 1) } else { (y, m + 1) }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn leap_years() {
    assert!(is_leap_year(2000));
    assert!(is_leap_year(2024));
    assert!(!is_leap_year(1900));
    assert!(!is_leap_year(2023));
  }

  #[test]
  fn days() {
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2023, 2), 28);
    assert_eq!(days_in_month(2024, 1), 31);
    assert_eq!(days_in_month(2024, 4), 30);
  }

  #[test]
  fn dow() {
    // 2024-01-01 is Monday
    assert_eq!(day_of_week(2024, 1, 1), 0);
    // 2024-01-07 is Sunday
    assert_eq!(day_of_week(2024, 1, 7), 6);
  }

  #[test]
  fn parse() {
    assert_eq!(parse_date("2025-05-09"), Some((2025, 5, 9)));
    assert_eq!(parse_date("bad"), None);
    assert_eq!(parse_datetime_local("2025-05-09T14:30"), Some((2025, 5, 9, 14, 30)));
  }

  // ── Segment parsing ──

  #[test]
  fn segments_mdy() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert_eq!(segs.len(), 5);
    assert_eq!(segs[0].kind, DateSegmentKind::Month);
    assert_eq!((segs[0].byte_start, segs[0].byte_len), (0, 2));
    assert_eq!(segs[1].kind, DateSegmentKind::Separator);
    assert_eq!((segs[1].byte_start, segs[1].byte_len), (2, 1));
    assert_eq!(segs[2].kind, DateSegmentKind::Day);
    assert_eq!((segs[2].byte_start, segs[2].byte_len), (3, 2));
    assert_eq!(segs[3].kind, DateSegmentKind::Separator);
    assert_eq!(segs[4].kind, DateSegmentKind::Year);
    assert_eq!((segs[4].byte_start, segs[4].byte_len), (6, 4));
  }

  #[test]
  fn segments_dmy_dot() {
    let segs = parse_pattern_segments("dd.mm.yyyy");
    assert_eq!(segs[0].kind, DateSegmentKind::Day);
    assert_eq!(segs[2].kind, DateSegmentKind::Month);
    assert_eq!(segs[4].kind, DateSegmentKind::Year);
  }

  #[test]
  fn segments_ymd() {
    let segs = parse_pattern_segments("yyyy-mm-dd");
    assert_eq!(segs[0].kind, DateSegmentKind::Year);
    assert_eq!(segs[2].kind, DateSegmentKind::Month);
    assert_eq!(segs[4].kind, DateSegmentKind::Day);
  }

  // ── Segment navigation ──

  #[test]
  fn segment_at_mdy() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    // "05/09/2025"
    assert_eq!(segment_at(&segs, 0), Some(0)); // 'm' of month
    assert_eq!(segment_at(&segs, 1), Some(0)); // second 'm'
    assert_eq!(segment_at(&segs, 2), None);    // '/' separator
    assert_eq!(segment_at(&segs, 3), Some(2)); // 'd' of day
    assert_eq!(segment_at(&segs, 6), Some(4)); // 'y' of year
  }

  #[test]
  fn editable_indices() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert_eq!(editable_segment_indices(&segs), vec![0, 2, 4]);
  }

  // ── Cursor movement ──

  #[test]
  fn cursor_left_skips_separator() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    // pos=3 (start of day), left → skip '/' at 2 → land at 1 (end of month)
    assert_eq!(cursor_left(&segs, 3), 1);
    // pos=6 (start of year), left → skip '/' at 5 → land at 4 (end of day)
    assert_eq!(cursor_left(&segs, 6), 4);
    // pos=1 (inside month), left → 0 (still in month, no skip)
    assert_eq!(cursor_left(&segs, 1), 0);
  }

  #[test]
  fn cursor_right_skips_separator() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    // pos=1 → right → 2, which is separator → jump to 3 (start of day)
    assert_eq!(cursor_right(&segs, 1, 10), 3);
  }

  #[test]
  fn cursor_right_at_end() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert_eq!(cursor_right(&segs, 9, 10), 10);
    assert_eq!(cursor_right(&segs, 10, 10), 10);
  }

  #[test]
  fn cursor_left_at_start() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert_eq!(cursor_left(&segs, 0), 0);
  }

  // ── Tab navigation ──

  #[test]
  fn next_segment_mdy() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    // From month (pos 0) → day segment
    assert_eq!(next_segment(&segs, 0), (3, 5));
    assert_eq!(next_segment(&segs, 1), (3, 5));
    // From day → year
    assert_eq!(next_segment(&segs, 3), (6, 10));
    // From year → stays at year (last segment)
    assert_eq!(next_segment(&segs, 7), (6, 10));
  }

  #[test]
  fn prev_segment_mdy() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    // From year → day
    assert_eq!(prev_segment(&segs, 7), (3, 5));
    // From day → month
    assert_eq!(prev_segment(&segs, 3), (0, 2));
    // From month → stays at month (first segment)
    assert_eq!(prev_segment(&segs, 0), (0, 2));
  }

  // ── Clamp to editable ──

  #[test]
  fn clamp_separator_to_next() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert_eq!(clamp_to_editable(&segs, 2), 3); // '/' → start of day
    assert_eq!(clamp_to_editable(&segs, 5), 6); // '/' → start of year
  }

  #[test]
  fn clamp_editable_unchanged() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert_eq!(clamp_to_editable(&segs, 0), 0);
    assert_eq!(clamp_to_editable(&segs, 1), 1);
    assert_eq!(clamp_to_editable(&segs, 3), 3);
    assert_eq!(clamp_to_editable(&segs, 8), 8);
  }

  // ── is_separator ──

  #[test]
  fn separator_detection() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert!(is_separator(&segs, 2));
    assert!(is_separator(&segs, 5));
    assert!(!is_separator(&segs, 0));
    assert!(!is_separator(&segs, 3));
    assert!(!is_separator(&segs, 9));
  }

  // ── Formatted parse roundtrip ──

  #[test]
  fn format_and_parse_roundtrip_mdy() {
    use crate::locale::format_date_pattern;
    let pattern = "mm/dd/yyyy";
    let formatted = format_date_pattern(pattern, 2025, 5, 9);
    assert_eq!(formatted, "05/09/2025");
    assert_eq!(parse_formatted_date(&formatted, pattern), Some((2025, 5, 9)));
  }

  #[test]
  fn format_and_parse_roundtrip_dmy_dot() {
    use crate::locale::format_date_pattern;
    let pattern = "dd.mm.yyyy";
    let formatted = format_date_pattern(pattern, 2025, 12, 31);
    assert_eq!(formatted, "31.12.2025");
    assert_eq!(parse_formatted_date(&formatted, pattern), Some((2025, 12, 31)));
  }

  #[test]
  fn parse_formatted_invalid() {
    assert_eq!(parse_formatted_date("13/09/2025", "mm/dd/yyyy"), None); // month 13
    assert_eq!(parse_formatted_date("02/30/2025", "mm/dd/yyyy"), None); // feb 30
    assert_eq!(parse_formatted_date("xx/09/2025", "mm/dd/yyyy"), None); // non-numeric
  }

  // ── Datetime pattern ──

  #[test]
  fn segments_datetime() {
    let segs = parse_pattern_segments("mm/dd/yyyy HH:MM");
    assert_eq!(segs.len(), 9);
    assert_eq!(segs[0].kind, DateSegmentKind::Month);   // 0..2
    assert_eq!(segs[1].kind, DateSegmentKind::Separator); // 2 '/'
    assert_eq!(segs[2].kind, DateSegmentKind::Day);      // 3..5
    assert_eq!(segs[3].kind, DateSegmentKind::Separator); // 5 '/'
    assert_eq!(segs[4].kind, DateSegmentKind::Year);     // 6..10
    assert_eq!(segs[5].kind, DateSegmentKind::Separator); // 10 ' '
    assert_eq!(segs[6].kind, DateSegmentKind::Hour);     // 11..13
    assert_eq!(segs[7].kind, DateSegmentKind::Separator); // 13 ':'
    assert_eq!(segs[8].kind, DateSegmentKind::Minute);   // 14..16
  }

  #[test]
  fn format_datetime_pattern_roundtrip() {
    let pattern = "mm/dd/yyyy HH:MM";
    let formatted = format_datetime_pattern(pattern, 2025, 5, 9, 14, 30);
    assert_eq!(formatted, "05/09/2025 14:30");
    assert_eq!(parse_formatted_datetime(&formatted, pattern), Some((2025, 5, 9, 14, 30)));
  }

  #[test]
  fn parse_formatted_datetime_invalid_hour() {
    assert_eq!(parse_formatted_datetime("05/09/2025 24:30", "mm/dd/yyyy HH:MM"), None);
  }

  #[test]
  fn parse_formatted_datetime_invalid_minute() {
    assert_eq!(parse_formatted_datetime("05/09/2025 14:60", "mm/dd/yyyy HH:MM"), None);
  }

  #[test]
  fn validate_datetime_formatted() {
    let segs = parse_pattern_segments("mm/dd/yyyy HH:MM");
    assert!(validate_formatted("05/09/2025 14:30", &segs));
    assert!(!validate_formatted("05/09/2025 25:30", &segs)); // hour 25
    assert!(!validate_formatted("05/09/2025 14:60", &segs)); // minute 60
    assert!(!validate_formatted("13/09/2025 14:30", &segs)); // month 13
  }

  #[test]
  fn datetime_tab_navigation() {
    let segs = parse_pattern_segments("mm/dd/yyyy HH:MM");
    // month → day → year → hour → minute
    assert_eq!(next_segment(&segs, 0), (3, 5));   // month → day
    assert_eq!(next_segment(&segs, 3), (6, 10));  // day → year
    assert_eq!(next_segment(&segs, 7), (11, 13)); // year → hour
    assert_eq!(next_segment(&segs, 11), (14, 16)); // hour → minute
    assert_eq!(next_segment(&segs, 14), (14, 16)); // minute → stays
    // reverse
    assert_eq!(prev_segment(&segs, 14), (11, 13)); // minute → hour
    assert_eq!(prev_segment(&segs, 11), (6, 10));  // hour → year
  }

  #[test]
  fn time_first_pattern() {
    let pattern = "HH:MM yyyy-mm-dd";
    let segs = parse_pattern_segments(pattern);
    assert_eq!(segs[0].kind, DateSegmentKind::Hour);
    assert_eq!((segs[0].byte_start, segs[0].byte_len), (0, 2));
    assert_eq!(segs[2].kind, DateSegmentKind::Minute);
    assert_eq!((segs[2].byte_start, segs[2].byte_len), (3, 2));
    assert_eq!(segs[4].kind, DateSegmentKind::Year);
    assert_eq!(segs[6].kind, DateSegmentKind::Month);
    assert_eq!(segs[8].kind, DateSegmentKind::Day);

    let formatted = format_datetime_pattern(pattern, 2025, 5, 9, 14, 30);
    assert_eq!(formatted, "14:30 2025-05-09");
    assert_eq!(parse_formatted_datetime(&formatted, pattern), Some((2025, 5, 9, 14, 30)));

    // Tab: hour → minute → year → month → day
    assert_eq!(next_segment(&segs, 0), (3, 5));   // hour → minute
    assert_eq!(next_segment(&segs, 3), (6, 10));  // minute → year
    assert_eq!(next_segment(&segs, 7), (11, 13)); // year → month
    assert_eq!(next_segment(&segs, 11), (14, 16)); // month → day
  }

  #[test]
  fn dot_separated_dmy_datetime() {
    let pattern = "dd.mm.yyyy HH:MM";
    let formatted = format_datetime_pattern(pattern, 2025, 12, 31, 23, 59);
    assert_eq!(formatted, "31.12.2025 23:59");
    assert_eq!(parse_formatted_datetime(&formatted, pattern), Some((2025, 12, 31, 23, 59)));
  }

  // ── Phase 3: Overwrite mode ──

  #[test]
  fn overwrite_digit_in_month() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    let r = date_overwrite_char("05/09/2025", 0, '1', &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "15/09/2025");
    assert_eq!(r.cursor, 1);
  }

  #[test]
  fn overwrite_second_digit_auto_advances() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    let r = date_overwrite_char("15/09/2025", 1, '2', &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "12/09/2025");
    assert_eq!(r.cursor, 3); // auto-advanced past '/' to day segment
  }

  #[test]
  fn overwrite_in_day_segment() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    let r = date_overwrite_char("05/09/2025", 3, '1', &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "05/19/2025");
    assert_eq!(r.cursor, 4);
  }

  #[test]
  fn overwrite_in_year_segment() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    let r = date_overwrite_char("05/09/2025", 6, '3', &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "05/09/3025");
    assert_eq!(r.cursor, 7);
  }

  #[test]
  fn overwrite_rejects_non_digit() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    let r = date_overwrite_char("05/09/2025", 0, 'a', &segs);
    assert!(!r.consumed);
    assert_eq!(r.text, "05/09/2025");
  }

  #[test]
  fn overwrite_dmy_dot_pattern() {
    let segs = parse_pattern_segments("dd.mm.yyyy");
    let r = date_overwrite_char("09.05.2025", 0, '3', &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "39.05.2025");
    assert_eq!(r.cursor, 1);
  }

  #[test]
  fn overwrite_last_year_digit_stays() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    let r = date_overwrite_char("05/09/2025", 9, '6', &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "05/09/2026");
    assert_eq!(r.cursor, 10); // at end, no more segments
  }

  // ── Phase 3: Backspace ──

  #[test]
  fn backspace_replaces_with_zero() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    // cursor at 2: backspace targets pos 1 ('5'), zeros it
    let r = date_backspace("05/09/2025", 2, &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "00/09/2025");
    assert_eq!(r.cursor, 1);

    // cursor at 1: backspace targets pos 0 ('0'), zeros it (already '0')
    let r2 = date_backspace("15/09/2025", 1, &segs);
    assert!(r2.consumed);
    assert_eq!(r2.text, "05/09/2025");
    assert_eq!(r2.cursor, 0);
  }

  #[test]
  fn backspace_across_separator() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    // cursor at 3 (start of day), backspace skips '/' and zeros pos 1 ('5')
    let r = date_backspace("05/09/2025", 3, &segs);
    assert!(r.consumed);
    assert_eq!(r.text, "00/09/2025");
    assert_eq!(r.cursor, 1);
  }

  #[test]
  fn backspace_at_zero_does_nothing() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    let r = date_backspace("05/09/2025", 0, &segs);
    assert!(!r.consumed);
    assert_eq!(r.text, "05/09/2025");
  }

  // ── Phase 4: Segment validation ──

  #[test]
  fn validate_month() {
    assert!(validate_segment(DateSegmentKind::Month, "01", 2025, 1));
    assert!(validate_segment(DateSegmentKind::Month, "12", 2025, 1));
    assert!(!validate_segment(DateSegmentKind::Month, "00", 2025, 1));
    assert!(!validate_segment(DateSegmentKind::Month, "13", 2025, 1));
  }

  #[test]
  fn validate_day() {
    assert!(validate_segment(DateSegmentKind::Day, "01", 2025, 1));
    assert!(validate_segment(DateSegmentKind::Day, "31", 2025, 1));
    assert!(!validate_segment(DateSegmentKind::Day, "00", 2025, 1));
    assert!(!validate_segment(DateSegmentKind::Day, "32", 2025, 1));
  }

  #[test]
  fn validate_day_feb() {
    assert!(validate_segment(DateSegmentKind::Day, "28", 2023, 2));
    assert!(!validate_segment(DateSegmentKind::Day, "29", 2023, 2));
    assert!(validate_segment(DateSegmentKind::Day, "29", 2024, 2)); // leap
  }

  #[test]
  fn validate_year() {
    assert!(validate_segment(DateSegmentKind::Year, "2025", 0, 0));
    assert!(validate_segment(DateSegmentKind::Year, "0001", 0, 0));
    assert!(!validate_segment(DateSegmentKind::Year, "0000", 0, 0));
  }

  #[test]
  fn validate_hour() {
    assert!(validate_segment(DateSegmentKind::Hour, "00", 0, 0));
    assert!(validate_segment(DateSegmentKind::Hour, "23", 0, 0));
    assert!(!validate_segment(DateSegmentKind::Hour, "24", 0, 0));
  }

  #[test]
  fn validate_minute() {
    assert!(validate_segment(DateSegmentKind::Minute, "00", 0, 0));
    assert!(validate_segment(DateSegmentKind::Minute, "59", 0, 0));
    assert!(!validate_segment(DateSegmentKind::Minute, "60", 0, 0));
  }

  // ── Phase 4: Full validation ──

  #[test]
  fn validate_full_valid() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert!(validate_formatted("05/09/2025", &segs));
    assert!(validate_formatted("12/31/2025", &segs));
    assert!(validate_formatted("02/29/2024", &segs)); // leap year
  }

  #[test]
  fn validate_full_invalid() {
    let segs = parse_pattern_segments("mm/dd/yyyy");
    assert!(!validate_formatted("13/09/2025", &segs)); // month 13
    assert!(!validate_formatted("02/30/2025", &segs)); // feb 30
    assert!(!validate_formatted("00/09/2025", &segs)); // month 0
    assert!(!validate_formatted("05/00/2025", &segs)); // day 0
    assert!(!validate_formatted("05/09/0000", &segs)); // year 0
  }

  #[test]
  fn validate_dmy_dot() {
    let segs = parse_pattern_segments("dd.mm.yyyy");
    assert!(validate_formatted("09.05.2025", &segs));
    assert!(!validate_formatted("31.02.2025", &segs)); // feb 31
  }
}
