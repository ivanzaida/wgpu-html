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
#[derive(Debug, Clone, Default)]
pub struct FontRegistry {
    faces: Vec<FontFace>,
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

    /// Push a face; returns its handle.
    pub fn register(&mut self, face: FontFace) -> FontHandle {
        let h = FontHandle(self.faces.len());
        self.faces.push(face);
        h
    }

    pub fn get(&self, handle: FontHandle) -> Option<&FontFace> {
        self.faces.get(handle.0)
    }

    pub fn iter(&self) -> impl Iterator<Item = (FontHandle, &FontFace)> {
        self.faces
            .iter()
            .enumerate()
            .map(|(i, f)| (FontHandle(i), f))
    }

    /// Find the best face for one family name plus a target weight and
    /// style. Family is matched case-insensitively (ASCII). Ties on
    /// score break toward the face registered later, so a host can
    /// override an earlier registration by re-registering.
    ///
    /// Returns `None` if no face has that family name.
    pub fn find(
        &self,
        family: &str,
        weight: u16,
        style: FontStyleAxis,
    ) -> Option<FontHandle> {
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
    pub fn find_first(
        &self,
        families: &[&str],
        weight: u16,
        style: FontStyleAxis,
    ) -> Option<FontHandle> {
        for fam in families {
            if let Some(h) = self.find(fam, weight, style) {
                return Some(h);
            }
        }
        None
    }
}

/// Case-insensitive ASCII comparison of family names. CSS treats family
/// names case-insensitively; non-ASCII characters compare verbatim.
fn family_eq(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .all(|(x, y)| x.eq_ignore_ascii_case(&y))
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
    let wrong_direction =
        (prefers_heavier && c < t) || (prefers_lighter && c > t);
    if wrong_direction {
        // 10_000 keeps wrong-direction matches strictly worse than any
        // right-direction match (max raw weight gap is 800).
        10_000 + dist
    } else {
        dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(b: &[u8]) -> Arc<[u8]> {
        Arc::from(b.to_vec().into_boxed_slice())
    }

    fn face(family: &str, weight: u16, style: FontStyleAxis, marker: u8) -> FontFace {
        FontFace {
            family: family.to_string(),
            weight,
            style,
            data: data(&[marker]),
        }
    }

    #[test]
    fn register_returns_sequential_handles() {
        let mut r = FontRegistry::new();
        let a = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let b = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
        assert_eq!(a, FontHandle(0));
        assert_eq!(b, FontHandle(1));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn find_resolves_exact_match() {
        let mut r = FontRegistry::new();
        let regular = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
        assert_eq!(
            r.find("Inter", 400, FontStyleAxis::Normal),
            Some(regular)
        );
        assert_eq!(r.find("Inter", 700, FontStyleAxis::Normal), Some(bold));
        // Resolves to the bytes we registered, not just the handle.
        assert_eq!(r.get(regular).unwrap().data[0], 1);
        assert_eq!(r.get(bold).unwrap().data[0], 2);
    }

    #[test]
    fn find_is_case_insensitive_on_family() {
        let mut r = FontRegistry::new();
        r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        assert!(r.find("INTER", 400, FontStyleAxis::Normal).is_some());
        assert!(r.find("inter", 400, FontStyleAxis::Normal).is_some());
        assert!(r.find("Roboto", 400, FontStyleAxis::Normal).is_none());
    }

    #[test]
    fn find_picks_closer_weight() {
        let mut r = FontRegistry::new();
        let regular = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
        // 600 → 700 is closer than 600 → 400.
        assert_eq!(r.find("Inter", 600, FontStyleAxis::Normal), Some(bold));
        // 500 → 400 (200 gap) is closer than 500 → 700 (200 gap, but
        // ties break to the later-registered, which is bold, also at 200 gap).
        // 450 → 400 wins (50 vs 250).
        assert_eq!(
            r.find("Inter", 450, FontStyleAxis::Normal),
            Some(regular)
        );
    }

    #[test]
    fn find_prefers_lighter_for_sub_400_targets() {
        let mut r = FontRegistry::new();
        let regular = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let _bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
        // Target 300: 400 is lighter → wrong direction, but 700 is also
        // wrong; 400 closer (100 vs 400). 400 wins.
        assert_eq!(r.find("Inter", 300, FontStyleAxis::Normal), Some(regular));
    }

    #[test]
    fn find_prefers_heavier_for_super_500_targets() {
        let mut r = FontRegistry::new();
        let _light = r.register(face("Inter", 300, FontStyleAxis::Normal, 1));
        let bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
        // Target 600: 700 is heavier (right direction, gap 100); 300 is
        // lighter (wrong direction, gap 300). 700 wins.
        assert_eq!(r.find("Inter", 600, FontStyleAxis::Normal), Some(bold));
    }

    #[test]
    fn style_axis_exact_beats_swap_beats_normal() {
        let mut r = FontRegistry::new();
        let normal = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let italic = r.register(face("Inter", 400, FontStyleAxis::Italic, 2));
        let oblique = r.register(face("Inter", 400, FontStyleAxis::Oblique, 3));
        // Exact:
        assert_eq!(r.find("Inter", 400, FontStyleAxis::Italic), Some(italic));
        assert_eq!(r.find("Inter", 400, FontStyleAxis::Oblique), Some(oblique));
        assert_eq!(r.find("Inter", 400, FontStyleAxis::Normal), Some(normal));
    }

    #[test]
    fn italic_swaps_to_oblique_when_no_italic() {
        let mut r = FontRegistry::new();
        let normal = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let oblique = r.register(face("Inter", 400, FontStyleAxis::Oblique, 2));
        // Asking for Italic with only Normal + Oblique present: Oblique
        // wins via the swap band (better than Normal).
        assert_eq!(
            r.find("Inter", 400, FontStyleAxis::Italic),
            Some(oblique)
        );
        // Sanity: Normal still resolves to Normal.
        assert_eq!(r.find("Inter", 400, FontStyleAxis::Normal), Some(normal));
    }

    #[test]
    fn re_register_overrides_on_ties() {
        // Two identical (family, weight, style) registrations: the
        // later one wins by the score-tie rule.
        let mut r = FontRegistry::new();
        let _first = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let second = r.register(face("Inter", 400, FontStyleAxis::Normal, 9));
        let h = r.find("Inter", 400, FontStyleAxis::Normal).unwrap();
        assert_eq!(h, second);
        assert_eq!(r.get(h).unwrap().data[0], 9);
    }

    #[test]
    fn find_first_walks_family_list() {
        let mut r = FontRegistry::new();
        let inter = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
        let _other = r.register(face("Helvetica", 400, FontStyleAxis::Normal, 2));
        // Roboto missing → falls through to Inter.
        assert_eq!(
            r.find_first(&["Roboto", "Inter", "Helvetica"], 400, FontStyleAxis::Normal),
            Some(inter)
        );
        // No families match → None.
        assert!(
            r.find_first(&["Garamond"], 400, FontStyleAxis::Normal)
                .is_none()
        );
    }

    #[test]
    fn empty_registry_returns_none() {
        let r = FontRegistry::new();
        assert!(r.is_empty());
        assert!(r.find("Inter", 400, FontStyleAxis::Normal).is_none());
        assert!(
            r.find_first(&["Inter"], 400, FontStyleAxis::Normal)
                .is_none()
        );
    }
}
