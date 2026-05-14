use lui_glyph::{AtlasEntry, AtlasRect};

#[test]
fn uv_min_at_origin() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 0,
      y: 0,
      w: 10,
      h: 10,
    },
  };
  assert_eq!(entry.uv_min(100, 100), [0.0, 0.0]);
}

#[test]
fn uv_max_at_origin() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 0,
      y: 0,
      w: 10,
      h: 10,
    },
  };
  assert_eq!(entry.uv_max(100, 100), [0.1, 0.1]);
}

#[test]
fn uv_min_at_offset() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 50,
      y: 25,
      w: 10,
      h: 10,
    },
  };
  assert_eq!(entry.uv_min(100, 100), [0.5, 0.25]);
}

#[test]
fn uv_max_at_offset() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 50,
      y: 25,
      w: 10,
      h: 10,
    },
  };
  assert_eq!(entry.uv_max(100, 100), [0.6, 0.35]);
}

#[test]
fn uv_full_atlas() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 0,
      y: 0,
      w: 256,
      h: 256,
    },
  };
  assert_eq!(entry.uv_min(256, 256), [0.0, 0.0]);
  assert_eq!(entry.uv_max(256, 256), [1.0, 1.0]);
}

#[test]
fn uv_min_always_less_than_uv_max() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 30,
      y: 40,
      w: 5,
      h: 8,
    },
  };
  let min = entry.uv_min(200, 200);
  let max = entry.uv_max(200, 200);
  assert!(min[0] < max[0]);
  assert!(min[1] < max[1]);
}

#[test]
fn uv_values_are_in_unit_range() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 100,
      y: 100,
      w: 50,
      h: 50,
    },
  };
  let min = entry.uv_min(256, 256);
  let max = entry.uv_max(256, 256);
  for v in [min[0], min[1], max[0], max[1]] {
    assert!(v >= 0.0 && v <= 1.0, "UV {} out of [0,1] range", v);
  }
}

#[test]
fn zero_size_entry_has_equal_min_max() {
  let entry = AtlasEntry {
    rect: AtlasRect {
      x: 10,
      y: 10,
      w: 0,
      h: 0,
    },
  };
  let min = entry.uv_min(100, 100);
  let max = entry.uv_max(100, 100);
  assert_eq!(min, max);
}
