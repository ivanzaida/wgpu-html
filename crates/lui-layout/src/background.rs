//! CSS background-image resolution helpers.

use crate::{gradient, BackgroundImagePaint, ImageCache, Rect};
use lui_models::{
    common::css_enums::{BackgroundRepeat, CssImage},
    Style,
};
use std::sync::Arc;

/// Parse a single token from `background-size` / `background-position`
/// into a length in physical pixels. Supports `<n>px`, bare `<n>`
/// (interpreted as pixels), `<n>%` (resolved against `container`), and
/// the keyword `auto` (returned as `None`). Returns `None` for any
/// unrecognised input — callers fall back to a sensible default.
fn parse_bg_axis(token: &str, container: f32) -> Option<f32> {
    let t = token.trim().to_ascii_lowercase();
    if t == "auto" || t.is_empty() {
        return None;
    }
    if let Some(stripped) = t.strip_suffix('%') {
        return stripped.trim().parse::<f32>().ok().map(|p| container * p / 100.0);
    }
    let numeric = t.strip_suffix("px").unwrap_or(&t);
    numeric.trim().parse::<f32>().ok()
}

/// Resolve `background-size` to a per-tile (width, height) pair in
/// physical pixels. Supports `auto`, `cover`, `contain`, single
/// `<length-percentage>` (applied to width, height auto), and a
/// `<lp> <lp>` pair. Aspect ratio is preserved when one axis is
/// `auto` (the standard CSS behaviour).
fn resolve_bg_size(value: Option<&str>, img_w: u32, img_h: u32, bg_w: f32, bg_h: f32) -> (f32, f32) {
    let intrinsic_w = img_w as f32;
    let intrinsic_h = img_h as f32;
    if intrinsic_w <= 0.0 || intrinsic_h <= 0.0 || bg_w <= 0.0 || bg_h <= 0.0 {
        return (intrinsic_w.max(0.0), intrinsic_h.max(0.0));
    }
    let raw = value.map(str::trim).unwrap_or("auto");
    let lower = raw.to_ascii_lowercase();
    if lower == "auto" || lower.is_empty() {
        return (intrinsic_w, intrinsic_h);
    }
    if lower == "cover" {
        let scale = (bg_w / intrinsic_w).max(bg_h / intrinsic_h);
        return (intrinsic_w * scale, intrinsic_h * scale);
    }
    if lower == "contain" {
        let scale = (bg_w / intrinsic_w).min(bg_h / intrinsic_h);
        return (intrinsic_w * scale, intrinsic_h * scale);
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    let aspect = intrinsic_h / intrinsic_w;
    match parts.as_slice() {
        [w_s] => {
            let w = parse_bg_axis(w_s, bg_w).unwrap_or(intrinsic_w);
            (w, w * aspect)
        }
        [w_s, h_s] => {
            let w_opt = parse_bg_axis(w_s, bg_w);
            let h_opt = parse_bg_axis(h_s, bg_h);
            match (w_opt, h_opt) {
                (Some(w), Some(h)) => (w, h),
                (Some(w), None) => (w, w * aspect),
                (None, Some(h)) => (h / aspect, h),
                (None, None) => (intrinsic_w, intrinsic_h),
            }
        }
        _ => (intrinsic_w, intrinsic_h),
    }
}

/// Resolve a single token of `background-position` for one axis to a
/// pixel offset within the background area. Accepts the per-axis
/// keywords (`left`/`right` map to 0% / 100% on the x axis,
/// `top`/`bottom` to 0% / 100% on the y axis, `center` to 50%) as
/// well as `<length>` and `<percentage>`. The CSS rule is "anchor
/// point of the image equals anchor point of the box" expressed as
/// `(box - tile) * percent + length_offset`.
fn resolve_bg_position_axis(token: &str, box_size: f32, tile_size: f32, is_x: bool) -> f32 {
    let t = token.trim().to_ascii_lowercase();
    let percent: Option<f32> = match t.as_str() {
        "left" if is_x => Some(0.0),
        "right" if is_x => Some(100.0),
        "top" if !is_x => Some(0.0),
        "bottom" if !is_x => Some(100.0),
        "center" => Some(50.0),
        _ => None,
    };
    if let Some(p) = percent {
        return (box_size - tile_size) * p / 100.0;
    }
    if let Some(stripped) = t.strip_suffix('%') {
        if let Ok(p) = stripped.trim().parse::<f32>() {
            return (box_size - tile_size) * p / 100.0;
        }
    }
    let numeric = t.strip_suffix("px").unwrap_or(&t);
    numeric.trim().parse::<f32>().unwrap_or(0.0)
}

/// Resolve `background-position` to `(off_x, off_y)` in physical
/// pixels relative to the background area's top-left corner. Default
/// is `0% 0%` (top-left).
fn resolve_bg_position(value: Option<&str>, bg_w: f32, bg_h: f32, tile_w: f32, tile_h: f32) -> (f32, f32) {
    let raw = value.map(str::trim).unwrap_or("");
    if raw.is_empty() {
        return (0.0, 0.0);
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    match parts.as_slice() {
        [single] => {
            // CSS: a single value is the x coordinate; y is `center`.
            let x = resolve_bg_position_axis(single, bg_w, tile_w, true);
            let y = resolve_bg_position_axis("center", bg_h, tile_h, false);
            (x, y)
        }
        [a, b] => {
            // Disambiguate axis-only keywords: if either token is a
            // y-axis-only keyword (top/bottom) it must be the y value
            // even when listed first. CSS lets you write
            // `top right` and `right top` interchangeably for the
            // two-keyword form.
            let is_y = |t: &str| matches!(t, "top" | "bottom");
            let is_x = |t: &str| matches!(t, "left" | "right");
            if is_y(&a.to_ascii_lowercase()) || is_x(&b.to_ascii_lowercase()) {
                let y = resolve_bg_position_axis(a, bg_h, tile_h, false);
                let x = resolve_bg_position_axis(b, bg_w, tile_w, true);
                (x, y)
            } else {
                let x = resolve_bg_position_axis(a, bg_w, tile_w, true);
                let y = resolve_bg_position_axis(b, bg_h, tile_h, false);
                (x, y)
            }
        }
        _ => (0.0, 0.0),
    }
}

/// Tile a single image across (a portion of) `bg` according to the
/// repeat mode, given the per-tile size and the initial tile offset
/// (relative to `bg`'s top-left). Returns the absolute on-screen
/// rectangle for every tile that intersects `bg`. For axes that don't
/// repeat, only the seed tile is emitted; for `repeat` / `repeat-x` /
/// `repeat-y` we walk both directions from the seed by `tile_w`/
/// `tile_h` until we leave `bg`.
fn compute_bg_tiles(
    bg: Rect,
    tile_w: f32,
    tile_h: f32,
    off_x: f32,
    off_y: f32,
    repeat: BackgroundRepeat,
) -> Vec<Rect> {
    use BackgroundRepeat as BR;
    let mut tiles = Vec::new();
    if tile_w <= 0.0 || tile_h <= 0.0 || bg.w <= 0.0 || bg.h <= 0.0 {
        return tiles;
    }
    let seed_x = bg.x + off_x;
    let seed_y = bg.y + off_y;
    let repeat_x = matches!(repeat, BR::Repeat | BR::RepeatX);
    let repeat_y = matches!(repeat, BR::Repeat | BR::RepeatY);

    let xs: Vec<f32> = if repeat_x {
        let mut start = seed_x;
        while start > bg.x {
            start -= tile_w;
        }
        let mut xs = Vec::new();
        let mut x = start;
        while x < bg.x + bg.w {
            xs.push(x);
            x += tile_w;
        }
        xs
    } else {
        // Skip the single tile entirely if it's outside the bg area.
        if seed_x + tile_w <= bg.x || seed_x >= bg.x + bg.w {
            Vec::new()
        } else {
            vec![seed_x]
        }
    };
    let ys: Vec<f32> = if repeat_y {
        let mut start = seed_y;
        while start > bg.y {
            start -= tile_h;
        }
        let mut ys = Vec::new();
        let mut y = start;
        while y < bg.y + bg.h {
            ys.push(y);
            y += tile_h;
        }
        ys
    } else {
        if seed_y + tile_h <= bg.y || seed_y >= bg.y + bg.h {
            Vec::new()
        } else {
            vec![seed_y]
        }
    };

    for &y in &ys {
        for &x in &xs {
            tiles.push(Rect::new(x, y, tile_w, tile_h));
        }
    }
    tiles
}

/// Top-level: turn `style.background_image` (+ associated longhands)
/// into a [`BackgroundImagePaint`] positioned within `bg`. Returns
/// `None` when there's no supported image reference, the image hasn't
/// finished loading yet, or the resolved tile size collapses to zero.
pub(crate) fn resolve_background_image(
    style: &Style,
    bg: Rect,
    images: &mut ImageCache,
) -> Option<BackgroundImagePaint> {
    use BackgroundRepeat as BR;

    let (image_id, data, img_w, img_h) = match style.background_image.as_ref()? {
        CssImage::Url(url) => {
            let img = images.load_image_url(url, None, None)?;
            (img.image_id, img.data, img.width, img.height)
        }
        CssImage::Function(func) => {
            let grad = gradient::parse_gradient(func)?;
            // Gradients have no intrinsic dimensions — use background box size
            let (tile_w, tile_h) =
                resolve_bg_size(style.background_size.as_deref(), bg.w as u32, bg.h as u32, bg.w, bg.h);
            if tile_w <= 0.0 || tile_h <= 0.0 {
                return None;
            }
            let w = (tile_w.round() as u32).max(1).min(4096);
            let h = (tile_h.round() as u32).max(1).min(4096);
            let pixels = gradient::rasterize(&grad, w, h);
            let id = gradient::gradient_image_id(func, w, h);

            let (off_x, off_y) =
                resolve_bg_position(style.background_position.as_deref(), bg.w, bg.h, tile_w, tile_h);
            let repeat = style.background_repeat.clone().unwrap_or(BR::Repeat);
            let tiles = compute_bg_tiles(bg, tile_w, tile_h, off_x, off_y, repeat);
            if tiles.is_empty() {
                return None;
            }
            return Some(BackgroundImagePaint {
                image_id: id,
                data: Arc::new(pixels),
                width: w,
                height: h,
                tiles,
            });
        }
    };

    let (tile_w, tile_h) = resolve_bg_size(style.background_size.as_deref(), img_w, img_h, bg.w, bg.h);
    if tile_w <= 0.0 || tile_h <= 0.0 {
        return None;
    }
    let (off_x, off_y) = resolve_bg_position(style.background_position.as_deref(), bg.w, bg.h, tile_w, tile_h);
    let repeat = style.background_repeat.clone().unwrap_or(BR::Repeat);
    let tiles = compute_bg_tiles(bg, tile_w, tile_h, off_x, off_y, repeat);
    if tiles.is_empty() {
        return None;
    }
    Some(BackgroundImagePaint {
        image_id,
        data,
        width: img_w,
        height: img_h,
        tiles,
    })
}
