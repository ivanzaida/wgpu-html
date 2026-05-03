//! Single-channel glyph atlas with a shelf packer.
//!
//! The atlas keeps a CPU-side `Vec<u8>` of `R8Unorm` pixels and a list
//! of dirty rectangles. Glyph rasterisers call `insert(w, h, src)` to
//! reserve space and copy a coverage mask; each insert appends a dirty
//! rect. Per frame, `flush_dirty` drains those rects so a caller can
//! upload only the changed regions to the GPU. A convenience method,
//! `upload(&Queue, &Texture)`, wires that drain to `wgpu::Queue::
//! write_texture` directly.
//!
//! Packing strategy (T2): a simple shelf packer.
//! - Glyphs are placed into horizontal "shelves" stacked top-to-bottom.
//! - A new shelf opens once a glyph won't fit on the current shelf.
//! - A shelf's height is fixed at the height of the first glyph put on it. Subsequent glyphs that don't fit vertically
//!   open a new shelf.
//! - There is no eviction or fragmentation reclamation — overflow returns `None` so the caller can decide what to do
//! (T7 brings LRU eviction; T2's caller can rebuild the atlas wholesale if needed).

/// One-pixel gutter around each glyph so bilinear filtering doesn't
/// bleed zero pixels from neighbouring entries into glyph edges.
const ATLAS_PAD: u32 = 1;

/// Integer rectangle inside the atlas, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasRect {
  pub x: u32,
  pub y: u32,
  pub w: u32,
  pub h: u32,
}

/// Result of a successful `Atlas::insert`. Carries both the pixel rect
/// (so the caller can compute UVs) and the same as a convenience-typed
/// rect for direct rendering.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasEntry {
  pub rect: AtlasRect,
}

impl AtlasEntry {
  /// Top-left UV in [0, 1].
  pub fn uv_min(&self, atlas_w: u32, atlas_h: u32) -> [f32; 2] {
    [self.rect.x as f32 / atlas_w as f32, self.rect.y as f32 / atlas_h as f32]
  }

  /// Bottom-right UV in [0, 1].
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

/// CPU-side atlas. Hand it glyph rasters; ask for dirty rects to
/// upload.
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
  /// Create an empty atlas of the given pixel dimensions. Width and
  /// height should be powers of two for best GPU upload behaviour
  /// but it's not enforced here.
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
  /// exactly `w * h` bytes, row-major, top-down (matches the atlas
  /// layout). Returns `None` if the atlas can't fit the rect — the
  /// caller can then resize / rebuild.
  pub fn insert(&mut self, w: u32, h: u32, src: &[u8]) -> Option<AtlasEntry> {
    debug_assert_eq!(src.len(), (w as usize) * (h as usize), "src must be w * h bytes");
    if w == 0 || h == 0 {
      // Zero-area glyphs (whitespace) get a deterministic rect at
      // (0, 0) so the caller can still produce a UV range. They
      // contribute no pixels to the atlas.
      return Some(AtlasEntry {
        rect: AtlasRect { x: 0, y: 0, w: 0, h: 0 },
      });
    }
    if w > self.width {
      return None;
    }
    let rect = self.allocate(w, h)?;
    self.write_pixels(rect, src);
    self.dirty.push(rect);
    Some(AtlasEntry { rect })
  }

  /// Drain pending dirty rects into `sink`, passing each rect along
  /// with a contiguous `&[u8]` of its pixels, row-major. After
  /// flushing, the dirty list is empty.
  pub fn flush_dirty<F: FnMut(AtlasRect, &[u8])>(&mut self, mut sink: F) {
    // We can't drain into the closure while also borrowing
    // `self.pixels`, so steal the dirty list into a local first.
    let dirty = std::mem::take(&mut self.dirty);
    for rect in dirty {
      let bytes = self.read_rect(rect);
      sink(rect, &bytes);
    }
  }

  /// Convenience: flush all pending dirty rects directly to a
  /// `wgpu::Texture` via `Queue::write_texture`. The texture must be
  /// `R8Unorm` and at least the atlas's dimensions.
  pub fn upload(&mut self, queue: &wgpu::Queue, texture: &wgpu::Texture) {
    self.flush_dirty(|rect, bytes| {
      queue.write_texture(
        wgpu::TexelCopyTextureInfo {
          texture,
          mip_level: 0,
          origin: wgpu::Origin3d {
            x: rect.x,
            y: rect.y,
            z: 0,
          },
          aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(rect.w),
          rows_per_image: Some(rect.h),
        },
        wgpu::Extent3d {
          width: rect.w,
          height: rect.h,
          depth_or_array_layers: 1,
        },
      );
    });
  }

  /// Wipe pixels and packer state. Doesn't shrink the backing
  /// allocation — useful when the caller decides to re-rasterise on
  /// e.g. a DPI change.
  pub fn clear(&mut self) {
    for p in &mut self.pixels {
      *p = 0;
    }
    self.shelves.clear();
    self.next_shelf_y = 0;
    // Force a full-atlas re-upload next flush.
    self.dirty.clear();
    self.dirty.push(AtlasRect {
      x: 0,
      y: 0,
      w: self.width,
      h: self.height,
    });
  }

  // --- internal -----------------------------------------------------------

  fn allocate(&mut self, w: u32, h: u32) -> Option<AtlasRect> {
    // First pass: find a shelf that has room horizontally and is
    // tall enough. Account for padding gutter between entries.
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
    // No fit — open a new shelf below the previous ones.
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
      let dst_start = (rect.y as usize + row) * stride + rect.x as usize;
      let src_start = row * rect.w as usize;
      self.pixels[dst_start..dst_start + rect.w as usize].copy_from_slice(&src[src_start..src_start + rect.w as usize]);
    }
    // Dilate the 1px padding gutter with the nearest edge pixel so
    // bilinear filtering at glyph edges blends with the correct
    // colour instead of black.  Each of the four sides, plus the
    // four corners, is filled.
    if rect.w > 0 && rect.h > 0 {
      let x = rect.x as usize;
      let y = rect.y as usize;
      let w = rect.w as usize;
      let h = rect.h as usize;
      let last_col = x + w - 1;
      let last_row = y + h - 1;
      // Top edge: copy first row
      if y > 0 {
        let tmp: Vec<u8> = self.pixels[y * stride + x..y * stride + x + w].to_vec();
        self.pixels[(y - 1) * stride + x..(y - 1) * stride + x + w].copy_from_slice(&tmp);
      }
      // Bottom edge: copy last row
      if last_row + 1 < self.height as usize {
        let tmp: Vec<u8> = self.pixels[last_row * stride + x..last_row * stride + x + w].to_vec();
        self.pixels[(last_row + 1) * stride + x..(last_row + 1) * stride + x + w]
          .copy_from_slice(&tmp);
      }
      // Left column + top-left / bottom-left corners
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
      // Right column + top-right / bottom-right corners
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

#[cfg(test)]
#[path = "atlas_tests.rs"]
mod tests_atlas;
