use super::helpers::*;
use crate::*;

/// Walk the layout tree and return true if every text leaf's glyphs
/// fit within its content_rect.  Returns (passed, failures) where
/// each failure is a description.
fn check_glyphs_fit(root: &LayoutBox) -> (bool, Vec<String>) {
  let mut failures = Vec::new();
  check_glyphs_fit_impl(root, &mut Vec::new(), &mut failures);
  (failures.is_empty(), failures)
}

fn check_glyphs_fit_impl(b: &LayoutBox, path: &mut Vec<usize>, out: &mut Vec<String>) {
  if let Some(run) = &b.text_run {
    let h = b.content_rect.h;
    for (i, g) in run.glyphs.iter().enumerate() {
      let bottom = g.y + g.h;
      if bottom > h + 0.01 {
        out.push(format!(
          "text leaf at {:?} glyph {i} ({:?}): y={:.1} h={:.1} bottom={:.1} exceeds content_rect.h={:.1} by {:.1}px",
          path,
          run.text.chars().nth(i),
          g.y,
          g.h,
          bottom,
          h,
          bottom - h,
        ));
      }
    }
  }
  for (i, child) in b.children.iter().enumerate() {
    path.push(i);
    check_glyphs_fit_impl(child, path, out);
    path.pop();
  }
}

#[test]
fn glyphs_within_content_rect_with_real_font() {
  let root = layout_with_fonts(
    r#"<body style="margin:0; font-family:sans-serif; font-size:32px;">
          <div>gyj0ABCDE</div>
        </body>"#,
    800.0,
    600.0,
  );
  let (ok, failures) = check_glyphs_fit(&root);
  assert!(ok, "glyphs extend past content_rect:\n{}", failures.join("\n"));
}

#[test]
fn glyphs_within_content_rect_small_font() {
  let root = layout_with_fonts(
    r#"<body style="margin:0; font-family:sans-serif; font-size:12px;">
          <div>hello world (gy0)</div>
        </body>"#,
    800.0,
    600.0,
  );
  let (ok, failures) = check_glyphs_fit(&root);
  assert!(ok, "glyphs extend past content_rect:\n{}", failures.join("\n"));
}

#[test]
fn glyphs_within_content_rect_flex_row() {
  // Tight flex-row containers are where clipping is most visible.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .row { display:flex; align-items:center; height:40px; }
        .row span { font-size:24px; }
       </style></head>
       <body style="margin:0; font-family:sans-serif;">
         <div class="row">
           <span>gy0px</span>
           <span>ABC</span>
         </div>
       </body></html>"#,
    800.0,
    600.0,
  );
  let (ok, failures) = check_glyphs_fit(&root);
  assert!(
    ok,
    "glyphs extend past content_rect in flex row:\n{}",
    failures.join("\n")
  );
}

#[test]
fn content_rect_at_least_run_height() {
  let root = layout_with_fonts(
    r#"<body style="margin:0; font-family:sans-serif; font-size:16px;">
          <div>The quick brown fox</div>
        </body>"#,
    800.0,
    600.0,
  );
  fn check(b: &LayoutBox) {
    if let Some(run) = &b.text_run {
      assert!(
        b.content_rect.h >= run.height - 0.01,
        "content_rect.h={:.2} < run.height={:.2}",
        b.content_rect.h,
        run.height,
      );
    }
    for child in &b.children {
      check(child);
    }
  }
  check(&root);
}
