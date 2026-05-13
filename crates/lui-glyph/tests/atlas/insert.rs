use lui_glyph::Atlas;

#[test]
fn new_atlas_has_correct_dimensions() {
    let atlas = Atlas::new(512, 256);
    assert_eq!(atlas.dimensions(), (512, 256));
}

#[test]
fn new_atlas_pixels_are_zero() {
    let atlas = Atlas::new(64, 64);
    assert!(atlas.pixels().iter().all(|&b| b == 0));
}

#[test]
fn new_atlas_pixel_count_matches_dimensions() {
    let atlas = Atlas::new(100, 50);
    assert_eq!(atlas.pixels().len(), 100 * 50);
}

#[test]
fn insert_zero_size_succeeds() {
    let mut atlas = Atlas::new(64, 64);
    let entry = atlas.insert(0, 0, &[]);
    assert!(entry.is_some());
    let rect = entry.unwrap().rect;
    assert_eq!(rect.w, 0);
    assert_eq!(rect.h, 0);
}

#[test]
fn insert_single_pixel() {
    let mut atlas = Atlas::new(64, 64);
    let entry = atlas.insert(1, 1, &[255]);
    assert!(entry.is_some());
    let rect = entry.unwrap().rect;
    assert_eq!(rect.w, 1);
    assert_eq!(rect.h, 1);
}

#[test]
fn insert_writes_correct_pixels() {
    let mut atlas = Atlas::new(64, 64);
    let src = vec![100u8; 4 * 3];
    let entry = atlas.insert(4, 3, &src).unwrap();
    let rect = entry.rect;

    let stride = 64;
    for row in 0..3 {
        for col in 0..4 {
            let px = atlas.pixels()[(rect.y as usize + row) * stride + rect.x as usize + col];
            assert_eq!(px, 100, "pixel at ({}, {}) should be 100", col, row);
        }
    }
}

#[test]
fn insert_too_wide_returns_none() {
    let mut atlas = Atlas::new(32, 32);
    let data = vec![0u8; 64 * 1];
    assert!(atlas.insert(64, 1, &data).is_none());
}

#[test]
fn insert_too_tall_returns_none() {
    let mut atlas = Atlas::new(32, 32);
    let data = vec![0u8; 1 * 64];
    assert!(atlas.insert(1, 64, &data).is_none());
}

#[test]
fn insert_exact_width_succeeds() {
    let mut atlas = Atlas::new(16, 16);
    let data = vec![128u8; 16 * 1];
    let entry = atlas.insert(16, 1, &data);
    assert!(entry.is_some());
}

#[test]
fn multiple_inserts_do_not_overlap() {
    let mut atlas = Atlas::new(256, 256);
    let a = atlas.insert(10, 10, &vec![1u8; 100]).unwrap().rect;
    let b = atlas.insert(10, 10, &vec![2u8; 100]).unwrap().rect;

    let a_right = a.x + a.w;
    let a_bottom = a.y + a.h;
    let b_right = b.x + b.w;
    let b_bottom = b.y + b.h;

    let no_x_overlap = a_right <= b.x || b_right <= a.x;
    let no_y_overlap = a_bottom <= b.y || b_bottom <= a.y;
    assert!(no_x_overlap || no_y_overlap, "rects overlap: a={:?}, b={:?}", a, b);
}

#[test]
fn many_inserts_until_full() {
    let mut atlas = Atlas::new(64, 64);
    let data = vec![255u8; 8 * 8];
    let mut count = 0;
    while atlas.insert(8, 8, &data).is_some() {
        count += 1;
        if count > 100 { break; }
    }
    assert!(count > 1, "should fit more than 1 glyph");
    assert!(count < 100, "should eventually run out of space");
}

#[test]
fn insert_returns_rects_within_atlas_bounds() {
    let mut atlas = Atlas::new(128, 128);
    for _ in 0..10 {
        if let Some(entry) = atlas.insert(12, 14, &vec![0u8; 12 * 14]) {
            let r = entry.rect;
            assert!(r.x + r.w <= 128, "x+w={} exceeds width", r.x + r.w);
            assert!(r.y + r.h <= 128, "y+h={} exceeds height", r.y + r.h);
        }
    }
}
