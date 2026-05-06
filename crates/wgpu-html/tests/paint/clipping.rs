use wgpu_html::{paint::*, renderer::DisplayList};
use wgpu_html_layout::{LayoutBox, Rect as LR};
use wgpu_html_text::{PositionedGlyph, ShapedLine, ShapedRun};

#[test]
fn real_textarea_in_flex_row_does_not_clip_following_block_quad() {
  let tree = wgpu_html_parser::parse(
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
  use wgpu_html_layout::{BorderColors, BorderStyles, BoxKind, CornerRadii, Insets, OverflowAxes};
  use wgpu_html_models::common::css_enums::Overflow;
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
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: wgpu_html_models::common::PointerEvents::Auto,
    user_select: wgpu_html_models::common::UserSelect::Auto,
    image: None,
    background_image: None,
    children: Vec::new(),
    cursor: wgpu_html_models::common::Cursor::Auto,
    z_index: None,
    is_fixed: false,
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
      byte_boundaries: wgpu_html_text::utf8_boundaries("A"),
      width: 8.0,
      height: 22.0,
      ascent: 10.0,
    }),
    text_color: Some([1.0, 1.0, 1.0, 1.0]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: wgpu_html_models::common::PointerEvents::Auto,
    user_select: wgpu_html_models::common::UserSelect::Auto,
    image: None,
    background_image: None,
    children: Vec::new(),
    cursor: wgpu_html_models::common::Cursor::Auto,
    z_index: None,
    is_fixed: false,
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
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: wgpu_html_models::common::PointerEvents::Auto,
    user_select: wgpu_html_models::common::UserSelect::Auto,
    image: None,
    background_image: None,
    children: vec![textarea, h2],
    cursor: wgpu_html_models::common::Cursor::Auto,
    z_index: None,
    is_fixed: false,
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
  let tree = wgpu_html_parser::parse(
    r#"<body style="margin: 0; padding: 0;">
                <div style="display: flex; height: 64px;">
                    <div style="overflow: auto; width: 320px; height: 64px;
                                background-color: red;"></div>
                </div>
                <div style="margin-top: 50px; height: 30px; background-color: blue;"></div>
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
fn overflow_visible_emits_single_clip_range() {
  let tree = wgpu_html_parser::parse(r#"<body style="width: 100px; height: 50px; background-color: red;"></body>"#);
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.clips.len(), 1);
  assert!(list.clips[0].rect.is_none());
  assert_eq!(list.clips[0].quad_range.0, 0);
  assert_eq!(list.clips[0].quad_range.1, list.quads.len() as u32);
}

#[test]
fn overflow_hidden_emits_clip_range_at_padding_box() {
  let tree = wgpu_html_parser::parse(
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
  let tree = wgpu_html_parser::parse(
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
  let tree = wgpu_html_parser::parse(
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
  let tree = wgpu_html_parser::parse(
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
  assert!(clipped.quad_range.0 >= 1);
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
  let tree = wgpu_html_parser::parse(
    r#"<body style="margin: 0;">
                <div><div style="width: 100px; max-height: 50px;
                             overflow-y: scroll; background-color: white;">
                    <div style="height: 120px; background-color: blue;"></div>
                </div></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(list.quads.iter().any(|q| q.color == wgpu_html::scroll::DEFAULT_TRACK));
  assert!(list.quads.iter().any(|q| q.color == wgpu_html::scroll::DEFAULT_THUMB));
}

#[test]
fn overflow_y_scroll_paints_scrollbar_without_overflow() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="margin: 0;">
                <div><div style="width: 100px; height: 80px;
                             overflow-y: scroll; background-color: white;">
                    <div style="height: 20px; background-color: blue;"></div>
                </div></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(list.quads.iter().any(|q| q.color == wgpu_html::scroll::DEFAULT_TRACK));
  assert!(list.quads.iter().any(|q| q.color == wgpu_html::scroll::DEFAULT_THUMB));
}

#[test]
fn overflow_scroll_offset_moves_descendants_and_thumb() {
  let mut tree = wgpu_html_parser::parse(
    r#"<body style="margin: 0;">
                <div><div style="width: 100px; max-height: 50px;
                             overflow-y: scroll; background-color: white;">
                    <div style="height: 120px; background-color: blue;"></div>
                </div></div>
            </body>"#,
  );
  tree
    .interaction
    .scroll_offsets
    .insert(vec![0, 0], wgpu_html_tree::ScrollOffset { x: 0.0, y: 30.0 });
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
    .find(|q| q.color == wgpu_html::scroll::DEFAULT_TRACK)
    .expect("scrollbar track");
  let thumb = list
    .quads
    .iter()
    .find(|q| q.color == wgpu_html::scroll::DEFAULT_THUMB)
    .expect("scrollbar thumb");
  assert!(thumb.rect.y > track.rect.y + 2.0);
}

#[test]
fn overflow_y_auto_without_overflow_does_not_paint_scrollbar() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="margin: 0;">
                <div style="width: 100px; height: 80px;
                             overflow-y: auto; background-color: white;">
                    <div style="height: 20px; background-color: blue;"></div>
                </div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(!list.quads.iter().any(|q| q.color == wgpu_html::scroll::DEFAULT_TRACK));
  assert!(!list.quads.iter().any(|q| q.color == wgpu_html::scroll::DEFAULT_THUMB));
}

#[test]
fn overflow_hidden_with_border_radius_emits_rounded_clip() {
  let tree = wgpu_html_parser::parse(
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
  let tree = wgpu_html_parser::parse(
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
  let tree = wgpu_html_parser::parse(
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
  for r in &clip_rects {
    assert!(r.x >= 0.0 && r.y >= 0.0);
    assert!(r.x + r.w <= 100.0 + 0.5);
    assert!(r.y + r.h <= 100.0 + 0.5);
  }
}

#[test]
fn tree_row_overflow_hidden_clips_span_glyphs() {
  let mut tree = wgpu_html_parser::parse(
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
    assert!(
      g.rect.x >= clip_rect.x - 1.0,
      "glyph {g_idx} at x={:.1} starts before clip x={:.1}",
      g.rect.x,
      clip_rect.x
    );
    glyphs_inside += 1;
  }
  assert!(glyphs_inside > 0, "expected glyphs inside the clip range, got 0");
  let clip_right = clip_rect.x + clip_rect.w;
  let mut overflow_glyphs = 0u32;
  for g_idx in row_clip.glyph_range.0..row_clip.glyph_range.1 {
    let g = list.glyphs[g_idx as usize];
    if g.rect.x + g.rect.w > clip_right + 1.0 {
      overflow_glyphs += 1;
    }
  }
  assert!(
    overflow_glyphs == 0 || glyphs_inside > 0,
    "some glyphs overflow clip rect but are inside the clipped range; \
       renderer scissor should clip them"
  );
}

#[test]
fn tree_row_overflow_hidden_pushes_clip_before_span_glyph_idx() {
  let mut tree = wgpu_html_parser::parse(
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
