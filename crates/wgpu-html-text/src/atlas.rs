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
//! - A shelf's height is fixed at the height of the first glyph put on
//!   it. Subsequent glyphs that don't fit vertically open a new shelf.
//! - There is no eviction or fragmentation reclamation — overflow
//!   returns `None` so the caller can decide what to do (T7 brings LRU
//!   eviction; T2's caller can rebuild the atlas wholesale if needed).

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
        [
            self.rect.x as f32 / atlas_w as f32,
            self.rect.y as f32 / atlas_h as f32,
        ]
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
        debug_assert_eq!(
            src.len(),
            (w as usize) * (h as usize),
            "src must be w * h bytes"
        );
        if w == 0 || h == 0 {
            // Zero-area glyphs (whitespace) get a deterministic rect at
            // (0, 0) so the caller can still produce a UV range. They
            // contribute no pixels to the atlas.
            return Some(AtlasEntry {
                rect: AtlasRect {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0,
                },
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
        // tall enough.
        for shelf in &mut self.shelves {
            if shelf.cursor_x + w <= self.width && h <= shelf.h {
                let rect = AtlasRect {
                    x: shelf.cursor_x,
                    y: shelf.y,
                    w,
                    h,
                };
                shelf.cursor_x += w;
                return Some(rect);
            }
        }
        // No fit — open a new shelf below the previous ones.
        if self.next_shelf_y + h > self.height {
            return None;
        }
        let shelf = Shelf {
            y: self.next_shelf_y,
            h,
            cursor_x: w,
        };
        let rect = AtlasRect {
            x: 0,
            y: shelf.y,
            w,
            h,
        };
        self.next_shelf_y += h;
        self.shelves.push(shelf);
        Some(rect)
    }

    fn write_pixels(&mut self, rect: AtlasRect, src: &[u8]) {
        let stride = self.width as usize;
        for row in 0..rect.h as usize {
            let dst_start = (rect.y as usize + row) * stride + rect.x as usize;
            let src_start = row * rect.w as usize;
            self.pixels[dst_start..dst_start + rect.w as usize]
                .copy_from_slice(&src[src_start..src_start + rect.w as usize]);
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
mod tests {
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
        assert_eq!(
            entry.rect,
            AtlasRect {
                x: 0,
                y: 0,
                w: 8,
                h: 4
            }
        );
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
        // Same shelf (y=0); next to the first.
        assert_eq!(
            b.rect,
            AtlasRect {
                x: 10,
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
        assert_eq!(b.rect.y, 8); // bumped onto a new shelf below
        assert_eq!(b.rect.x, 0);
    }

    #[test]
    fn horizontal_overflow_opens_new_shelf() {
        // 64-wide atlas; glyph 50 fits, next 20 doesn't on same shelf.
        let mut atlas = Atlas::new(64, 64);
        let _a = atlas.insert(50, 8, &solid(50, 8, 0)).unwrap();
        let b = atlas.insert(20, 8, &solid(20, 8, 0)).unwrap();
        assert_eq!(
            b.rect,
            AtlasRect {
                x: 0,
                y: 8,
                w: 20,
                h: 8
            }
        );
    }

    #[test]
    fn full_atlas_returns_none() {
        let mut atlas = Atlas::new(8, 8);
        // First insert fills the whole atlas vertically.
        let _ = atlas.insert(8, 8, &solid(8, 8, 0)).unwrap();
        // Second insert can't fit anywhere.
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
        assert_eq!(
            e.rect,
            AtlasRect {
                x: 0,
                y: 0,
                w: 0,
                h: 0
            }
        );
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
        assert_eq!(
            b.rect,
            AtlasRect {
                x: 0,
                y: 0,
                w: 8,
                h: 8
            }
        );
        // Two dirty rects accumulated: the full-atlas wipe and the new
        // 8×8 insert.
        let mut count = 0;
        atlas.flush_dirty(|_, _| count += 1);
        assert_eq!(count, 2);
    }
}
