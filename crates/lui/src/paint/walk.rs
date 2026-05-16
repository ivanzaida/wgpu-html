use lui_core::{
  display_list::{DisplayList, Rect as DlRect},
  transform::{parse_transform, parse_transform_origin, Transform2D},
};
use lui_glyph::TextContext;
use lui_layout::{BoxKind, LayoutBox};

use super::{background, border, clip, form, image, scrollbar, shadow, style, text};

pub fn paint_box_sel(
  b: &LayoutBox,
  dl: &mut DisplayList,
  clip_stack: &mut Vec<clip::ClipFrame>,
  text_ctx: &mut TextContext,
  scroll_offset_x: f32,
  scroll_offset_y: f32,
  parent_opacity: f32,
  dpi_scale: f32,
  path: &mut Vec<usize>,
  selection: Option<&lui_core::TextSelection>,
  sel_colors: &lui_core::SelectionColors,
  form_ctx: &form::FormPaintCtx,
  image_cache: &image::ImageCache,
) {
  if !style::is_visible(b.style) {
    return;
  }

  let opacity = parent_opacity * style::css_opacity(b.style);
  if opacity <= 0.0 {
    return;
  }

  let dx = scroll_offset_x;
  let dy = scroll_offset_y;

  let border_rect = {
    let br = b.border_rect();
    DlRect::new(br.x + dx, br.y + dy, br.width, br.height)
  };

  let xf = resolve_box_transform(b, border_rect.w, border_rect.h);
  let quad_start = dl.quads.len();
  let glyph_start = dl.glyphs.len();
  let image_start = dl.images.len();

  let (radii_h, radii_v) = style::border_radii(b.style, border_rect.w, border_rect.h);

  let is_anon = matches!(b.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline);
  if !is_anon {
    shadow::paint_box_shadows(b.style.box_shadow, border_rect, radii_h, radii_v, opacity, dl);
    background::paint_background(b, border_rect, radii_h, radii_v, opacity, dl);
    image::paint_background_image(b, border_rect, opacity, image_cache, dl);
    border::paint_borders(b, border_rect, radii_h, radii_v, opacity, dl);
  }

  image::paint_img_element(b, dx, dy, opacity, image_cache, text_ctx, dpi_scale, dl);

  if b.node.element().is_text() {
    text::paint_text_with_selection(
      b,
      b.content.x + dx,
      b.content.y + dy,
      opacity,
      text_ctx,
      dl,
      dpi_scale,
      path,
      selection,
      sel_colors,
    );
  }

  let font_size = style::css_f32(b.style.font_size).max(1.0);
  let line_height = font_size * 1.2;
  text::paint_text_decoration(
    b,
    b.content.x + dx,
    b.content.y + dy,
    b.content.width,
    line_height,
    opacity,
    dl,
  );

  text::paint_list_marker(b, b.content.x + dx, b.content.y + dy, opacity, text_ctx, dl, dpi_scale);

  form::paint_form_control(
    b,
    b.content.x + dx,
    b.content.y + dy,
    opacity,
    text_ctx,
    dl,
    dpi_scale,
    path,
    form_ctx,
  );

  let clipped = clip::should_clip(b);
  let parent_clip = if clipped {
    Some(clip::push_overflow_clip(b, dx, dy, clip_stack, dl))
  } else {
    None
  };

  let (scroll_x, scroll_y) = clip::scroll_offset(b);
  let child_dx = dx - scroll_x;
  let child_dy = dy - scroll_y;

  let mut child_order: Vec<usize> = (0..b.children.len()).collect();
  child_order.sort_by_key(|&i| z_index_sort_key(&b.children[i]));

  for &i in &child_order {
    path.push(i);
    paint_box_sel(
      &b.children[i],
      dl,
      clip_stack,
      text_ctx,
      child_dx,
      child_dy,
      opacity,
      dpi_scale,
      path,
      selection,
      sel_colors,
      form_ctx,
      image_cache,
    );
    path.pop();
  }

  if let Some(ref parent) = parent_clip {
    clip::pop_overflow_clip(parent, clip_stack, dl);
  }

  scrollbar::paint_scrollbars(b, dx, dy, opacity, dl);

  if let Some((mat, origin)) = xf {
    let xf_mat = [mat.a, mat.b, mat.c, mat.d];
    let abs_ox = border_rect.x + origin[0];
    let abs_oy = border_rect.y + origin[1];
    for q in &mut dl.quads[quad_start..] {
      q.transform = xf_mat;
      q.transform_origin = [abs_ox - q.rect.x, abs_oy - q.rect.y];
    }
    for g in &mut dl.glyphs[glyph_start..] {
      g.transform = xf_mat;
      g.transform_origin = [abs_ox - g.rect.x, abs_oy - g.rect.y];
    }
    for img in &mut dl.images[image_start..] {
      img.transform = xf_mat;
      img.transform_origin = [abs_ox - img.rect.x, abs_oy - img.rect.y];
    }
  }
}

fn resolve_box_transform(b: &LayoutBox, w: f32, h: f32) -> Option<(Transform2D, [f32; 2])> {
  let val = b.style.transform?;
  let mat = resolve_transform_value(val, w, h)?;
  let origin_str = b.style.transform_origin.and_then(css_value_as_str);
  let (ox, oy) = parse_transform_origin(origin_str, w, h);
  Some((mat, [ox, oy]))
}

fn resolve_transform_value(val: &lui_core::CssValue, w: f32, h: f32) -> Option<Transform2D> {
  match val {
    lui_core::CssValue::String(s) | lui_core::CssValue::Unknown(s) => parse_transform(s, w, h),
    lui_core::CssValue::Function { function, args } => {
      let formatted = format_css_function(function, args);
      parse_transform(&formatted, w, h)
    }
    _ => None,
  }
}

fn format_css_function(func: &lui_core::css_function::CssFunction, args: &[lui_core::CssValue]) -> String {
  let mut s = format!("{}(", func.name());
  for (i, arg) in args.iter().enumerate() {
    if i > 0 {
      s.push_str(", ");
    }
    match arg {
      lui_core::CssValue::Number(n) => s.push_str(&format!("{n}")),
      lui_core::CssValue::Percentage(n) => s.push_str(&format!("{n}%")),
      lui_core::CssValue::Dimension { value, unit } => s.push_str(&format!("{value}{}", unit.as_str())),
      _ => {}
    }
  }
  s.push(')');
  s
}

fn css_value_as_str(v: &lui_core::CssValue) -> Option<&str> {
  match v {
    lui_core::CssValue::String(s) | lui_core::CssValue::Unknown(s) => Some(s.as_ref()),
    _ => None,
  }
}

fn z_index_sort_key(b: &LayoutBox) -> (i32, i32) {
  match b.z_index {
    Some(z) if z < 0 => (-1, z),
    Some(z) => (1, z),
    None => (0, 0),
  }
}
