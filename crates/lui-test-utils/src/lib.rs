//! Visual regression testing utilities.
//!
//! Compare rendered RGBA pixels against reference PNGs.
//!
//! ```rust,no_run
//! use lui_test_utils::{load_png, compare_rgba, DiffResult};
//!
//! let expected = load_png("tests/expected/card.png").unwrap();
//! let actual = load_png("tests/actual/card.png").unwrap();
//! let result = compare_rgba(&expected, &actual, 5);
//! assert!(result.match_ratio() > 0.99, "visual regression: {:.1}% match", result.match_ratio() * 100.0);
//! ```

use std::path::Path;

/// RGBA image buffer.
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

/// Result of comparing two images.
pub struct DiffResult {
    pub total_pixels: u32,
    pub matching_pixels: u32,
    pub max_channel_diff: u8,
    pub diff_image: Option<RgbaImage>,
}

impl DiffResult {
    pub fn match_ratio(&self) -> f64 {
        if self.total_pixels == 0 { return 1.0; }
        self.matching_pixels as f64 / self.total_pixels as f64
    }

    pub fn match_percent(&self) -> f64 {
        self.match_ratio() * 100.0
    }
}

/// Load a PNG file into an RGBA buffer.
pub fn load_png(path: impl AsRef<Path>) -> Result<RgbaImage, String> {
    let file = std::fs::File::open(path.as_ref())
        .map_err(|e| format!("failed to open {}: {e}", path.as_ref().display()))?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info()
        .map_err(|e| format!("failed to decode PNG: {e}"))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)
        .map_err(|e| format!("failed to read PNG frame: {e}"))?;
    buf.truncate(info.buffer_size());

    let data = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => {
            let mut rgba = Vec::with_capacity(buf.len() / 3 * 4);
            for chunk in buf.chunks(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255);
            }
            rgba
        }
        other => return Err(format!("unsupported color type: {other:?}")),
    };

    Ok(RgbaImage { width: info.width, height: info.height, data })
}

/// Save an RGBA buffer as a PNG file.
pub fn save_png(image: &RgbaImage, path: impl AsRef<Path>) -> Result<(), String> {
    let file = std::fs::File::create(path.as_ref())
        .map_err(|e| format!("failed to create {}: {e}", path.as_ref().display()))?;
    let mut encoder = png::Encoder::new(file, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()
        .map_err(|e| format!("failed to write PNG header: {e}"))?;
    writer.write_image_data(&image.data)
        .map_err(|e| format!("failed to write PNG data: {e}"))?;
    Ok(())
}

/// Compare two RGBA images pixel-by-pixel.
///
/// `threshold` is the max allowed per-channel difference (0-255) for a pixel
/// to count as "matching". Use 0 for exact match, 2-5 for anti-aliasing tolerance.
///
/// If `generate_diff` is true, produces a diff image: green = match, red = mismatch.
pub fn compare_rgba(expected: &RgbaImage, actual: &RgbaImage, threshold: u8) -> DiffResult {
    compare_rgba_inner(expected, actual, threshold, false)
}

/// Same as `compare_rgba` but also generates a diff image.
pub fn compare_rgba_with_diff(expected: &RgbaImage, actual: &RgbaImage, threshold: u8) -> DiffResult {
    compare_rgba_inner(expected, actual, threshold, true)
}

fn compare_rgba_inner(expected: &RgbaImage, actual: &RgbaImage, threshold: u8, generate_diff: bool) -> DiffResult {
    let w = expected.width.min(actual.width);
    let h = expected.height.min(actual.height);
    let total = w * h;

    let diff_w = expected.width.max(actual.width);
    let diff_h = expected.height.max(actual.height);
    let mut diff_data = if generate_diff {
        Some(vec![0u8; (diff_w * diff_h * 4) as usize])
    } else {
        None
    };

    let mut matching = 0u32;
    let mut max_diff = 0u8;

    for y in 0..h {
        for x in 0..w {
            let ei = ((y * expected.width + x) * 4) as usize;
            let ai = ((y * actual.width + x) * 4) as usize;

            let dr = (expected.data[ei] as i16 - actual.data[ai] as i16).unsigned_abs() as u8;
            let dg = (expected.data[ei + 1] as i16 - actual.data[ai + 1] as i16).unsigned_abs() as u8;
            let db = (expected.data[ei + 2] as i16 - actual.data[ai + 2] as i16).unsigned_abs() as u8;
            let da = (expected.data[ei + 3] as i16 - actual.data[ai + 3] as i16).unsigned_abs() as u8;

            let ch_max = dr.max(dg).max(db).max(da);
            max_diff = max_diff.max(ch_max);
            let is_match = ch_max <= threshold;

            if is_match { matching += 1; }

            if let Some(ref mut diff) = diff_data {
                let di = ((y * diff_w + x) * 4) as usize;
                if is_match {
                    diff[di] = actual.data[ai] / 2;
                    diff[di + 1] = (actual.data[ai + 1] / 2).saturating_add(128);
                    diff[di + 2] = actual.data[ai + 2] / 2;
                    diff[di + 3] = 255;
                } else {
                    diff[di] = 255;
                    diff[di + 1] = 0;
                    diff[di + 2] = 0;
                    diff[di + 3] = 255;
                }
            }
        }
    }

    // Size mismatch pixels count as mismatched
    let size_mismatch = (diff_w * diff_h) - total;
    let total_with_mismatch = total + size_mismatch;

    if let Some(ref mut diff) = diff_data {
        for y in h..diff_h {
            for x in 0..diff_w {
                let di = ((y * diff_w + x) * 4) as usize;
                diff[di] = 255; diff[di + 1] = 100; diff[di + 2] = 0; diff[di + 3] = 255;
            }
        }
        for y in 0..h {
            for x in w..diff_w {
                let di = ((y * diff_w + x) * 4) as usize;
                diff[di] = 255; diff[di + 1] = 100; diff[di + 2] = 0; diff[di + 3] = 255;
            }
        }
    }

    DiffResult {
        total_pixels: total_with_mismatch,
        matching_pixels: matching,
        max_channel_diff: max_diff,
        diff_image: diff_data.map(|data| RgbaImage { width: diff_w, height: diff_h, data }),
    }
}

/// Assert that two images match within a tolerance.
///
/// Panics with a detailed message if the match ratio is below `min_ratio`.
/// If `diff_path` is provided, saves the diff image on failure.
pub fn assert_screenshots_match(
    expected_path: impl AsRef<Path>,
    actual_path: impl AsRef<Path>,
    threshold: u8,
    min_ratio: f64,
    diff_path: Option<&Path>,
) {
    let expected = load_png(&expected_path).expect("failed to load expected screenshot");
    let actual = load_png(&actual_path).expect("failed to load actual screenshot");
    let result = compare_rgba_with_diff(&expected, &actual, threshold);

    if result.match_ratio() < min_ratio {
        if let (Some(diff_path), Some(ref diff)) = (diff_path, &result.diff_image) {
            let _ = save_png(diff, diff_path);
            eprintln!("diff image saved to {}", diff_path.display());
        }
        panic!(
            "screenshot mismatch: {:.1}% match (need {:.1}%), max channel diff={}, expected={}x{}, actual={}x{}\n  expected: {}\n  actual: {}",
            result.match_percent(), min_ratio * 100.0, result.max_channel_diff,
            expected.width, expected.height, actual.width, actual.height,
            expected_path.as_ref().display(), actual_path.as_ref().display(),
        );
    }
}
