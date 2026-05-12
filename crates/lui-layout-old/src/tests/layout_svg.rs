use super::helpers::*;
use crate::*;
#[test]
fn svg_replaced_element_size_from_attributes() {
  // <svg width="100px" height="80px"> should produce a 100×80 layout box
  // and have image data attached (rasterised from the path child).
  let tree = make(
    r#"<body style="margin: 0;">
            <svg width="100px" height="80px" viewBox="0 0 100 80">
              <path d="M0 0 L100 80" fill="black"/>
            </svg>
        </body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let svg_box = &root.children[0];
  assert_eq!(svg_box.border_rect.w, 100.0, "SVG width");
  assert_eq!(svg_box.border_rect.h, 80.0, "SVG height");
  // Should have rasterised image data attached.
  assert!(svg_box.image.is_some(), "SVG should produce image data");
  let img = svg_box.image.as_ref().unwrap();
  assert_eq!(img.width, 100, "raster width matches layout width");
  assert_eq!(img.height, 80, "raster height matches layout height");
}

#[test]
fn svg_inside_flex_card_has_image() {
  // Mirror the demo: SVG inside a flex-column card, with CSS fill targeting path.
  let tree = make(
    r#"<body style="margin: 0;">
          <style>
            .card { display: flex; flex-direction: column; align-items: center; width: 180px; padding: 16px; }
            .fill-orange path { fill: #e07b39; }
          </style>
          <div class="card">
            <svg class="fill-orange" width="160" height="160" viewBox="0 0 1200 1200">
              <path d="M0 0 L1200 1200 L0 1200 Z"/>
            </svg>
          </div>
        </body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  // root(body) → [style, div.card]; style element has no box
  let card = root.children.iter().find(|c| c.border_rect.w > 0.0).expect("card");
  let svg_box = &card.children[0];
  assert!(svg_box.border_rect.w > 0.0, "SVG border_rect has width");
  assert!(svg_box.border_rect.h > 0.0, "SVG border_rect has height");
  // paint.rs checks content_rect, not border_rect
  assert!(
    svg_box.content_rect.w > 0.0,
    "SVG content_rect has width (paint needs this)"
  );
  assert!(
    svg_box.content_rect.h > 0.0,
    "SVG content_rect has height (paint needs this)"
  );
  assert!(
    svg_box.image.is_some(),
    "SVG inside flex card must produce image data; border={:?} content={:?}",
    svg_box.border_rect,
    svg_box.content_rect
  );
  // Verify the rasterized image has non-zero content (not all alpha=0)
  let img = svg_box.image.as_ref().unwrap();
  assert!(img.width > 0 && img.height > 0, "image has dimensions");
  let has_opaque = img.data.chunks_exact(4).any(|p| p[3] > 0);
  assert!(
    has_opaque,
    "rasterised SVG must have at least one non-transparent pixel"
  );
}

// ── tree-row wrapping / overlap regression tests ────────────────

#[test]
fn tree_row_fixed_width_50px_spans_no_overlap() {
  // Reproduces text-shrink.html: .tree-row with width:50px,
  // flex-shrink:0, white-space:nowrap, overflow:hidden.
  // Even when content overflows the row, span children must not
  // overlap each other.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .tree-rows { display: flex; flex-direction: column; width: 600px; }
        .tree-row  { width: 50px; display: flex; align-items: center; height: 18px;
                     flex-shrink: 0; white-space: nowrap; overflow: hidden; }
        .tag  { color: #5DB0D7; }
        .br   { color: #9AA0A6; }
        .atn  { color: #9AA0A6; margin-left: 4px; }
        .atv  { color: #F28B82; }
        .text-node { color: #E8EAED; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="tree-rows">
          <div class="tree-row">
            <span class="br">&lt;</span>
            <span class="tag">div</span>
            <span class="atn"> class</span>
            <span class="br">=</span>
            <span class="atv">"child child-1"</span>
            <span class="br">&gt;</span>
            <span class="text-node">1</span>
            <span class="br">&lt;/</span>
            <span class="tag">div</span>
            <span class="br">&gt;</span>
          </div>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  let body = &root.children[1];
  let rows = &body.children[0];
  let row = &rows.children[0];

  // Row width must be exactly 50px (flex-shrink:0 prevents shrinking).
  assert!(
    (row.border_rect.w - 50.0).abs() < 1.0,
    "tree-row width should be 50px, got {:.1}",
    row.border_rect.w
  );

  // Span children must not overlap each other even though content overflows.
  assert_no_overlap(&row.children);

  // Each span's text box must be clamped to its span width.
  for (i, span) in row.children.iter().enumerate() {
    for child in &span.children {
      if child.text_run.is_some() {
        assert!(
          child.content_rect.w <= span.content_rect.w + 1.0,
          "span {i}: text box w={:.1} exceeds span w={:.1}",
          child.content_rect.w,
          span.content_rect.w
        );
      }
    }
  }

  // Verify span content_rect boundaries: each span's right edge
  // must be ≤ next span's left edge. This is the strongest check
  // against visual overlap.
  for i in 1..row.children.len() {
    let prev_right = row.children[i - 1].content_rect.x + row.children[i - 1].content_rect.w;
    assert!(
      row.children[i].content_rect.x >= prev_right - 0.5,
      "span {i} at x={:.1} overlaps span {} ending at {prev_right:.1}",
      row.children[i].content_rect.x,
      i - 1
    );
  }
}

#[test]
fn tree_row_flex_shrink_zero_preserves_order_in_narrow_container() {
  // When tree-row has flex-shrink:0 and width:50px, each span must
  // start at or after the previous span's right edge, preserving
  // left-to-right order with no backward overlaps.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .tree-row { width: 50px; display: flex; align-items: center; height: 18px;
                    flex-shrink: 0; white-space: nowrap; overflow: hidden; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="tree-row">
          <span>&lt;</span>
          <span>body</span>
          <span class="atn"> class</span>
          <span>=</span>
          <span>"parent"</span>
          <span>&gt;</span>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  let body = &root.children[1];
  let row = &body.children[0];
  assert_no_overlap(&row.children);
}

#[test]
fn tree_rows_column_container_no_vertical_overlap() {
  // Multiple .tree-row elements in a column flex container must not
  // overlap vertically. Each 18px row stacked at 18px intervals.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .tree-rows { display: flex; flex-direction: column; padding: 8px 0; }
        .tree-row  { width: 200px; display: flex; align-items: center; height: 18px;
                     flex-shrink: 0; white-space: nowrap; overflow: hidden; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="tree-rows">
          <div class="tree-row"><span>&lt;body&gt;</span></div>
          <div class="tree-row"><span>&lt;div class="parent"&gt;</span></div>
          <div class="tree-row"><span>&lt;div class="child"&gt;1&lt;/div&gt;</span></div>
          <div class="tree-row"><span>&lt;/div&gt;</span></div>
          <div class="tree-row"><span>&lt;/body&gt;</span></div>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  let body = &root.children[1];
  let rows = &body.children[0];

  // Each row must have height 18px (flex-shrink:0 + explicit height).
  for (i, row) in rows.children.iter().enumerate() {
    assert!(
      (row.border_rect.h - 18.0).abs() < 1.0,
      "row {i}: expected height 18px, got {:.1}",
      row.border_rect.h
    );
  }

  // Rows must not overlap vertically.
  let rows_margin = rows.children.iter().map(|r| r.margin_rect).collect::<Vec<_>>();
  for i in 1..rows_margin.len() {
    let prev_bottom = rows_margin[i - 1].y + rows_margin[i - 1].h;
    assert!(
      rows_margin[i].y >= prev_bottom - 0.5,
      "row {i} starts at y={:.1} but previous row ends at y={prev_bottom:.1}",
      rows_margin[i].y
    );
  }
}

#[test]
fn tree_row_text_box_clamped_when_content_overflows_width() {
  // When a span's text is wider than the tree-row's 50px, the text
  // box inside each span must be clamped to that span's computed
  // width, never exceeding it.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .tree-row { width: 50px; display: flex; align-items: center; height: 18px;
                    flex-shrink: 0; white-space: nowrap; overflow: hidden; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="tree-row">
          <span>this-is-a-very-long-class-name-that-overflows</span>
          <span>still-no-overlap</span>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  let body = &root.children[1];
  let row = &body.children[0];

  // Spans must not overlap.
  assert_no_overlap(&row.children);

  // Each span's text box must be clamped.
  for (i, span) in row.children.iter().enumerate() {
    for child in &span.children {
      if child.text_run.is_some() {
        assert!(
          child.content_rect.w <= span.content_rect.w + 1.0,
          "span {i}: text box w={:.1} exceeds span w={:.1}",
          child.content_rect.w,
          span.content_rect.w
        );
      }
    }
  }
}

#[test]
fn tree_row_overflow_hidden_propagated_to_layout() {
  // Regression: verify overflow:hidden on a flex tree-row is
  // actually carried through to the LayoutBox, so the paint pass
  // can push a clip.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .tree-row { width: 50px; display: flex; align-items: center; height: 18px;
                    flex-shrink: 0; white-space: nowrap; overflow: hidden; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="tree-row">
          <span>hello world this is long text</span>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  let body = &root.children[1];
  let row = &body.children[0];

  // The tree-row itself must have overflow:hidden on both axes.
  assert!(
    row.overflow.clips_any(),
    "tree-row must have overflow:hidden; got x={:?} y={:?}",
    row.overflow.x,
    row.overflow.y
  );
  assert!(
    row.overflow.clips_both(),
    "tree-row must clip both axes with overflow:hidden"
  );

  // Span children have default overflow:visible
  for (i, child) in row.children.iter().enumerate() {
    assert!(
      !child.overflow.clips_any(),
      "span child {i} should have overflow:visible, got {:?} x={:?} y={:?}",
      i,
      child.overflow.x,
      child.overflow.y
    );
  }
}

#[test]
fn tree_row_spans_glyphs_no_overlap_at_pixel_level() {
  let root = layout_with_fonts(
    r#"<html><head><style>
        .tree-row { width: 50px; display: flex; align-items: center; height: 18px;
                    flex-shrink: 0; white-space: nowrap; overflow: hidden; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="tree-row">
          <span>&lt;</span>
          <span>div</span>
          <span> class</span>
          <span>=</span>
          <span>"app-root"</span>
          <span>&gt;</span>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );

  let body = &root.children[1];
  let row = &body.children[0];

  assert_no_overlap(&row.children);

  // Collect per-span glyph bounding rects.  Assign each glyph the
  // span index it came from so we can tell intra-span kerning
  // overlaps from cross-span collisions.
  struct GlyphRect {
    span: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
  }
  let mut glyphs = Vec::new();
  for (span_idx, span) in row.children.iter().enumerate() {
    for child in &span.children {
      if let Some(run) = &child.text_run {
        for g in &run.glyphs {
          glyphs.push(GlyphRect {
            span: span_idx,
            x: child.content_rect.x + g.x,
            y: child.content_rect.y + g.y,
            w: g.w,
            h: g.h,
          });
        }
      }
    }
  }

  assert!(!glyphs.is_empty(), "expected glyphs");

  // Sort by x then y.
  glyphs.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap().then(a.y.partial_cmp(&b.y).unwrap()));

  for i in 1..glyphs.len() {
    let prev = &glyphs[i - 1];
    let curr = &glyphs[i];
    if prev.span == curr.span {
      continue; // intra-span kerning overlap is expected
    }
    let pr = prev.x + prev.w;
    let cr = curr.x + curr.w;
    let pb = prev.y + prev.h;
    let cb = curr.y + curr.h;
    let x_overlap = curr.x < pr - 0.01 && cr > prev.x + 0.01;
    let y_overlap = curr.y < pb - 0.01 && cb > prev.y + 0.01;
    assert!(
      !(x_overlap && y_overlap),
      "span {} glyph at ({:.1},{:.1} {:.1}x{:.1}) overlaps span {} glyph at \
       ({:.1},{:.1} {:.1}x{:.1})",
      curr.span,
      curr.x,
      curr.y,
      curr.w,
      curr.h,
      prev.span,
      prev.x,
      prev.y,
      prev.w,
      prev.h
    );
  }
}

#[test]
fn border_color_falls_back_to_current_color() {
  // `border: 2px solid` without an explicit color must use the
  // element's `color` (currentColor) per CSS spec.
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
        <div style="width: 100px; height: 40px; color: rgb(255, 0, 0);
                    border: 2px solid;"></div>
      </body>"#,
    800.0,
    600.0,
  );
  let div = &root.children[0];
  assert_eq!(div.border.top, 2.0);
  assert_eq!(div.border_colors.top, Some([1.0, 0.0, 0.0, 1.0]));
  assert_eq!(div.border_colors.right, Some([1.0, 0.0, 0.0, 1.0]));
  assert_eq!(div.border_colors.bottom, Some([1.0, 0.0, 0.0, 1.0]));
  assert_eq!(div.border_colors.left, Some([1.0, 0.0, 0.0, 1.0]));
}
