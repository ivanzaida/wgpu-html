use lui_glyph::Atlas;

#[test]
fn flush_dirty_returns_rects_after_insert() {
    let mut atlas = Atlas::new(64, 64);
    atlas.insert(4, 4, &vec![128u8; 16]);

    let mut rects = Vec::new();
    atlas.flush_dirty(|rect, _data| rects.push(rect));

    assert_eq!(rects.len(), 1);
}

#[test]
fn flush_dirty_clears_pending_rects() {
    let mut atlas = Atlas::new(64, 64);
    atlas.insert(4, 4, &vec![128u8; 16]);

    atlas.flush_dirty(|_, _| {});

    let mut count = 0;
    atlas.flush_dirty(|_, _| count += 1);
    assert_eq!(count, 0, "second flush should have nothing");
}

#[test]
fn flush_dirty_data_is_not_empty() {
    let mut atlas = Atlas::new(64, 64);
    atlas.insert(3, 3, &vec![200u8; 9]);

    atlas.flush_dirty(|rect, data| {
        assert!(!data.is_empty());
        assert_eq!(data.len(), (rect.w * rect.h) as usize);
    });
}

#[test]
fn clear_resets_pixels_to_zero() {
    let mut atlas = Atlas::new(32, 32);
    atlas.insert(5, 5, &vec![255u8; 25]);
    atlas.clear();

    assert!(atlas.pixels().iter().all(|&b| b == 0));
}

#[test]
fn clear_allows_reinsertion() {
    let mut atlas = Atlas::new(32, 32);
    let data = vec![100u8; 10 * 10];

    // Fill until full
    while atlas.insert(10, 10, &data).is_some() {}

    atlas.clear();

    // Should be able to insert again
    assert!(atlas.insert(10, 10, &data).is_some());
}

#[test]
fn clear_emits_full_dirty_rect() {
    let mut atlas = Atlas::new(64, 128);
    atlas.clear();

    let mut rects = Vec::new();
    atlas.flush_dirty(|rect, _| rects.push(rect));

    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].x, 0);
    assert_eq!(rects[0].y, 0);
    assert_eq!(rects[0].w, 64);
    assert_eq!(rects[0].h, 128);
}

#[test]
fn flush_dirty_no_inserts_yields_nothing() {
    let mut atlas = Atlas::new(64, 64);
    let mut count = 0;
    atlas.flush_dirty(|_, _| count += 1);
    assert_eq!(count, 0);
}

#[test]
fn bilinear_gutter_copies_edge_pixels() {
    let mut atlas = Atlas::new(64, 64);
    let src = vec![200u8; 4 * 4];
    let entry = atlas.insert(4, 4, &src).unwrap();
    let rect = entry.rect;

    let stride = 64;
    // The gutter should copy edge pixel values to adjacent pixels
    // Check right gutter (if space exists)
    if rect.x + rect.w < 64 {
        let right_gutter = atlas.pixels()[(rect.y as usize) * stride + (rect.x + rect.w) as usize];
        let last_col = atlas.pixels()[(rect.y as usize) * stride + (rect.x + rect.w - 1) as usize];
        assert_eq!(right_gutter, last_col, "right gutter should match last column pixel");
    }
}

#[test]
fn shelf_packing_uses_horizontal_space() {
    let mut atlas = Atlas::new(64, 64);
    let data_a = vec![1u8; 10 * 10];
    let data_b = vec![2u8; 10 * 8];

    let a = atlas.insert(10, 10, &data_a).unwrap().rect;
    let b = atlas.insert(10, 8, &data_b).unwrap().rect;

    // Both should be on the same shelf (same y) since b fits in a's shelf height
    assert_eq!(a.y, b.y, "second insert should share the shelf");
    assert!(b.x > a.x, "second insert should be to the right");
}

#[test]
fn different_heights_create_new_shelves() {
    let mut atlas = Atlas::new(64, 64);
    // Fill first shelf completely
    let wide_data = vec![1u8; 60 * 10];
    atlas.insert(60, 10, &wide_data).unwrap();

    // Next insert won't fit horizontally → new shelf
    let data = vec![2u8; 10 * 10];
    let b = atlas.insert(10, 10, &data).unwrap().rect;

    assert!(b.y > 0, "should be on a new shelf below");
}
