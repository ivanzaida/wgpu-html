use super::*;

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
  assert!(chunks[0].1.iter().all(|&b| b == 1));
  assert!(chunks[1].1.iter().all(|&b| b == 2));

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
