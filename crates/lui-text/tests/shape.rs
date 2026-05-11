use std::sync::Arc;

use lui_text::{Atlas, FontStyleAxis, TextContext, parse_line_height_multiplier};
use lui_tree::{FontFace, FontRegistry, system_font_variants};

fn system_font_bytes() -> Option<Arc<[u8]>> {
  let variants = system_font_variants();
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
  registry.register(FontFace::regular("test", font_data));

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
  registry.register(FontFace::regular("test", font_data));

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

fn make_ctx() -> Option<(TextContext, lui_text::FontHandle)> {
  let font_data = system_font_bytes()?;
  let mut registry = FontRegistry::new();
  registry.register(FontFace::regular("test", font_data));
  let mut ctx = TextContext::new(2048);
  ctx.sync_fonts(&registry);
  let handle = ctx.pick_font(&["test"], 400, FontStyleAxis::Normal)?;
  Some((ctx, handle))
}

/// Every glyph quad height must exactly match its atlas entry height.
/// A mismatch stretches the UV mapping, making glyphs appear taller
/// and shifting text down.
#[test]
fn glyph_quad_height_matches_atlas_bitmap() {
  let Some((mut ctx, handle)) = make_ctx() else {
    eprintln!("skipping: no system font");
    return;
  };

  for size in [12.0, 16.0, 24.0] {
    let run = ctx
      .shape_and_pack(
        "Hgqypj active email@example.com",
        handle,
        size,
        size * 1.25,
        0.0,
        400,
        FontStyleAxis::Normal,
        None,
        [0.0; 4],
      )
      .expect("shaped");

    let (atlas_w, atlas_h) = ctx.atlas.dimensions();

    for (i, g) in run.glyphs.iter().enumerate() {
      // Recover atlas pixel height from UV span
      let uv_height = g.uv_max[1] - g.uv_min[1];
      let atlas_px_height = (uv_height * atlas_h as f32).round();

      assert!(
        (g.h - atlas_px_height).abs() < 0.5,
        "size={size} glyph {i}: quad_h ({:.1}) != atlas_h ({:.1}). \
         UV stretches the glyph, causing vertical shift.",
        g.h,
        atlas_px_height,
      );
    }
  }
}

/// UV coordinates must cover exactly the atlas rect — no over-sampling
/// into the gutter and no under-sampling that clips edges.
#[test]
fn glyph_uv_covers_exact_atlas_rect() {
  let Some((mut ctx, handle)) = make_ctx() else {
    eprintln!("skipping: no system font");
    return;
  };

  let run = ctx
    .shape_and_pack(
      "Hgqypj",
      handle,
      24.0,
      30.0,
      0.0,
      400,
      FontStyleAxis::Normal,
      None,
      [0.0; 4],
    )
    .expect("shaped");

  let (atlas_w, atlas_h) = ctx.atlas.dimensions();

  for (i, g) in run.glyphs.iter().enumerate() {
    if g.w == 0.0 || g.h == 0.0 {
      continue;
    }

    let uv_w_px = (g.uv_max[0] - g.uv_min[0]) * atlas_w as f32;
    let uv_h_px = (g.uv_max[1] - g.uv_min[1]) * atlas_h as f32;

    // UV pixel span must be integer (atlas rects are pixel-aligned)
    assert!(
      (uv_w_px - uv_w_px.round()).abs() < 0.01,
      "glyph {i}: UV width {uv_w_px:.3} is not pixel-aligned"
    );
    assert!(
      (uv_h_px - uv_h_px.round()).abs() < 0.01,
      "glyph {i}: UV height {uv_h_px:.3} is not pixel-aligned"
    );

    // Quad size must match UV pixel extent
    assert!(
      (g.w - uv_w_px.round()).abs() < 0.5,
      "glyph {i}: quad_w ({:.1}) != uv_w ({:.1})",
      g.w,
      uv_w_px.round()
    );
    assert!(
      (g.h - uv_h_px.round()).abs() < 0.5,
      "glyph {i}: quad_h ({:.1}) != uv_h ({:.1})",
      g.h,
      uv_h_px.round()
    );
  }
}

/// All glyphs must fit within the run's reported [0, height) vertical
/// extent. No glyph quad should have negative y or extend past run.height.
#[test]
fn glyphs_fit_within_run_height() {
  let Some((mut ctx, handle)) = make_ctx() else {
    eprintln!("skipping: no system font");
    return;
  };

  for size in [12.0, 16.0, 24.0, 48.0] {
    let line_h = size * 1.25;
    let run = ctx
      .shape_and_pack(
        "Hgqypj AaBb",
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

    for (i, g) in run.glyphs.iter().enumerate() {
      assert!(
        g.y >= -1.0,
        "size={size} glyph {i}: y={:.1} is above the run top (rounding tolerance -1px)",
        g.y,
      );
      let bottom = g.y + g.h;
      assert!(
        bottom <= run.height + 1.0,
        "size={size} glyph {i}: bottom={bottom:.1} exceeds run.height={:.1} \
         (rounding tolerance +1px)",
        run.height,
      );
    }
  }
}

/// The ascent value must place the baseline so that glyphs with
/// descenders (g, y, p, q, j) extend below it, and ascenders (H, b, d)
/// extend above it. Specifically: ascent should be roughly 70-90% of
/// the line height for typical Latin fonts.
#[test]
fn ascent_is_reasonable_fraction_of_line_height() {
  let Some((mut ctx, handle)) = make_ctx() else {
    eprintln!("skipping: no system font");
    return;
  };

  let size = 16.0;
  let line_h = size * 1.25;
  let run = ctx
    .shape_and_pack(
      "Hgqypj",
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

  let ratio = run.ascent / run.height;
  eprintln!("ascent={:.1} height={:.1} ratio={:.2}", run.ascent, run.height, ratio);
  assert!(
    ratio > 0.5 && ratio < 0.95,
    "ascent/height ratio {ratio:.2} is outside expected range [0.5, 0.95]"
  );
}

/// The dirty rect returned by flush_dirty must include the 1px
/// dilation gutter so the GPU texture receives the dilated edge
/// pixels. Without this, bilinear filtering at glyph edges blends
/// with stale zeros, clipping the top row of every glyph.
#[test]
fn dirty_rect_includes_gutter() {
  let mut atlas = Atlas::new(128, 128);

  // Insert a 4x4 glyph — first shelf starts at y=0, so there's
  // no top gutter row. But right/bottom gutters must be included.
  let src = vec![0xAA_u8; 16];
  let entry = atlas.insert(4, 4, &src).unwrap();
  let rect = entry.rect;

  let mut dirty: Vec<(u32, u32, u32, u32)> = Vec::new();
  atlas.flush_dirty(|r, _| dirty.push((r.x, r.y, r.w, r.h)));

  assert_eq!(dirty.len(), 1);
  let (dx, dy, dw, dh) = dirty[0];

  // First glyph at (0,0): no left/top gutter at atlas edge,
  // but right and bottom gutters must be included.
  assert_eq!(dx, rect.x, "dirty x must match glyph x (no left gutter at edge)");
  assert_eq!(dy, rect.y, "dirty y must match glyph y (no top gutter at edge)");
  assert_eq!(dw, rect.w + 1, "dirty width must include right gutter");
  assert_eq!(dh, rect.h + 1, "dirty height must include bottom gutter");

  // Insert a second glyph that lands on a NEW shelf (force it by
  // using a tall glyph that won't fit on the first shelf).
  let src2 = vec![0xBB_u8; 6 * 6];
  let entry2 = atlas.insert(6, 6, &src2).unwrap();
  let rect2 = entry2.rect;

  dirty.clear();
  atlas.flush_dirty(|r, _| dirty.push((r.x, r.y, r.w, r.h)));

  assert_eq!(dirty.len(), 1);
  let (dx2, dy2, dw2, dh2) = dirty[0];

  // Second glyph is on a new shelf below the first, so it has a
  // top gutter (y > 0). All four gutters must be present.
  assert!(rect2.y > 0, "second glyph must be on a lower shelf");
  assert_eq!(dx2, rect2.x, "dirty x at atlas left edge");
  assert_eq!(dy2, rect2.y - 1, "dirty y must include top gutter");
  assert_eq!(dw2, rect2.w + 1, "dirty width must include right gutter");
  assert_eq!(dh2, rect2.h + 2, "dirty height must include top + bottom gutter");
}

/// Atlas gutter must contain dilated edge pixels, not zeros. This
/// ensures bilinear filtering at glyph edges blends with the correct
/// value rather than transparent black.
#[test]
fn atlas_gutter_is_dilated_not_zero() {
  let mut atlas = Atlas::new(128, 128);

  // Insert a 4x4 glyph where every pixel is 0xFF
  let src = vec![0xFF_u8; 16];
  let entry = atlas.insert(4, 4, &src).unwrap();
  let rect = entry.rect;

  let stride = 128;
  let pixels = atlas.pixels();

  // Bottom gutter row (rect.y + rect.h) should be dilated from last row
  if rect.y + rect.h < 128 {
    let gutter_y = (rect.y + rect.h) as usize;
    for col in rect.x as usize..(rect.x + rect.w) as usize {
      let gutter_val = pixels[gutter_y * stride + col];
      assert_eq!(
        gutter_val, 0xFF,
        "bottom gutter at ({col}, {gutter_y}) = {gutter_val:#x}, expected 0xFF (dilated)"
      );
    }
  }

  // Right gutter column (rect.x + rect.w) should be dilated from last col
  if rect.x + rect.w < 128 {
    let gutter_x = (rect.x + rect.w) as usize;
    for row in rect.y as usize..(rect.y + rect.h) as usize {
      let gutter_val = pixels[row * stride + gutter_x];
      assert_eq!(
        gutter_val, 0xFF,
        "right gutter at ({gutter_x}, {row}) = {gutter_val:#x}, expected 0xFF (dilated)"
      );
    }
  }
}

/// The first row of every glyph bitmap in the atlas must contain
/// at least one nonzero pixel. If it's all zeros the top of the
/// glyph is invisible regardless of positioning.
#[test]
fn atlas_first_row_of_glyph_is_nonzero() {
  let Some((mut ctx, handle)) = make_ctx() else {
    eprintln!("skipping: no system font");
    return;
  };

  for size in [12.0, 16.0, 24.0, 32.0] {
    let run = ctx
      .shape_and_pack(
        "HMWpqg",
        handle,
        size,
        size * 1.25,
        0.0,
        400,
        FontStyleAxis::Normal,
        None,
        [0.0; 4],
      )
      .expect("shaped");

    let (atlas_w, _) = ctx.atlas.dimensions();
    let pixels = ctx.atlas.pixels();

    for (i, g) in run.glyphs.iter().enumerate() {
      if g.w == 0.0 || g.h == 0.0 {
        continue;
      }
      let ax = (g.uv_min[0] * atlas_w as f32).round() as usize;
      let ay = (g.uv_min[1] * atlas_w as f32).round() as usize;
      let gw = g.w as usize;
      let row_start = ay * atlas_w as usize + ax;
      let first_row = &pixels[row_start..row_start + gw];
      let nonzero = first_row.iter().filter(|&&p| p > 0).count();
      assert!(
        nonzero > 0,
        "size={size} glyph[{i}]: first atlas row is all zeros — top of glyph will be invisible"
      );
    }
  }
}

/// Simulates the GPU-side atlas after flush_dirty. For every shaped
/// glyph the texel row directly above the glyph rect (the top
/// gutter) must be nonzero on the GPU. Zero gutter + bilinear
/// filtering = first visible row blended with black.
#[test]
fn gpu_atlas_top_gutter_nonzero_for_shaped_glyphs() {
  let Some((mut ctx, handle)) = make_ctx() else {
    eprintln!("skipping: no system font");
    return;
  };

  let run = ctx
    .shape_and_pack(
      "Hg",
      handle,
      24.0,
      30.0,
      0.0,
      400,
      FontStyleAxis::Normal,
      None,
      [0.0; 4],
    )
    .expect("shaped");

  let (atlas_w, atlas_h) = ctx.atlas.dimensions();
  let mut gpu = vec![0u8; (atlas_w * atlas_h) as usize];
  ctx.atlas.flush_dirty(|rect, bytes| {
    for row in 0..rect.h as usize {
      let dst = (rect.y as usize + row) * atlas_w as usize + rect.x as usize;
      let src = row * rect.w as usize;
      gpu[dst..dst + rect.w as usize].copy_from_slice(&bytes[src..src + rect.w as usize]);
    }
  });

  for (i, g) in run.glyphs.iter().enumerate() {
    if g.w == 0.0 || g.h == 0.0 {
      continue;
    }
    let ax = (g.uv_min[0] * atlas_w as f32).round() as usize;
    let ay = (g.uv_min[1] * atlas_w as f32).round() as usize;
    let gw = g.w as usize;

    if ay == 0 {
      continue; // first shelf — ClampToEdge handles this
    }

    let gutter_start = (ay - 1) * atlas_w as usize + ax;
    let gutter = &gpu[gutter_start..gutter_start + gw];
    let nonzero = gutter.iter().filter(|&&p| p > 0).count();
    assert!(
      nonzero > 0,
      "glyph[{i}]: GPU top-gutter row (y={}) is all zeros — \
       bilinear filtering will blend first row with black",
      ay - 1,
    );
  }
}

#[test]
fn parse_line_height_multiplier_lucide() {
  let lucide_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../lui-devtools/fonts/lucide.ttf");
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
