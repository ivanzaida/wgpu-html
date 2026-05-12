use super::helpers::*;
use crate::*;
// ── placeholder rendering ─────────────────────────────────────────────────
//
// The layout-test pipeline doesn't register a font, so
// `shape_text_run` short-circuits to `(None, …)` — `text_run` is
// `None` even when the placeholder code path fires. We therefore
// gate placeholder presence on `text_color`, which the helper
// always sets when it decides to emit a placeholder run.

fn any_box_with_placeholder(b: &LayoutBox) -> bool {
  b.text_color.is_some() || b.children.iter().any(any_box_with_placeholder)
}

fn first_box_with_placeholder(b: &LayoutBox) -> Option<&LayoutBox> {
  if b.text_color.is_some() {
    return Some(b);
  }
  for c in &b.children {
    if let Some(found) = first_box_with_placeholder(c) {
      return Some(found);
    }
  }
  None
}

#[test]
fn input_with_placeholder_attaches_placeholder_run() {
  // An empty `<input>` with a `placeholder` should drive the
  // layout box through the placeholder code path (visible via
  // `text_color` being set even when no font is registered).
  //
  // The UA stylesheet sets `color: fieldtext` (black) on inputs,
  // so the placeholder helper halves the alpha → `[0, 0, 0, 0.5]`.
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="text" placeholder="Type here">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let input = first_box_with_placeholder(&body).expect("input box should carry placeholder color");
  let color = input.text_color.unwrap();
  assert_eq!(color, [0.0, 0.0, 0.0, 0.5]);
}

#[test]
fn input_with_value_suppresses_placeholder() {
  // A non-empty `value` overrides the placeholder; no
  // placeholder color should be set.
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="text" placeholder="Type here" value="actual content">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert!(
    !any_box_with_placeholder(&body),
    "value=\"…\" should suppress placeholder rendering"
  );
}

#[test]
fn input_without_placeholder_has_no_placeholder_run() {
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="text">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert!(!any_box_with_placeholder(&body));
}

#[test]
fn input_type_hidden_does_not_render_placeholder() {
  // `type="hidden"` is `display: none` per UA — even with a
  // placeholder, no placeholder run should be emitted.
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="hidden" placeholder="invisible">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert!(!any_box_with_placeholder(&body));
}

#[test]
fn input_with_empty_placeholder_attribute_has_no_placeholder_run() {
  // `placeholder=""` is empty — no glyphs to shape, no run.
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="text" placeholder="">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert!(!any_box_with_placeholder(&body));
}

#[test]
fn textarea_with_placeholder_attaches_placeholder_run() {
  // Empty `<textarea>` with a `placeholder` attribute drives
  // the placeholder code path the same way.
  let tree = make(
    r#"<body style="margin: 0;">
            <textarea placeholder="A few words..."></textarea>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let ta = first_box_with_placeholder(&body).expect("textarea placeholder");
  assert!(ta.text_color.is_some());
}

#[test]
fn empty_input_content_height_defaults_to_line_height() {
  // An empty `<input>` collapses to zero measured height (no
  // children to lay out), which would put the placeholder text
  // run below the padding box and clip it at the input's bottom
  // border. Layout fills the missing measured height with one
  // line of the cascaded font so the run renders fully inside
  // the box.
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="text" style="border: 0; padding: 0;">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  fn first_input_box(b: &LayoutBox) -> Option<&LayoutBox> {
    if matches!(b.kind, BoxKind::Block) && b.children.is_empty() {
      return Some(b);
    }
    for c in &b.children {
      if let Some(found) = first_input_box(c) {
        return Some(found);
      }
    }
    None
  }
  let input = first_input_box(&body).expect("input box");
  // No font registered → `font_size_px` defaults to 16, so
  // `line-height: normal` lands at ~1.25 × 16 = 20.
  assert!(
    input.content_rect.h >= 16.0,
    "empty input should default to one line of content height (got {})",
    input.content_rect.h
  );
}

#[test]
fn empty_textarea_content_height_defaults_to_line_height() {
  // Same default as `<input>` — an empty textarea would
  // otherwise collapse to zero measured height.
  let tree = make(
    r#"<body style="margin: 0;">
            <textarea style="border: 0; padding: 0;"></textarea>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  fn first_ta(b: &LayoutBox) -> Option<&LayoutBox> {
    if matches!(b.kind, BoxKind::Block) && b.children.is_empty() {
      return Some(b);
    }
    for c in &b.children {
      if let Some(found) = first_ta(c) {
        return Some(found);
      }
    }
    None
  }
  let ta = first_ta(&body).expect("textarea box");
  assert!(
    ta.content_rect.h >= 16.0,
    "empty textarea should default to one line of content height (got {})",
    ta.content_rect.h
  );
}

#[test]
fn placeholder_respects_user_padding_shorthand() {
  // User CSS `padding: 8px 10px` should override the UA's
  // `padding-block: 1px; padding-inline: 2px`, so the input's
  // content_rect is inset by 8/10/8/10. The placeholder shaping
  // uses that content_rect — the box's content edges should
  // therefore match the user-specified padding.
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="text" placeholder="x"
                   style="padding: 8px 10px; border: 0; width: 100px;">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let input = first_box_with_placeholder(&body).expect("input box should carry a placeholder");
  let cr = input.content_rect;
  let br = input.border_rect;
  // border = 0, padding = 8 vertical / 10 horizontal.
  assert!(
    (cr.x - (br.x + 10.0)).abs() < 0.01,
    "left padding 10px not applied: cr.x={} br.x={}",
    cr.x,
    br.x
  );
  assert!(
    (cr.y - (br.y + 8.0)).abs() < 0.01,
    "top padding 8px not applied: cr.y={} br.y={}",
    cr.y,
    br.y
  );
  assert!(
    (cr.w - (br.w - 20.0)).abs() < 0.01,
    "horizontal padding 20px (10+10) not applied: cr.w={} br.w={}",
    cr.w,
    br.w
  );
}

#[test]
fn placeholder_color_uses_cascaded_color_with_half_alpha() {
  // When the cascaded `color` is set, placeholder colour =
  // `color` × alpha 0.5. With CSS `color: red`, alpha is 0.5
  // and the channels track red's linearised RGB.
  let tree = make(
    r#"<body style="margin: 0;">
            <input type="text" placeholder="hint" style="color: red;">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let input = first_box_with_placeholder(&body).expect("input box should carry placeholder color");
  let color = input.text_color.unwrap();
  assert!(
    (color[3] - 0.5).abs() < 1e-4,
    "alpha should be halved (got {})",
    color[3]
  );
  // Linearised red ≈ 1.0; green/blue stay at 0.
  assert!(color[0] > 0.9, "red channel ≈ 1 (got {})", color[0]);
  assert_eq!(color[1], 0.0);
  assert_eq!(color[2], 0.0);
}

#[test]
fn textarea_in_flex_row_does_not_inflate_height() {
  // Regression guard for the "no text after textarea" symptom.
  // A flex row containing `<label>` + `<textarea height: 64px>`
  // should advance the body's block flow by ~64-72px (textarea
  // content + UA padding/border), not by hundreds of pixels —
  // which would push following siblings off-screen.
  let tree = make(
    r#"<body style="margin: 0; padding: 0;">
            <h2 style="font-size: 11px; margin: 0;">First</h2>
            <div style="display: flex; gap: 0;">
                <label>Bio</label>
                <textarea style="min-width: 320px; height: 64px;"></textarea>
            </div>
            <h2 style="font-size: 11px; margin: 0;">Second</h2>
        </body>"#,
  );
  let body = layout(&tree, 1280.0, 720.0).unwrap();
  let kids = &body.children;
  assert_eq!(kids.len(), 3, "body kids: {}", kids.len());
  let row = &kids[1];
  let after = &kids[2];
  assert!(
    after.margin_rect.y < row.margin_rect.y + 200.0,
    "h2 after textarea row sits at y={}, row starts at y={} (delta {} > 200) — \
         textarea row is sized way larger than its 64px height",
    after.margin_rect.y,
    row.margin_rect.y,
    after.margin_rect.y - row.margin_rect.y
  );
}
