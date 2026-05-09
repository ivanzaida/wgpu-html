use crate::*;

pub(crate) fn first_child(b: &LayoutBox) -> &LayoutBox {
  b.children.first().expect("expected a child")
}

pub(crate) fn make(html: &str) -> CascadedTree {
  wgpu_html_style::cascade(&wgpu_html_parser::parse(html))
}

pub(crate) fn layout_scaled(tree: &CascadedTree, viewport_w: f32, viewport_h: f32, scale: f32) -> LayoutBox {
  let mut text_ctx = wgpu_html_text::TextContext::new(64);
  let mut image_cache = AssetIo::new(wgpu_html_assets::blocking::BlockingFetcher::new());
  layout_with_text(tree, &mut text_ctx, &mut image_cache, viewport_w, viewport_h, scale).unwrap()
}

/// Layout with system fonts registered so text has real widths.
pub(crate) fn layout_with_fonts(html: &str, viewport_w: f32, viewport_h: f32) -> LayoutBox {
  let mut tree = wgpu_html_parser::parse(html);
  tree.register_system_fonts("sans-serif");
  let cascaded = wgpu_html_style::cascade(&tree);
  let mut text_ctx = wgpu_html_text::TextContext::new(64);
  text_ctx.sync_fonts(&tree.fonts);
  let mut image_cache = AssetIo::new(wgpu_html_assets::blocking::BlockingFetcher::new());
  layout_with_text(&cascaded, &mut text_ctx, &mut image_cache, viewport_w, viewport_h, 1.0).unwrap()
}

pub(crate) fn synthetic_text_layout() -> LayoutBox {
  let r = Rect::new(10.0, 20.0, 100.0, 24.0);
  LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: CornerRadii::zero(),
    border: Insets::zero(),
    border_colors: BorderColors::default(),
    border_styles: BorderStyles::default(),
    border_radius: CornerRadii::zero(),
    kind: BoxKind::Text,
    text_run: Some(ShapedRun {
      glyphs: vec![
        PositionedGlyph {
          x: 0.0,
          y: 0.0,
          w: 10.0,
          h: 20.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.0, 0.0, 0.0, 1.0],
        },
        PositionedGlyph {
          x: 10.0,
          y: 0.0,
          w: 10.0,
          h: 20.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.0, 0.0, 0.0, 1.0],
        },
        PositionedGlyph {
          x: 20.0,
          y: 0.0,
          w: 10.0,
          h: 20.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.0, 0.0, 0.0, 1.0],
        },
      ],
      lines: vec![wgpu_html_text::ShapedLine {
        top: 0.0,
        height: 20.0,
        glyph_range: (0, 3),
      }],
      glyph_chars: vec![],
      text: "abc".to_string(),
      byte_boundaries: wgpu_html_text::utf8_boundaries("abc"),
      width: 30.0,
      height: 20.0,
      ascent: 16.0,
    }),
    text_color: Some([0.0, 0.0, 0.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes::visible(),
    resize: Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: Cursor::Default,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: LuiProperties::default(),
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  }
}

/// Collect per-run glyph extents: (run_start, run_end) for each text run.
pub(crate) fn collect_run_extents(root: &LayoutBox) -> Vec<(f32, f32)> {
  let mut runs = Vec::new();
  collect_run_extents_recursive(root, &mut runs);
  runs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
  runs
}

fn collect_run_extents_recursive(b: &LayoutBox, out: &mut Vec<(f32, f32)>) {
  if let Some(run) = &b.text_run {
    if !run.glyphs.is_empty() {
      let first_x = b.content_rect.x + run.glyphs.first().unwrap().x;
      let last = run.glyphs.last().unwrap();
      let last_end = b.content_rect.x + last.x + last.w;
      out.push((first_x, last_end));
    }
  }
  for child in &b.children {
    collect_run_extents_recursive(child, out);
  }
}

/// Assert that text runs from different spans don't overlap.
pub(crate) fn assert_runs_no_overlap(runs: &[(f32, f32)]) {
  for i in 1..runs.len() {
    let (_, prev_end) = runs[i - 1];
    let (next_start, _) = runs[i];
    assert!(
      next_start >= prev_end - 1.0,
      "run {i} at x={next_start:.1} overlaps with previous ending at {prev_end:.1}; \
       runs: [{:.1}..{:.1}] vs [{:.1}..{:.1}]",
      runs[i - 1].0,
      prev_end,
      next_start,
      runs[i].1,
    );
  }
}

pub(crate) fn assert_no_overlap(children: &[LayoutBox]) {
  for (i, child) in children.iter().enumerate() {
    assert!(
      child.margin_rect.w > 0.5,
      "child {i} has near-zero width: {}",
      child.margin_rect.w
    );
  }
  for i in 1..children.len() {
    let prev_end = children[i - 1].margin_rect.x + children[i - 1].margin_rect.w;
    assert!(
      children[i].margin_rect.x >= prev_end - 0.5,
      "child {i} at x={:.1} overlaps with previous ending at {prev_end:.1}",
      children[i].margin_rect.x
    );
  }
}
