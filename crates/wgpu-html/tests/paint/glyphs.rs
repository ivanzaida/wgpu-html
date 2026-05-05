use wgpu_html::{paint::*, renderer::DisplayList};
use wgpu_html_layout::LayoutBox;

fn simple_body_with_glyphs() -> LayoutBox {
  let r = wgpu_html_layout::Rect::new(0.0, 0.0, 800.0, 24.0);
  LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: wgpu_html_layout::CornerRadii::zero(),
    border: wgpu_html_layout::Insets::zero(),
    border_colors: wgpu_html_layout::BorderColors::default(),
    border_styles: wgpu_html_layout::BorderStyles::default(),
    border_radius: wgpu_html_layout::CornerRadii::zero(),
    kind: wgpu_html_layout::BoxKind::Text,
    text_run: Some(wgpu_html_text::ShapedRun {
      glyphs: vec![
        wgpu_html_text::PositionedGlyph {
          x: 0.0,
          y: 0.0,
          w: 10.0,
          h: 16.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [1.0, 1.0, 1.0, 1.0],
        },
        wgpu_html_text::PositionedGlyph {
          x: 10.0,
          y: 2.0,
          w: 12.0,
          h: 20.0,
          uv_min: [0.0, 0.0],
          uv_max: [1.0, 1.0],
          color: [1.0, 1.0, 1.0, 1.0],
        },
      ],
      lines: vec![wgpu_html_text::ShapedLine {
        top: 0.0,
        height: 24.0,
        glyph_range: (0, 2),
      }],
      glyph_chars: vec![],
      text: "ab".to_string(),
      byte_boundaries: wgpu_html_text::utf8_boundaries("ab"),
      width: 22.0,
      height: 24.0,
      ascent: 18.0,
    }),
    text_color: Some([1.0; 4]),
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: wgpu_html_layout::OverflowAxes::visible(),
    resize: wgpu_html_layout::Resize::None,
    opacity: 1.0,
    pointer_events: wgpu_html_models::common::PointerEvents::Auto,
    user_select: wgpu_html_models::common::UserSelect::Auto,
    cursor: wgpu_html_models::common::Cursor::Auto,
    z_index: None,
    image: None,
    background_image: None,
    children: Vec::new(),
    is_fixed: false,
  }
}

fn assert_glyphs_fit_clips(list: &wgpu_html::renderer::DisplayList) {
  for cmd in &list.commands {
    if cmd.kind != wgpu_html::renderer::DisplayCommandKind::Glyph {
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
fn tree_row_painted_glyphs_dont_overlap() {
  let mut tree = wgpu_html_parser::parse(
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
  wgpu_html_tree::register_system_fonts(&mut tree, "sans-serif");
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
fn glyphs_fit_within_default_clip() {
  let root = simple_body_with_glyphs();
  let mut list = DisplayList::new();
  paint_layout(&root, &mut list);
  assert_glyphs_fit_clips(&list);
}

#[test]
fn glyphs_fit_within_overflow_hidden_clip() {
  let mut tree = wgpu_html_parser::parse(
    r#"<body style="margin:0; width:200px; height:80px; overflow:hidden;
                          font-family:sans-serif; font-size:24px; color:white;">
           <span>gy0j</span>
         </body>"#,
  );
  wgpu_html_tree::register_system_fonts(&mut tree, "sans-serif");
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_glyphs_fit_clips(&list);
}

#[test]
fn glyphs_in_tight_overflow_hidden_flex_row() {
  let mut tree = wgpu_html_parser::parse(
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
  wgpu_html_tree::register_system_fonts(&mut tree, "sans-serif");
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_glyphs_fit_clips(&list);
}
