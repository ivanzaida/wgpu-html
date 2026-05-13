//! Font registry — document-owned database of registered font faces.
//! Lookup follows simplified CSS-Fonts-3 matching.

use crate::font_face::{FontFace, FontHandle, FontStyleAxis};

/// All registered font faces available to one document.
#[derive(Debug, Clone)]
pub struct FontRegistry {
    faces: Vec<FontFace>,
    generation: u64,
}

impl Default for FontRegistry {
    fn default() -> Self { Self { faces: Vec::new(), generation: 0 } }
}

impl FontRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn len(&self) -> usize { self.faces.len() }
    pub fn is_empty(&self) -> bool { self.faces.is_empty() }
    pub fn generation(&self) -> u64 { self.generation }

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

    /// Find the best face for `family` + weight + style.
    pub fn find(&self, family: &str, weight: u16, style: FontStyleAxis) -> Option<FontHandle> {
        let mut best: Option<(u32, FontHandle)> = None;
        for (h, face) in self.iter() {
            if !family_eq(&face.family, family) { continue; }
            let score = match_score(face, weight, style);
            match best {
                Some((b, _)) if score > b => {}
                _ => best = Some((score, h)),
            }
        }
        best.map(|(_, h)| h)
    }

    /// Walk a CSS family list left-to-right. Falls back to any
    /// registered face when the list contains a generic keyword.
    pub fn find_first(&self, families: &[&str], weight: u16, style: FontStyleAxis) -> Option<FontHandle> {
        for fam in families {
            if let Some(h) = self.find(fam, weight, style) { return Some(h); }
        }
        if !families.iter().any(|f| is_generic_family(f)) { return None; }
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

// ── Matching helpers ────────────────────────────────────────────────────

fn is_generic_family(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(),
        "sans-serif" | "serif" | "monospace" | "cursive" | "fantasy"
        | "system-ui" | "ui-sans-serif" | "ui-serif" | "ui-monospace"
        | "ui-rounded" | "math" | "emoji" | "fangsong"
        | "-apple-system" | "blinkmacsystemfont"
    )
}

fn family_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).all(|(x, y)| x.eq_ignore_ascii_case(&y))
}

fn match_score(face: &FontFace, weight: u16, style: FontStyleAxis) -> u32 {
    style_band(face.style, style) * 1_000_000 + weight_distance(face.weight, weight)
}

fn style_band(candidate: FontStyleAxis, target: FontStyleAxis) -> u32 {
    use FontStyleAxis::*;
    match (target, candidate) {
        (Normal, Normal) | (Italic, Italic) | (Oblique, Oblique) => 0,
        (Italic, Oblique) | (Oblique, Italic) => 1,
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
    if wrong_direction { 10_000 + dist } else { dist }
}
