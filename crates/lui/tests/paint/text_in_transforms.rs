use lui::paint::*;
use lui::text::TextContext;

fn paint_with_fonts(html: &str, w: f32, h: f32) -> lui::renderer::DisplayList {
  let mut tree = lui_parser::parse(html);
  tree.register_system_fonts("DemoSans");
  let mut ctx = TextContext::new(2048);
  let mut ic = lui_layout::ImageCache::default();
  paint_tree_with_text(&tree, &mut ctx, &mut ic, w, h, 1.0, 0.0)
}

#[test]
fn rotated_card_text_wraps_within_container() {
  let list = paint_with_fonts(
    r#"<body style="margin:0;font-family:DemoSans,sans-serif">
      <div style="transform:rotate(15deg);width:200px;padding:16px;background:#222;border-radius:8px">
        <p style="margin:0">Icon and text rotated together here</p>
      </div>
    </body>"#,
    600.0, 400.0,
  );
  assert!(!list.glyphs.is_empty(), "should have glyphs");
  let visible = list.glyphs.iter().filter(|g| g.rect.w > 0.0 && g.rect.h > 0.0).count();
  assert!(visible >= 10, "expected at least 10 visible glyphs, got {visible}");
}

#[test]
fn scaled_card_text_fully_present() {
  let list = paint_with_fonts(
    r#"<body style="margin:0;font-family:DemoSans,sans-serif">
      <div style="transform:scale(1.3);transform-origin:left top;width:200px;padding:16px;background:#222;border-radius:8px">
        <p style="margin:0">Scaled card with check icon text</p>
      </div>
    </body>"#,
    600.0, 400.0,
  );
  assert!(!list.glyphs.is_empty(), "should have glyphs");
  let visible = list.glyphs.iter().filter(|g| g.rect.w > 0.0 && g.rect.h > 0.0).count();
  assert!(visible >= 15, "expected at least 15 visible glyphs, got {visible}");
}

#[test]
fn flex_row_with_svg_and_text_rotated() {
  // Reproduces the demo layout: flex row with SVG icon + text in a rotated card
  let list = paint_with_fonts(
    r#"<body style="margin:0;font-family:DemoSans,sans-serif">
      <div style="transform:rotate(15deg);width:220px;padding:16px;background:#222;border-radius:8px;display:flex;align-items:center;gap:12px">
        <div style="width:48px;height:48px;background:red;border-radius:24px"></div>
        <p style="margin:0">Icon plus text rotated</p>
      </div>
    </body>"#,
    600.0, 400.0,
  );
  // SVG placeholder (red circle) + text
  assert!(list.quads.len() >= 2, "should have icon + card bg quads");
  assert!(!list.glyphs.is_empty(), "should have text glyphs");

  // Print glyph positions for debugging
  let min_x = list.glyphs.iter().map(|g| g.rect.x).fold(f32::MAX, f32::min);
  let max_x = list.glyphs.iter().map(|g| g.rect.x + g.rect.w).fold(f32::MIN, f32::max);
  let text_width = max_x - min_x;
  eprintln!("glyphs: {} total, x range: {min_x:.1}..{max_x:.1} (span={text_width:.1})", list.glyphs.len());

  // Verify no glyph is clipped to a restrictive scissor
  for cmd in &list.commands {
    if cmd.kind != lui::renderer::DisplayCommandKind::Glyph { continue; }
    let g = &list.glyphs[cmd.index as usize];
    let clip = &list.clips[cmd.clip_index as usize];
    if let Some(cr) = clip.rect {
      let glyph_right = g.rect.x + g.rect.w;
      let clip_right = cr.x + cr.w;
      if glyph_right > clip_right + 1.0 {
        eprintln!("CLIP: glyph at x={:.1}..{:.1} outside clip {:.1}..{:.1}",
          g.rect.x, glyph_right, cr.x, clip_right);
      }
    }
  }
}

#[test]
fn text_not_clipped_by_untransformed_scissor() {
  // Key test: when a container is rotated, the text inside should NOT
  // be clipped by a scissor rect based on the untransformed bounds.
  // The scissor should either be expanded or disabled for transformed content.
  let list = paint_with_fonts(
    r#"<body style="margin:0;font-family:DemoSans,sans-serif">
      <div style="transform:rotate(30deg);width:200px;height:60px;padding:10px;background:#222;overflow:visible">
        <span>Text that should not be clipped by a tight rect</span>
      </div>
    </body>"#,
    600.0, 400.0,
  );

  // With overflow:visible and no parent clips, glyphs should have
  // no restrictive clip rect (rect = None) or a viewport-sized clip.
  for cmd in &list.commands {
    if cmd.kind != lui::renderer::DisplayCommandKind::Glyph { continue; }
    let clip = &list.clips[cmd.clip_index as usize];
    if let Some(cr) = clip.rect {
      // If there IS a clip rect, it should be at least viewport-sized
      // or the full container size — not a tight untransformed rect
      // that would chop rotated content.
      assert!(cr.w >= 200.0 || cr.h >= 60.0,
        "clip rect too small for transformed content: {:?}", cr);
    }
  }
}
