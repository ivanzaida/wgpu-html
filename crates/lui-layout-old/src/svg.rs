//! SVG-as-replaced-element support.
//!
//! When the layout engine encounters an `<svg>` node it treats the entire
//! SVG subtree as a single atomic replaced element (analogous to `<img>`).
//! The subtree is serialised back to an SVG string, rasterised synchronously
//! with `resvg`, and stored as `ImageData` on the resulting `LayoutBox`.
//!
//! A simple hash-of-(svg-string, w, h) cache keeps repeated identical SVGs
//! from being rasterised on every frame.

use std::{
  collections::HashMap,
  sync::{Mutex, OnceLock},
};

// resvg 0.44 exposes render as a free function; there is no resvg::Tree.
use lui_models::common::css_enums::{CssColor, CssLength};
use lui_models::{Svg, SvgPath, common::html_enums::SvgLength};
use lui_style::CascadedNode;
use lui_tree::Element;
use resvg::{tiny_skia, usvg};

use crate::ImageData;

// ---------------------------------------------------------------------------
// Raster cache
// ---------------------------------------------------------------------------

struct SvgCacheEntry {
  data: std::sync::Arc<Vec<u8>>,
  width: u32,
  height: u32,
  image_id: u64,
}

fn svg_cache() -> &'static Mutex<HashMap<u64, SvgCacheEntry>> {
  static C: OnceLock<Mutex<HashMap<u64, SvgCacheEntry>>> = OnceLock::new();
  C.get_or_init(|| Mutex::new(HashMap::new()))
}

fn hash_svg_key(svg_xml: &str, w: u32, h: u32) -> u64 {
  use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
  };
  let mut s = DefaultHasher::new();
  svg_xml.hash(&mut s);
  w.hash(&mut s);
  h.hash(&mut s);
  s.finish()
}

// ---------------------------------------------------------------------------
// Intrinsic CSS-pixel size helpers
// ---------------------------------------------------------------------------

/// Convert an `SvgLength` to a CSS-pixel value, if it is absolute.
/// Percentage / Em / Rem / Raw values that don't parse return `None`.
pub fn svg_length_to_css_px(l: &SvgLength) -> Option<f32> {
  match l {
    SvgLength::Px(v) => Some(*v),
    SvgLength::Raw(s) => {
      // Raw is used by `parse_svg_length` only as last resort.
      // Try a plain numeric parse (assumes px).
      s.trim().parse::<f32>().ok()
    }
    _ => None,
  }
}

/// Return the SVG element's intrinsic size in CSS pixels from its
/// `width` / `height` attributes, if both are absolute.  Returns
/// `(None, None)` when either is absent or non-absolute.
pub fn svg_intrinsic_css_size(svg: &Svg) -> (Option<f32>, Option<f32>) {
  let w = svg.width.as_ref().and_then(svg_length_to_css_px);
  let h = svg.height.as_ref().and_then(svg_length_to_css_px);
  (w, h)
}

// ---------------------------------------------------------------------------
// SVG XML serialiser
// ---------------------------------------------------------------------------

fn escape_attr(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('"', "&quot;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
}

/// Convert a `CssColor` to an SVG-compatible paint string.
fn css_color_to_svg(c: &CssColor) -> String {
  match c {
    CssColor::Named(n) => n.to_string(),
    CssColor::Hex(h) => h.to_string(),
    CssColor::Rgb(r, g, b) => format!("rgb({},{},{})", r, g, b),
    CssColor::Rgba(r, g, b, a) => format!("rgba({},{},{},{})", r, g, b, a),
    CssColor::Hsl(h, s, l) => format!("hsl({},{:.1}%,{:.1}%)", h, s, l),
    CssColor::Hsla(h, s, l, a) => format!("hsla({},{:.1}%,{:.1}%,{})", h, s, l, a),
    CssColor::Transparent => "none".to_string(),
    CssColor::CurrentColor => "currentColor".to_string(),
    CssColor::Function(f) => f.to_string(),
  }
}

/// Convert a `CssLength` to a numeric SVG value string (px / unitless).
fn css_length_to_svg(l: &CssLength) -> Option<String> {
  match l {
    CssLength::Px(v) => Some(format!("{}", v)),
    CssLength::Em(v) => Some(format!("{}em", v)),
    CssLength::Rem(v) => Some(format!("{}rem", v)),
    CssLength::Vw(v) => Some(format!("{}vw", v)),
    CssLength::Vh(v) => Some(format!("{}vh", v)),
    CssLength::Percent(v) => Some(format!("{}%", v)),
    CssLength::Zero => Some("0".to_string()),
    _ => None,
  }
}

fn write_svg_path(node: &CascadedNode, p: &SvgPath, out: &mut String) {
  let s = &node.style;
  out.push_str("<path");
  if let Some(d) = &p.d {
    out.push_str(" d=\"");
    out.push_str(&escape_attr(d));
    out.push('"');
  }
  // fill: CSS cascade wins over HTML attr, which wins over no value.
  let fill_str: Option<String> = s
    .svg_fill
    .as_ref()
    .map(css_color_to_svg)
    .or_else(|| p.fill.as_ref().map(|s| s.to_string()));
  if let Some(v) = &fill_str {
    out.push_str(" fill=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // fill-rule
  let fill_rule: Option<&str> = s.svg_fill_rule.as_deref().or(p.fill_rule.as_deref());
  if let Some(v) = fill_rule {
    out.push_str(" fill-rule=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // fill-opacity
  if let Some(v) = s.svg_fill_opacity {
    out.push_str(&format!(" fill-opacity=\"{}\"", v));
  }
  // stroke
  let stroke_str: Option<String> = s
    .svg_stroke
    .as_ref()
    .map(css_color_to_svg)
    .or_else(|| p.stroke.as_ref().map(|s| s.to_string()));
  if let Some(v) = &stroke_str {
    out.push_str(" stroke=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // stroke-width
  let sw: Option<String> = s
    .svg_stroke_width
    .as_ref()
    .and_then(css_length_to_svg)
    .or_else(|| p.stroke_width.as_ref().map(|s| s.to_string()));
  if let Some(v) = &sw {
    out.push_str(" stroke-width=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // stroke-opacity
  if let Some(v) = s.svg_stroke_opacity {
    out.push_str(&format!(" stroke-opacity=\"{}\"", v));
  }
  // stroke-linecap
  if let Some(v) = &s.svg_stroke_linecap {
    out.push_str(" stroke-linecap=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // stroke-linejoin
  if let Some(v) = &s.svg_stroke_linejoin {
    out.push_str(" stroke-linejoin=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // stroke-dasharray
  if let Some(v) = s.svg_stroke_dasharray.as_deref() {
    out.push_str(" stroke-dasharray=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // stroke-dashoffset
  if let Some(v) = s.svg_stroke_dashoffset.as_ref().and_then(css_length_to_svg) {
    out.push_str(&format!(" stroke-dashoffset=\"{}\"", v));
  }
  // opacity (general)
  let opacity: Option<String> = s
    .opacity
    .map(|v| format!("{}", v))
    .or_else(|| p.opacity.as_ref().map(|s| s.to_string()));
  if let Some(v) = &opacity {
    out.push_str(" opacity=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  // transform (HTML attr only — CSS transform handled by layout, not SVG)
  if let Some(v) = &p.transform {
    out.push_str(" transform=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  out.push_str("/>");
}

fn write_svg_child(child: &CascadedNode, out: &mut String) {
  match &child.element {
    Element::SvgPath(p) => write_svg_path(child, p, out),
    Element::SvgElement(el) => write_svg_element(child, el, out),
    _ => {}
  }
}

fn write_svg_element(node: &CascadedNode, el: &lui_models::SvgElement, out: &mut String) {
  out.push('<');
  out.push_str(&el.tag);
  for (k, v) in &el.attrs {
    out.push(' ');
    out.push_str(k);
    out.push_str("=\"");
    out.push_str(&escape_attr(v));
    out.push('"');
  }
  if node.children.is_empty() {
    out.push_str("/>");
  } else {
    out.push('>');
    for child in &node.children {
      write_svg_child(child, out);
      if let Element::Text(t) = &child.element {
        out.push_str(&escape_attr(t));
      }
    }
    out.push_str("</");
    out.push_str(&el.tag);
    out.push('>');
  }
}

/// Serialise an `Element::Svg` `CascadedNode` (with its children) to a
/// minimal but valid SVG 1.1 string suitable for `resvg`.
pub fn serialize_svg_node(node: &CascadedNode) -> String {
  let svg = match &node.element {
    Element::Svg(s) => s,
    _ => return String::new(),
  };

  let mut out = String::with_capacity(512);
  out.push_str(r#"<svg xmlns="http://www.w3.org/2000/svg""#);

  if let Some(w) = &svg.width {
    match w {
      SvgLength::Px(v) => {
        out.push_str(&format!(" width=\"{}px\"", v));
      }
      SvgLength::Raw(s) => {
        out.push_str(&format!(" width=\"{}\"", escape_attr(s)));
      }
      SvgLength::Percent(v) => {
        out.push_str(&format!(" width=\"{}%\"", v));
      }
      _ => {}
    }
  }
  if let Some(h) = &svg.height {
    match h {
      SvgLength::Px(v) => {
        out.push_str(&format!(" height=\"{}px\"", v));
      }
      SvgLength::Raw(s) => {
        out.push_str(&format!(" height=\"{}\"", escape_attr(s)));
      }
      SvgLength::Percent(v) => {
        out.push_str(&format!(" height=\"{}%\"", v));
      }
      _ => {}
    }
  }
  if let Some(vb) = &svg.view_box {
    out.push_str(" viewBox=\"");
    out.push_str(&escape_attr(vb));
    out.push('"');
  }
  // fill / stroke on the SVG root: CSS cascade wins, HTML attr fallback.
  let root_fill = node
    .style
    .svg_fill
    .as_ref()
    .map(css_color_to_svg)
    .or_else(|| svg.fill.as_ref().map(|s| s.to_string()));
  if let Some(f) = &root_fill {
    out.push_str(" fill=\"");
    out.push_str(&escape_attr(f));
    out.push('"');
  }
  let root_stroke = node
    .style
    .svg_stroke
    .as_ref()
    .map(css_color_to_svg)
    .or_else(|| svg.stroke.as_ref().map(|s| s.to_string()));
  if let Some(s) = &root_stroke {
    out.push_str(" stroke=\"");
    out.push_str(&escape_attr(s));
    out.push('"');
  }
  out.push('>');

  for child in &node.children {
    write_svg_child(child, &mut out);
  }

  out.push_str("</svg>");
  out
}

// ---------------------------------------------------------------------------
// Rasteriser
// ---------------------------------------------------------------------------

/// Rasterise `svg_xml` at exactly `out_w × out_h` physical pixels.
/// The SVG viewport is scaled to fill the requested rectangle while
/// honouring the `viewBox` mapping (if any) built into the string.
///
/// Returns straight (un-premultiplied) RGBA8 bytes or `None` on error.
fn rasterize_raw(svg_xml: &str, out_w: u32, out_h: u32) -> Option<Vec<u8>> {
  let opt = usvg::Options::default();
  let tree = usvg::Tree::from_str(svg_xml, &opt).ok()?;

  let mut pixmap = tiny_skia::Pixmap::new(out_w, out_h)?;

  // Scale the SVG's intrinsic size to fill the requested rect.
  let svg_size = tree.size();
  let svg_w = svg_size.width();
  let svg_h = svg_size.height();
  if svg_w <= 0.0 || svg_h <= 0.0 {
    return None;
  }
  let sx = out_w as f32 / svg_w;
  let sy = out_h as f32 / svg_h;
  let transform = tiny_skia::Transform::from_scale(sx, sy);

  // resvg 0.44: free function render(tree, transform, pixmap)
  resvg::render(&tree, transform, &mut pixmap.as_mut());

  // tiny-skia stores premultiplied RGBA; un-premultiply for the renderer.
  let raw = pixmap.data();
  let mut straight = raw.to_vec();
  for chunk in straight.chunks_exact_mut(4) {
    let a = chunk[3];
    if a != 0 && a != 255 {
      let inv = 255.0 / a as f32;
      chunk[0] = (chunk[0] as f32 * inv).min(255.0) as u8;
      chunk[1] = (chunk[1] as f32 * inv).min(255.0) as u8;
      chunk[2] = (chunk[2] as f32 * inv).min(255.0) as u8;
    }
  }
  Some(straight)
}

/// Rasterise `svg_xml` at `out_w × out_h` and wrap the result as an
/// [`ImageData`], consulting and updating the module-level raster cache.
/// Returns `None` only if parsing or pixmap allocation fails.
pub fn make_svg_image_data(svg_xml: &str, out_w: u32, out_h: u32) -> Option<ImageData> {
  if out_w == 0 || out_h == 0 {
    return None;
  }

  let key = hash_svg_key(svg_xml, out_w, out_h);

  // Fast path: already rasterised at the same size.
  {
    let cache = svg_cache().lock().ok()?;
    if let Some(entry) = cache.get(&key) {
      return Some(ImageData {
        image_id: entry.image_id,
        data: entry.data.clone(),
        width: entry.width,
        height: entry.height,
        frames: None,
      });
    }
  }

  let pixels = rasterize_raw(svg_xml, out_w, out_h)?;
  let data = std::sync::Arc::new(pixels);
  let entry = SvgCacheEntry {
    data: data.clone(),
    width: out_w,
    height: out_h,
    image_id: key,
  };

  if let Ok(mut cache) = svg_cache().lock() {
    cache.insert(key, entry);
  }

  Some(ImageData {
    image_id: key,
    data,
    width: out_w,
    height: out_h,
    frames: None,
  })
}
