use super::helpers::*;
use crate::*;

// ── helpers ─────────────────────────────────────────────────────────

fn find_form_control(b: &LayoutBox) -> Option<&FormControlInfo> {
  if b.form_control.is_some() {
    return b.form_control.as_ref();
  }
  for c in &b.children {
    if let Some(fc) = find_form_control(c) {
      return Some(fc);
    }
  }
  None
}

fn find_box_with_form_control(b: &LayoutBox) -> Option<&LayoutBox> {
  if b.form_control.is_some() {
    return Some(b);
  }
  for c in &b.children {
    if let Some(found) = find_box_with_form_control(c) {
      return Some(found);
    }
  }
  None
}

fn find_all_form_controls(b: &LayoutBox, out: &mut Vec<FormControlInfo>) {
  if let Some(fc) = &b.form_control {
    out.push(fc.clone());
  }
  for c in &b.children {
    find_all_form_controls(c, out);
  }
}

fn find_box_with_text_color(b: &LayoutBox) -> Option<&LayoutBox> {
  if b.text_color.is_some() && b.form_control.is_none() {
    return Some(b);
  }
  for c in &b.children {
    if let Some(found) = find_box_with_text_color(c) {
      return Some(found);
    }
  }
  None
}

// ── Checkbox ────────────────────────────────────────────────────────

#[test]
fn checkbox_has_form_control_info() {
  let tree = make(r#"<body style="margin:0;"><input type="checkbox" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("checkbox should have FormControlInfo");
  assert!(matches!(fc.kind, FormControlKind::Checkbox { checked: false }));
}

#[test]
fn checkbox_checked_attribute() {
  let tree = make(r#"<body style="margin:0;"><input type="checkbox" checked /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("checkbox should have FormControlInfo");
  assert!(matches!(fc.kind, FormControlKind::Checkbox { checked: true }));
}

#[test]
fn checkbox_has_intrinsic_height() {
  let tree = make(r#"<body style="margin:0;"><input type="checkbox" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("checkbox box");
  assert!(b.border_rect.h > 5.0, "checkbox should have height, got {}", b.border_rect.h);
}

#[test]
fn checkbox_no_text_run() {
  let tree = make(r#"<body style="margin:0;"><input type="checkbox" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("checkbox box");
  assert!(b.text_run.is_none(), "checkbox should not have a text run");
}

// ── Radio ───────────────────────────────────────────────────────────

#[test]
fn radio_has_form_control_info() {
  let tree = make(r#"<body style="margin:0;"><input type="radio" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("radio should have FormControlInfo");
  assert!(matches!(fc.kind, FormControlKind::Radio { checked: false }));
}

#[test]
fn radio_checked_attribute() {
  let tree = make(r#"<body style="margin:0;"><input type="radio" checked /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("radio should have FormControlInfo");
  assert!(matches!(fc.kind, FormControlKind::Radio { checked: true }));
}

#[test]
fn radio_has_intrinsic_height() {
  let tree = make(r#"<body style="margin:0;"><input type="radio" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("radio box");
  assert!(b.border_rect.h > 5.0);
}

#[test]
fn radio_no_text_run() {
  let tree = make(r#"<body style="margin:0;"><input type="radio" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("radio box");
  assert!(b.text_run.is_none(), "radio should not have a text run");
}

// ── Range ───────────────────────────────────────────────────────────

#[test]
fn range_has_form_control_info_with_defaults() {
  let tree = make(r#"<body style="margin:0;"><input type="range" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("range should have FormControlInfo");
  match fc.kind {
    FormControlKind::Range { value, min, max } => {
      assert_eq!(min, 0.0);
      assert_eq!(max, 100.0);
      assert!((value - 50.0).abs() < 0.01, "default range value should be 50, got {value}");
    }
    _ => panic!("expected Range, got {:?}", fc.kind),
  }
}

#[test]
fn range_respects_min_max_value() {
  let tree = make(
    r#"<body style="margin:0;"><input type="range" min="10" max="20" value="15" /></body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("range");
  match fc.kind {
    FormControlKind::Range { value, min, max } => {
      assert_eq!(min, 10.0);
      assert_eq!(max, 20.0);
      assert!((value - 15.0).abs() < 0.01);
    }
    _ => panic!("expected Range"),
  }
}

#[test]
fn range_clamps_value_to_bounds() {
  let tree = make(
    r#"<body style="margin:0;"><input type="range" min="0" max="10" value="999" /></body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("range");
  match fc.kind {
    FormControlKind::Range { value, .. } => {
      assert!((value - 10.0).abs() < 0.01, "value should be clamped to max");
    }
    _ => panic!("expected Range"),
  }
}

#[test]
fn range_has_intrinsic_height() {
  let tree = make(r#"<body style="margin:0;"><input type="range" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("range box");
  assert!(b.border_rect.h > 5.0, "range should have height");
}

#[test]
fn range_no_text_run() {
  let tree = make(r#"<body style="margin:0;"><input type="range" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("range box");
  assert!(b.text_run.is_none(), "range should not have a text run");
}

// ── Color ───────────────────────────────────────────────────────────

#[test]
fn color_has_form_control_info_default_black() {
  let tree = make(r#"<body style="margin:0;"><input type="color" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("color should have FormControlInfo");
  match fc.kind {
    FormControlKind::Color { r, g, b, a } => {
      assert!(r < 0.01 && g < 0.01 && b < 0.01, "default color should be black");
      assert!((a - 1.0).abs() < 0.01, "default alpha should be 1.0");
    }
    _ => panic!("expected Color"),
  }
}

#[test]
fn color_parses_hex_value() {
  let tree = make(r##"<body style="margin:0;"><input type="color" value="#ff0000" /></body>"##);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("color");
  match fc.kind {
    FormControlKind::Color { r, g, b, .. } => {
      assert!(r > 0.9, "red channel should be high for #ff0000, got {r}");
      assert!(g < 0.01, "green should be 0");
      assert!(b < 0.01, "blue should be 0");
    }
    _ => panic!("expected Color"),
  }
}

#[test]
fn color_has_intrinsic_size() {
  let tree = make(r#"<body style="margin:0;"><input type="color" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("color box");
  assert!(b.border_rect.w > 20.0, "color should have width");
  assert!(b.border_rect.h > 10.0, "color should have height");
}

#[test]
fn color_no_text_run() {
  let tree = make(r#"<body style="margin:0;"><input type="color" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("color box");
  assert!(b.text_run.is_none(), "color should not have a text run");
}

// ── File ────────────────────────────────────────────────────────────

#[test]
fn file_has_form_control_info() {
  let tree = make(r#"<body style="margin:0;"><input type="file" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let fc = find_form_control(&root).expect("file should have FormControlInfo");
  assert!(matches!(fc.kind, FormControlKind::File));
}

// ── Hidden ──────────────────────────────────────────────────────────

#[test]
fn hidden_input_has_no_layout() {
  let tree = make(r#"<body style="margin:0;"><input type="hidden" value="secret" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root);
  assert!(b.is_none(), "hidden input should not have FormControlInfo");
  // Should be display:none — zero size
  let input_box = &root.children[0];
  assert!(
    input_box.border_rect.w < 0.01 && input_box.border_rect.h < 0.01,
    "hidden input should be zero size"
  );
}

// ── Text-like inputs (text, email, tel, etc.) ──────────────────────

#[test]
fn text_input_has_no_form_control_info() {
  let tree = make(r#"<body style="margin:0;"><input type="text" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert!(find_form_control(&root).is_none(), "text input should not have FormControlInfo");
}

#[test]
fn text_input_has_inline_block_display() {
  let tree = make(r#"<body style="margin:0;"><input type="text" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let input = &root.children[0];
  assert!(input.border_rect.h > 0.0, "text input should have height from UA line-height");
}

#[test]
fn email_input_has_no_form_control_info() {
  let tree = make(r#"<body style="margin:0;"><input type="email" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert!(find_form_control(&root).is_none());
}

// ── Button-like inputs ──────────────────────────────────────────────

#[test]
fn submit_input_has_no_form_control_info() {
  let tree = make(r#"<body style="margin:0;"><input type="submit" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert!(find_form_control(&root).is_none(), "submit should not have FormControlInfo");
}

#[test]
fn submit_input_has_button_styling() {
  let tree = make(r#"<body style="margin:0;"><input type="submit" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let input = &root.children[0];
  assert!(input.border_rect.h > 0.0, "submit should have height");
  assert!(input.background.is_some(), "submit should have background color");
}

#[test]
fn reset_input_has_button_styling() {
  let tree = make(r#"<body style="margin:0;"><input type="reset" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let input = &root.children[0];
  assert!(input.background.is_some(), "reset should have background color");
}

#[test]
fn button_input_has_button_styling() {
  let tree = make(r#"<body style="margin:0;"><input type="button" value="Click" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let input = &root.children[0];
  assert!(input.background.is_some(), "button should have background color");
}

// ── Submit/Reset default labels (with fonts) ────────────────────────

#[test]
fn submit_shows_default_label() {
  let root = layout_with_fonts(
    r#"<body style="margin:0;"><input type="submit" /></body>"#,
    800.0,
    600.0,
  );
  let input = &root.children[0];
  let run = input.text_run.as_ref().expect("submit should have text run with default label");
  assert_eq!(run.text, "Submit");
}

#[test]
fn reset_shows_default_label() {
  let root = layout_with_fonts(
    r#"<body style="margin:0;"><input type="reset" /></body>"#,
    800.0,
    600.0,
  );
  let input = &root.children[0];
  let run = input.text_run.as_ref().expect("reset should have text run with default label");
  assert_eq!(run.text, "Reset");
}

#[test]
fn submit_value_overrides_default() {
  let root = layout_with_fonts(
    r#"<body style="margin:0;"><input type="submit" value="Go" /></body>"#,
    800.0,
    600.0,
  );
  let input = &root.children[0];
  let run = input.text_run.as_ref().expect("submit should have text run");
  assert_eq!(run.text, "Go");
}

// ── Multiple inputs in a form ───────────────────────────────────────

#[test]
fn form_with_mixed_inputs_has_correct_form_controls() {
  let tree = make(
    r#"<body style="margin:0;">
      <input type="checkbox" />
      <input type="radio" />
      <input type="range" />
      <input type="color" />
      <input type="text" />
      <input type="submit" />
    </body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let mut fcs = Vec::new();
  find_all_form_controls(&root, &mut fcs);
  assert_eq!(fcs.len(), 4, "should have 4 form controls (checkbox, radio, range, color)");
  assert!(matches!(fcs[0].kind, FormControlKind::Checkbox { .. }));
  assert!(matches!(fcs[1].kind, FormControlKind::Radio { .. }));
  assert!(matches!(fcs[2].kind, FormControlKind::Range { .. }));
  assert!(matches!(fcs[3].kind, FormControlKind::Color { .. }));
}

// ── Foreground color for form controls ──────────────────────────────

#[test]
fn checkbox_has_text_color_for_checkmark_rendering() {
  let tree = make(r#"<body style="margin:0;"><input type="checkbox" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("checkbox box");
  assert!(
    b.text_color.is_some(),
    "checkbox should have text_color for checkmark/dot rendering"
  );
}

#[test]
fn radio_has_text_color_for_dot_rendering() {
  let tree = make(r#"<body style="margin:0;"><input type="radio" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("radio box");
  assert!(b.text_color.is_some());
}

// ── Content rect must fit inside parent ─────────────────────────────

#[test]
fn range_content_rect_fits_inside_parent() {
  let tree = make(
    r#"<body style="margin:0;">
      <div style="width:200px;">
        <input type="range" />
      </div>
    </body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let container = &root.children[0];
  let range = find_box_with_form_control(&root).expect("range box");
  let parent_left = container.content_rect.x;
  let parent_right = parent_left + container.content_rect.w;
  assert!(
    range.content_rect.x >= parent_left - 0.5,
    "range content left ({}) should be >= parent left ({})",
    range.content_rect.x, parent_left,
  );
  assert!(
    range.content_rect.x + range.content_rect.w <= parent_right + 0.5,
    "range content right ({}) should be <= parent right ({})",
    range.content_rect.x + range.content_rect.w, parent_right,
  );
}

#[test]
fn checkbox_content_rect_fits_inside_parent() {
  let tree = make(
    r#"<body style="margin:0;">
      <div style="width:200px;">
        <input type="checkbox" />
      </div>
    </body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let container = &root.children[0];
  let cb = find_box_with_form_control(&root).expect("checkbox box");
  let parent_right = container.content_rect.x + container.content_rect.w;
  assert!(
    cb.content_rect.x + cb.content_rect.w <= parent_right + 0.5,
    "checkbox content right ({}) should be <= parent right ({})",
    cb.content_rect.x + cb.content_rect.w, parent_right,
  );
}

#[test]
fn range_has_zero_border_width() {
  let tree = make(r#"<body style="margin:0;"><input type="range" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("range box");
  assert!(
    b.border.left < 0.01 && b.border.right < 0.01,
    "range should have zero border, got left={} right={}",
    b.border.left, b.border.right,
  );
}

#[test]
fn range_in_flex_column_matches_sibling_width() {
  let tree = make(
    r#"<body style="margin:0;">
      <div style="display:flex; flex-direction:column; width:300px;">
        <span>Range</span>
        <input type="range" />
      </div>
    </body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let container = &root.children[0];
  let span = &container.children[0];
  let range = find_box_with_form_control(&root).expect("range box");

  eprintln!("container content: x={} w={}", container.content_rect.x, container.content_rect.w);
  eprintln!("span border:      x={} w={}", span.border_rect.x, span.border_rect.w);
  eprintln!("range border:     x={} w={}", range.border_rect.x, range.border_rect.w);
  eprintln!("range content:    x={} w={}", range.content_rect.x, range.content_rect.w);
  eprintln!("range margin:     l={} r={}", range.margin_rect.x - container.content_rect.x,
    (container.content_rect.x + container.content_rect.w) - (range.margin_rect.x + range.margin_rect.w));
  eprintln!("range border:     l={} r={} t={} b={}", range.border.left, range.border.right, range.border.top, range.border.bottom);
  eprintln!("range margin_rect: x={} w={}", range.margin_rect.x, range.margin_rect.w);
  eprintln!("range border_rect: x={} w={}", range.border_rect.x, range.border_rect.w);
  eprintln!("gap: margin_w - border_w = {}", range.margin_rect.w - range.border_rect.w);
  eprintln!("range padding:    l={} r={}",
    range.content_rect.x - range.border_rect.x - range.border.left,
    (range.border_rect.x + range.border_rect.w) - (range.content_rect.x + range.content_rect.w) - range.border.right);

  let container_left = container.content_rect.x;
  let container_right = container_left + container.content_rect.w;
  assert!(
    (range.content_rect.x - container_left).abs() < 1.0,
    "range content should start at container left: range.x={} container.x={}",
    range.content_rect.x, container_left,
  );
  assert!(
    ((range.content_rect.x + range.content_rect.w) - container_right).abs() < 1.0,
    "range content should end at container right: range.right={} container.right={}",
    range.content_rect.x + range.content_rect.w, container_right,
  );
}

#[test]
fn checkbox_has_zero_border_width() {
  let tree = make(r#"<body style="margin:0;"><input type="checkbox" /></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let b = find_box_with_form_control(&root).expect("checkbox box");
  assert!(
    b.border.left < 0.01 && b.border.right < 0.01,
    "checkbox should have zero border, got left={} right={}",
    b.border.left, b.border.right,
  );
}
