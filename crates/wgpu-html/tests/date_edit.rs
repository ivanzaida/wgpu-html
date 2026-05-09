use wgpu_html_tree::{Tree, text_input, set_date_value};

fn make_date_tree(value: &str) -> Tree {
  let html = format!(r#"<input type="date" value="{value}"/>"#);
  wgpu_html_parser::parse(&html)
}

fn make_datetime_tree(value: &str) -> Tree {
  let html = format!(r#"<input type="datetime-local" value="{value}"/>"#);
  wgpu_html_parser::parse(&html)
}

fn input_path(tree: &Tree) -> Vec<usize> {
  tree.query_selector_path("input").expect("input not found")
}

fn input_value(tree: &Tree) -> String {
  let path = input_path(tree);
  let node = tree.root.as_ref().unwrap().at_path(&path).unwrap();
  match &node.element {
    wgpu_html_tree::Element::Input(inp) => inp.value.as_deref().unwrap_or("").to_string(),
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
  tree.interaction.edit_cursor = Some(wgpu_html_tree::EditCursor::collapsed(0));
  assert!(text_input(&mut tree, "1"));
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(&dv[..2], "15"); // first char overwritten
}

#[test]
fn type_non_digit_rejected() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  tree.interaction.edit_cursor = Some(wgpu_html_tree::EditCursor::collapsed(0));
  assert!(!text_input(&mut tree, "a"));
  let dv = tree.interaction.date_display_value.as_deref().unwrap();
  assert_eq!(dv, "05/09/2025"); // unchanged
}

#[test]
fn type_auto_advances_past_separator() {
  let mut tree = make_date_tree("2025-01-01");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  tree.interaction.edit_cursor = Some(wgpu_html_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "1");
  text_input(&mut tree, "2");
  // After typing "12" in month, cursor should auto-advance past '/' to day segment (pos 3)
  let cursor = tree.interaction.edit_cursor.as_ref().unwrap().cursor;
  assert_eq!(cursor, 3);
}

// ── Blur roundtrip ──

#[test]
fn blur_valid_date_writes_iso() {
  let mut tree = make_date_tree("2025-01-01");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Overwrite month to "05"
  tree.interaction.edit_cursor = Some(wgpu_html_tree::EditCursor::collapsed(0));
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
fn blur_invalid_date_reverts() {
  let mut tree = make_date_tree("2025-05-09");
  let path = input_path(&tree);
  tree.focus(Some(&path));

  // Overwrite month to "00" (invalid)
  tree.interaction.edit_cursor = Some(wgpu_html_tree::EditCursor::collapsed(0));
  text_input(&mut tree, "0");
  text_input(&mut tree, "0");

  // Blur — should revert
  tree.focus(None);
  assert_eq!(input_value(&tree), "2025-05-09"); // unchanged
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
  tree.interaction.edit_cursor = Some(wgpu_html_tree::EditCursor::collapsed(1));

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

  tree.interaction.edit_cursor = Some(wgpu_html_tree::EditCursor::collapsed(0));

  // Tab should select the next segment (day)
  tree.key_down("Tab", "Tab", false);
  let ec = tree.interaction.edit_cursor.as_ref().unwrap();
  assert!(ec.has_selection());
  let (start, end) = ec.selection_range();
  assert_eq!(start, 3); // day segment start
  assert_eq!(end, 5);   // day segment end
}
