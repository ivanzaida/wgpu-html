//! Single-channel R8 glyph atlas with a shelf packer.
//!
//! Glyph rasters are inserted via `insert(w, h, src)`, which reserves
//! space and copies the coverage mask. Dirty rects are tracked so the
//! GPU can upload only changed regions via `flush_dirty`.

/// One-pixel gutter so bilinear filtering doesn't bleed zeros from
/// neighbouring entries into glyph edges.
const ATLAS_PAD: u32 = 1;

/// Integer rectangle inside the atlas, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasRect {
  pub x: u32,
  pub y: u32,
  pub w: u32,
  pub h: u32,
}

/// Result of a successful `Atlas::insert`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasEntry {
  pub rect: AtlasRect,
}

impl AtlasEntry {
  pub fn uv_min(&self, atlas_w: u32, atlas_h: u32) -> [f32; 2] {
    [self.rect.x as f32 / atlas_w as f32, self.rect.y as f32 / atlas_h as f32]
  }

  pub fn uv_max(&self, atlas_w: u32, atlas_h: u32) -> [f32; 2] {
    [
      (self.rect.x + self.rect.w) as f32 / atlas_w as f32,
      (self.rect.y + self.rect.h) as f32 / atlas_h as f32,
    ]
  }
}

#[derive(Debug)]
struct Shelf {
  y: u32,
  h: u32,
  cursor_x: u32,
}

/// CPU-side R8 atlas with shelf packing and dirty-rect tracking.
#[derive(Debug)]
pub struct Atlas {
  width: u32,
  height: u32,
  pixels: Vec<u8>,
  shelves: Vec<Shelf>,
  next_shelf_y: u32,
  dirty: Vec<AtlasRect>,
}

impl Atlas {
  pub fn new(width: u32, height: u32) -> Self {
    let pixels = vec![0u8; (width as usize) * (height as usize)];
    Self {
      width,
      height,
      pixels,
      shelves: Vec::new(),
      next_shelf_y: 0,
      dirty: Vec::new(),
    }
  }

  pub fn dimensions(&self) -> (u32, u32) {
    (self.width, self.height)
  }

  pub fn pixels(&self) -> &[u8] {
    &self.pixels
  }

  /// Reserve a `w × h` region and copy `src` into it. `src` must be
  /// exactly `w * h` bytes, row-major, top-down.
  pub fn insert(&mut self, w: u32, h: u32, src: &[u8]) -> Option<AtlasEntry> {
    debug_assert_eq!(src.len(), (w as usize) * (h as usize));
    if w == 0 || h == 0 {
      return Some(AtlasEntry {
        rect: AtlasRect { x: 0, y: 0, w: 0, h: 0 },
      });
    }
    if w > self.width {
      return None;
    }
    let rect = self.allocate(w, h)?;
    self.write_pixels(rect, src);

    let dx = if rect.x > 0 { 1u32 } else { 0 };
    let dy = if rect.y > 0 { 1u32 } else { 0 };
    let dr = if rect.x + rect.w < self.width { 1u32 } else { 0 };
    let db = if rect.y + rect.h < self.height { 1u32 } else { 0 };
    self.dirty.push(AtlasRect {
      x: rect.x - dx,
      y: rect.y - dy,
      w: rect.w + dx + dr,
      h: rect.h + dy + db,
    });
    Some(AtlasEntry { rect })
  }

  /// Drain pending dirty rects. Calls `sink(rect, &pixels)` for each.
  pub fn flush_dirty<F: FnMut(AtlasRect, &[u8])>(&mut self, mut sink: F) {
    let dirty = std::mem::take(&mut self.dirty);
    for rect in dirty {
      let bytes = self.read_rect(rect);
      sink(rect, &bytes);
    }
  }

  /// Wipe pixels and packer state (e.g. on DPI change).
  pub fn clear(&mut self) {
    self.pixels.fill(0);
    self.shelves.clear();
    self.next_shelf_y = 0;
    self.dirty.clear();
    self.dirty.push(AtlasRect {
      x: 0,
      y: 0,
      w: self.width,
      h: self.height,
    });
  }

  // ── internal ──────────────────────────────────────────────────

  fn allocate(&mut self, w: u32, h: u32) -> Option<AtlasRect> {
    for shelf in &mut self.shelves {
      if shelf.cursor_x + w + ATLAS_PAD <= self.width && h <= shelf.h {
        let rect = AtlasRect {
          x: shelf.cursor_x,
          y: shelf.y,
          w,
          h,
        };
        shelf.cursor_x += w + ATLAS_PAD;
        return Some(rect);
      }
    }
    if self.next_shelf_y + h + ATLAS_PAD > self.height {
      return None;
    }
    let shelf = Shelf {
      y: self.next_shelf_y,
      h,
      cursor_x: w + ATLAS_PAD,
    };
    let rect = AtlasRect { x: 0, y: shelf.y, w, h };
    self.next_shelf_y += h + ATLAS_PAD;
    self.shelves.push(shelf);
    Some(rect)
  }

  fn write_pixels(&mut self, rect: AtlasRect, src: &[u8]) {
    let stride = self.width as usize;
    for row in 0..rect.h as usize {
      let dst = (rect.y as usize + row) * stride + rect.x as usize;
      let src_off = row * rect.w as usize;
      self.pixels[dst..dst + rect.w as usize].copy_from_slice(&src[src_off..src_off + rect.w as usize]);
    }
    if rect.w == 0 || rect.h == 0 {
      return;
    }
    let x = rect.x as usize;
    let y = rect.y as usize;
    let w = rect.w as usize;
    let h = rect.h as usize;
    let last_col = x + w - 1;
    let last_row = y + h - 1;
    if y > 0 {
      let tmp: Vec<u8> = self.pixels[y * stride + x..y * stride + x + w].to_vec();
      self.pixels[(y - 1) * stride + x..(y - 1) * stride + x + w].copy_from_slice(&tmp);
    }
    if last_row + 1 < self.height as usize {
      let tmp: Vec<u8> = self.pixels[last_row * stride + x..last_row * stride + x + w].to_vec();
      self.pixels[(last_row + 1) * stride + x..(last_row + 1) * stride + x + w].copy_from_slice(&tmp);
    }
    if x > 0 {
      let prev = x - 1;
      for row in y..y + h {
        self.pixels[row * stride + prev] = self.pixels[row * stride + x];
      }
      if y > 0 {
        self.pixels[(y - 1) * stride + prev] = self.pixels[y * stride + x];
      }
      if last_row + 1 < self.height as usize {
        self.pixels[(last_row + 1) * stride + prev] = self.pixels[last_row * stride + x];
      }
    }
    if last_col + 1 < self.width as usize {
      let next = last_col + 1;
      for row in y..y + h {
        self.pixels[row * stride + next] = self.pixels[row * stride + last_col];
      }
      if y > 0 {
        self.pixels[(y - 1) * stride + next] = self.pixels[y * stride + last_col];
      }
      if last_row + 1 < self.height as usize {
        self.pixels[(last_row + 1) * stride + next] = self.pixels[last_row * stride + last_col];
      }
    }
  }

  fn read_rect(&self, rect: AtlasRect) -> Vec<u8> {
    let stride = self.width as usize;
    let mut out = Vec::with_capacity((rect.w * rect.h) as usize);
    for row in 0..rect.h as usize {
      let start = (rect.y as usize + row) * stride + rect.x as usize;
      out.extend_from_slice(&self.pixels[start..start + rect.w as usize]);
    }
    out
  }
}
