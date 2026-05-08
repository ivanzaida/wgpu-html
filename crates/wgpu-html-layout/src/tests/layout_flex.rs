use super::helpers::*;
use crate::*;
// ---------------------------------------------------------------------------
// flex layout
// ---------------------------------------------------------------------------

#[test]
fn flex_row_default_packs_at_start() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 200px; height: 100px;">
            <div style="width: 30px; height: 30px;"></div>
            <div style="width: 40px; height: 30px;"></div>
            <div style="width: 50px; height: 30px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  assert_eq!(kids.len(), 3);
  assert_eq!(kids[0].margin_rect.x, 0.0);
  assert_eq!(kids[1].margin_rect.x, 30.0);
  assert_eq!(kids[2].margin_rect.x, 70.0);
}

#[test]
fn flex_row_justify_center() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; justify-content: center; width: 100px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  // Total used = 40, free = 60, center → start at 30.
  assert_eq!(body.children[0].margin_rect.x, 30.0);
  assert_eq!(body.children[1].margin_rect.x, 50.0);
}

#[test]
fn flex_row_justify_end() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; justify-content: flex-end; width: 100px; height: 50px;">
            <div style="width: 30px; height: 20px;"></div>
            <div style="width: 30px; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.x, 40.0);
  assert_eq!(body.children[1].margin_rect.x, 70.0);
}

#[test]
fn flex_row_justify_space_between() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; justify-content: space-between; width: 120px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
  assert_eq!(xs, vec![0.0, 50.0, 100.0]);
}

#[test]
fn flex_row_justify_space_evenly() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; justify-content: space-evenly; width: 120px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
  assert_eq!(xs, vec![15.0, 50.0, 85.0]);
}

#[test]
fn flex_row_gap_separates_children() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; gap: 8px; width: 200px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
  assert_eq!(xs, vec![0.0, 28.0, 56.0]);
}

#[test]
fn flex_row_align_items_center() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; align-items: center; width: 200px; height: 100px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 60px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.y, 40.0);
  assert_eq!(body.children[1].margin_rect.y, 20.0);
}

#[test]
fn flex_row_align_items_end() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; align-items: flex-end; width: 200px; height: 100px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 60px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.y, 80.0);
  assert_eq!(body.children[1].margin_rect.y, 40.0);
}

#[test]
fn flex_row_align_items_stretch_fills_unspecified_height() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; align-items: stretch; width: 200px; height: 80px;">
            <div style="width: 20px;"></div>
            <div style="width: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].border_rect.h, 80.0);
  assert_eq!(body.children[1].border_rect.h, 80.0);
}

#[test]
fn flex_row_spans_with_text_do_not_overlap() {
  // A flex row containing multiple spans with text content.
  // Each span should get its text's intrinsic width and advance
  // the cursor — no overlap between adjacent spans.
  let body = layout_with_fonts(
    r#"<body style="margin: 0; display: flex; align-items: center; width: 600px; height: 18px; font-family: sans-serif;">
            <span>hello</span>
            <span>world</span>
            <span>test</span>
        </body>"#,
    800.0,
    600.0,
  );
  assert_no_overlap(&body.children);
  // Verify text runs from different spans don't overlap.
  let runs = collect_run_extents(&body);
  assert_runs_no_overlap(&runs);
}

#[test]
fn flex_row_spans_glyph_positions_sequential() {
  // Verifies that the actual glyph x-positions from adjacent spans
  // are sequential (no overlap). This catches bugs where layout
  // boxes are correct but glyphs within them are positioned wrong.
  let body = layout_with_fonts(
    r#"<body style="margin: 0; display: flex; align-items: center; width: 600px; height: 20px; font-family: sans-serif;">
            <span>AB</span>
            <span>CD</span>
        </body>"#,
    800.0,
    600.0,
  );
  let runs = collect_run_extents(&body);
  assert!(runs.len() >= 2, "expected at least 2 text runs, got {}", runs.len());
  assert_runs_no_overlap(&runs);
}

#[test]
fn flex_row_devtools_tree_row_glyphs_no_overlap() {
  // Reproduces the exact devtools tree-row: <div> with class attr.
  // Glyph positions must not overlap across span boundaries.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .row { display: flex; align-items: center; height: 18px; }
        .tag { color: #5DB0D7; }
        .br  { color: #9AA0A6; }
        .atn { color: #9AA0A6; margin-left: 4px; }
        .atv { color: #F28B82; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="row">
          <span class="br">&lt;</span>
          <span class="tag">div</span>
          <span class="atn">class</span>
          <span class="br">=</span>
          <span class="atv">"app-root"</span>
          <span class="br">&gt;</span>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  let body = &root.children[1];
  let row = &body.children[0];
  assert_no_overlap(&row.children);
  let runs = collect_run_extents(row);
  assert!(runs.len() >= 3, "expected text runs, got {}", runs.len());
  assert_runs_no_overlap(&runs);
}

#[test]
fn flex_row_shrunk_spans_text_clipped_to_box() {
  // When a flex row is narrower than the total text width and items
  // shrink (flex-shrink: 1, default), the text box must be clamped
  // to the span's width so the paint clip has correct bounds.
  let body = layout_with_fonts(
    r#"<body style="margin: 0; display: flex; align-items: center; width: 60px; height: 18px;
                    white-space: nowrap; font-family: sans-serif;">
            <span>hello</span>
            <span>world</span>
        </body>"#,
    800.0,
    600.0,
  );
  let span0 = &body.children[0];
  let span1 = &body.children[1];
  let span0_right = span0.content_rect.x + span0.content_rect.w;
  // Span boxes must not overlap.
  assert!(
    span1.content_rect.x >= span0_right - 0.5,
    "span1 starts at {:.1} before span0 ends at {span0_right:.1}",
    span1.content_rect.x
  );
  // Text box inside each span must be clamped to the span width.
  for (i, span) in body.children.iter().enumerate() {
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
fn flex_row_nowrap_shrunk_text_box_clamped() {
  // With white-space:nowrap, text shapes at full width but the text
  // box must be clamped to the container (flex-shrunk) width. This
  // verifies make_text_leaf clamps box_w to max_width_px.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .row { display: flex; align-items: center; height: 18px;
               white-space: nowrap; overflow: hidden; width: 80px; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="row">
          <span>hello</span>
          <span>world</span>
          <span>test</span>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  let body = &root.children[1];
  let row = &body.children[0];
  // Each span's text box must be clamped to the span's width.
  // Glyphs inside the run may extend past (they were shaped at full
  // width) but the paint pass clips them to the text box bounds.
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
  // Span boxes must not overlap.
  assert_no_overlap(&row.children);
}

#[test]
fn flex_row_spans_in_column_flex_tree_row() {
  // Reproduces the devtools tree panel: a column flex container
  // holds multiple flex-row "tree-row" divs, each containing spans
  // with tag names and attributes. The spans must not overlap.
  let root = layout_with_fonts(
    r#"<html><head><style>
        .rows { display: flex; flex-direction: column; width: 600px; height: 400px; }
        .row  { display: flex; align-items: center; height: 18px; white-space: nowrap; overflow: hidden; }
        .tag  { color: #5DB0D7; }
        .br   { color: #9AA0A6; }
        .atn  { color: #9AA0A6; margin-left: 4px; }
        .atv  { color: #F28B82; }
      </style></head>
      <body style="margin: 0; font-family: sans-serif;">
        <div class="rows">
          <div class="row">
            <span class="br">&lt;</span>
            <span class="tag">div</span>
            <span class="atn"> class</span>
            <span class="br">=</span>
            <span class="atv">"app-root"</span>
            <span class="br">&gt;</span>
          </div>
        </div>
      </body></html>"#,
    800.0,
    600.0,
  );
  // Navigate: html > body > .rows > .row
  let body = &root.children[1];
  let rows = &body.children[0];
  let row = &rows.children[0];
  assert_no_overlap(&row.children);
}

#[test]
fn flex_column_direction_stacks_vertically() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-direction: column; width: 100px; height: 200px;">
            <div style="width: 100%; height: 30px;"></div>
            <div style="width: 100%; height: 50px;"></div>
            <div style="width: 100%; height: 70px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let ys: Vec<f32> = body.children.iter().map(|c| c.margin_rect.y).collect();
  assert_eq!(ys, vec![0.0, 30.0, 80.0]);
}

#[test]
fn flex_row_reverse_orders_right_to_left() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-direction: row-reverse; width: 200px; height: 50px;">
            <div style="width: 30px; height: 30px;"></div>
            <div style="width: 30px; height: 30px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.x, 170.0);
  assert_eq!(body.children[1].margin_rect.x, 140.0);
}

#[test]
fn flex_test_html_layout() {
  // Direct port of crates/wgpu-html-demo/html/flex-test.html minus the
  // text content (which we don't render yet).
  let tree = make(
    r#"
        <style>
            body { margin: 0; }
            #parent {
                width: 100px;
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 10px;
            }
            .child { width: 30px; height: 30px; }
        </style>
        <div id="parent">
            <div class="child"></div>
            <div class="child"></div>
            <div class="child"></div>
        </div>
        "#,
  );
  // The parser wraps <style> + <div id="parent"> in a synthetic
  // <body>; the test's `body { margin: 0 }` overrides the UA's
  // `body { margin: 8px }` so coordinates start at the viewport
  // origin. Pull #parent out via its known border-box width.
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let parent = body
    .children
    .iter()
    .find(|c| c.border_rect.w == 120.0)
    .expect("parent div");
  // content-box: width=100 → border-box = 100 + 10*2 padding = 120.
  assert_eq!(parent.border_rect.w, 120.0);
  // 3 × 30 = 90 main used inside content_w=100, free=10 → space-between
  // gaps of 5, items at content_x=10, 45, 80.
  let xs: Vec<f32> = parent.children.iter().map(|c| c.margin_rect.x).collect();
  assert_eq!(xs, vec![10.0, 45.0, 80.0]);
}

// ---------------------------------------------------------------------------
// flex: grow / shrink / basis / wrap / align-content / align-self / order /
//       auto-margin / min-max / row-gap / column-gap
// ---------------------------------------------------------------------------

#[test]
fn flex_grow_splits_remaining_main_equally() {
  // Two grow=1 items inside a 200px row container → each takes 100px.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="flex-grow: 1; height: 20px;"></div>
            <div style="flex-grow: 1; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  assert!((kids[0].border_rect.w - 100.0).abs() < 0.01);
  assert!((kids[1].border_rect.w - 100.0).abs() < 0.01);
  assert!((kids[1].margin_rect.x - 100.0).abs() < 0.01);
}

#[test]
fn flex_grow_weighted_by_factor() {
  // grow ratios 1 : 2 split a 300px row as 100 : 200.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 300px; height: 50px;">
            <div style="flex-grow: 1; height: 20px;"></div>
            <div style="flex-grow: 2; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  assert!((kids[0].border_rect.w - 100.0).abs() < 0.01);
  assert!((kids[1].border_rect.w - 200.0).abs() < 0.01);
}

#[test]
fn flex_basis_overrides_width_for_main_size() {
  // flex-basis takes over from width for main-axis sizing.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 400px; height: 50px;">
            <div style="width: 200px; flex-basis: 100px; height: 20px; flex-grow: 0; flex-shrink: 0;"></div>
            <div style="width: 100px; flex-grow: 0; flex-shrink: 0; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  // flex-basis: 100px → first item 100, second still 100, total 200.
  assert!((body.children[0].border_rect.w - 100.0).abs() < 0.01);
  assert!((body.children[1].margin_rect.x - 100.0).abs() < 0.01);
}

#[test]
fn flex_shorthand_one_value_is_grow_with_zero_basis() {
  // `flex: 1` → grow=1, shrink=1, basis=0. Two items split the
  // entire 200px container even though they have no explicit width.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="flex: 1; height: 20px;"></div>
            <div style="flex: 1; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert!((body.children[0].border_rect.w - 100.0).abs() < 0.01);
  assert!((body.children[1].border_rect.w - 100.0).abs() < 0.01);
}

#[test]
fn flex_column_auto_height_flex_one_keeps_content_height() {
  // `flex: 1` expands to a 0% basis. In an auto-height column flex
  // container that percentage is indefinite, so the item must keep
  // its natural content height instead of receiving a 0px height
  // override from flex layout.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-direction: column; width: 120px;">
            <div style="flex: 1; padding: 10px;">
                <div style="height: 40px;"></div>
            </div>
            <div style="height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let first = &body.children[0];
  let second = &body.children[1];
  assert!((first.border_rect.h - 60.0).abs() < 0.01);
  assert!((second.margin_rect.y - 60.0).abs() < 0.01);
}

#[test]
fn flex_column_block_child_stacks_its_own_children() {
  // A block-level div (no explicit display) inside a column flex container
  // should compute its content height from its block-flow children.
  // This reproduces a devtools bug where nested block children overlap.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-direction: column; width: 200px;">
            <div>
                <div style="display: flex; height: 22px;"></div>
                <div style="display: flex; height: 18px;"></div>
                <div style="display: flex; height: 18px;"></div>
            </div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let wrapper = &body.children[0];
  assert!(
    (wrapper.border_rect.h - 58.0).abs() < 0.01,
    "wrapper height should be 58, got {}",
    wrapper.border_rect.h
  );
  let ys: Vec<f32> = wrapper
    .children
    .iter()
    .map(|c| c.margin_rect.y - wrapper.content_rect.y)
    .collect();
  assert_eq!(
    ys,
    vec![0.0, 22.0, 40.0],
    "children should stack at y=0, 22, 40; got {ys:?}"
  );
}

#[test]
fn flex_column_block_child_with_border_stacks_children() {
  // Same as above but the wrapper has a border-bottom (matching
  // the devtools style-group wrapper). The border adds to the
  // wrapper's box height but must not affect child stacking.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-direction: column; width: 200px;">
            <div style="border-bottom: 1px solid #333;">
                <div style="display: flex; height: 22px;"></div>
                <div style="display: flex; height: 18px;"></div>
                <div style="display: flex; height: 18px;"></div>
            </div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let wrapper = &body.children[0];
  // Content = 58, border-bottom = 1 → border_rect.h = 59
  assert!(
    (wrapper.border_rect.h - 59.0).abs() < 0.01,
    "wrapper border_rect.h should be 59, got {}",
    wrapper.border_rect.h
  );
  let ys: Vec<f32> = wrapper
    .children
    .iter()
    .map(|c| c.margin_rect.y - wrapper.content_rect.y)
    .collect();
  assert_eq!(
    ys,
    vec![0.0, 22.0, 40.0],
    "children should stack at y=0, 22, 40; got {ys:?}"
  );
}

#[test]
fn flex_column_block_child_with_stylesheet_classes() {
  // The devtools uses a linked stylesheet for child styles.
  // Verify that CSS-class-driven display:flex children inside a
  // block wrapper inside a flex column stack correctly.
  let tree = {
    let mut t = wgpu_html_parser::parse(
      r#"<html><head></head>
         <body style="margin: 0; display: flex; flex-direction: column; width: 300px;">
           <div style="border-bottom: 1px solid #333;">
             <div class="hdr"><span>Title</span></div>
             <div class="row"><span>prop</span><span>value</span></div>
             <div class="row"><span>prop</span><span>value</span></div>
             <div class="end">}</div>
           </div>
         </body></html>"#,
    );
    t.register_linked_stylesheet(
      "test.css",
      r#"
        .hdr { display: flex; align-items: center; height: 22px; padding: 0 12px; }
        .row { display: flex; align-items: center; height: 18px; padding: 0 12px 0 28px; }
        .end { display: flex; align-items: center; height: 18px; padding: 0 12px; }
      "#,
    );
    wgpu_html_style::cascade(&t)
  };
  let root = layout(&tree, 800.0, 600.0).unwrap();
  // html → head + body. body is the second child (index 1).
  let body = &root.children[1];
  let wrapper = &body.children[0];
  // Content height = 22 + 18 + 18 + 18 = 76, border-bottom = 1 → 77
  assert!(
    (wrapper.border_rect.h - 77.0).abs() < 0.01,
    "wrapper border_rect.h should be 77, got {}",
    wrapper.border_rect.h
  );
  // Children should stack vertically.
  let ys: Vec<f32> = wrapper
    .children
    .iter()
    .map(|c| c.margin_rect.y - wrapper.content_rect.y)
    .collect();
  assert_eq!(ys, vec![0.0, 22.0, 40.0, 58.0], "children should stack; got {ys:?}");
}

#[test]
fn flex_column_deeply_nested_block_child_stacks() {
  // Reproduces the full devtools nesting: multiple flex layers
  // around a block wrapper containing display:flex children.
  let tree = {
    let mut t = wgpu_html_parser::parse(
      r#"<html><head></head>
         <body style="margin:0; display:flex; flex-direction:column;">
           <div style="display:flex; flex-grow:1;">
             <div style="display:flex; flex-direction:column; flex-grow:1;">
               <div class="sc">
                 <div style="border-bottom: 1px solid #333;">
                   <div class="hdr"><span>Layout</span></div>
                   <div class="row"><span>display</span><span>flex</span></div>
                   <div class="row"><span>width</span><span>200px</span></div>
                   <div class="end">}</div>
                 </div>
               </div>
             </div>
           </div>
         </body></html>"#,
    );
    t.register_linked_stylesheet(
      "test.css",
      r#"
        .sc  { display: flex; flex-direction: column; flex-grow: 1; overflow: auto; }
        .hdr { display: flex; align-items: center; height: 22px; padding: 0 12px; }
        .row { display: flex; align-items: center; height: 18px; padding: 0 28px; }
        .end { display: flex; align-items: center; height: 18px; padding: 0 12px; }
      "#,
    );
    wgpu_html_style::cascade(&t)
  };
  let root = layout(&tree, 800.0, 600.0).unwrap();
  // html > body > outer-flex > inner-col-flex > .sc > wrapper
  let body = &root.children[1];
  let outer = &body.children[0];
  let col = &outer.children[0];
  let sc = &col.children[0];
  let wrapper = &sc.children[0];
  let content_h = 22.0 + 18.0 + 18.0 + 18.0; // 76
  assert!(
    (wrapper.content_rect.h - content_h).abs() < 0.5,
    "wrapper content_rect.h should be ~{content_h}, got {}",
    wrapper.content_rect.h
  );
  // Children within the wrapper should stack vertically.
  let ys: Vec<f32> = wrapper
    .children
    .iter()
    .map(|c| c.margin_rect.y - wrapper.content_rect.y)
    .collect();
  assert!(
    (ys[1] - 22.0).abs() < 0.5 && (ys[2] - 40.0).abs() < 0.5,
    "children should stack; got {ys:?}"
  );
}

#[test]
fn flex_column_nested_flex_column_many_children() {
  // A flex-column child inside a flex-column parent with many
  // grandchildren. The .rule class is display:flex; flex-direction:column.
  // Matches the devtools structure exactly.
  //
  // CRITICAL: body has height:100% so the flex chain has DEFINITE
  // main-axis sizes at every level, matching the real devtools
  // rendering where the viewport provides a fixed height.
  let tree = {
    let mut t = wgpu_html_parser::parse(
      r#"<html><head></head>
         <body style="margin:0; display:flex; flex-direction:column; height:600px;">
           <div style="display:flex; flex-grow:1;">
             <div style="display:flex; flex-direction:column; flex-grow:1;">
               <div class="sc">
                 <div class="rule">
                   <div class="hdr"><span>Layout</span></div>
                   <div class="row"><span>a</span><span>b</span></div>
                   <div class="row"><span>c</span><span>d</span></div>
                   <div class="row"><span>e</span><span>f</span></div>
                   <div class="row"><span>g</span><span>h</span></div>
                   <div class="row"><span>i</span><span>j</span></div>
                   <div class="row"><span>k</span><span>l</span></div>
                   <div class="row"><span>m</span><span>n</span></div>
                   <div class="row"><span>o</span><span>p</span></div>
                   <div class="end">}</div>
                 </div>
               </div>
             </div>
           </div>
         </body></html>"#,
    );
    t.register_linked_stylesheet(
      "test.css",
      r#"
        .sc   { display:flex; flex-direction:column; flex-grow:1; overflow:auto; }
        .rule { display:flex; flex-direction:column; border-bottom:1px solid #333; flex-shrink:0; }
        .hdr  { display:flex; align-items:center; height:22px; padding:0 12px; }
        .row  { display:flex; align-items:center; height:18px; padding:0 12px 0 28px; }
        .end  { display:flex; align-items:center; height:18px; padding:0 12px; }
      "#,
    );
    wgpu_html_style::cascade(&t)
  };
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let body = &root.children[1];
  let outer = &body.children[0];
  let col = &outer.children[0];
  let sc = &col.children[0];
  let rule = &sc.children[0];
  // 22 + 8*18 + 18 = 184 content + 1 border = 185 border_rect
  let expected_content = 22.0 + 8.0 * 18.0 + 18.0;
  assert!(
    (rule.content_rect.h - expected_content).abs() < 0.5,
    "rule content_rect.h should be ~{expected_content}, got {}",
    rule.content_rect.h
  );
  // Check all 10 children stack
  let ys: Vec<f32> = rule
    .children
    .iter()
    .map(|c| c.margin_rect.y - rule.content_rect.y)
    .collect();
  let mut expected_ys = vec![0.0, 22.0];
  for i in 2..10 {
    expected_ys.push(22.0 + (i - 1) as f32 * 18.0);
  }
  for (i, (actual, expected)) in ys.iter().zip(expected_ys.iter()).enumerate() {
    assert!(
      (actual - expected).abs() < 0.5,
      "child {i}: expected y={expected}, got {actual}; all ys={ys:?}"
    );
  }
}

#[test]
fn flex_shrink_reduces_overflowing_items() {
  // Three items 100px each in a 200px container with shrink=1
  // (default). Total would be 300; each item shrinks by 100/300 of
  // its base size → final 200/3 ≈ 66.67 each.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="width: 100px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let expected = 200.0_f32 / 3.0;
  for child in &body.children {
    assert!(
      (child.border_rect.w - expected).abs() < 0.1,
      "got {} expected {}",
      child.border_rect.w,
      expected
    );
  }
}

#[test]
fn flex_min_width_floors_shrunk_item() {
  // Without min-width, three 100px items would all shrink to 66.67 in
  // a 200px container. `min-width: 80px` on the first floors it.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="width: 100px; min-width: 80px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert!((body.children[0].border_rect.w - 80.0).abs() < 0.05);
  // Remaining 120 split between two items → 60 each.
  assert!((body.children[1].border_rect.w - 60.0).abs() < 0.5);
  assert!((body.children[2].border_rect.w - 60.0).abs() < 0.5);
}

#[test]
fn flex_max_width_caps_grown_item() {
  // grow=1 on both, but `max-width: 80px` caps the first.
  // Remaining free space goes to the second.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="flex-grow: 1; max-width: 80px; height: 20px;"></div>
            <div style="flex-grow: 1; height: 20px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert!((body.children[0].border_rect.w - 80.0).abs() < 0.5);
  assert!((body.children[1].border_rect.w - 120.0).abs() < 0.5);
}

#[test]
fn flex_wrap_breaks_to_new_line() {
  // Three 80px items in a 200px wrap row → two lines: [80, 80] and [80].
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap; width: 200px;">
            <div style="width: 80px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 80px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 80px; height: 30px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  // First two on line 1 (y=0).
  assert_eq!(kids[0].margin_rect.y, 0.0);
  assert_eq!(kids[1].margin_rect.y, 0.0);
  // Third wraps to line 2 (y=30).
  assert_eq!(kids[2].margin_rect.y, 30.0);
  // Body content height = both lines' max cross sizes = 60.
  assert_eq!(body.content_rect.h, 60.0);
}

#[test]
fn flex_column_wrap_with_indefinite_height_stays_single_line() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-direction: column; flex-wrap: wrap; width: 120px;">
            <div style="width: 20px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 20px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 20px; height: 30px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.y, 0.0);
  assert_eq!(body.children[1].margin_rect.y, 30.0);
  assert_eq!(body.children[2].margin_rect.y, 60.0);
  assert_eq!(body.children[0].margin_rect.x, 0.0);
  assert_eq!(body.children[1].margin_rect.x, 0.0);
  assert_eq!(body.children[2].margin_rect.x, 0.0);
  assert_eq!(body.content_rect.h, 90.0);
}

#[test]
fn flex_percent_cross_size_with_indefinite_cross_disables_stretch_but_lays_out_as_auto() {
  let tree = make(
    r#"<body style="margin: 0; display: flex; align-items: stretch; width: 200px;">
            <div style="width: 20px; height: 50%; padding: 5px;"></div>
            <div style="width: 20px; height: 40px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].border_rect.h, 10.0);
  assert_eq!(body.children[1].border_rect.h, 40.0);
}

#[test]
fn flex_align_self_overrides_align_items() {
  // align-items: center; one item overrides with align-self: flex-end.
  let tree = make(
    r#"<body style="margin: 0; display: flex; align-items: center;
                          width: 200px; height: 100px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px; align-self: flex-end;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.y, 40.0); // centered
  assert_eq!(body.children[1].margin_rect.y, 80.0); // pinned to bottom
}

#[test]
fn flex_align_content_center_with_two_lines() {
  // Two lines of 30px each in a 100px container → free 40, center
  // → first line at y=20, second at y=50.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center;
                          width: 100px; height: 100px;">
            <div style="width: 100px; height: 30px;"></div>
            <div style="width: 100px; height: 30px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.y, 20.0);
  assert_eq!(body.children[1].margin_rect.y, 50.0);
}

#[test]
fn flex_auto_margin_main_axis_pushes_to_end() {
  // `margin-left: auto` on the second item shoves it to the right.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="width: 30px; height: 20px;"></div>
            <div style="width: 30px; height: 20px; margin-left: auto;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.x, 0.0);
  // First item used 30, free = 200 - 60 = 140 → second at 30+140 = 170.
  assert_eq!(body.children[1].margin_rect.x, 170.0);
}

#[test]
fn flex_order_reorders_visual_layout() {
  // order: -1 puts the third item in front of the first two.
  let tree = make(
    r#"<body style="margin: 0; display: flex; width: 300px; height: 50px;">
            <div style="width: 50px; height: 20px;"></div>
            <div style="width: 50px; height: 20px;"></div>
            <div style="width: 50px; height: 20px; order: -1;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  // Source order is preserved on the layout tree (so hit-testing
  // stays consistent), but visual main-axis positions reflect the
  // re-ordered placement.
  let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
  // The third source-order child (order=-1) is laid out first → x=0.
  assert_eq!(xs[2], 0.0);
  assert_eq!(xs[0], 50.0);
  assert_eq!(xs[1], 100.0);
}

#[test]
fn flex_row_gap_and_column_gap_independent() {
  // row-gap (cross axis) and column-gap (main axis) act on
  // different axes when wrap is on.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          row-gap: 10px; column-gap: 5px;
                          width: 100px;">
            <div style="width: 40px; height: 20px; flex-shrink: 0;"></div>
            <div style="width: 40px; height: 20px; flex-shrink: 0;"></div>
            <div style="width: 40px; height: 20px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  // Line 1: items at x=0, x=45 (40 + 5 column-gap), but third
  // wraps because 40 + 5 + 40 + 5 + 40 = 130 > 100 → line 2.
  assert_eq!(kids[0].margin_rect.x, 0.0);
  assert_eq!(kids[1].margin_rect.x, 45.0);
  // Third is on line 2 with row-gap 10 below 20px line 1 → y=30.
  assert_eq!(kids[2].margin_rect.y, 30.0);
}

// --------------------------------------------------------------------------
// Cross-axis alignment regressions: a `flex-wrap: wrap` container whose
// items happen to fit on one line is *single-line* per CSS-Flex-1 §9.4
// step 15 — `align-content` has no effect there. These tests pin that
// behaviour so the `flex-grow.html` `.row-wrap` case (6×120 px items
// in a wide viewport, single line, `align-content: center`) doesn't
// silently start centering.
// --------------------------------------------------------------------------

#[test]
fn flex_wrap_single_line_ignores_align_content_per_spec() {
  // `flex-wrap: wrap` is set, but the items fit in one line, so the
  // container is single-line. `align-content: center` must be a
  // no-op: items stay at the top of the 160px-tall line (default
  // align-items: stretch + explicit item height ≡ flex-start).
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center;
                          width: 800px; height: 160px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  // All three items fit on one line → align-content is dropped.
  for c in &body.children {
    assert_eq!(c.margin_rect.y, 0.0, "single-line items pin to start");
    assert_eq!(c.margin_rect.h, 40.0, "explicit height is preserved");
  }
}

#[test]
fn flex_wrap_single_line_align_items_center_does_center() {
  // The fix for the flex-grow demo's 3rd row: pair the wrap
  // container with `align-items: center` so each item centers
  // within its (single) line. Same items as above; line cross size
  // = container height = 160; items 40px tall → centered y = 60.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-items: center;
                          width: 800px; height: 160px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  for c in &body.children {
    assert_eq!(
      c.margin_rect.y, 60.0,
      "align-items: center centers each item in the single line"
    );
  }
}

#[test]
fn flex_wrap_actually_wraps_when_container_too_narrow() {
  // Drop the container width so 6 items of 120px each can't all
  // fit on one line. Now the container is multi-line and
  // `align-content: center` distributes free cross space.
  //
  // Container 400px wide, 8px gap → 3 items per line
  // (3*120 + 2*8 = 376 ≤ 400). 6 items → 2 lines.
  // total_lines_cross = 2 * 40 (line cross) + 1 * 8 (cross-gap) = 88.
  // Container cross = 200 → free = 112 → center start offset = 56.
  // Line 1 at y=56, line 2 at y = 56 + 40 + 8 = 104.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center; gap: 8px;
                          width: 400px; height: 200px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  for c in &kids[0..3] {
    assert_eq!(c.margin_rect.y, 56.0);
  }
  for c in &kids[3..6] {
    assert_eq!(c.margin_rect.y, 104.0);
  }
}

#[test]
fn flex_align_self_center_overrides_default_alignment_on_single_line() {
  // Even on a single-line container with `align-content: center`
  // (which is ignored), an item with `align-self: center` still
  // centers itself per `align-self`. Documents that the per-item
  // override route works regardless of `align-content`.
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center;
                          width: 800px; height: 160px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;
                          align-self: center;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  assert_eq!(kids[0].margin_rect.y, 0.0, "default keeps top alignment");
  assert_eq!(kids[1].margin_rect.y, 60.0, "align-self: center wins");
  assert_eq!(kids[2].margin_rect.y, 0.0, "third stays at top");
}

#[test]
fn flex_wrap_no_height_does_not_apply_align_content() {
  // Multi-line container with *no* explicit cross size: `align-content`
  // has nothing to distribute. Lines pack to start regardless of
  // the value, per CSS-Flex-1 §9.6 ("only meaningful with definite
  // cross size").
  let tree = make(
    r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center; gap: 0px;
                          width: 200px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  // Two lines (one item each), no extra cross space → both pack
  // to start: y=0 and y=40.
  assert_eq!(body.children[0].margin_rect.y, 0.0);
  assert_eq!(body.children[1].margin_rect.y, 40.0);
}

#[test]
fn img_html_width_height_respected_in_flex_row() {
  // <img width="64" height="64"> inside a flex row should produce
  // a 64×64 box — HTML attributes must not be overridden by
  // align-items: stretch (the default).
  let tree = make(
    r#"<body style="margin: 0; display: flex; gap: 16px;">
            <img width="64" height="64" src="https://upload.wikimedia.org/wikipedia/commons/4/47/PNG_transparency_demonstration_1.png?_=20240708155759">
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let img = &body.children[0];
  assert_eq!(img.content_rect.w, 64.0, "img width should be 64");
  assert_eq!(img.content_rect.h, 64.0, "img height should be 64");
}

#[test]
fn img_html_width_height_not_stretched_by_taller_sibling() {
  // When a flex row has a taller sibling, the img with HTML
  // width/height should NOT stretch to the line height.
  let tree = make(
    r#"<body style="margin: 0; display: flex; gap: 16px;">
            <img width="64" height="64" src="https://upload.wikimedia.org/wikipedia/commons/4/47/PNG_transparency_demonstration_1.png?_=20240708155759">
            <div style="width: 100px; height: 200px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let img = &body.children[0];
  assert_eq!(img.content_rect.w, 64.0, "img width should be 64");
  assert_eq!(
    img.content_rect.h, 64.0,
    "img height should remain 64, not stretched to 200"
  );
}

#[test]
fn img_no_attributes_stretches_in_flex_row() {
  // Without HTML width/height, an img with no loaded data should
  // stretch on the cross axis (default align-items: stretch).
  let tree = make(
    r#"<body style="margin: 0; display: flex;">
            <img src="https://upload.wikimedia.org/wikipedia/commons/4/47/PNG_transparency_demonstration_1.png?_=20240708155759">
            <div style="width: 100px; height: 200px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let img = &body.children[0];
  assert_eq!(
    img.content_rect.h, 200.0,
    "img without attrs should stretch to line height"
  );
}

#[test]
fn img_html_attrs_in_nested_flex() {
  // Matches the demo structure: body > div.row(flex) > img
  let tree = make(
    r#"<body style="margin: 0;">
            <div style="display: flex; gap: 16px; padding: 16px;">
                <img width="64" height="64" src="https://example.com/img.png">
            </div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let row = &body.children[0];
  let img = &row.children[0];
  assert_eq!(img.content_rect.w, 64.0, "img width should be 64");
  assert_eq!(img.content_rect.h, 64.0, "img height should be 64");
}

#[test]
fn img_html_attrs_respected_after_image_loads() {
  // Simulate what happens after the image loads: layout is called
  // with an ImageCache that has the image available. The intrinsic
  // size is large (e.g. 800×600) but HTML attrs say 64×64.
  let html = r#"<body style="margin: 0;">
      <div style="display: flex; gap: 16px; padding: 16px;">
          <img width="64" height="64" src="https://upload.wikimedia.org/wikipedia/commons/4/47/PNG_transparency_demonstration_1.png?_=20240708155759">
      </div>
  </body>"#;
  let tree = wgpu_html_parser::parse(html);
  let cascaded = wgpu_html_style::cascade(&tree);
  let mut text_ctx = wgpu_html_text::TextContext::new(64);
  let mut io = AssetIo::new(wgpu_html_assets::blocking::BlockingFetcher::new());

  // First pass: image is Pending
  let body = layout_with_text(&cascaded, &mut text_ctx, &mut io, 800.0, 600.0, 1.0).unwrap();
  let img = &body.children[0].children[0];
  assert_eq!(img.content_rect.w, 64.0, "first pass: width=64");
  assert_eq!(img.content_rect.h, 64.0, "first pass: height=64");

  // Verify the parser correctly parsed width/height attributes
  let root = tree.root.as_ref().unwrap();
  let flex_div = &root.children[0];
  let img_node = &flex_div.children[0];
  match &img_node.element {
    wgpu_html_tree::Element::Img(img_el) => {
      assert_eq!(img_el.width, Some(64), "parsed img.width");
      assert_eq!(img_el.height, Some(64), "parsed img.height");
    }
    _ => panic!("expected Img element"),
  }

  // Wait for the image to load
  std::thread::sleep(std::time::Duration::from_secs(3));

  // Second pass: image is loaded (intrinsic size likely > 64)
  let body = layout_with_text(&cascaded, &mut text_ctx, &mut io, 800.0, 600.0, 1.0).unwrap();
  let img = &body.children[0].children[0];
  assert_eq!(img.content_rect.w, 64.0, "after load: width must still be 64");
  assert_eq!(img.content_rect.h, 64.0, "after load: height must still be 64");

  // Verify the image data is present and the box truly has
  // content_rect matching what the renderer will use.
  let img_data = img.image.as_ref().expect("image should be loaded");
  assert!(
    img_data.width > 64 || img_data.height > 64,
    "intrinsic image should be larger than 64 to prove layout constrains it"
  );
  // Even though image data is large, the layout box content_rect
  // is 64×64 — the renderer paints within content_rect.
  assert_eq!(img.content_rect.w, 64.0);
  assert_eq!(img.content_rect.h, 64.0);
}

// --------------------------------------------------------------------------
// Gap included in intrinsic width
// --------------------------------------------------------------------------

#[test]
fn flex_row_gap_spaces_children_correctly() {
  let html = r#"<div style="display: flex; gap: 20px; width: 200px;"><div style="width: 40px; height: 40px;"></div><div style="width: 40px; height: 40px;"></div><div style="width: 40px; height: 40px;"></div></div>"#;
  let cascaded = super::helpers::make(html);
  let root = layout(&cascaded, 800.0, 600.0).unwrap();
  fn find_flex(b: &LayoutBox) -> Option<&LayoutBox> {
    if b.children.len() == 3 && b.children[0].content_rect.w > 0.0 {
      return Some(b);
    }
    b.children.iter().find_map(find_flex)
  }
  let flex = find_flex(&root).expect("should find flex container with 3 children");
  let c0 = &flex.children[0];
  let c1 = &flex.children[1];
  let c2 = &flex.children[2];
  let gap_01 = c1.content_rect.x - (c0.content_rect.x + c0.content_rect.w);
  let gap_12 = c2.content_rect.x - (c1.content_rect.x + c1.content_rect.w);
  assert!(
    (gap_01 - 20.0).abs() < 1.0,
    "gap between child 0 and 1 should be 20px, got {gap_01}"
  );
  assert!(
    (gap_12 - 20.0).abs() < 1.0,
    "gap between child 1 and 2 should be 20px, got {gap_12}"
  );
}
