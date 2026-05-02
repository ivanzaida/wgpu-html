use std::sync::Arc;

use super::*;

fn system_font_bytes() -> Option<Arc<[u8]>> {
  let variants = wgpu_html_tree::system_font_variants();
  variants.first().map(|v| v.data.clone())
}

/// The run height must always equal the requested line-height.
/// Glyph bitmaps may overflow above or below (CSS allows this —
/// the line box is never grown for descenders). The key invariant
/// is that the height is deterministic and content-independent.
#[test]
fn run_height_equals_line_height() {
  let Some(font_data) = system_font_bytes() else {
    eprintln!("skipping: no system font found");
    return;
  };

  let mut registry = FontRegistry::new();
  registry.register(wgpu_html_tree::FontFace::regular("test", font_data));

  let mut ctx = TextContext::new(2048);
  ctx.sync_fonts(&registry);

  let handle = ctx
    .pick_font(&["test"], 400, FontStyleAxis::Normal)
    .expect("font registered");

  for size in [12.0, 16.0, 24.0, 32.0, 48.0] {
    let line_h = size * 1.25;
    let run = ctx
      .shape_and_pack(
        "Hello World gqypj",
        handle,
        size,
        line_h,
        0.0,
        400,
        FontStyleAxis::Normal,
        None,
        [0.0, 0.0, 0.0, 1.0],
      )
      .expect("shaped");

    eprintln!(
      "size={:.0} lh={:.1} run.h={:.1} ascent={:.1}",
      size, line_h, run.height, run.ascent,
    );

    assert_eq!(
      run.height, line_h,
      "size={size}: run height ({:.1}) must equal line-height ({:.1})",
      run.height, line_h,
    );
  }
}

/// Run height must NOT depend on which characters appear in the
/// text. CSS line-height determines the box height; glyphs are
/// allowed to overflow (browsers never expand the line box for
/// descenders). If heights differ, flex `align-items: center`
/// places items at different offsets depending on content.
#[test]
fn run_height_independent_of_glyph_content() {
  let Some(font_data) = system_font_bytes() else {
    eprintln!("skipping: no system font found");
    return;
  };

  let mut registry = FontRegistry::new();
  registry.register(wgpu_html_tree::FontFace::regular("test", font_data));

  let mut ctx = TextContext::new(2048);
  ctx.sync_fonts(&registry);

  let handle = ctx
    .pick_font(&["test"], 400, FontStyleAxis::Normal)
    .expect("font registered");

  let size = 16.0;
  let line_h = size * 1.25; // 20.0

  let no_descenders = ctx
    .shape_and_pack(
      "<",
      handle,
      size,
      line_h,
      0.0,
      400,
      FontStyleAxis::Normal,
      None,
      [0.0; 4],
    )
    .expect("shaped");

  let with_descenders = ctx
    .shape_and_pack(
      "body",
      handle,
      size,
      line_h,
      0.0,
      400,
      FontStyleAxis::Normal,
      None,
      [0.0; 4],
    )
    .expect("shaped");

  eprintln!(
    "no_desc h={:.1}  with_desc h={:.1}  line_h={:.1}",
    no_descenders.height, with_descenders.height, line_h,
  );

  assert_eq!(
    no_descenders.height, with_descenders.height,
    "run height must equal line-height regardless of glyph content \
           (no_desc={:.1}, with_desc={:.1}, line_h={:.1})",
    no_descenders.height, with_descenders.height, line_h,
  );
}

#[test]
fn parse_line_height_multiplier_lucide() {
  let lucide_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../wgpu-html-devtools/fonts/lucide.ttf");
  let Ok(bytes) = std::fs::read(lucide_path) else {
    eprintln!("skipping: no lucide font at {lucide_path}");
    return;
  };
  let mult = parse_line_height_multiplier(&bytes).unwrap();
  eprintln!("Lucide multiplier: {mult}");
  // OS/2 USE_TYPO_METRICS: (1000 - 0 + 90) / 1000 = 1.09
  assert!(
    (mult - 1.09).abs() < 0.01,
    "expected ~1.09 from OS/2 typo metrics, got {mult}"
  );
}
