use super::helpers::*;
use crate::*;

fn full_and_incremental(
  html: &str,
  mutate: impl FnOnce(&mut lui_tree::Tree),
  dirty: &[Vec<usize>],
  vw: f32,
  vh: f32,
) -> (LayoutBox, LayoutBox) {
  let mut tree = lui_parser::parse(html);
  let cascaded = lui_style::cascade(&tree);
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut image_cache = ImageCache::default();
  let mut prev = layout_with_text(&cascaded, &mut text_ctx, &mut image_cache, vw, vh, 1.0).unwrap();

  mutate(&mut tree);
  let cascaded2 = lui_style::cascade(&tree);

  let mut text_ctx2 = lui_text::TextContext::new(64);
  let mut image_cache2 = ImageCache::default();
  layout_incremental(
    &cascaded2,
    &mut prev,
    dirty,
    &mut text_ctx2,
    &mut image_cache2,
    vw,
    vh,
    1.0,
    &lui_tree::DefaultLocale,
    None,
    None,
  );
  let incremental = prev;

  let mut text_ctx3 = lui_text::TextContext::new(64);
  let mut image_cache3 = ImageCache::default();
  let full = layout_with_text(&cascaded2, &mut text_ctx3, &mut image_cache3, vw, vh, 1.0).unwrap();

  (incremental, full)
}

fn assert_rects_close(a: &Rect, b: &Rect, label: &str) {
  let eps = 0.5;
  assert!(
    (a.x - b.x).abs() < eps && (a.y - b.y).abs() < eps && (a.w - b.w).abs() < eps && (a.h - b.h).abs() < eps,
    "{label}: rects differ\n  incremental: ({:.1}, {:.1}, {:.1}, {:.1})\n  full:        ({:.1}, {:.1}, {:.1}, {:.1})",
    a.x,
    a.y,
    a.w,
    a.h,
    b.x,
    b.y,
    b.w,
    b.h,
  );
}

fn assert_trees_match(inc: &LayoutBox, full: &LayoutBox, path: &str) {
  assert_rects_close(&inc.margin_rect, &full.margin_rect, &format!("{path}.margin_rect"));
  assert_rects_close(&inc.border_rect, &full.border_rect, &format!("{path}.border_rect"));
  assert_rects_close(&inc.content_rect, &full.content_rect, &format!("{path}.content_rect"));
  assert_eq!(
    inc.children.len(),
    full.children.len(),
    "{path}: child count mismatch ({} vs {})",
    inc.children.len(),
    full.children.len(),
  );
  for (i, (ic, fc)) in inc.children.iter().zip(full.children.iter()).enumerate() {
    assert_trees_match(ic, fc, &format!("{path}[{i}]"));
  }
}

// ── Block flow: clean siblings unchanged ────────────────────────────

#[test]
fn clean_sibling_after_dirty_stays_in_place() {
  let html = r#"<body style="margin:0;">
    <div style="height:50px;"></div>
    <div style="height:50px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(html, |_| {}, &[vec![0]], 800.0, 600.0);
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn dirty_child_height_change_shifts_siblings() {
  let html = r#"<body style="margin:0;">
    <div id="a" style="height:50px;"></div>
    <div id="b" style="height:30px;"></div>
    <div id="c" style="height:20px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:80px;".into());
        }
      }
    },
    &[vec![0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn dirty_child_shrinks_shifts_siblings_up() {
  let html = r#"<body style="margin:0;">
    <div style="height:100px;"></div>
    <div style="height:40px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:30px;".into());
        }
      }
    },
    &[vec![0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn multiple_dirty_paths() {
  let html = r#"<body style="margin:0;">
    <div style="height:40px;"></div>
    <div style="height:40px;"></div>
    <div style="height:40px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:60px;".into());
        }
      }
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[2])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:80px;".into());
        }
      }
    },
    &[vec![0], vec![2]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Auto-height propagation ────────────────────────────────────────

#[test]
fn auto_height_parent_grows_when_child_grows() {
  let html = r#"<body style="margin:0;">
    <div>
      <div style="height:50px;"></div>
    </div>
    <div style="height:30px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:100px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn explicit_height_parent_absorbs_child_change() {
  let html = r#"<body style="margin:0;">
    <div style="height:200px;">
      <div style="height:50px;"></div>
    </div>
    <div style="height:30px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:150px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Flex container fallback ────────────────────────────────────────

#[test]
fn flex_container_with_dirty_child_fully_relayouts() {
  let html = r#"<body style="margin:0;">
    <div style="display:flex; gap:10px;">
      <div style="width:100px; height:50px;"></div>
      <div style="width:100px; height:50px;"></div>
    </div>
    <div style="height:20px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("width:100px; height:80px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn flex_container_height_change_shifts_following_block_sibling() {
  let html = r#"<body style="margin:0;">
    <div style="display:flex;">
      <div style="width:50px; height:40px;"></div>
    </div>
    <div style="height:30px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("width:50px; height:100px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Column flex: recurse like block flow ───────────────────────────

#[test]
fn column_flex_dirty_child_only_relayouts_that_child() {
  let html = r#"<body style="margin:0;">
    <div style="display:flex; flex-direction:column; gap:10px;">
      <div style="height:50px;"></div>
      <div style="height:50px;"></div>
      <div style="height:50px;"></div>
    </div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:80px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn column_flex_nested_dirty_input() {
  let html = r#"<body style="margin:0;">
    <div style="display:flex; flex-direction:column; gap:5px;">
      <div style="display:flex; flex-direction:column;">
        <span>Label</span>
        <input type="text" value="hello" style="display:block; height:30px;" />
      </div>
      <div style="height:40px;"></div>
    </div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0, 1])) {
        if let lui_tree::Element::Input(inp) = &mut node.element {
          inp.value = Some("hello world".into());
        }
      }
    },
    &[vec![0, 0, 1]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn row_flex_dirty_child_fully_relayouts() {
  let html = r#"<body style="margin:0;">
    <div style="display:flex; flex-direction:row; gap:10px;">
      <div style="width:100px; height:50px;"></div>
      <div style="width:100px; height:50px;"></div>
    </div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("width:100px; height:80px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Grid container fallback ────────────────────────────────────────

#[test]
fn grid_container_with_dirty_child_fully_relayouts() {
  let html = r#"<body style="margin:0;">
    <div style="display:grid; grid-template-columns: 1fr 1fr; gap:5px;">
      <div style="height:40px;"></div>
      <div style="height:40px;"></div>
    </div>
    <div style="height:20px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:80px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Width changes ──────────────────────────────────────────────────

#[test]
fn child_width_change_same_height_no_sibling_shift() {
  let html = r#"<body style="margin:0;">
    <div style="width:100px; height:50px;"></div>
    <div style="height:50px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("width:300px; height:50px;".into());
        }
      }
    },
    &[vec![0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn child_width_change_with_height_change() {
  let html = r#"<body style="margin:0;">
    <div style="width:200px; height:40px;"></div>
    <div style="height:30px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("width:500px; height:80px;".into());
        }
      }
    },
    &[vec![0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

#[test]
fn child_width_grows_inside_auto_width_parent() {
  let html = r#"<body style="margin:0;">
    <div>
      <div style="width:100px; height:30px;"></div>
    </div>
    <div style="height:20px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("width:400px; height:30px;".into());
        }
      }
    },
    &[vec![0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── No-op dirty path (same content) ───────────────────────────────

#[test]
fn dirty_path_with_no_actual_change_preserves_layout() {
  let html = r#"<body style="margin:0;">
    <div style="height:50px;"></div>
    <div style="height:50px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(html, |_| {}, &[vec![0]], 800.0, 600.0);
  assert_trees_match(&inc, &full, "root");
}

// ── Deeply nested dirty path ───────────────────────────────────────

#[test]
fn deeply_nested_dirty_propagates_height() {
  let html = r#"<body style="margin:0;">
    <div>
      <div>
        <div style="height:30px;"></div>
      </div>
    </div>
    <div style="height:20px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0, 0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:80px;".into());
        }
      }
    },
    &[vec![0, 0, 0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Empty dirty paths → no crash ───────────────────────────────────

#[test]
fn empty_dirty_paths_is_noop() {
  let html = r#"<body style="margin:0;">
    <div style="height:50px;"></div>
  </body>"#;
  let tree = lui_parser::parse(html);
  let cascaded = lui_style::cascade(&tree);
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut image_cache = ImageCache::default();
  let mut prev = layout_with_text(&cascaded, &mut text_ctx, &mut image_cache, 800.0, 600.0, 1.0).unwrap();

  let original_h = prev.margin_rect.h;
  let changed = layout_incremental(
    &cascaded,
    &mut prev,
    &[],
    &mut text_ctx,
    &mut image_cache,
    800.0,
    600.0,
    1.0,
    &lui_tree::DefaultLocale,
    None,
    None,
  );
  assert!(!changed);
  assert!((prev.margin_rect.h - original_h).abs() < 0.01);
}

// ── Out-of-flow children don't affect cursor ───────────────────────

#[test]
fn absolute_child_does_not_shift_siblings() {
  let html = r#"<body style="margin:0; position:relative;">
    <div style="position:absolute; height:200px;"></div>
    <div style="height:40px;"></div>
    <div style="height:40px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("position:absolute; height:500px;".into());
        }
      }
    },
    &[vec![0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Many siblings: only dirty + following shift ────────────────────

#[test]
fn many_siblings_dirty_in_middle() {
  let html = r#"<body style="margin:0;">
    <div style="height:20px;"></div>
    <div style="height:20px;"></div>
    <div style="height:20px;"></div>
    <div style="height:20px;"></div>
    <div style="height:20px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[2])) {
        if let lui_tree::Element::Div(div) = &mut node.element {
          div.style = Some("height:50px;".into());
        }
      }
    },
    &[vec![2]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}

// ── Form control input value change ────────────────────────────────

#[test]
fn input_value_change_does_not_affect_siblings() {
  let html = r#"<body style="margin:0;">
    <input type="text" value="hello" style="display:block; height:30px;" />
    <div style="height:50px;"></div>
  </body>"#;
  let (inc, full) = full_and_incremental(
    html,
    |tree| {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
        if let lui_tree::Element::Input(inp) = &mut node.element {
          inp.value = Some("hello world".into());
        }
      }
    },
    &[vec![0]],
    800.0,
    600.0,
  );
  assert_trees_match(&inc, &full, "root");
}
