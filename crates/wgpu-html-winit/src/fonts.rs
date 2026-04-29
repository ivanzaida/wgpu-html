//! System-font discovery for hosts that don't ship their own fonts.
//!
//! Scans a small built-in table of well-known font paths
//! (Windows: Segoe UI / Arial / Calibri / Tahoma; Linux:
//! DejaVu in the usual locations; macOS: Helvetica / Arial) and
//! returns whatever the first readable row contains as a
//! `Vec<SystemFontVariant>` (regular, bold, italic,
//! bold-italic — any subset that's actually on disk).
//!
//! Hosts call [`register_system_fonts`] to push them into a
//! [`Tree`] under a given family name, matching the family the
//! HTML's CSS expects.
//!
//! The bytes are cached behind a `OnceLock` and shared by
//! `Arc<[u8]>`, so repeat calls don't re-read from disk and
//! `wgpu-html-text` recognises the faces as already loaded on
//! second-and-later sync.

use std::sync::{Arc, OnceLock};

use wgpu_html_tree::{FontFace, FontHandle, FontStyleAxis, Tree};

/// One font family's worth of system-font paths: regular, bold,
/// italic, bold-italic. Empty path = "this variant isn't on disk
/// for this family"; the loader skips it. The table is scanned
/// top-to-bottom — the first row whose `regular` is readable
/// wins. Other variants of that row are loaded if present.
const FONT_FAMILIES: &[[&str; 4]] = &[
    // Windows — Segoe UI is the system UI font on modern Windows.
    [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\segoeuib.ttf",
        "C:\\Windows\\Fonts\\segoeuii.ttf",
        "C:\\Windows\\Fonts\\segoeuiz.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\arialbd.ttf",
        "C:\\Windows\\Fonts\\ariali.ttf",
        "C:\\Windows\\Fonts\\arialbi.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\calibri.ttf",
        "C:\\Windows\\Fonts\\calibrib.ttf",
        "C:\\Windows\\Fonts\\calibrii.ttf",
        "C:\\Windows\\Fonts\\calibriz.ttf",
    ],
    [
        "C:\\Windows\\Fonts\\tahoma.ttf",
        "C:\\Windows\\Fonts\\tahomabd.ttf",
        "",
        "",
    ],
    // Linux — DejaVu lives in a few different paths across distros.
    [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-BoldOblique.ttf",
    ],
    [
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-BoldOblique.ttf",
    ],
    [
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Oblique.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-BoldOblique.ttf",
    ],
    // macOS — `.ttc` collections cover all variants in one file,
    // so the bold / italic slots are empty (cosmic-text picks
    // within the collection by `(weight, style)`).
    ["/System/Library/Fonts/Helvetica.ttc", "", "", ""],
    ["/Library/Fonts/Arial.ttf", "", "", ""],
];

/// One concrete font face the system-font loader resolved.
#[derive(Clone)]
pub struct SystemFontVariant {
    pub weight: u16,
    pub style: FontStyleAxis,
    pub data: Arc<[u8]>,
}

/// Scan [`FONT_FAMILIES`] and return every variant from the first
/// row whose regular file is readable. The result is cached for
/// the process; repeat calls hand back the same `Arc`s.
pub fn system_font_variants() -> &'static [SystemFontVariant] {
    static FACES: OnceLock<Vec<SystemFontVariant>> = OnceLock::new();
    FACES
        .get_or_init(|| {
            for row in FONT_FAMILIES {
                let regular_path = row[0];
                let Ok(reg_bytes) = std::fs::read(regular_path) else {
                    continue;
                };
                let mut out = vec![SystemFontVariant {
                    weight: 400,
                    style: FontStyleAxis::Normal,
                    data: Arc::from(reg_bytes.into_boxed_slice()),
                }];
                let variants: [(&str, u16, FontStyleAxis); 3] = [
                    (row[1], 700, FontStyleAxis::Normal),
                    (row[2], 400, FontStyleAxis::Italic),
                    (row[3], 700, FontStyleAxis::Italic),
                ];
                for (path, weight, style) in variants {
                    if path.is_empty() {
                        continue;
                    }
                    if let Ok(bytes) = std::fs::read(path) {
                        out.push(SystemFontVariant {
                            weight,
                            style,
                            data: Arc::from(bytes.into_boxed_slice()),
                        });
                    }
                }
                return out;
            }
            Vec::new()
        })
        .as_slice()
}

/// Convenience: register every variant from [`system_font_variants`]
/// on `tree` under the given `family` name. Returns the number of
/// variants registered (may be `0` on machines whose font paths
/// don't match the built-in table).
///
/// `family` is the name your CSS will reference, e.g.
/// `font-family: "DemoSans", sans-serif;` would call this with
/// `"DemoSans"`.
pub fn register_system_fonts(tree: &mut Tree, family: &str) -> usize {
    let variants = system_font_variants();
    for face in variants {
        tree.register_font(FontFace {
            family: family.to_owned(),
            weight: face.weight,
            style: face.style,
            data: face.data.clone(),
        });
    }
    variants.len()
}

// ── Icon fonts ──────────────────────────────────────────────────────────────

/// The Lucide icon font, embedded at compile time (ISC license).
///
/// Glyphs are mapped to Unicode Private Use Area codepoints
/// (U+E000 and up). Use HTML numeric character references like
/// `&#xe151;` (search) or Rust `'\u{e151}'` to reference them.
/// See <https://lucide.dev/icons> for the full icon catalogue and
/// codepoint assignments.
pub static LUCIDE_FONT_DATA: &[u8] = include_bytes!("../fonts/lucide.ttf");

/// Register the embedded Lucide icon font on `tree` under the
/// given `family` name. The font contains a single 400-weight
/// normal-style face. Returns the [`FontHandle`].
///
/// Usage in HTML after registering with `family = "lucide"`:
///
/// ```html
/// <span style="font-family: lucide; font-size: 24px;">&#xe151;</span>
/// ```
///
/// The `color` CSS property tints icons just like regular text.
pub fn register_lucide_icons(tree: &mut Tree, family: &str) -> FontHandle {
    tree.register_font(FontFace::regular(family, Arc::from(LUCIDE_FONT_DATA)))
}
