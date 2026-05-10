use lui_text::{Atlas, AtlasRect};

fn solid(w: u32, h: u32, value: u8) -> Vec<u8> {
  vec![value; (w * h) as usize]
}

#[test]
fn new_atlas_is_empty_and_clean() {
  let atlas = Atlas::new(64, 64);
  assert_eq!(atlas.dimensions(), (64, 64));
  assert!(atlas.pixels().iter().all(|&b| b == 0));
}

#[test]
fn insert_places_glyph_top_left() {
  let mut atlas = Atlas::new(64, 64);
  let entry = atlas.insert(8, 4, &solid(8, 4, 0xAA)).unwrap();
  assert_eq!(entry.rect, AtlasRect { x: 0, y: 0, w: 8, h: 4 });
  // Pixels were copied into the right rows.
  for row in 0..4 {
    for col in 0..8 {
      assert_eq!(atlas.pixels()[row * 64 + col], 0xAA);
    }
  }
}

#[test]
fn second_insert_packs_on_same_shelf() {
  let mut atlas = Atlas::new(64, 64);
  let _a = atlas.insert(10, 8, &solid(10, 8, 0x10)).unwrap();
  let b = atlas.insert(12, 6, &solid(12, 6, 0x20)).unwrap();
  // Same shelf (y=0); next to the first with 1px pad gutter.
  assert_eq!(
    b.rect,
    AtlasRect {
      x: 11,
      y: 0,
      w: 12,
      h: 6
    }
  );
}

#[test]
fn shelf_height_is_first_insert_height() {
  // Shelf height pinned to the first glyph (8). A taller glyph
  // (10) should open a new shelf rather than fit into the first.
  let mut atlas = Atlas::new(64, 64);
  let _a = atlas.insert(10, 8, &solid(10, 8, 0)).unwrap();
  let b = atlas.insert(10, 10, &solid(10, 10, 0)).unwrap();
  assert_eq!(b.rect.y, 9); // bumped onto a new shelf below (8h + 1 pad)
  assert_eq!(b.rect.x, 0);
}

#[test]
fn horizontal_overflow_opens_new_shelf() {
  // 64-wide atlas; glyph 50 fits, next 20 doesn't on same shelf
  // (50 + 1 pad + 20 = 71 > 64).
  let mut atlas = Atlas::new(64, 64);
  let _a = atlas.insert(50, 8, &solid(50, 8, 0)).unwrap();
  let b = atlas.insert(20, 8, &solid(20, 8, 0)).unwrap();
  assert_eq!(
    b.rect,
    AtlasRect {
      x: 0,
      y: 9,
      w: 20,
      h: 8
    }
  );
}

#[test]
fn full_atlas_returns_none() {
  // 8×8 atlas with 1px pad: an 8×8 glyph would need
  // next_shelf_y = 8+1 = 9 which overflows the atlas.
  let mut atlas = Atlas::new(8, 8);
  assert!(atlas.insert(8, 8, &solid(8, 8, 0)).is_none());
  // A 7×7 fits; the next one doesn't.
  let _ = atlas.insert(7, 7, &solid(7, 7, 0)).unwrap();
  assert!(atlas.insert(1, 1, &solid(1, 1, 0)).is_none());
}

#[test]
fn glyph_wider_than_atlas_returns_none() {
  let mut atlas = Atlas::new(8, 8);
  assert!(atlas.insert(9, 4, &solid(9, 4, 0)).is_none());
}

#[test]
fn zero_size_insert_returns_empty_entry() {
  let mut atlas = Atlas::new(8, 8);
  // Whitespace glyphs etc. — they don't take pixels but still
  // give the caller a valid (empty) AtlasEntry.
  let e = atlas.insert(0, 0, &[]).unwrap();
  assert_eq!(e.rect, AtlasRect { x: 0, y: 0, w: 0, h: 0 });
}

#[test]
fn flush_dirty_drains_and_returns_pixels() {
  let mut atlas = Atlas::new(64, 64);
  let _a = atlas.insert(4, 4, &solid(4, 4, 1)).unwrap();
  let _b = atlas.insert(4, 4, &solid(4, 4, 2)).unwrap();

  let mut chunks: Vec<(AtlasRect, Vec<u8>)> = Vec::new();
  atlas.flush_dirty(|rect, bytes| chunks.push((rect, bytes.to_vec())));
  assert_eq!(chunks.len(), 2);
  // Dirty rects now include the dilation gutter. The gutter column
  // shared between _a and _b may have been overwritten by _b's
  // left-edge dilation, so check the glyph core region instead.
  let r0 = &chunks[0].0;
  for row in 0..4u32 {
    for col in 0..4u32 {
      let idx = ((row.wrapping_sub(r0.y)) * r0.w + (col.wrapping_sub(r0.x))) as usize;
      assert_eq!(chunks[0].1[idx], 1, "glyph _a core at ({col},{row})");
    }
  }
  let r1 = &chunks[1].0;
  let b_x = _b.rect.x;
  for row in 0..4u32 {
    for col in b_x..b_x + 4 {
      let idx = ((row.wrapping_sub(r1.y)) * r1.w + (col.wrapping_sub(r1.x))) as usize;
      assert_eq!(chunks[1].1[idx], 2, "glyph _b core at ({col},{row})");
    }
  }

  // A second flush sees nothing — the list was drained.
  let mut count = 0;
  atlas.flush_dirty(|_, _| count += 1);
  assert_eq!(count, 0);
}

#[test]
fn uv_extents_are_normalised() {
  let mut atlas = Atlas::new(100, 50);
  let entry = atlas.insert(10, 10, &solid(10, 10, 0)).unwrap();
  let (w, h) = atlas.dimensions();
  let lo = entry.uv_min(w, h);
  let hi = entry.uv_max(w, h);
  // Edge-of-atlas rect — no inset used by this method (inset is
  // applied at shape time).
  assert_eq!(lo, [0.0, 0.0]);
  assert_eq!(hi, [0.10, 0.20]);
}

/// The flushed bytes for a glyph on the second shelf must include the
/// dilated top-gutter row. Without it the GPU texture has zeros above
/// the glyph, and bilinear filtering blends the first visible row with
/// black — visually cutting off the top of every glyph.
#[test]
fn flushed_top_gutter_row_matches_first_bitmap_row() {
  let mut atlas = Atlas::new(64, 64);

  // Fill shelf 0 with a wide glyph so the next insert opens shelf 1.
  let _ = atlas.insert(60, 8, &solid(60, 8, 0x11)).unwrap();
  atlas.flush_dirty(|_, _| {}); // drain

  // Insert a glyph whose first row is 0xAA. On the second shelf
  // the top gutter (row y-1) must be dilated and flushed.
  let mut src = vec![0xCC_u8; 6 * 6];
  src[0..6].fill(0xAA);
  let entry = atlas.insert(6, 6, &src).unwrap();
  let rect = entry.rect;
  assert!(rect.y > 0, "glyph must be on a lower shelf");

  let mut flushed: Vec<(AtlasRect, Vec<u8>)> = Vec::new();
  atlas.flush_dirty(|r, bytes| flushed.push((r, bytes.to_vec())));
  assert_eq!(flushed.len(), 1);

  let (dirty, bytes) = &flushed[0];

  // The dirty rect must start one row above the glyph (the gutter).
  assert_eq!(
    dirty.y,
    rect.y - 1,
    "dirty rect must include top gutter row"
  );

  // The first row in the flushed bytes IS the gutter row. It must
  // equal the glyph's own first row (dilated copy), not zeros.
  let gutter_row: &[u8] = &bytes[0..dirty.w as usize];
  // The gutter columns that overlap the glyph rect should be 0xAA
  // (the glyph's first row value).
  for col in 0..rect.w {
    let idx = (rect.x - dirty.x + col) as usize;
    assert_eq!(
      gutter_row[idx], 0xAA,
      "top gutter at col {col} is {:#x}, expected 0xAA (dilated first row)",
      gutter_row[idx]
    );
  }
}

/// Simulates what the GPU sees. After insert + flush_dirty, build a
/// "GPU texture" from the flushed rects. The pixel directly above
/// every glyph (the top gutter) must not be zero — otherwise bilinear
/// filtering at the glyph's top edge blends with black, clipping the
/// first visible row.
#[test]
fn gpu_texture_top_gutter_is_nonzero() {
  let mut atlas = Atlas::new(64, 64);

  // Two glyphs on separate shelves so the second has a top gutter.
  let _ = atlas.insert(10, 8, &solid(10, 8, 0xFF)).unwrap();
  let b = atlas.insert(10, 10, &solid(10, 10, 0xFF)).unwrap();

  // Simulate GPU: start with all zeros, apply flushed dirty rects.
  let mut gpu = vec![0u8; 64 * 64];
  atlas.flush_dirty(|rect, bytes| {
    for row in 0..rect.h as usize {
      let dst = (rect.y as usize + row) * 64 + rect.x as usize;
      let src = row * rect.w as usize;
      gpu[dst..dst + rect.w as usize].copy_from_slice(&bytes[src..src + rect.w as usize]);
    }
  });

  // Check the row directly above glyph B. It's the top gutter.
  let gutter_y = b.rect.y as usize - 1;
  for col in b.rect.x as usize..(b.rect.x + b.rect.w) as usize {
    let val = gpu[gutter_y * 64 + col];
    assert_ne!(
      val, 0,
      "GPU pixel at ({col}, {gutter_y}) is zero — bilinear filtering \
       will blend the first row of glyph B with black, clipping the top"
    );
  }
}

#[test]
fn clear_resets_packer_and_marks_full_dirty() {
  let mut atlas = Atlas::new(16, 16);
  let _a = atlas.insert(8, 8, &solid(8, 8, 0xFF)).unwrap();
  atlas.clear();
  // Pixels zeroed.
  assert!(atlas.pixels().iter().all(|&b| b == 0));
  // Re-insert lands at top-left again (packer reset).
  let b = atlas.insert(8, 8, &solid(8, 8, 0)).unwrap();
  assert_eq!(b.rect, AtlasRect { x: 0, y: 0, w: 8, h: 8 });
  // Two dirty rects accumulated: the full-atlas wipe and the new
  // 8×8 insert.
  let mut count = 0;
  atlas.flush_dirty(|_, _| count += 1);
  assert_eq!(count, 2);
}
