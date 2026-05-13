//! Text shaping: converts a string + font properties into positioned glyphs.

use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Weight, Style};

use crate::font::FontContext;
use crate::font_face::{FontHandle, FontStyleAxis};

/// Properties that affect text shaping.
#[derive(Debug, Clone)]
pub struct TextStyle<'a> {
    pub font_size: f32,
    pub line_height: f32,
    pub font_family: &'a str,
    pub weight: u16,
    pub letter_spacing: f32,
    pub word_spacing: f32,
}

impl Default for TextStyle<'_> {
    fn default() -> Self {
        Self { font_size: 16.0, line_height: 19.2, font_family: "sans-serif", weight: 400,
            letter_spacing: 0.0, word_spacing: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u16,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Normalized atlas UVs [u_min, v_min].
    pub uv_min: [f32; 2],
    /// Normalized atlas UVs [u_max, v_max].
    pub uv_max: [f32; 2],
    /// Linear RGBA foreground color.
    pub color: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct ShapedRun {
    pub glyphs: Vec<PositionedGlyph>,
    /// For each glyph, its 0-based character index in `text`.
    /// Empty on synthetic (test-built) runs — callers fall back to
    /// identity mapping when this is empty.
    pub glyph_chars: Vec<usize>,
    /// The source text that produced this run.
    pub text: String,
    /// UTF-8 byte offsets for every character boundary in `text`.
    /// Length is `text.chars().count() + 1`.
    pub byte_boundaries: Vec<usize>,
    pub width: f32,
    pub height: f32,
    /// Distance from the top of the first line to the baseline.
    pub ascent: f32,
    pub line_height: f32,
    pub font_size: f32,
    pub line_count: usize,
}

impl ShapedRun {
    pub fn char_count(&self) -> usize {
        self.byte_boundaries.len().saturating_sub(1)
    }

    pub fn glyph_to_char_index(&self, glyph_idx: usize) -> usize {
        if self.glyph_chars.is_empty() {
            return glyph_idx;
        }
        self.glyph_chars.get(glyph_idx).copied().unwrap_or_else(|| {
            self.glyph_chars.last().copied().map(|c| c + 1).unwrap_or(glyph_idx)
        })
    }

    pub fn char_to_glyph_index(&self, char_idx: usize) -> usize {
        if self.glyph_chars.is_empty() {
            return char_idx.min(self.glyphs.len());
        }
        self.glyph_chars
            .iter()
            .position(|&c| c >= char_idx)
            .unwrap_or(self.glyphs.len())
    }

    pub fn byte_offset_for_boundary(&self, glyph_index: usize) -> usize {
        if self.byte_boundaries.is_empty() {
            return 0;
        }
        let idx = glyph_index.min(self.byte_boundaries.len().saturating_sub(1));
        self.byte_boundaries[idx]
    }
}

pub fn utf8_boundaries(text: &str) -> Vec<usize> {
    let mut out = Vec::with_capacity(text.chars().count() + 1);
    out.push(0);
    for (idx, ch) in text.char_indices() {
        out.push(idx + ch.len_utf8());
    }
    out
}

/// Metrics-only result — cheaper than `ShapedRun` when glyph data isn't needed.
#[derive(Debug, Clone, Copy)]
pub struct RunMetrics {
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
}

#[derive(Debug, Clone)]
pub struct ShapedLine {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
}

// ── System-font shaping (backward-compatible) ──────────────────────────

impl FontContext {
    /// Shape a single text span using system fonts (no custom registry).
    pub fn shape(&mut self, text: &str, style: &TextStyle) -> ShapedRun {
        let attrs = build_attrs(style);
        self.shape_with_attrs(text, &attrs, style.font_size, style.line_height)
    }

    /// Shape text and break into lines fitting `max_width`.
    pub fn break_into_lines(&mut self, text: &str, style: &TextStyle, max_width: f32) -> Vec<ShapedLine> {
        let attrs = build_attrs(style);
        self.shape_lines(text, &attrs, style.font_size, style.line_height, max_width)
    }

    // ── Custom-font shaping (via FontHandle) ───────────────────────────

    /// Shape using a registered custom font handle.
    pub fn shape_with_handle(
        &mut self,
        text: &str,
        handle: FontHandle,
        font_size: f32,
        line_height: f32,
        weight: u16,
        style_axis: FontStyleAxis,
    ) -> Option<ShapedRun> {
        let fontdb_id = self.fontdb_id(handle)?;
        let family = self.system.db_mut().face(fontdb_id)?
            .families.first().map(|(n, _)| n.clone())?;

        let attrs = Attrs::new()
            .family(Family::Name(&family))
            .weight(Weight(weight))
            .style(convert_style(style_axis));

        Some(self.shape_with_attrs(text, &attrs, font_size, line_height))
    }

    /// Measure a custom-font run — metrics only.
    pub fn measure_with_handle(
        &mut self,
        text: &str,
        handle: FontHandle,
        font_size: f32,
        line_height: f32,
        weight: u16,
        style_axis: FontStyleAxis,
    ) -> Option<RunMetrics> {
        let fontdb_id = self.fontdb_id(handle)?;
        let family = self.system.db_mut().face(fontdb_id)?
            .families.first().map(|(n, _)| n.clone())?;

        let attrs = Attrs::new()
            .family(Family::Name(&family))
            .weight(Weight(weight))
            .style(convert_style(style_axis));

        Some(self.measure_with_attrs(text, &attrs, font_size, line_height))
    }

    /// Shape using a CSS family list resolved through the custom registry.
    /// Falls back to system fonts if no registered family matches.
    pub fn shape_with_families(
        &mut self,
        text: &str,
        families: &[&str],
        font_size: f32,
        line_height: f32,
        weight: u16,
        style_axis: FontStyleAxis,
    ) -> ShapedRun {
        let resolved = self.resolve_family(families, weight, style_axis);
        if let Some(family) = resolved {
            let attrs = Attrs::new()
                .family(Family::Name(&family))
                .weight(Weight(weight))
                .style(convert_style(style_axis));
            self.shape_with_attrs(text, &attrs, font_size, line_height)
        } else {
            let ts = TextStyle {
                font_family: families.first().copied().unwrap_or("sans-serif"),
                font_size, line_height, weight, ..Default::default()
            };
            let attrs = build_attrs(&ts);
            self.shape_with_attrs(text, &attrs, font_size, line_height)
        }
    }

    /// Measure a families-resolved run — metrics only.
    pub fn measure_with_families(
        &mut self,
        text: &str,
        families: &[&str],
        font_size: f32,
        line_height: f32,
        weight: u16,
        style_axis: FontStyleAxis,
    ) -> RunMetrics {
        let resolved = self.resolve_family(families, weight, style_axis);
        if let Some(family) = resolved {
            let attrs = Attrs::new()
                .family(Family::Name(&family))
                .weight(Weight(weight))
                .style(convert_style(style_axis));
            self.measure_with_attrs(text, &attrs, font_size, line_height)
        } else {
            let ts = TextStyle {
                font_family: families.first().copied().unwrap_or("sans-serif"),
                font_size, line_height, weight, ..Default::default()
            };
            let attrs = build_attrs(&ts);
            self.measure_with_attrs(text, &attrs, font_size, line_height)
        }
    }

    // ── Internal helpers ───────────────────────────────────────────────

    fn shape_with_attrs(&mut self, text: &str, attrs: &Attrs, font_size: f32, line_height: f32) -> ShapedRun {
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.system, metrics);
        buffer.set_size(None, None);
        buffer.set_text(text, attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.system, false);

        let bb = utf8_boundaries(text);
        let mut glyphs = Vec::new();
        let mut glyph_chars = Vec::new();
        let mut line_count = 0;
        let mut ascent = 0.0;
        for run in buffer.layout_runs() {
            if line_count == 0 {
                ascent = run.line_y;
            }
            line_count += 1;
            for g in run.glyphs {
                glyphs.push(PositionedGlyph {
                    glyph_id: g.glyph_id, x: g.x, y: g.y,
                    w: g.w, h: line_height,
                    uv_min: [0.0; 2], uv_max: [0.0; 2], color: [0.0; 4],
                });
                let char_idx = bb.partition_point(|&b| b < g.start)
                    .min(bb.len().saturating_sub(1));
                glyph_chars.push(char_idx);
            }
        }
        let width = glyphs.iter().map(|g| g.x + g.w).fold(0.0f32, f32::max);
        ShapedRun {
            glyphs, glyph_chars,
            text: text.to_owned(), byte_boundaries: bb,
            width, ascent,
            height: line_count as f32 * line_height,
            line_height, font_size, line_count,
        }
    }

    /// Measure only — shape text and return metrics without collecting glyphs.
    /// Used by layout for intrinsic sizing when glyph positions aren't needed.
    pub fn measure_only(&mut self, text: &str, style: &TextStyle) -> RunMetrics {
        let attrs = build_attrs(style);
        self.measure_with_attrs(text, &attrs, style.font_size, style.line_height)
    }

    fn measure_with_attrs(&mut self, text: &str, attrs: &Attrs, font_size: f32, line_height: f32) -> RunMetrics {
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.system, metrics);
        buffer.set_size(None, None);
        buffer.set_text(text, attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(&mut self.system, false);

        let mut line_count = 0;
        let mut ascent = 0.0;
        let mut width: f32 = 0.0;
        for run in buffer.layout_runs() {
            if line_count == 0 {
                ascent = run.line_y;
            }
            line_count += 1;
            let run_width = run.glyphs.iter().map(|g| g.x + g.w).fold(0.0f32, f32::max);
            width = width.max(run_width);
        }
        RunMetrics { width, ascent, height: line_count as f32 * line_height }
    }

    fn shape_lines(&mut self, text: &str, attrs: &Attrs, font_size: f32, line_height: f32, max_width: f32) -> Vec<ShapedLine> {
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.system, metrics);
        buffer.set_size(Some(font_size), Some(line_height));
        buffer.set_text(text, attrs, Shaping::Advanced, None);
        buffer.set_size(Some(max_width), Some(line_height));
        buffer.shape_until_scroll(&mut self.system, false);

        buffer.layout_runs().map(|run| {
            let mut glyphs = Vec::new();
            for g in run.glyphs {
                glyphs.push(PositionedGlyph {
                    glyph_id: g.glyph_id, x: g.x, y: g.y,
                    w: g.w, h: line_height,
                    uv_min: [0.0; 2], uv_max: [0.0; 2], color: [0.0; 4],
                });
            }
            let line_width = glyphs.iter().map(|g| g.x + g.w).fold(0.0f32, f32::max);
            ShapedLine { glyphs, width: line_width, height: line_height }
        }).collect()
    }
}

// ── Style → cosmic-text attrs ──────────────────────────────────────────

pub(crate) fn build_attrs<'a>(style: &'a TextStyle<'a>) -> Attrs<'a> {
    let family = if style.font_family.is_empty() || style.font_family == "sans-serif" {
        Family::SansSerif
    } else if style.font_family == "serif" {
        Family::Serif
    } else if style.font_family == "monospace" {
        Family::Monospace
    } else {
        Family::Name(style.font_family)
    };
    Attrs::new().family(family).weight(Weight(style.weight))
}

pub(crate) fn convert_style(axis: FontStyleAxis) -> Style {
    match axis {
        FontStyleAxis::Normal => Style::Normal,
        FontStyleAxis::Italic => Style::Italic,
        FontStyleAxis::Oblique => Style::Oblique,
    }
}

// ── Font metric parsing ───────────────────────────────────────────────

/// Parse `line-height: normal` multiplier from raw font bytes.
///
/// Uses the browser algorithm:
/// 1. If OS/2 has `USE_TYPO_METRICS`, use typo ascender/descender/lineGap.
/// 2. Otherwise use OS/2 winAscent + winDescent.
/// 3. Fall back to hhea ascender/descender/lineGap.
pub fn parse_line_height_multiplier(data: &[u8]) -> Option<f32> {
    let offset_table_start = if data.len() >= 12 && &data[0..4] == b"ttcf" {
        let n = u32::from_be_bytes(data[8..12].try_into().ok()?) as usize;
        if n == 0 || data.len() < 16 { return None; }
        u32::from_be_bytes(data[12..16].try_into().ok()?) as usize
    } else {
        0
    };

    let d = &data[offset_table_start..];
    if d.len() < 12 { return None; }
    let num_tables = u16::from_be_bytes(d[4..6].try_into().ok()?) as usize;

    let mut head_off = None;
    let mut hhea_off = None;
    let mut os2_off = None;
    for i in 0..num_tables {
        let rec = 12 + i * 16;
        if rec + 16 > d.len() { break; }
        let tag = &d[rec..rec + 4];
        let off = u32::from_be_bytes(d[rec + 8..rec + 12].try_into().ok()?) as usize;
        match tag {
            b"head" => head_off = Some(off),
            b"hhea" => hhea_off = Some(off),
            b"OS/2" => os2_off = Some(off),
            _ => {}
        }
    }

    let head = head_off?;
    if head + 20 > data.len() { return None; }
    let upem = u16::from_be_bytes(data[head + 18..head + 20].try_into().ok()?) as f32;
    if upem == 0.0 { return None; }

    if let Some(os2) = os2_off {
        if os2 + 78 <= data.len() {
            let fs_selection = u16::from_be_bytes(data[os2 + 62..os2 + 64].try_into().ok()?);
            let use_typo_metrics = fs_selection & (1 << 7) != 0;

            if use_typo_metrics {
                let typo_asc = i16::from_be_bytes(data[os2 + 68..os2 + 70].try_into().ok()?) as f32;
                let typo_desc = i16::from_be_bytes(data[os2 + 70..os2 + 72].try_into().ok()?) as f32;
                let typo_gap = i16::from_be_bytes(data[os2 + 72..os2 + 74].try_into().ok()?) as f32;
                return Some((typo_asc - typo_desc + typo_gap) / upem);
            } else {
                let win_asc = u16::from_be_bytes(data[os2 + 74..os2 + 76].try_into().ok()?) as f32;
                let win_desc = u16::from_be_bytes(data[os2 + 76..os2 + 78].try_into().ok()?) as f32;
                return Some((win_asc + win_desc) / upem);
            }
        }
    }

    let hhea = hhea_off?;
    if hhea + 10 > data.len() { return None; }
    let ascender = i16::from_be_bytes(data[hhea + 4..hhea + 6].try_into().ok()?) as f32;
    let descender = i16::from_be_bytes(data[hhea + 6..hhea + 8].try_into().ok()?) as f32;
    let line_gap = i16::from_be_bytes(data[hhea + 8..hhea + 10].try_into().ok()?) as f32;

    Some((ascender - descender + line_gap) / upem)
}
