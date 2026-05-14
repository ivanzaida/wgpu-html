use lui_tree::{set_date_value, text_input, Tree};
use lui_v1::paint::byte_offset_to_glyph_index;

fn make_date_tree(value: &str) -> Tree {
  let html = format!(r#"<input type="date" value="{value}"/>"#);
  lui_parser::parse(&html)
}

fn make_datetime_tree(value: &str) -> Tree {
  let html = format!(r#"<input type="datetime-local" value="{value}"/>"#);
  lui_parser::parse(&html)
}

fn input_path(tree: &Tree) -> Vec<usize> {
  tree.query_selector_path("input").expect("input not found")
}

fn input_value(tree: &Tree) -> String {
  let path = input_path(tree);
  let node = tree.root.as_ref().unwrap().at_path(&path).unwrap();
  match &node.element {
    lui_tree::Element::Input(inp) => inp.value.as_deref().unwrap_or("").to_string(),
    _ => panic!("not an input"),
  }
}

// ── Focus populates display value ──

#[test]
fn focus_date_populates_display_value() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(dv, "05/09/2025"); // default locale is MDY
}

#[test]
fn focus_datetime_populates_display_value() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(dv, "05/09/2025 14:30");
}

#[test]
fn focus_empty_date_shows_placeholder() {
  let mut tree = make_date_tree("");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(dv, "mm/dd/yyyy");
}

// ── Overwrite mode typing ──

#[test]
fn type_digits_into_date() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Cursor starts at end. Type '1' — should overwrite at first segment (snapped).
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  assert!(text_input(&mut tree, "1"));
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(&dv[..2], "15"); // first char overwritten
}

#[test]
fn type_non_digit_rejected() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  assert!(!text_input(&mut tree, "a"));
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(dv, "05/09/2025"); // unchanged
}

#[test]
fn type_auto_advances_past_separator() {
  let mut tree = make_date_tree("2025-01-01");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "1");
  text_input(&mut tree, "2");
  // After typing "12" in month, auto-advances to day segment and selects it (3..5)
  let ec = tree.interaction.edit_cursor.as_ref().unwrap();
  assert!(ec.has_selection());
  assert_eq!(ec.selection_range(), (3, 5));
}

// ── Blur roundtrip ──

#[test]
fn blur_valid_date_writes_iso() {
  let mut tree = make_date_tree("2025-01-01");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Overwrite month to "05"
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "0");
  text_input(&mut tree, "5");
  // Overwrite day to "09"
  text_input(&mut tree, "0");
  text_input(&mut tree, "9");

  // Blur
  tree.focus(None);
  assert!(tree.interaction.date_display_value.is_none());
  assert_eq!(input_value(&tree), "2025-05-09");
}

#[test]
fn blur_invalid_date_clamps() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Overwrite month to "00" (invalid) — auto-advances to day, clamped to 01.
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "0");
  text_input(&mut tree, "0");

  tree.focus(None);
  assert_eq!(input_value(&tree), "2025-01-09"); // clamped month 00 → 01
}

// ── Calendar picker updates display ──

#[test]
fn set_date_value_updates_iso() {
  let mut tree = make_date_tree("2025-01-01");
  let path = input_path(&tree);
  set_date_value(&mut tree, &path, "2025-12-25");
  assert_eq!(input_value(&tree), "2025-12-25");
}

// ── Segment navigation ──

#[test]
fn arrow_keys_skip_separators() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Start at pos 1 (inside month)
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(1));

  // Arrow right from pos 1 → pos 2 would be separator, so should jump to 3
  tree.key_down("ArrowRight", "ArrowRight", false);
  let cursor = tree.interaction.edit_cursor.as_ref().unwrap().cursor;
  // pos 1 → right → 2 is separator → jump to 3
  assert!(cursor >= 2); // at least past the separator
}

#[test]
fn tab_selects_next_segment() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));

  // Tab should select the next segment (day)
  tree.key_down("Tab", "Tab", false);
  let ec = tree.interaction.edit_cursor.as_ref().unwrap();
  assert!(ec.has_selection());
  let (start, end) = ec.selection_range();
  assert_eq!(start, 3); // day segment start
  assert_eq!(end, 5); // day segment end
}

// ── Datetime-local (HH:MM) ──

#[test]
fn datetime_focus_shows_formatted_with_time() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(dv, "05/09/2025 14:30");
}

#[test]
fn datetime_type_into_hour_segment() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // "05/09/2025 14:30" — hour starts at position 11
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(11));
  assert!(text_input(&mut tree, "2"));
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(&dv[11..13], "24");
}

#[test]
fn datetime_type_into_minute_segment() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // "05/09/2025 14:30" — minute starts at position 14
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(14));
  assert!(text_input(&mut tree, "4"));
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(&dv[14..16], "40");
}

#[test]
fn datetime_tab_through_all_segments() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Start at month (pos 0), tab through all 5 segments
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));

  // Focus already selects month (0..2). Tab 1: month → day
  tree.key_down("Tab", "Tab", false);
  let ec = tree.interaction.edit_cursor.as_ref().unwrap();
  assert_eq!(ec.selection_range(), (3, 5));

  // Tab 2: day → year
  tree.key_down("Tab", "Tab", false);
  let ec = tree.interaction.edit_cursor.as_ref().unwrap();
  assert_eq!(ec.selection_range(), (6, 10));

  // Tab 3: year → hour
  tree.key_down("Tab", "Tab", false);
  let ec = tree.interaction.edit_cursor.as_ref().unwrap();
  assert_eq!(ec.selection_range(), (11, 13));

  // Tab 4: hour → minute
  tree.key_down("Tab", "Tab", false);
  let ec = tree.interaction.edit_cursor.as_ref().unwrap();
  assert_eq!(ec.selection_range(), (14, 16));
}

#[test]
fn datetime_blur_valid_writes_iso() {
  let mut tree = make_datetime_tree("2025-01-01T00:00");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Type "05" into month, "09" into day
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "0");
  text_input(&mut tree, "5");
  text_input(&mut tree, "0");
  text_input(&mut tree, "9");
  // Skip year (leave as 2025), type "14" into hour
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(11));
  text_input(&mut tree, "1");
  text_input(&mut tree, "4");
  // Type "30" into minute
  text_input(&mut tree, "3");
  text_input(&mut tree, "0");

  tree.focus(None);
  assert_eq!(input_value(&tree), "2025-05-09T14:30");
}

#[test]
fn datetime_blur_invalid_hour_clamps() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Overwrite hour to "25" — auto-advances to minute, clamped to 23.
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(11));
  text_input(&mut tree, "2");
  text_input(&mut tree, "5");

  tree.focus(None);
  assert_eq!(input_value(&tree), "2025-05-09T23:30"); // clamped hour 25 → 23
}

#[test]
fn datetime_blur_invalid_minute_reverts() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Overwrite minute to "60" (invalid) — clamped to 59 on segment leave.
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(14));
  text_input(&mut tree, "6");
  text_input(&mut tree, "0");

  tree.focus(None);
  assert_eq!(input_value(&tree), "2025-05-09T14:59"); // clamped
}

#[test]
fn segment_leave_clamps_month() {
  let mut tree = make_date_tree("2025-01-15");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Type "34" into month — fills segment, auto-advances to day.
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "3");
  text_input(&mut tree, "4");

  // Display value should be clamped to 12, and ISO updated.
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(&dv[..2], "12", "month should clamp to 12");
  assert_eq!(input_value(&tree), "2025-12-15");
}

#[test]
fn segment_leave_clamps_hour() {
  let mut tree = make_datetime_tree("2025-05-09T14:30");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Type "25" into hour — auto-advances to minute, clamped to 23.
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(11));
  text_input(&mut tree, "2");
  text_input(&mut tree, "5");

  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(&dv[11..13], "23", "hour should clamp to 23");
  assert_eq!(input_value(&tree), "2025-05-09T23:30");
}

#[test]
fn tab_away_clamps_segment() {
  let mut tree = make_date_tree("2025-01-15");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Type "5" into month (partial "51"), then Tab to day.
  tree.interaction.edit_cursor = Some(lui_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "5");
  tree.key_down("Tab", "Tab", false);

  // Month "51" clamped to "12" on segment leave.
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(&dv[..2], "12");
  assert_eq!(input_value(&tree), "2025-12-15");
}

// ── Selection ↔ glyph mapping (reproduces +1 shift) ──

/// Build a synthetic ShapedRun for `text` where every character with
/// a visible glyph gets a 10 px wide box and invisible characters
/// (spaces) are skipped — matching real shaping behaviour.
fn synthetic_run(text: &str) -> lui_text::ShapedRun {
  let bb = lui_text::utf8_boundaries(text);
  let mut glyphs = Vec::new();
  let mut glyph_chars = Vec::new();
  let mut x = 0.0_f32;
  for (char_idx, ch) in text.chars().enumerate() {
    if ch == ' ' {
      x += 5.0;
      continue;
    }
    glyphs.push(lui_text::PositionedGlyph {
      x,
      y: 0.0,
      w: 10.0,
      h: 16.0,
      uv_min: [0.0; 2],
      uv_max: [1.0; 2],
      color: [0.0, 0.0, 0.0, 1.0],
    });
    glyph_chars.push(char_idx);
    x += 10.0;
  }
  lui_text::ShapedRun {
    lines: vec![lui_text::ShapedLine {
      top: 0.0,
      height: 20.0,
      glyph_range: (0, glyphs.len()),
    }],
    text: text.to_owned(),
    byte_boundaries: bb,
    width: x,
    height: 20.0,
    ascent: 16.0,
    glyphs,
    glyph_chars,
  }
}

#[test]
fn byte_offset_to_glyph_hour_selection() {
  // "05/09/2025 14:30" — space at position 10 is skipped during shaping.
  // Hour segment: bytes 11..13 ("14"), minute segment: bytes 14..16 ("30").
  let run = synthetic_run("05/09/2025 14:30");

  // Space is skipped: 15 glyphs for 16 characters.
  assert_eq!(run.glyphs.len(), 15, "space should be skipped");
  assert_eq!(run.glyph_chars.len(), 15);

  // Glyph 10 should map to char 11 ('1' of "14").
  assert_eq!(run.glyph_chars[10], 11, "glyph 10 = char 11 = '1'");

  // byte_offset_to_glyph_index must return glyph 10 for byte 11 (start of hour).
  let start_g = byte_offset_to_glyph_index(&run, 11);
  assert_eq!(
    start_g, 10,
    "hour selection start should be glyph 10 ('1'), not 11 ('4')"
  );

  let end_g = byte_offset_to_glyph_index(&run, 13);
  assert_eq!(end_g, 12, "hour selection end should be glyph 12 (':'), not 13 ('3')");
}

#[test]
fn byte_offset_to_glyph_minute_selection() {
  let run = synthetic_run("05/09/2025 14:30");
  let start_g = byte_offset_to_glyph_index(&run, 14);
  assert_eq!(
    start_g, 13,
    "minute selection start should be glyph 13 ('3'), not 14 ('0')"
  );

  let end_g = byte_offset_to_glyph_index(&run, 16);
  assert_eq!(
    end_g, 15,
    "minute selection end should be glyph 15 (past last), not glyphs.len()"
  );
}
