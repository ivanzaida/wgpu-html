//! Font registry — owned by `Tree`.
//!
//! See `docs/text.md` §3 for rationale: fonts belong to the document,
//! not to a process-global store. Each `Tree` carries its own
//! `FontRegistry`; cascade and layout consult that registry alone.
//!
//! This module is intentionally pure data: it knows nothing about
//! cosmic-text, rustybuzz, or the GPU. The `wgpu-html-text` crate
//! consumes `&FontRegistry` to build its shaper-side database.

use std::sync::Arc;

/// CSS `font-style` axis. `Oblique` and `Italic` are interchangeable
/// during font matching (preferred order: exact > the other > normal).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontStyleAxis {
  Normal,
  Italic,
  Oblique,
}

impl Default for FontStyleAxis {
  fn default() -> Self {
    Self::Normal
  }
}

/// One physical font face: a single `(family, weight, style)` pairing
/// backed by some font bytes (typically OTF / TTF).
///
/// `data` is `Arc<[u8]>` so the same font file can be registered under
/// multiple aliases without copying, and a downstream font system can
/// hold the same bytes without taking ownership.
#[derive(Debug, Clone)]
pub struct FontFace {
  pub family: String,
  pub weight: u16,
  pub style: FontStyleAxis,
  pub data: Arc<[u8]>,
}

impl FontFace {
  /// Convenience: 400-weight, normal-style face for the given family.
  pub fn regular(family: impl Into<String>, data: Arc<[u8]>) -> Self {
    Self {
      family: family.into(),
      weight: 400,
      style: FontStyleAxis::Normal,
      data,
    }
  }
}

/// Stable index into a `FontRegistry`. Returned by
/// `Tree::register_font` and the registry's lookup methods.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FontHandle(pub usize);

/// All faces available to one document.
///
/// Lookup follows the simplified CSS-Fonts-3 matching described in
/// `docs/text.md` §6. The registry is stored in registration order;
/// when several faces tie on the matching score, the one registered
/// later wins.
#[derive(Debug, Clone)]
pub struct FontRegistry {
  faces: Vec<FontFace>,
  /// Monotonically increasing counter, bumped on every `register()`
  /// call. Consumers (e.g. `TextContext::sync_fonts`) compare this
  /// against their last-seen value to skip redundant work.
  generation: u64,
}

impl Default for FontRegistry {
  fn default() -> Self {
    Self {
      faces: Vec::new(),
      generation: 0,
    }
  }
}

impl FontRegistry {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn len(&self) -> usize {
    self.faces.len()
  }

  pub fn is_empty(&self) -> bool {
    self.faces.is_empty()
  }

  /// Current generation counter. Bumped on every `register()`.
  pub fn generation(&self) -> u64 {
    self.generation
  }

  /// Push a face; returns its handle.
  pub fn register(&mut self, face: FontFace) -> FontHandle {
    let h = FontHandle(self.faces.len());
    self.faces.push(face);
    self.generation += 1;
    h
  }

  pub fn get(&self, handle: FontHandle) -> Option<&FontFace> {
    self.faces.get(handle.0)
  }

  pub fn iter(&self) -> impl Iterator<Item = (FontHandle, &FontFace)> {
    self.faces.iter().enumerate().map(|(i, f)| (FontHandle(i), f))
  }

  /// Find the best face for one family name plus a target weight and
  /// style. Family is matched case-insensitively (ASCII). Ties on
  /// score break toward the face registered later, so a host can
  /// override an earlier registration by re-registering.
  ///
  /// Returns `None` if no face has that family name.
  pub fn find(&self, family: &str, weight: u16, style: FontStyleAxis) -> Option<FontHandle> {
    let mut best: Option<(u32, FontHandle)> = None;
    for (h, face) in self.iter() {
      if !family_eq(&face.family, family) {
        continue;
      }
      let score = match_score(face, weight, style);
      match best {
        // Strict <=: later registrations win on ties.
        Some((b, _)) if score > b => {}
        _ => best = Some((score, h)),
      }
    }
    best.map(|(_, h)| h)
  }

  /// Walk a CSS-style family list left-to-right, returning the first
  /// family that has any registered face (then picking the best
  /// `(weight, style)` within it). This is the entry point a layout
  /// engine should call.
  ///
  /// Generic-family fallback: if no name in the list matches a
  /// registered family but the list contains a CSS generic
  /// keyword (`sans-serif`, `serif`, `monospace`, `cursive`,
  /// `fantasy`, `system-ui`, `ui-*`, `-apple-system`, …), the
  /// best `(weight, style)`-scoring face from the entire
  /// registry is returned. This makes plain `font-family:
  /// sans-serif` resolve even when the host registered fonts
  /// under a specific name like `"Inter"`.
  pub fn find_first(&self, families: &[&str], weight: u16, style: FontStyleAxis) -> Option<FontHandle> {
    for fam in families {
      if let Some(h) = self.find(fam, weight, style) {
        return Some(h);
      }
    }
    if !families.iter().any(|f| is_generic_family(f)) {
      return None;
    }
    // Generic fallback: pick the best-scoring face across all
    // registered families. Unlike `find` (where later registrations
    // win ties to allow overrides), here first-wins-on-ties so
    // the primary text font (registered first) is preferred over
    // icon/symbol fonts (registered later at the same weight).
    let mut best: Option<(u32, FontHandle)> = None;
    for (h, face) in self.iter() {
      let score = match_score(face, weight, style);
      match best {
        Some((b, _)) if score >= b => {}
        _ => best = Some((score, h)),
      }
    }
    best.map(|(_, h)| h)
  }
}

/// Whether `name` is one of the CSS generic font-family keywords
/// (CSS-Fonts-4 §3.1.1). Used as the trigger for `find_first`'s
/// "any registered face" fallback.
fn is_generic_family(name: &str) -> bool {
  let n = name.to_ascii_lowercase();
  matches!(
    n.as_str(),
    "sans-serif"
      | "serif"
      | "monospace"
      | "cursive"
      | "fantasy"
      | "system-ui"
      | "ui-sans-serif"
      | "ui-serif"
      | "ui-monospace"
      | "ui-rounded"
      | "math"
      | "emoji"
      | "fangsong"
      | "-apple-system"
      | "blinkmacsystemfont"
  )
}

/// Case-insensitive ASCII comparison of family names. CSS treats family
/// names case-insensitively; non-ASCII characters compare verbatim.
fn family_eq(a: &str, b: &str) -> bool {
  a.len() == b.len() && a.bytes().zip(b.bytes()).all(|(x, y)| x.eq_ignore_ascii_case(&y))
}

/// Score a candidate face against a `(weight, style)` target. Lower is
/// better. Combines a style-distance band with weight distance, with a
/// large penalty for being on the wrong side of the target weight when
/// the target prefers a direction (CSS-Fonts-3 §5.2.4, simplified):
///
/// - `target < 400` prefers lighter, then heavier.
/// - `target > 500` prefers heavier, then lighter.
/// - `target ∈ [400, 500]` is bidirectional: closest wins.
fn match_score(face: &FontFace, weight: u16, style: FontStyleAxis) -> u32 {
  style_band(face.style, style) * 1_000_000 + weight_distance(face.weight, weight)
}

fn style_band(candidate: FontStyleAxis, target: FontStyleAxis) -> u32 {
  use FontStyleAxis::*;
  match (target, candidate) {
    (Normal, Normal) | (Italic, Italic) | (Oblique, Oblique) => 0,
    // Italic and Oblique are interchangeable per CSS, with a small
    // penalty so an exact match still wins when both exist.
    (Italic, Oblique) | (Oblique, Italic) => 1,
    // Falling back from an italic-ish target to Normal (or vice
    // versa) is the worst non-empty outcome.
    _ => 2,
  }
}

fn weight_distance(candidate: u16, target: u16) -> u32 {
  let t = target as i32;
  let c = candidate as i32;
  let dist = (c - t).unsigned_abs();
  let prefers_heavier = t > 500;
  let prefers_lighter = t < 400;
  let wrong_direction = (prefers_heavier && c < t) || (prefers_lighter && c > t);
  if wrong_direction {
    // 10_000 keeps wrong-direction matches strictly worse than any
    // right-direction match (max raw weight gap is 800).
    10_000 + dist
  } else {
    dist
  }
}

#[cfg(test)]
#[path = "fonts_tests.rs"]
mod tests_fonts;
