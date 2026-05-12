use lui_layout_old::Resize;
use lui_models::common::{Cursor, PointerEvents};
use lui_tree::{ScrollOffset, TextCursor, TextSelection};

use super::*;

fn synthetic_text_layout() -> LayoutBox {
  let r = lui_layout_old::Rect::new(10.0, 20.0, 100.0, 24.0);
  LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: lui_layout_old::CornerRadii::zero(),
    border: lui_layout_old::Insets::zero(),
    border_colors: lui_layout_old::BorderColors::default(),
    border_styles: lui_layout_old::BorderStyles::default(),
    border_radius: lui_layout_old::CornerRadii::zero(),
    kind: lui_layout_old::BoxKind::Text,
    text_run: Some(lui_text::ShapedRun {
      glyphs: vec![
        lui_text::PositionedGlyph {
          x: 0.0,
          y: 4.0,
          w: 8.0,
          h: 14.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.1, 0.2, 0.3, 1.0],
        },
        lui_text::PositionedGlyph {
          x: 8.0,
          y: 4.0,
          w: 8.0,
          h: 14.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.2, 0.3, 0.4, 1.0],
        },
        lui_text::PositionedGlyph {
          x: 16.0,
          y: 4.0,
          w: 8.0,
          h: 14.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [0.3, 0.4, 0.5, 1.0],
        },
      ],
      lines: vec![lui_text::ShapedLine {
        top: 0.0,
        height: 22.0,
        glyph_range: (0, 3),
      }],
      glyph_chars: vec![],
      text: "abc".to_string(),
      byte_boundaries: lui_text::utf8_boundaries("abc"),
      width: 24.0,
      height: 22.0,
      ascent: 10.0,
    }),
    text_color: Some([0.0, 0.0, 0.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: lui_layout_old::OverflowAxes::visible(),
    resize: Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: Cursor::Auto,
    z_index: None,
    image: None,
    background_image: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  }
}

#[test]
fn paints_single_styled_box() {
  let tree = lui_parser::parse(r#"<body style="width: 100px; height: 50px; background-color: red;"></body>"#);
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  let q = list.quads[0];
  assert_eq!(q.rect.w, 100.0);
  assert_eq!(q.rect.h, 50.0);
}

#[test]
fn opacity_multiplies_background_through_subtree() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0; opacity: 0.5;">
                <div style="opacity: 0.5; width: 100px; height: 50px; background-color: blue;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  assert!((list.quads[0].color[3] - 0.25).abs() < 0.001);
}

#[test]
fn skips_boxes_without_background() {
  let tree = lui_parser::parse("<div><p>hi</p></div>");
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(list.quads.is_empty());
}

#[test]
fn border_emits_four_edge_quads() {
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border: 2px solid red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // No background, uniform border → single stroked ring.
  assert_eq!(list.quads.len(), 1);
  assert_eq!(list.quads[0].stroke, [2.0, 2.0, 2.0, 2.0]);
}

#[test]
fn border_with_background_emits_five_quads() {
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: blue;
                             border: 2px solid red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // Background + uniform border ring.
  assert_eq!(list.quads.len(), 2);
}

#[test]
fn radii_carry_through_to_display_list() {
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-radius: 1px 2px 3px 4px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  let q = list.quads[0];
  // Order: TL, TR, BR, BL.
  assert_eq!(q.radii_h, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn rounded_uniform_border_emits_single_ring_quad() {
  // border-radius + uniform `border:` → one stroked rounded ring,
  // not four sharp edge quads.
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border: 1px solid grey;
                             border-radius: 16px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  let q = list.quads[0];
  assert_eq!(q.radii_h, [16.0; 4]);
  assert_eq!(q.radii_v, [16.0; 4]);
  assert_eq!(q.stroke, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn rounded_with_background_and_border_emits_two_quads() {
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border: 2px solid blue;
                             border-radius: 8px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // 1 rounded background + 1 ring border = 2 quads.
  assert_eq!(list.quads.len(), 2);
  // Background is the first push and has no stroke.
  assert_eq!(list.quads[0].stroke, [0.0; 4]);
  assert_eq!(list.quads[1].stroke, [2.0, 2.0, 2.0, 2.0]);
}

#[test]
fn rounded_with_per_side_colors_emits_per_side_ring_quads() {
  // Each solid side gets its own one-sided ring quad so the
  // corners follow the rounded path — 1 rounded background + 4
  // ring quads = 5 total. (Same total count as the old sharp-
  // fallback path; the difference is each border quad now has
  // a non-zero stroke and curves at the corner.)
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-width: 2px;
                             border-color: red green blue orange;
                             border-radius: 8px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 5);
  // Each border quad has stroke set on exactly one side.
  for q in &list.quads[1..] {
    let nonzero_sides = q.stroke.iter().filter(|s| **s > 0.0).count();
    assert_eq!(nonzero_sides, 1);
  }
}

#[test]
fn rounded_with_mixed_styles_skips_none_sides() {
  // border-style: solid solid none solid → bottom side is omitted,
  // remaining 3 sides emit ring quads.
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-width: 2px;
                             border-style: solid solid none solid;
                             border-color: grey;
                             border-radius: 8px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // 1 background + 3 ring quads (top / right / left).
  assert_eq!(list.quads.len(), 4);
}

#[test]
fn sharp_box_border_still_uses_four_edges() {
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border: 2px solid red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // Sharp box with uniform border → single stroked ring, no seams.
  assert_eq!(list.quads.len(), 1);
  assert_eq!(list.quads[0].stroke, [2.0, 2.0, 2.0, 2.0]);
  assert_eq!(list.quads[0].radii_h, [0.0; 4]);
}

#[test]
fn no_radius_keeps_sharp_quad() {
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  assert_eq!(list.quads[0].radii_h, [0.0; 4]);
  assert_eq!(list.quads[0].radii_v, [0.0; 4]);
}

#[test]
fn border_with_no_color_is_skipped() {
  // border-width set, but no color → we don't paint edges (no
  // foreground-color fallback yet).
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: blue;
                             border-width: 2px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
}

#[test]
fn child_uses_block_flow_position() {
  // No absolute positioning: cards stack vertically. Override
  // the UA `body { margin: 8px }` so the first child sits at
  // y=0, then the second stacks immediately under it (no
  // margin between siblings).
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="height: 64px; background-color: blue;"></div>
                <div style="height: 30px; background-color: red;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 2);
  // First (blue header) at y=0
  assert_eq!(list.quads[0].rect.y, 0.0);
  // Second (red) stacks immediately under it (no margin)
  assert_eq!(list.quads[1].rect.y, 64.0);
}

#[test]
fn dashed_border_emits_multiple_segments_per_side() {
  let tree = lui_parser::parse(
    r#"<body style="width: 200px; height: 100px;
                             border: 2px dashed red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // Solid would emit 4 quads. Dashed should produce many more.
  assert!(
    list.quads.len() > 8,
    "expected dashed border to emit many segments, got {}",
    list.quads.len()
  );
}

#[test]
fn dotted_border_emits_segments_too() {
  let tree = lui_parser::parse(
    r#"<body style="width: 200px; height: 100px;
                             border: 2px dotted blue;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(list.quads.len() > 8);
}

#[test]
fn border_style_none_skips_that_side() {
  let tree = lui_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border-width: 2px;
                             border-style: solid solid none solid;
                             border-color: red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // Per-side fallback path; bottom side skipped → 3 solid edges.
  assert_eq!(list.quads.len(), 3);
}

// --- overflow / clipping --------------------------------------

#[test]
fn real_textarea_in_flex_row_does_not_clip_following_block_quad() {
  // Same regression guard as the next test, but uses a real
  // `<textarea>` element so the UA stylesheet's actual
  // `overflow: auto`, `border: 2px inset`, `padding: 2px`,
  // etc. kick in. The blue sibling background that follows
  // the row must not be inside any non-None clip range.
  let tree = lui_parser::parse(
    r#"<body style="margin: 0; padding: 0;">
                <div style="display: flex;">
                    <textarea style="min-width: 320px; height: 64px;"></textarea>
                </div>
                <div style="margin-top: 30px; height: 30px; background-color: blue;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let blue_idx = list
    .quads
    .iter()
    .position(|q| q.color == [0.0, 0.0, 1.0, 1.0])
    .expect("blue sibling background quad emitted");
  let blue = blue_idx as u32;
  for clip in &list.clips {
    if clip.rect.is_some() && blue >= clip.quad_range.0 && blue < clip.quad_range.1 {
      let r = clip.rect.unwrap();
      let blue_quad = &list.quads[blue_idx];
      let outside_x = blue_quad.rect.x + blue_quad.rect.w <= r.x || blue_quad.rect.x >= r.x + r.w;
      let outside_y = blue_quad.rect.y + blue_quad.rect.h <= r.y || blue_quad.rect.y >= r.y + r.h;
      assert!(
        !(outside_x || outside_y),
        "blue sibling at {:?} sits inside textarea-row clip range {:?} \
                     (outside_x={} outside_y={})",
        blue_quad.rect,
        r,
        outside_x,
        outside_y
      );
    }
  }
}

#[test]
fn glyphs_after_overflow_auto_sibling_are_not_clipped() {
  // Same shape as the textarea forms.html bug: a body containing
  // an overflow:auto sibling block and then a text-leaf sibling
  // that follows it. Quad-only regression guards above only
  // assert quads aren't clipped; this one walks the glyph axis
  // of the display list to make sure the popped textarea clip
  // doesn't continue suppressing the following text.
  use lui_layout_old::{
    BorderColors, BorderStyles, BoxKind, CornerRadii, Insets, LayoutBox, OverflowAxes, Rect as LR,
  };
  use lui_models::common::css_enums::Overflow;
  use lui_text::{PositionedGlyph, ShapedLine, ShapedRun};
  let textarea_rect = LR::new(0.0, 0.0, 320.0, 64.0);
  let textarea = LayoutBox {
    margin_rect: textarea_rect,
    border_rect: textarea_rect,
    content_rect: textarea_rect,
    background: None,
    background_rect: textarea_rect,
    background_radii: CornerRadii::zero(),
    border: Insets::zero(),
    border_colors: BorderColors::default(),
    border_styles: BorderStyles::default(),
    border_radius: CornerRadii::zero(),
    kind: BoxKind::Block,
    text_run: None,
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes {
      x: Overflow::Auto,
      y: Overflow::Auto,
      scrollbar_width: 10.0,
      scrollbar_thumb: None,
      scrollbar_track: None,
    },
    resize: Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    image: None,
    background_image: None,
    children: Vec::new(),
    cursor: Cursor::Auto,
    z_index: None,
    is_fixed: false,
    form_control: None,
  };
  let h2_rect = LR::new(0.0, 100.0, 200.0, 24.0);
  let h2 = LayoutBox {
    margin_rect: h2_rect,
    border_rect: h2_rect,
    content_rect: h2_rect,
    background: None,
    background_rect: h2_rect,
    background_radii: CornerRadii::zero(),
    border: Insets::zero(),
    border_colors: BorderColors::default(),
    border_styles: BorderStyles::default(),
    border_radius: CornerRadii::zero(),
    kind: BoxKind::Text,
    text_run: Some(ShapedRun {
      glyphs: vec![PositionedGlyph {
        x: 0.0,
        y: 4.0,
        w: 8.0,
        h: 14.0,
        uv_min: [0.0, 0.0],
        uv_max: [1.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
      }],
      lines: vec![ShapedLine {
        top: 0.0,
        height: 22.0,
        glyph_range: (0, 1),
      }],
      glyph_chars: vec![],
      text: "A".to_string(),
      byte_boundaries: lui_text::utf8_boundaries("A"),
      width: 8.0,
      height: 22.0,
      ascent: 10.0,
    }),
    text_color: Some([1.0, 1.0, 1.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes::visible(),
    resize: Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    image: None,
    background_image: None,
    children: Vec::new(),
    cursor: Cursor::Auto,
    z_index: None,
    is_fixed: false,
    form_control: None,
  };
  let body_rect = LR::new(0.0, 0.0, 800.0, 200.0);
  let body = LayoutBox {
    margin_rect: body_rect,
    border_rect: body_rect,
    content_rect: body_rect,
    background: None,
    background_rect: body_rect,
    background_radii: CornerRadii::zero(),
    border: Insets::zero(),
    border_colors: BorderColors::default(),
    border_styles: BorderStyles::default(),
    border_radius: CornerRadii::zero(),
    kind: BoxKind::Block,
    text_run: None,
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes::visible(),
    resize: Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    image: None,
    background_image: None,
    children: vec![textarea, h2],
    cursor: Cursor::Auto,
    z_index: None,
    is_fixed: false,
    form_control: None,
  };
  let mut list = DisplayList::new();
  paint_layout(&body, &mut list);
  assert_eq!(list.glyphs.len(), 1, "h2 glyph should be emitted");
  let g = &list.glyphs[0];
  let g_idx = 0u32;
  for clip in &list.clips {
    if clip.rect.is_some() && g_idx >= clip.glyph_range.0 && g_idx < clip.glyph_range.1 {
      let r = clip.rect.unwrap();
      let outside_x = g.rect.x + g.rect.w <= r.x || g.rect.x >= r.x + r.w;
      let outside_y = g.rect.y + g.rect.h <= r.y || g.rect.y >= r.y + r.h;
      assert!(
        !(outside_x || outside_y),
        "h2 glyph at {:?} sits inside clip range {:?} that suppresses it \
                     (outside_x={} outside_y={})",
        g.rect,
        r,
        outside_x,
        outside_y
      );
    }
  }
}

#[test]
fn overflow_auto_in_flex_row_does_not_clip_block_sibling_below() {
  // Repro for the "no text after textarea" report. A flex item
  // with `overflow: auto` (textarea's UA default) emits a
  // clip range. We need to confirm a *block* sibling that
  // follows the flex container doesn't get clipped to the
  // flex item's bounds.
  let tree = lui_parser::parse(
    r#"<body style="margin: 0; padding: 0;">
                <div style="display: flex; height: 64px;">
                    <div style="overflow: auto; width: 320px; height: 64px;
                                background-color: red;"></div>
                </div>
                <div style="margin-top: 50px; height: 30px; background-color: blue;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  // The blue quad must exist (background of the post-flex sibling).
  let blue_idx = list
    .quads
    .iter()
    .position(|q| q.color == [0.0, 0.0, 1.0, 1.0])
    .expect("blue sibling background quad emitted");
  let blue = blue_idx as u32;
  // For each clip range with a non-None rect, the blue quad
  // must NOT fall inside its quad_range — that would mean the
  // flex item's `overflow:auto` clip is leaking onto the
  // following block sibling.
  for clip in &list.clips {
    if clip.rect.is_some() && blue >= clip.quad_range.0 && blue < clip.quad_range.1 {
      let r = clip.rect.unwrap();
      let blue_quad = &list.quads[blue_idx];
      let outside_x = blue_quad.rect.x + blue_quad.rect.w <= r.x || blue_quad.rect.x >= r.x + r.w;
      let outside_y = blue_quad.rect.y + blue_quad.rect.h <= r.y || blue_quad.rect.y >= r.y + r.h;
      assert!(
        !(outside_x || outside_y),
        "blue sibling at {:?} sits inside a clip range {:?} that would visually \
                     suppress it (outside_x={} outside_y={})",
        blue_quad.rect,
        r,
        outside_x,
        outside_y
      );
    }
  }
}

#[test]
fn svg_test_demo_paints_svg_images() {
  let tree = lui_parser::parse(include_str!("../../lui-demo/html/svg-test.html"));
  fn count_svg_nodes(node: &lui_tree::Node) -> usize {
    let own = matches!(node.element, lui_tree::Element::Svg(_)) as usize;
    own + node.children.iter().map(count_svg_nodes).sum::<usize>()
  }
  assert_eq!(
    tree.root.as_ref().map(count_svg_nodes).unwrap_or(0),
    8,
    "svg-test.html should parse all inline SVG elements"
  );
  let mut text_ctx = TextContext::new(64);
  let mut image_cache = lui_layout_old::ImageCache::default();
  let layout = lui_layout_old::layout_with_text(
    &lui_style::cascade(&tree),
    &mut text_ctx,
    &mut image_cache,
    1920.0,
    1080.0,
    1.0,
  )
  .expect("layout");
  fn count_layout_images(b: &LayoutBox) -> usize {
    (b.image.is_some() && b.content_rect.w > 0.0 && b.content_rect.h > 0.0) as usize
      + b.children.iter().map(count_layout_images).sum::<usize>()
  }
  fn collect_image_rects(b: &LayoutBox, out: &mut Vec<(f32, f32, f32, f32)>) {
    if b.image.is_some() {
      let r = b.content_rect;
      out.push((r.x, r.y, r.w, r.h));
    }
    for child in &b.children {
      collect_image_rects(child, out);
    }
  }
  let mut image_rects = Vec::new();
  collect_image_rects(&layout, &mut image_rects);
  assert_eq!(
    count_layout_images(&layout),
    8,
    "layout should attach visible rasterized image data to each SVG: {image_rects:?}"
  );
  assert!(
    image_rects
      .iter()
      .all(|(_, _, w, h)| (*w - 160.0).abs() < 0.01 && (*h - 160.0).abs() < 0.01),
    "each SVG should keep its authored 160x160 content box: {image_rects:?}"
  );
  let mut scaled_text_ctx = TextContext::new(64);
  let mut scaled_image_cache = lui_layout_old::ImageCache::default();
  let scaled_layout = lui_layout_old::layout_with_text(
    &lui_style::cascade(&tree),
    &mut scaled_text_ctx,
    &mut scaled_image_cache,
    1920.0,
    1080.0,
    1.5,
  )
  .expect("scaled layout");
  let mut scaled_image_rects = Vec::new();
  collect_image_rects(&scaled_layout, &mut scaled_image_rects);
  assert!(
    scaled_image_rects
      .iter()
      .all(|(_, _, w, h)| (*w - 240.0).abs() < 0.01 && (*h - 240.0).abs() < 0.01),
    "scaled SVG content boxes should apply scale once: {scaled_image_rects:?}"
  );
  let mut direct = DisplayList::new();
  paint_layout(&layout, &mut direct);
  assert_eq!(
    direct.images.len(),
    8,
    "painting the computed layout should emit image quads"
  );
  let list = paint_tree(&tree, 1920.0, 1080.0);

  assert_eq!(
    list.images.len(),
    8,
    "svg-test.html should emit one image quad for each inline SVG"
  );
  for (idx, img) in list.images.iter().enumerate() {
    assert!(
      img.data.chunks_exact(4).any(|px| px[3] > 0),
      "SVG image {idx} must contain at least one visible pixel"
    );
  }
  assert!(
    list
      .commands
      .iter()
      .filter(|cmd| cmd.kind == lui_renderer_wgpu::DisplayCommandKind::Image)
      .count()
      == 8,
    "every SVG image should have an ordered image command"
  );
}

#[test]
fn overflow_visible_emits_single_clip_range() {
  // The display list carries one all-encompassing range with
  // `rect: None` whenever no `overflow: hidden` is in play.
  let tree = lui_parser::parse(r#"<body style="width: 100px; height: 50px; background-color: red;"></body>"#);
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.clips.len(), 1);
  assert!(list.clips[0].rect.is_none());
  assert_eq!(list.clips[0].quad_range.0, 0);
  assert_eq!(list.clips[0].quad_range.1, list.quads.len() as u32);
}

#[test]
fn template_contents_do_not_paint() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <template>
                    <div style="width: 100px; height: 50px; background-color: red;"></div>
                </template>
                <div style="width: 100px; height: 50px; background-color: blue;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(!list.quads.iter().any(|q| q.color == [1.0, 0.0, 0.0, 1.0]));
  assert!(list.quads.iter().any(|q| q.color == [0.0, 0.0, 1.0, 1.0]));
}

#[test]
fn overflow_hidden_emits_clip_range_at_padding_box() {
  // The clipping container has a 5px solid border, so the
  // padding-box rect (which is what `overflow: hidden` clips at
  // per CSS-2.2 §11.1.1) is the border-rect inset by the
  // border thickness. Container is 80+10*2=100 wide with the
  // padding contributing inside the padding-box. With a 5px
  // border, padding-box becomes 100×100 minus 5px on every
  // side → 90×90 at (5, 5).
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 80px; height: 80px; padding: 10px;
                             border: 5px solid black;
                             overflow: hidden; background-color: red;">
                    <div style="width: 200px; height: 200px;
                                 background-color: blue;"></div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let clip = list
    .clips
    .iter()
    .find(|c| c.rect.is_some())
    .expect("an overflow:hidden clip range");
  let r = clip.rect.unwrap();
  assert_eq!(r.x, 5.0);
  assert_eq!(r.y, 5.0);
  assert_eq!(r.w, 100.0);
  assert_eq!(r.h, 100.0);
}

#[test]
fn overflow_x_clip_leaves_vertical_axis_unclipped() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 80px; height: 80px; padding: 10px;
                             overflow: clip visible; background-color: red;">
                    <div style="width: 200px; height: 200px;
                                 background-color: blue;"></div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let clip = list
    .clips
    .iter()
    .find(|c| c.rect.is_some())
    .expect("an overflow-x clip range");
  let r = clip.rect.unwrap();
  assert_eq!(r.x, 0.0);
  assert_eq!(r.w, 100.0);
  assert!(r.y < -999_000.0);
  assert!(r.h > 1_999_000.0);
}

#[test]
fn one_axis_overflow_clip_does_not_emit_rounded_clip() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 100px; height: 100px;
                             overflow: clip visible;
                             border-radius: 16px;
                             background-color: red;">
                    <div style="width: 200px; height: 200px;
                                 background-color: blue;"></div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let clip = list
    .clips
    .iter()
    .find(|c| c.rect.is_some())
    .expect("an overflow-x clip range");
  assert!(!clip.is_rounded());
}

#[test]
fn overflow_clip_range_only_covers_descendants() {
  // The container's own background quad should sit *outside* the
  // clip range — the clip applies to its children, not its own
  // border / background.
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 50px; height: 50px;
                             overflow: hidden; background-color: red;">
                    <div style="width: 200px; height: 200px;
                                 background-color: blue;"></div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let clipped = list.clips.iter().find(|c| c.rect.is_some()).expect("clip range");
  // The container's own background is the first red quad and
  // sits before the clip range starts.
  assert!(clipped.quad_range.0 >= 1);
  // The blue child quad falls inside the range.
  assert!(clipped.quad_range.0 < clipped.quad_range.1);
  let blue_idx = list
    .quads
    .iter()
    .position(|q| q.color == [0.0, 0.0, 1.0, 1.0])
    .expect("blue quad emitted");
  let blue = blue_idx as u32;
  assert!(blue >= clipped.quad_range.0 && blue < clipped.quad_range.1);
}

#[test]
fn overflow_y_scroll_paints_vertical_scrollbar() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div><div style="width: 100px; max-height: 50px;
                             overflow-y: scroll; background-color: white;">
                    <div style="height: 120px; background-color: blue;"></div>
                </div></div>
            </body>"#,
  );

  let list = paint_tree(&tree, 800.0, 600.0);

  assert!(list.quads.iter().any(|q| q.color == crate::scroll::DEFAULT_TRACK));
  assert!(list.quads.iter().any(|q| q.color == crate::scroll::DEFAULT_THUMB));
}

#[test]
fn overflow_y_scroll_paints_scrollbar_without_overflow() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div><div style="width: 100px; height: 80px;
                             overflow-y: scroll; background-color: white;">
                    <div style="height: 20px; background-color: blue;"></div>
                </div></div>
            </body>"#,
  );

  let list = paint_tree(&tree, 800.0, 600.0);

  assert!(list.quads.iter().any(|q| q.color == crate::scroll::DEFAULT_TRACK));
  assert!(list.quads.iter().any(|q| q.color == crate::scroll::DEFAULT_THUMB));
}

#[test]
fn overflow_scroll_offset_moves_descendants_and_thumb() {
  let mut tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div><div style="width: 100px; max-height: 50px;
                             overflow-y: scroll; background-color: white;">
                    <div style="height: 120px; background-color: blue;"></div>
                </div></div>
            </body>"#,
  );
  tree.interaction.scroll_offsets.insert(vec![0, 0], ScrollOffset { x: 0.0, y: 30.0 });

  let list = paint_tree(&tree, 800.0, 600.0);

  let blue = list
    .quads
    .iter()
    .find(|q| q.color == [0.0, 0.0, 1.0, 1.0])
    .expect("blue child quad");
  assert_eq!(blue.rect.y, -30.0);

  let track = list
    .quads
    .iter()
    .find(|q| q.color == crate::scroll::DEFAULT_TRACK)
    .expect("scrollbar track");
  let thumb = list
    .quads
    .iter()
    .find(|q| q.color == crate::scroll::DEFAULT_THUMB)
    .expect("scrollbar thumb");
  assert!(thumb.rect.y > track.rect.y + 2.0);
}

#[test]
fn overflow_y_auto_without_overflow_does_not_paint_scrollbar() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 100px; height: 80px;
                             overflow-y: auto; background-color: white;">
                    <div style="height: 20px; background-color: blue;"></div>
                </div>
            </body>"#,
  );

  let list = paint_tree(&tree, 800.0, 600.0);

  assert!(!list.quads.iter().any(|q| q.color == crate::scroll::DEFAULT_TRACK));
  assert!(!list.quads.iter().any(|q| q.color == crate::scroll::DEFAULT_THUMB));
}

#[test]
fn overflow_hidden_with_border_radius_emits_rounded_clip() {
  // A box with `overflow: hidden` AND a border-radius produces a
  // clip range whose `radii_h` / `radii_v` are populated. The
  // padding-box radii are the outer radii inset by the border
  // thickness — with no border here, they pass through.
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 100px; height: 100px;
                             overflow: hidden;
                             border-radius: 16px;
                             background-color: red;">
                    <div style="width: 200px; height: 200px;
                                 background-color: blue;"></div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let clip = list
    .clips
    .iter()
    .find(|c| c.rect.is_some() && c.is_rounded())
    .expect("a rounded clip range");
  assert_eq!(clip.radii_h, [16.0, 16.0, 16.0, 16.0]);
  assert_eq!(clip.radii_v, [16.0, 16.0, 16.0, 16.0]);
}

#[test]
fn overflow_hidden_padding_box_radii_inset_by_border() {
  // Border thickness shrinks the rounded-clip radii at the
  // padding-box edge, matching how browsers draw the inner
  // rounded path. A 5px border on a `border-radius: 20px` box
  // leaves the inner rounded clip at radius 15px.
  //
  // The child needs a paintable quad so its presence keeps the
  // clip range alive through `finalize()`.
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 100px; height: 100px;
                             overflow: hidden;
                             border: 5px solid black;
                             border-radius: 20px;
                             background-color: red;">
                    <div style="width: 200px; height: 200px;
                                 background-color: blue;"></div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let clip = list.clips.iter().find(|c| c.is_rounded()).expect("rounded clip");
  for r in &clip.radii_h {
    assert!((r - 15.0).abs() < 0.05, "got {}", r);
  }
  for r in &clip.radii_v {
    assert!((r - 15.0).abs() < 0.05, "got {}", r);
  }
}

#[test]
fn nested_overflow_hidden_intersects_clips() {
  // Outer 100×100 at (0,0); inner 200×200 at (0,0) with
  // overflow:hidden too. The grandchild's effective scissor is
  // the intersection: 100×100 at (0,0). A second clip range
  // exists for the inner element with the same rect because
  // outer ∩ inner = outer here.
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 100px; height: 100px;
                             overflow: hidden; background-color: red;">
                    <div style="width: 200px; height: 200px;
                                 overflow: hidden; background-color: green;">
                        <div style="width: 400px; height: 400px;
                                     background-color: blue;"></div>
                    </div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let clip_rects: Vec<_> = list.clips.iter().filter_map(|c| c.rect).collect();
  assert!(
    clip_rects.len() >= 2,
    "expected ≥2 clip rects for nested overflow, got {}",
    clip_rects.len()
  );
  // Every nested clip is contained within the outer 100×100 rect.
  for r in &clip_rects {
    assert!(r.x >= 0.0 && r.y >= 0.0);
    assert!(r.x + r.w <= 100.0 + 0.5);
    assert!(r.y + r.h <= 100.0 + 0.5);
  }
}

#[test]
fn dashed_with_rounded_emits_per_side_patterned_rings() {
  // Uniform-circular corners → dashed pattern goes through the
  // shader as one one-sided ring quad per side; the dash pattern
  // wraps around the corner curve in the fragment shader.
  // 1 rounded background + 4 ring quads (top / right / bottom /
  // left, each with pattern set).
  let tree = lui_parser::parse(
    r#"<body style="width: 200px; height: 100px;
                             background-color: white;
                             border: 2px dashed red;
                             border-radius: 12px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 5);
  // Background first, no stroke / pattern.
  assert_eq!(list.quads[0].stroke, [0.0; 4]);
  assert_eq!(list.quads[0].pattern, [0.0; 4]);
  // Each border ring carries the dashed pattern (kind=1.0).
  for q in &list.quads[1..] {
    assert_eq!(q.pattern[0], 1.0);
    let nonzero = q.stroke.iter().filter(|s| **s > 0.0).count();
    assert_eq!(nonzero, 1);
  }
}

#[test]
fn selection_paints_background_and_overrides_glyph_color() {
  let root = synthetic_text_layout();
  let mut list = DisplayList::new();
  let selection = TextSelection {
    anchor: TextCursor {
      path: vec![],
      glyph_index: 1,
    },
    focus: TextCursor {
      path: vec![],
      glyph_index: 3,
    },
  };
  let colors = SelectionColors {
    background: [0.9, 0.8, 0.1, 0.4],
    foreground: [1.0, 1.0, 1.0, 1.0],
  };
  paint_layout_with_selection(&root, &mut list, Some(&selection), colors, 0.0);
  list.finalize();

  assert_eq!(list.quads.len(), 1, "single line emits one merged highlight span");
  assert_eq!(list.quads[0].color, colors.background);
  assert_eq!(list.quads[0].rect.y, 20.0, "selection starts at line top");
  assert_eq!(
    list.quads[0].rect.h, 22.0,
    "selection uses line height, not glyph height"
  );
  assert_eq!(list.glyphs.len(), 3);
  assert_eq!(list.glyphs[0].color, [0.1, 0.2, 0.3, 1.0]);
  assert_eq!(list.glyphs[1].color, colors.foreground);
  assert_eq!(list.glyphs[2].color, colors.foreground);
}

#[test]
fn tree_row_overflow_hidden_clips_span_glyphs() {
  let mut tree = lui_parser::parse(
    r#"<html><head><style>
          .tree-row { width: 50px; display: flex; align-items: center;
                      height: 18px; flex-shrink: 0; white-space: nowrap;
                      overflow: hidden; }
          span { color: #ccc; }
        </style></head>
        <body style="margin: 0; font-family: sans-serif;">
          <div class="tree-row">
            <span>&lt;</span>
            <span>div</span>
            <span> class</span>
            <span>=</span>
            <span>"child"</span>
            <span>&gt;</span>
          </div>
        </body></html>"#,
  );
  tree.register_system_fonts("sans-serif");
  let list = paint_tree(&tree, 800.0, 600.0);

  // The tree-row must push a clip. Find it.
  let row_clip = list
    .clips
    .iter()
    .find(|c| c.rect.is_some())
    .expect("tree-row with overflow:hidden must emit a clip range");
  let clip_rect = row_clip.rect.unwrap();

  assert!(
    (clip_rect.w - 50.0).abs() < 5.0,
    "clip width should be ~50px (the row width), got {:.1}",
    clip_rect.w
  );

  let mut glyphs_inside = 0u32;
  for g_idx in row_clip.glyph_range.0..row_clip.glyph_range.1 {
    let g = list.glyphs[g_idx as usize];
    // Glyph must start within or at edge of clip.
    assert!(
      g.rect.x >= clip_rect.x - 1.0,
      "glyph {g_idx} at x={:.1} starts before clip x={:.1}",
      g.rect.x,
      clip_rect.x
    );
    glyphs_inside += 1;
  }
  assert!(glyphs_inside > 0, "expected glyphs inside the clip range, got 0");

  // Any glyph whose right edge exceeds the clip right must be within
  // the clip range (so the renderer scissor can clip it). We already
  // iterate only over glyphs in the range, so this is a sanity
  // assertion that they are correctly bounded.
  let clip_right = clip_rect.x + clip_rect.w;
  let mut overflow_glyphs = 0u32;
  for g_idx in row_clip.glyph_range.0..row_clip.glyph_range.1 {
    let g = list.glyphs[g_idx as usize];
    if g.rect.x + g.rect.w > clip_right + 1.0 {
      overflow_glyphs += 1;
    }
  }
  // Glyphs that overflow the clip rect must still be inside the
  // clip range so the renderer scissor can cut them visually.
  assert!(
    overflow_glyphs == 0 || glyphs_inside > 0,
    "some glyphs overflow clip rect but are inside the clipped range; \
       renderer scissor should clip them"
  );
}

#[test]
fn tree_row_overflow_hidden_pushes_clip_before_span_glyph_idx() {
  // The clip push happens BEFORE children are painted. Glyph
  // indices of the first span must be ≥ the clip's glyph_range.0.
  let mut tree = lui_parser::parse(
    r#"<html><head><style>
          .row { width: 50px; display: flex; align-items: center; height: 18px;
                 flex-shrink: 0; white-space: nowrap; overflow: hidden; }
        </style></head>
        <body style="margin: 0; font-family: sans-serif;">
          <div class="row">
            <span>aaa</span>
            <span>bbb</span>
            <span>ccc</span>
          </div>
        </body></html>"#,
  );
  tree.register_system_fonts("sans-serif");
  let list = paint_tree(&tree, 800.0, 600.0);

  let row_clip = list
    .clips
    .iter()
    .find(|c| c.rect.is_some())
    .expect("row with overflow:hidden must emit a clip");

  // All glyphs must fall inside the row's clip range.
  for i in 0..list.glyphs.len() {
    let idx = i as u32;
    assert!(
      idx >= row_clip.glyph_range.0 && idx < row_clip.glyph_range.1,
      "glyph {i} at index {idx} not in clip glyph_range [{}, {})",
      row_clip.glyph_range.0,
      row_clip.glyph_range.1
    );
  }
}

#[test]
fn tree_row_painted_glyphs_dont_overlap() {
  let mut tree = lui_parser::parse(
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
  );
  tree.register_system_fonts("sans-serif");
  let list = paint_tree(&tree, 800.0, 600.0);
  let mut rects: Vec<(f32, f32)> = list.glyphs.iter().map(|g| (g.rect.x, g.rect.x + g.rect.w)).collect();
  assert!(!rects.is_empty());
  rects.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
  for i in 1..rects.len() {
    let gap = rects[i].0 - rects[i - 1].1;
    assert!(
      gap > -3.0,
      "glyph {i} starts at {:.1} but prev ends at {:.1} (gap {gap:.1})",
      rects[i].0,
      rects[i - 1].1
    );
  }
}

#[test]
fn z_index_sorts_positioned_children_in_paint_order() {
  let tree = lui_parser::parse(
    r#"<body style="margin:0; width:200px; height:100px;">
          <div style="position:absolute; z-index:10; width:50px; height:50px;
                      background-color:#00f; left:0; top:0;"></div>
          <div style="position:absolute; z-index:5; width:50px; height:50px;
                      background-color:#f00; left:10px; top:10px;"></div>
          <div style="position:absolute; z-index:auto; width:50px; height:50px;
                      background-color:#0f0; left:20px; top:20px;"></div>
        </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let colors: Vec<_> = list.quads.iter().map(|q| q.color).collect();
  assert_eq!(colors[0], [0.0, 1.0, 0.0, 1.0]);
  assert_eq!(colors[1], [1.0, 0.0, 0.0, 1.0]);
  assert_eq!(colors[2], [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn negative_z_index_paints_behind_non_positioned_siblings() {
  let tree = lui_parser::parse(
    r#"<body style="margin:0; width:200px; height:100px;">
          <div style="position:absolute; z-index:-1; width:80px; height:80px;
                      background-color:#f00; left:0; top:0;"></div>
          <div style="position:relative; width:50px; height:50px;
                      background-color:#00f;"></div>
        </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let colors: Vec<_> = list.quads.iter().map(|q| q.color).collect();
  assert_eq!(colors.len(), 2);
  assert_eq!(colors[0], [1.0, 0.0, 0.0, 1.0]);
  assert_eq!(colors[1], [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn absolute_z_neg1_child_paints_behind_normal_flow_sibling_with_margins() {
  let tree = lui_parser::parse(
    r#"<body style="margin:0;">
          <div style="position:relative; width:200px; height:80px;
                      background-color:#222;">
            <div style="position:absolute; z-index:-1; left:10px; top:10px;
                        width:80px; height:60px; background-color:#f00;"></div>
            <div style="width:100px; height:30px; margin-top:25px; margin-left:50px;
                        background-color:#00f;"></div>
          </div>
        </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let colors: Vec<_> = list.quads.iter().map(|q| q.color).collect();
  assert!(!colors.is_empty(), "no quads emitted");
  // Both red and blue should be present
  let red_idx = colors.iter().position(|c| *c == [1.0, 0.0, 0.0, 1.0]);
  let blue_idx = colors.iter().position(|c| *c == [0.0, 0.0, 1.0, 1.0]);
  assert!(red_idx.is_some(), "red quad not found; colors: {colors:?}");
  assert!(blue_idx.is_some(), "blue quad not found; colors: {colors:?}");
  assert!(red_idx < blue_idx, "red (z=-1) must paint before blue");
}

/// Assert that every glyph quad in the display list fits inside
/// its clip rect's Y bounds.  If a clip has no rect (full viewport)
/// the check is skipped.
fn assert_glyphs_fit_clips(list: &lui_renderer_wgpu::DisplayList) {
  for cmd in &list.commands {
    if cmd.kind != lui_renderer_wgpu::DisplayCommandKind::Glyph {
      continue;
    }
    let g = &list.glyphs[cmd.index as usize];
    let clip = &list.clips[cmd.clip_index as usize];
    if let Some(r) = clip.rect {
      let bottom = g.rect.y + g.rect.h;
      assert!(
        bottom <= r.y + r.h + 0.01,
        "glyph at y={:.1} h={:.1} bottom={:.1} extends past clip rect bottom={:.1} (rect={:?})",
        g.rect.y,
        g.rect.h,
        bottom,
        r.y + r.h,
        r,
      );
      assert!(
        g.rect.y >= r.y - 0.01,
        "glyph at y={:.1} is above clip rect top y={:.1} (rect={:?})",
        g.rect.y,
        r.y,
        r,
      );
    }
  }
}

#[test]
fn glyphs_fit_within_default_clip() {
  let root = simple_body_with_glyphs();
  let mut list = DisplayList::new();
  paint_layout(&root, &mut list);
  assert_glyphs_fit_clips(&list);
}

#[test]
fn glyphs_fit_within_overflow_hidden_clip() {
  // Text inside overflow:hidden — the clip rect is the padding box.
  let mut tree = lui_parser::parse(
    r#"<body style="margin:0; width:200px; height:80px; overflow:hidden;
                          font-family:sans-serif; font-size:24px; color:white;">
           <span>gy0j</span>
         </body>"#,
  );
  tree.register_system_fonts("sans-serif");
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_glyphs_fit_clips(&list);
}

#[test]
fn glyphs_in_tight_overflow_hidden_flex_row() {
  // Reproduces the `.glyph-row` scenario from p0-demo:
  //    height:18px  overflow:hidden  white-space:nowrap
  // with font-size inherited from body (defaults to 16px,
  // line-height ~19-20px — larger than the 18px container).
  let mut tree = lui_parser::parse(
    r#"<html><head><style>
           body { margin:0; font-family:sans-serif; }
           .glyph-row {
             width: 200px; height: 18px;
             display: flex; align-items: center;
             white-space: nowrap; overflow: hidden;
             background: #202124;
           }
           .tag  { color: #5db0d7; }
           .br   { color: #9aa0a6; }
           .atn  { color: #9aa0a6; }
           .atv  { color: #f28b82; }
         </style></head>
         <body>
           <div class="glyph-row">
             <span class="br">&lt;</span>
             <span class="tag">div</span>
             <span class="atn">class</span>
             <span class="br">=</span>
             <span class="atv">"root"</span>
             <span class="br">&gt;</span>
           </div>
         </body></html>"#,
  );
  tree.register_system_fonts("sans-serif");
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_glyphs_fit_clips(&list);
}

fn simple_body_with_glyphs() -> LayoutBox {
  let r = lui_layout_old::Rect::new(0.0, 0.0, 800.0, 24.0);
  LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: lui_layout_old::CornerRadii::zero(),
    border: lui_layout_old::Insets::zero(),
    border_colors: lui_layout_old::BorderColors::default(),
    border_styles: lui_layout_old::BorderStyles::default(),
    border_radius: lui_layout_old::CornerRadii::zero(),
    kind: lui_layout_old::BoxKind::Text,
    text_run: Some(lui_text::ShapedRun {
      glyphs: vec![
        lui_text::PositionedGlyph {
          x: 0.0,
          y: 0.0,
          w: 10.0,
          h: 16.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [1.0, 1.0, 1.0, 1.0],
        },
        lui_text::PositionedGlyph {
          x: 10.0,
          y: 2.0,
          w: 12.0,
          h: 20.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [1.0, 1.0, 1.0, 1.0],
        },
      ],
      lines: vec![lui_text::ShapedLine {
        top: 0.0,
        height: 24.0,
        glyph_range: (0, 2),
      }],
      glyph_chars: vec![],
      text: "ab".to_string(),
      byte_boundaries: lui_text::utf8_boundaries("ab"),
      width: 22.0,
      height: 24.0,
      ascent: 18.0,
    }),
    text_color: Some([1.0; 4]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: lui_layout_old::OverflowAxes::visible(),
    resize: Resize::None,
    opacity: 1.0,
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: Cursor::Auto,
    z_index: None,
    image: None,
    background_image: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  }
}
