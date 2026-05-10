//! Inline formatting context (IFC) and rich-text paragraph layout.
//!
//! Extracted from `lib.rs` to keep the block-layout file focused.
//! The IFC handles inline-level children: text wrapping, baseline
//! alignment, atomic inlines, and the rich-text paragraph path that
//! feeds cosmic-text's `set_rich_text`.


use lui_models::{
    common::css_enums::{
        BoxSizing, CssColor, CssLength, Cursor, Display, PointerEvents, Resize, UserSelect, VerticalAlign,
    },
    Style,
};
use lui_style::{CascadedNode, PseudoElementStyle};
use lui_text::{ParagraphSpan, PositionedGlyph, ShapedLine, ShapedRun};
use lui_tree::Element;

use crate::{
    box_model::{
        clamp_corner_radii, compute_background_box, resolve_border_widths,
        resolve_insets_margin, resolve_insets_padding,
    },
    color::{self, resolve_color, Color},
    incremental::{
        file_button_from_pseudo, lui_calendar_from_pseudo,
        lui_color_from_pseudo, lui_popup_from_pseudo, resolve_lui_properties,
    },
    layout_profile,
    length,
    positioned::{
        resolved_cursor, resolved_opacity, resolved_pointer_events,
        resolved_user_select, resolved_z_index,
    },
    types::*,
    BlockOverrides, Ctx, LayoutBox,
};

// These are all in crate root (lib.rs or text_shaping.rs with pub(crate) use):
use crate::{
    apply_text_transform, compute_placeholder_run, compute_value_run,
    empty_box, font_size_px, font_style_axis, font_weight_value,
    form_control_default_line_height, form_control_info, has_native_appearance,
    layout_block, line_height_px, make_text_leaf, normalize_text_for_style,
    parse_family_list, resolve_text_decorations,
    split_collapsed_first_word_prefix_and_tail,
    style_collapses_whitespace, style_wraps_text, style_breaks_all,
    trim_collapsed_whitespace_edges,
};

/// Inline-level test: an element whose default formatting puts it on
/// a line with its siblings. Honours an explicit `display` override
/// (`inline / inline-block / inline-flex`) but otherwise defaults by
/// HTML element kind. Block-by-default elements like `<div>` /
/// `<p>` / headings are *not* inline-level.
fn is_inline_level(node: &CascadedNode) -> bool {
  if let Some(d) = node.style.display.as_ref() {
    use lui_models::common::css_enums::Display::*;
    return matches!(d, Inline | InlineBlock | InlineFlex | Ruby | RubyText);
  }
  matches!(
    &node.element,
    Element::Text(_)
      | Element::Span(_)
      | Element::A(_)
      | Element::Strong(_)
      | Element::B(_)
      | Element::Em(_)
      | Element::I(_)
      | Element::U(_)
      | Element::S(_)
      | Element::Small(_)
      | Element::Mark(_)
      | Element::Code(_)
      | Element::Kbd(_)
      | Element::Samp(_)
      | Element::Var(_)
      | Element::Abbr(_)
      | Element::Cite(_)
      | Element::Dfn(_)
      | Element::Sub(_)
      | Element::Sup(_)
      | Element::Time(_)
      | Element::Br(_)
      | Element::Wbr(_)
      | Element::Bdi(_)
      | Element::Bdo(_)
      | Element::Ins(_)
      | Element::Del(_)
      | Element::Label(_)
      | Element::Output(_)
      | Element::Data(_)
      | Element::Ruby(_)
      | Element::Rt(_)
      | Element::Rp(_)
      | Element::Img(_)
      | Element::CustomElement(_)
  )
}

fn make_pseudo_node(pe: &PseudoElementStyle) -> CascadedNode {
  CascadedNode {
    element: Element::Span(lui_models::Span::default()),
    style: pe.style.clone(),
    children: vec![CascadedNode {
      element: Element::Text(pe.content_text.clone()),
      style: Style::default(),
      children: vec![],
      before: None,
      after: None,
      first_line: None,
      first_letter: None,
      placeholder: None,
      selection: None,
      marker: None,
      lui_pseudo: vec![],
    }],
    before: None,
    after: None,
    first_line: None,
    first_letter: None,
    placeholder: None,
    selection: None,
    marker: None,
    lui_pseudo: vec![],
  }
}

pub(crate) fn effective_children(node: &CascadedNode) -> Vec<std::borrow::Cow<'_, CascadedNode>> {
  use std::borrow::Cow;
  let mut out = Vec::with_capacity(node.children.len() + 3);
  if let Some(ref pe) = node.marker {
    out.push(Cow::Owned(make_pseudo_node(pe)));
  }
  if let Some(ref pe) = node.before {
    out.push(Cow::Owned(make_pseudo_node(pe)));
  }
  for child in &node.children {
    out.push(Cow::Borrowed(child));
  }
  if let Some(ref pe) = node.after {
    out.push(Cow::Owned(make_pseudo_node(pe)));
  }
  out
}

fn has_pseudo_elements(node: &CascadedNode) -> bool {
  node.before.is_some() || node.after.is_some() || node.marker.is_some()
}

/// True when every child of `node` is an inline-level box, so the
/// whole block becomes one inline formatting context. Empty parents
/// stay in block-flow (with zero content) — they have nothing to
/// flow.
pub(crate) fn all_children_inline_level(node: &CascadedNode) -> bool {
  let has_real = !node.children.is_empty();
  let has_pseudo = has_pseudo_elements(node);
  if !has_real && !has_pseudo {
    return false;
  }
  let real_inline = node.children.iter().all(is_inline_level);
  let pseudo_inline = node.before.as_ref().map_or(true, |pe| {
    pe.style.display.map_or(true, |d| matches!(d, Display::Inline | Display::InlineBlock))
  }) && node.after.as_ref().map_or(true, |pe| {
    pe.style.display.map_or(true, |d| matches!(d, Display::Inline | Display::InlineBlock))
  });
  real_inline && pseudo_inline
}

/// Result of laying out one inline-level subtree at a temporary
/// origin. The caller composes these on a horizontal cursor and
/// re-aligns each on the line's baseline by adjusting `box_.y` after
/// the fact.
///
/// Currently unused — the IFC switched to the rich-text paragraph
/// path (`layout_inline_paragraph`), which feeds cosmic-text's
/// `set_rich_text` and never goes through `layout_inline_subtree`.
/// The struct is left in place so future work that needs to compose
/// layouts horizontally on a custom path (e.g. inline-block content)
/// can re-use it without reinventing the shape.
#[allow(dead_code)]
struct InlineLayout {
  box_: LayoutBox,
  width: f32,
  ascent: f32,
  descent: f32,
  vertical_align: Option<VerticalAlign>,
}

/// Lay out one inline-level subtree starting at `(origin_x, origin_y)`.
/// Text leaves shape into a single `BoxKind::Text` with the run +
/// foreground colour. Inline elements recurse, position their
/// children on a baseline (so a `<small>` and a `<strong>` flow on
/// the same line), and wrap the result in a `BoxKind::Block` whose
/// background — if any — covers the inline element's content extent
/// (this is what makes `<mark>` paintable).
///
/// Currently unused — see the note on [`InlineLayout`].
#[allow(dead_code)]
fn layout_inline_subtree(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  container_w: f32,
  ctx: &mut Ctx,
) -> InlineLayout {
  // `display: none` removes the inline subtree from the line —
  // zero width, zero ascent / descent, no paintable box.
  if matches!(node.style.display, Some(Display::None)) {
    return InlineLayout {
      box_: empty_box(origin_x, origin_y),
      width: 0.0,
      ascent: 0.0,
      descent: 0.0,
      vertical_align: node.style.vertical_align.clone(),
    };
  }

  if let Element::Text(s) = &node.element {
    let max_width = if style_wraps_text(&node.style) && container_w.is_finite() && container_w > 0.0 {
      Some(container_w)
    } else {
      None
    };
    let (box_, w, h, ascent) = make_text_leaf(s, &node.style, origin_x, origin_y, max_width, false, ctx);
    let descent = (h - ascent).max(0.0);
    return InlineLayout {
      box_,
      width: w,
      ascent,
      descent,
      vertical_align: node.style.vertical_align.clone(),
    };
  }

  if is_atomic_inline(node) {
    return layout_atomic_inline_subtree(node, origin_x, origin_y, container_w, ctx);
  }

  if matches!(&node.element, Element::Img(_)) {
    if is_empty_inline_img(node) {
      return InlineLayout {
        box_: empty_box(origin_x, origin_y),
        width: 0.0,
        ascent: 0.0,
        descent: 0.0,
        vertical_align: node.style.vertical_align.clone(),
      };
    }
    let box_ = layout_block(
      node,
      origin_x,
      origin_y,
      container_w,
      f32::INFINITY,
      Rect::new(origin_x, origin_y, container_w, f32::INFINITY),
      BlockOverrides::default(),
      ctx,
    );
    let width = box_.margin_rect.w;
    let height = box_.margin_rect.h;
    return InlineLayout {
      box_,
      width,
      ascent: height,
      descent: 0.0,
      vertical_align: node.style.vertical_align.clone(),
    };
  }

  // Inline element: walk children at a horizontal cursor, then
  // baseline-align them inside this element.
  let mut cursor_x = 0.0_f32;
  let mut max_ascent = 0.0_f32;
  let mut max_descent = 0.0_f32;
  let mut child_layouts: Vec<InlineLayout> = Vec::new();
  for child in &node.children {
    let cl = layout_inline_subtree(
      child,
      origin_x + cursor_x,
      origin_y,
      (container_w - cursor_x).max(0.0),
      ctx,
    );
    if cl.ascent > max_ascent {
      max_ascent = cl.ascent;
    }
    if cl.descent > max_descent {
      max_descent = cl.descent;
    }
    cursor_x += cl.width;
    child_layouts.push(cl);
  }

  let line_h = max_ascent + max_descent;
  let baseline_y = origin_y + max_ascent;
  let mut final_children: Vec<LayoutBox> = Vec::with_capacity(child_layouts.len());
  for (child, cl) in node.children.iter().zip(child_layouts.into_iter()) {
    let cur_top = cl.box_.margin_rect.y;
    let font_size = font_size_px(&child.style).unwrap_or(16.0) * ctx.scale;
    let va_dy = vertical_align_dy(
      &child.style.vertical_align,
      cl.ascent,
      cl.descent,
      max_ascent,
      max_descent,
      line_h,
      font_size,
      ctx.scale,
    );
    let target_top = baseline_y - cl.ascent - va_dy;
    let dy = target_top - cur_top;
    let mut b = cl.box_;
    translate_box_y_in_place(&mut b, dy);
    final_children.push(b);
  }

  let bg = node.style.background_color.as_ref().and_then(resolve_color);
  let r = Rect::new(origin_x, origin_y, cursor_x, line_h);
  let box_ = LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: bg,
    background_rect: r,
    background_radii: CornerRadii::zero(),
    border: Insets::zero(),
    border_colors: BorderColors::default(),
    border_styles: BorderStyles::default(),
    border_radius: CornerRadii::zero(),
    kind: BoxKind::Block,
    text_run: None,
    text_color: None,
    // Decorations live on text leaves (cascade inheritance has
    // already propagated `text-decoration` down to every text
    // descendant). The inline wrapper itself draws nothing.
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes::visible(),
    resize: Resize::None,
    text_overflow: None,
    transform: None,
    transform_origin: (0.0, 0.0),
    opacity: resolved_opacity(&node.style),
    pointer_events: resolved_pointer_events(&node.style),
    user_select: resolved_user_select(&node.style),
    cursor: resolved_cursor(&node.style),
    z_index: resolved_z_index(&node.style),
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    file_button: None,
    children: final_children,
    is_fixed: false,
    form_control: None,
  };
  InlineLayout {
    box_,
    width: cursor_x,
    ascent: max_ascent,
    descent: max_descent,
    vertical_align: node.style.vertical_align.clone(),
  }
}

fn layout_atomic_inline_subtree(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  container_w: f32,
  ctx: &mut Ctx,
) -> InlineLayout {
  let style = &node.style;
  let margin = resolve_insets_margin(style, container_w, ctx);
  let mut border = resolve_border_widths(style, container_w, ctx);
  let mut padding = resolve_insets_padding(style, container_w, ctx);
  if has_native_appearance(node) {
    border = Insets::zero();
    padding = Insets::zero();
  }
  let box_sizing = style.box_sizing.clone().unwrap_or(BoxSizing::ContentBox);

  let specified_w = length::resolve(style.width.as_ref(), container_w, ctx).map(|specified| match box_sizing {
    BoxSizing::ContentBox => specified,
    BoxSizing::BorderBox => (specified - border.horizontal() - padding.horizontal()).max(0.0),
  });
  let content_x = origin_x + margin.left + border.left + padding.left;
  let content_y = origin_y + margin.top + border.top + padding.top;

  let (mut children, measured_w, measured_h, max_ascent, _max_descent) =
    layout_inline_children_no_wrap(node, content_x, content_y, specified_w.unwrap_or(container_w), ctx);

  let inner_width = specified_w.unwrap_or(measured_w);
  let specified_h = length::resolve(style.height.as_ref(), 0.0, ctx).map(|specified| match box_sizing {
    BoxSizing::ContentBox => specified,
    BoxSizing::BorderBox => (specified - border.vertical() - padding.vertical()).max(0.0),
  });
  let mut inner_height = specified_h.unwrap_or(measured_h);

  // Empty form controls (`<input>`, `<textarea>`, `<select>`,
  // `<button>` with no children) collapse to `inner_height = 0`
  // because they have nothing to measure. Browsers give them a
  // default content height equal to one line of the cascaded
  // font, so the placeholder text run we attach below has room
  // to render and the box visually matches the user's typed
  // content height. This is also what `<input value="">` would
  // need once value rendering lands.
  if specified_h.is_none() && form_control_default_line_height(node) {
    let font_size = font_size_px(style).unwrap_or(16.0);
    let line_h = line_height_px(style, font_size) * ctx.scale;
    if inner_height < line_h {
      inner_height = line_h;
    }
  }

  if inner_height > measured_h && max_ascent > 0.0 {
    let shift = inner_height - measured_h;
    for child in &mut children {
      translate_box_y_in_place(child, shift);
    }
  }

  let border_rect = Rect::new(
    origin_x + margin.left,
    origin_y + margin.top,
    border.horizontal() + padding.horizontal() + inner_width,
    border.vertical() + padding.vertical() + inner_height,
  );
  let content_rect = Rect::new(content_x, content_y, inner_width, inner_height);
  let margin_rect = Rect::new(
    origin_x,
    origin_y,
    margin.horizontal() + border_rect.w,
    margin.vertical() + border_rect.h,
  );

  let fg = color::resolve_foreground(style.color.as_ref(), color::BLACK);
  let background = style.background_color.as_ref().and_then(|c| color::resolve_with_current(c, fg));
  let accent_color = style.accent_color.as_ref().and_then(|c| color::resolve_with_current(c, fg));
  let lui = resolve_lui_properties(&style.custom_properties, fg);
  let resolve_border = |c: &CssColor| color::resolve_with_current(c, fg);
  let border_colors = BorderColors {
    top: style.border_top_color.as_ref().and_then(resolve_border).or(Some(fg)),
    right: style.border_right_color.as_ref().and_then(resolve_border).or(Some(fg)),
    bottom: style.border_bottom_color.as_ref().and_then(resolve_border).or(Some(fg)),
    left: style.border_left_color.as_ref().and_then(resolve_border).or(Some(fg)),
  };
  let border_styles = BorderStyles {
    top: style.border_top_style.clone(),
    right: style.border_right_style.clone(),
    bottom: style.border_bottom_style.clone(),
    left: style.border_left_style.clone(),
  };
  let resolve_corner = |h: Option<&CssLength>, v: Option<&CssLength>, ctx: &mut Ctx| -> Radius {
    let h_px = length::resolve(h, container_w, ctx).unwrap_or(0.0).max(0.0);
    let v_px = match v {
      Some(_) => length::resolve(v, inner_height.max(1.0), ctx).unwrap_or(0.0).max(0.0),
      None => h_px,
    };
    Radius { h: h_px, v: v_px }
  };
  let mut border_radius = CornerRadii {
    top_left: resolve_corner(
      style.border_top_left_radius.as_ref(),
      style.border_top_left_radius_v.as_ref(),
      ctx,
    ),
    top_right: resolve_corner(
      style.border_top_right_radius.as_ref(),
      style.border_top_right_radius_v.as_ref(),
      ctx,
    ),
    bottom_right: resolve_corner(
      style.border_bottom_right_radius.as_ref(),
      style.border_bottom_right_radius_v.as_ref(),
      ctx,
    ),
    bottom_left: resolve_corner(
      style.border_bottom_left_radius.as_ref(),
      style.border_bottom_left_radius_v.as_ref(),
      ctx,
    ),
  };
  clamp_corner_radii(&mut border_radius, border_rect.w, border_rect.h);
  let (background_rect, background_radii) =
    compute_background_box(style, border_rect, content_rect, border, padding, &border_radius);

  // Approximate inline-block baseline by the baseline of its last text
  // line, without adding padding-top directly into ascent; this keeps
  // neighboring inline text on the same baseline while the full box
  // height still contributes via descent.
  let inline_ascent = margin.top + border.top + max_ascent.max(0.0);
  let inline_descent = (margin_rect.h - inline_ascent).max(0.0);

  // For form controls (`<input>`, `<textarea>`), attach the value
  // text or the placeholder attribute as the box's text run.

  let (value_run, value_color) = compute_value_run(node, content_rect, ctx);
  let (placeholder_run, placeholder_color) = if value_run.is_some() {
    (value_run, value_color)
  } else {
    compute_placeholder_run(node, content_rect, ctx)
  };

  let fc = form_control_info(node);
  let text_color = placeholder_color.or_else(|| if fc.is_some() { Some(fg) } else { None });

  InlineLayout {
    box_: LayoutBox {
      margin_rect,
      border_rect,
      content_rect,
      background,
      background_rect,
      background_radii,
      border,
      border_colors,
      border_styles,
      border_radius,
      kind: BoxKind::Block,
      text_run: placeholder_run,
      text_color,
      text_unselectable: true,
      text_decorations: Vec::new(),
      overflow: OverflowAxes::visible(),
      resize: Resize::None,
      text_overflow: None,
    transform: None,
    transform_origin: (0.0, 0.0),
      opacity: resolved_opacity(style),
      pointer_events: resolved_pointer_events(style),
      user_select: resolved_user_select(style),
      cursor: resolved_cursor(style),
      z_index: resolved_z_index(style),
      image: None,
      background_image: None,
      first_line_color: None,
      first_letter_color: None,
      selection_bg: None,
      selection_fg: None,
      accent_color,
      lui,
      lui_popup: lui_popup_from_pseudo(node),
      lui_color_picker: lui_color_from_pseudo(node),
      lui_calendar: lui_calendar_from_pseudo(node),
      file_button: file_button_from_pseudo(node),
      children,
      is_fixed: false,
      form_control: fc,
    },
    width: margin_rect.w,
    ascent: inline_ascent,
    descent: inline_descent,
    vertical_align: node.style.vertical_align.clone(),
  }
}

fn layout_inline_children_no_wrap(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  _container_w: f32,
  ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32, f32, f32) {
  let mut cursor_x = 0.0_f32;
  let mut max_ascent = 0.0_f32;
  let mut max_descent = 0.0_f32;
  let mut child_layouts: Vec<InlineLayout> = Vec::new();
  for child in &node.children {
    let cl = layout_inline_subtree(child, origin_x + cursor_x, origin_y, f32::INFINITY, ctx);
    max_ascent = max_ascent.max(cl.ascent);
    max_descent = max_descent.max(cl.descent);
    cursor_x += cl.width;
    child_layouts.push(cl);
  }

  let line_h = max_ascent + max_descent;
  let baseline_y = origin_y + max_ascent;
  let mut final_children: Vec<LayoutBox> = Vec::with_capacity(child_layouts.len());
  for (child, cl) in node.children.iter().zip(child_layouts.into_iter()) {
    let font_size = font_size_px(&child.style).unwrap_or(16.0) * ctx.scale;
    let va_dy = vertical_align_dy(
      &child.style.vertical_align,
      cl.ascent,
      cl.descent,
      max_ascent,
      max_descent,
      line_h,
      font_size,
      ctx.scale,
    );
    let target_top = baseline_y - cl.ascent - va_dy;
    let dy = target_top - cl.box_.margin_rect.y;
    let mut b = cl.box_;
    if dy != 0.0 {
      translate_box_y_in_place(&mut b, dy);
    }
    final_children.push(b);
  }

  (final_children, cursor_x, line_h, max_ascent, max_descent)
}

fn is_atomic_inline(node: &CascadedNode) -> bool {
  matches!(node.style.display, Some(Display::InlineBlock | Display::InlineFlex))
}

/// Lay out a block's inline-level children as a stack of line boxes
/// at `(origin_x, origin_y)`. Returns the final children (already
/// positioned absolutely) plus the paragraph's used width (max line
/// width) and height (sum of line heights).
///
/// Behaviour:
/// - **Single text-leaf child** — shapes the leaf with cosmic-text's soft-wrap (`Some(container_w)`) so the paragraph
///   breaks at actual word boundaries inside the run.
/// - **Multiple inline children** — greedy element-boundary wrap. Each child is shaped on its own line at a scratch
///   origin; the IFC accumulates them onto the current line and rolls over to a new line when `cursor_x + child.width >
///   container_w`. Breaks land between elements (a `<strong>` either fits on the line or moves whole to the next line);
///   breaks *inside* a multi-leaf sentence are still pending — that's the cross-leaf rich-text shape pass tracked under
///   T7.
///
/// Each completed line is baseline-aligned independently (its max
/// ascent over its own children) and shifted by `horizontal_align_offset`
/// for `text-align`. `justify` falls through to `left`.
pub(crate) fn layout_inline_block_children(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  container_w: f32,
  ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
  let text_align = node.style.text_align.as_ref();

  // Single-text-leaf fast path: cosmic-text's word-boundary wrap
  // gives the right answer for plain paragraphs.
  if node.children.len() == 1 && !has_pseudo_elements(node) {
    if let Element::Text(s) = &node.children[0].element {
      let child_style = &node.children[0].style;
      let (box_, w, h, _ascent) = make_text_leaf(s, child_style, origin_x, origin_y, Some(container_w), true, ctx);
      // Heuristic text-align: the wrapped run's `width` is the
      // *widest* line, so right / center align by shifting the
      // whole box. Multi-line per-line align (the proper
      // CSS behaviour) lands with the rich-text path.
      let align_dx = horizontal_align_offset(text_align, container_w, w);
      let mut b = box_;
      if align_dx != 0.0 {
        translate_box_x_in_place(&mut b, align_dx);
      }
      return (vec![b], w, h);
    }
  }

  if contains_atomic_inline(node) {
    return layout_inline_mixed_children(node, origin_x, origin_y, container_w, ctx);
  }

  // Multi-child IFC: rich-text paragraph shape. We flatten the
  // inline subtree into a list of `(text, attrs)` spans (one per
  // source text leaf), feed cosmic-text via `set_rich_text` so its
  // word-boundary breaks land *between* spans without losing per-
  // span attributes, then re-expand the result into anonymous Block
  // boxes for per-line backgrounds (`<mark>`) and decoration bars
  // (`<a>` / `<u>` / `<s>`), plus one `BoxKind::Text` containing
  // every glyph (with each glyph's source colour baked in by
  // `shape_paragraph`).
  layout_inline_paragraph(node, origin_x, origin_y, container_w, text_align, ctx)
}

fn layout_inline_mixed_children(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  container_w: f32,
  ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
  fn first_line_width(cl: &InlineLayout) -> f32 {
    let Some(run) = cl.box_.text_run.as_ref() else {
      return cl.width;
    };
    let Some(first) = run.lines.first() else {
      return cl.width;
    };
    let mut max_right = 0.0_f32;
    for g in &run.glyphs[first.glyph_range.0..first.glyph_range.1] {
      max_right = max_right.max(g.x + g.w);
    }
    if max_right > 0.0 { max_right } else { cl.width }
  }

  fn text_inline_layout(
    text: &str,
    style: &Style,
    origin_x: f32,
    origin_y: f32,
    max_width_px: Option<f32>,
    ctx: &mut Ctx,
  ) -> InlineLayout {
    let (box_, w, h, ascent) = make_text_leaf(text, style, origin_x, origin_y, max_width_px, false, ctx);
    let descent = (h - ascent).max(0.0);
    InlineLayout {
      box_,
      width: w,
      ascent,
      descent,
      vertical_align: style.vertical_align.clone(),
    }
  }

  struct Line {
    items: Vec<(InlineLayout, f32)>,
    width: f32,
    ascent: f32,
    descent: f32,
    y: f32,
  }

  let wrap = container_w.is_finite() && container_w > 0.0 && style_wraps_text(&node.style);
  let font_px = font_size_px(&node.style).unwrap_or(16.0);
  let hard_break_height = line_height_px(&node.style, font_px) * ctx.scale;
  let mut lines: Vec<Line> = Vec::new();
  let mut current = Line {
    items: Vec::new(),
    width: 0.0,
    ascent: 0.0,
    descent: 0.0,
    y: origin_y,
  };
  let mut cursor_y = origin_y;

  for child in &node.children {
    let child_font_size = font_size_px(&child.style).unwrap_or(16.0) * ctx.scale;
    if matches!(&child.element, Element::Br(_)) {
      let line_h = (current.ascent + current.descent).max(hard_break_height);
      cursor_y += line_h;
      lines.push(current);
      current = Line {
        items: Vec::new(),
        width: 0.0,
        ascent: 0.0,
        descent: 0.0,
        y: cursor_y,
      };
      continue;
    }
    if wrap && !current.items.is_empty() && matches!(&child.element, Element::Text(_)) {
      let remaining = (container_w - current.width).max(0.0);
      let min_inline_room = font_size_px(&child.style).unwrap_or(font_px) * ctx.scale;
      if remaining < min_inline_room {
        let line_h = (current.ascent + current.descent).max(hard_break_height);
        cursor_y += line_h;
        lines.push(current);
        current = Line {
          items: Vec::new(),
          width: 0.0,
          ascent: 0.0,
          descent: 0.0,
          y: cursor_y,
        };
      }
    }
    let mut cl = layout_inline_subtree(
      child,
      origin_x + current.width,
      cursor_y,
      (container_w - current.width).max(0.0),
      ctx,
    );
    let wrapped_under_remainder = wrap
      && !current.items.is_empty()
      && matches!(&child.element, Element::Text(_))
      && cl
      .box_
      .text_run
      .as_ref()
      .map(|run| run.lines.len() > 1)
      .unwrap_or(false);
    if wrapped_under_remainder {
      let mut kept_head_on_line = false;
      if let Element::Text(raw) = &child.element {
        let remaining = (container_w - current.width).max(0.0);
        if let Some((head, tail)) = split_collapsed_first_word_prefix_and_tail(raw, &child.style) {
          let head_cl = text_inline_layout(&head, &child.style, origin_x + current.width, cursor_y, None, ctx);
          if head_cl.width > 0.0 && head_cl.width <= remaining {
            current.width += head_cl.width;
            current.ascent = current.ascent.max(head_cl.ascent);
            current.descent = current.descent.max(head_cl.descent);
            current.items.push((head_cl, child_font_size));
            kept_head_on_line = true;
            if !tail.trim().is_empty() {
              let line_h = (current.ascent + current.descent).max(hard_break_height);
              cursor_y += line_h;
              lines.push(current);
              current = Line {
                items: Vec::new(),
                width: 0.0,
                ascent: 0.0,
                descent: 0.0,
                y: cursor_y,
              };
              cl = text_inline_layout(&tail, &child.style, origin_x, cursor_y, Some(container_w), ctx);
            } else {
              continue;
            }
          }
        }
      }
      if !kept_head_on_line {
        let line_h = (current.ascent + current.descent).max(hard_break_height);
        cursor_y += line_h;
        lines.push(current);
        current = Line {
          items: Vec::new(),
          width: 0.0,
          ascent: 0.0,
          descent: 0.0,
          y: cursor_y,
        };
        cl = layout_inline_subtree(child, origin_x, cursor_y, container_w, ctx);
      }
    }
    let fit_width = first_line_width(&cl);
    if wrap && !current.items.is_empty() && current.width + fit_width > container_w {
      let line_h = current.ascent + current.descent;
      cursor_y += line_h;
      lines.push(current);
      current = Line {
        items: Vec::new(),
        width: 0.0,
        ascent: 0.0,
        descent: 0.0,
        y: cursor_y,
      };
      cl = layout_inline_subtree(child, origin_x, cursor_y, container_w, ctx);
    }
    current.width += cl.width;
    current.ascent = current.ascent.max(cl.ascent);
    current.descent = current.descent.max(cl.descent);
    current.items.push((cl, child_font_size));
  }
  lines.push(current);

  let mut final_children: Vec<LayoutBox> = Vec::new();
  let mut max_width = 0.0_f32;
  let mut total_h = 0.0_f32;
  for line in lines {
    max_width = max_width.max(line.width);
    let line_h = line.ascent + line.descent;
    total_h = (line.y - origin_y) + line_h;
    let baseline_y = line.y + line.ascent;
    let align_dx = horizontal_align_offset(node.style.text_align.as_ref(), container_w, line.width);
    for (cl, font_size) in line.items {
      let va_dy = vertical_align_dy(
        &cl.vertical_align,
        cl.ascent,
        cl.descent,
        line.ascent,
        line.descent,
        line_h,
        font_size,
        ctx.scale,
      );
      let target_top = baseline_y - cl.ascent - va_dy;
      let dy = target_top - cl.box_.margin_rect.y;
      let mut b = cl.box_;
      if dy != 0.0 {
        translate_box_y_in_place(&mut b, dy);
      }
      if align_dx != 0.0 {
        translate_box_x_in_place(&mut b, align_dx);
      }
      final_children.push(b);
    }
  }

  (final_children, max_width, total_h)
}

fn contains_atomic_inline(node: &CascadedNode) -> bool {
  node.children.iter().any(|child| {
    matches!(&child.element, Element::Img(_))
      || is_atomic_inline(child)
      || (!child.children.is_empty() && contains_atomic_inline(child))
  })
}

fn is_empty_inline_img(node: &CascadedNode) -> bool {
  match &node.element {
    Element::Img(img) => {
      img.src.is_none()
        && img.width.is_none()
        && img.height.is_none()
        && node.style.width.is_none()
        && node.style.height.is_none()
    }
    _ => false,
  }
}

/// Compute the vertical offset (dy) for CSS `vertical-align` relative to
/// the baseline. A positive return value shifts the box **up** (decreases
/// its y coordinate). The caller subtracts this from the baseline-aligned
/// target position.
///
/// All length parameters (`font_size`, `line_ascent`, etc.) must be in
/// physical pixels (CSS px × scale).
fn vertical_align_dy(
    va: &Option<VerticalAlign>,
    child_ascent: f32,
    child_descent: f32,
    line_ascent: f32,
    line_descent: f32,
    line_h: f32,
    font_size: f32,
    scale: f32,
) -> f32 {
    let va = match va {
        Some(v) => v,
        None => return 0.0,
    };
    match va {
        VerticalAlign::Baseline => 0.0,
        VerticalAlign::Sub => -font_size * 0.2,
        VerticalAlign::Super => font_size * 0.4,
        VerticalAlign::Top => line_ascent - child_ascent,
        VerticalAlign::Bottom => child_descent - line_descent,
        VerticalAlign::Middle => child_ascent * 0.5 - font_size * 0.25,
        VerticalAlign::TextTop => font_size - child_ascent,
        VerticalAlign::TextBottom => line_descent - child_descent,
        VerticalAlign::Length(len) => match len {
            CssLength::Px(v) => *v * scale,
            CssLength::Em(v) => *v * font_size,
            CssLength::Rem(v) => *v * 16.0 * scale,
            CssLength::Percent(v) => *v * 0.01 * line_h,
            _ => 0.0,
        },
    }
}

// ---------------------------------------------------------------------------
// Rich-text paragraph path
// ---------------------------------------------------------------------------

/// One source text leaf the paragraph plan has flattened from the
/// inline subtree. Attributes are already resolved against the
/// cascade (font matched to a concrete family name, colour reduced
/// to linear RGBA, sizes converted to physical pixels).
struct SpanData {
  text: String,
  family: String,
  weight: u16,
  style_axis: lui_text::FontStyleAxis,
  size_px: f32,
  line_height_px: f32,
  color: Color,
}

/// One inline element the paragraph plan crossed. `leaf_range` is
/// the half-open interval of `SpanData` indices the element wraps —
/// used to assemble per-line backgrounds and decoration bars after
/// shaping.
struct InlineBlockSpan {
  leaf_range: (u32, u32),
  background: Option<Color>,
  decorations: Vec<TextDecorationLine>,
  decoration_color: Color,
  opacity: f32,
}

#[derive(Default)]
struct ParagraphPlan {
  spans: Vec<SpanData>,
  inline_blocks: Vec<InlineBlockSpan>,
}

#[derive(Default)]
struct ParagraphCollapseState {
  prev_space: bool,
}

fn color_with_opacity(mut color: Color, opacity: f32) -> Color {
  color[3] *= opacity.clamp(0.0, 1.0);
  color
}

fn push_paragraph_span(node: &CascadedNode, text: String, plan: &mut ParagraphPlan, ctx: &mut Ctx, opacity: f32) {
  if text.is_empty() {
    return;
  }
  let families = parse_family_list(node.style.font_family.as_deref());
  let family_refs: Vec<&str> = families.iter().map(String::as_str).collect();
  let weight = font_weight_value(node.style.font_weight.as_ref());
  let axis = font_style_axis(node.style.font_style.as_ref());
  let family = {
    layout_profile::count_text_shape(&mut ctx.profiler); // resolve_family
    ctx
      .text
      .ctx
      .resolve_family(&family_refs, weight, axis)
      .unwrap_or_default()
  };

  let size_css = font_size_px(&node.style).unwrap_or(16.0);
  let line_h_css = line_height_px(&node.style, size_css);
  let color = node
    .style
    .color
    .as_ref()
    .and_then(resolve_color)
    .unwrap_or([0.0, 0.0, 0.0, 1.0]);

  plan.spans.push(SpanData {
    text,
    family,
    weight,
    style_axis: axis,
    size_px: size_css * ctx.scale,
    line_height_px: line_h_css * ctx.scale,
    color: color_with_opacity(color, opacity),
  });
}

/// Walk one inline-level subtree depth-first, appending to `plan`.
/// `Element::Text` becomes a span; an inline element wrapping
/// children that contributed any spans is recorded as an
/// `InlineBlockSpan` if it has a background or decoration, so its
/// per-line bounds can be reconstructed after shaping.
fn collect_paragraph_spans(
  node: &CascadedNode,
  plan: &mut ParagraphPlan,
  ctx: &mut Ctx,
  collapse: &mut ParagraphCollapseState,
  inherited_opacity: f32,
) {
  layout_profile::count_text_shape(&mut ctx.profiler); // collect_spans
  if matches!(node.style.display, Some(Display::None)) {
    return;
  }
  let opacity = inherited_opacity * resolved_opacity(&node.style);

  if matches!(&node.element, Element::Br(_)) {
    collapse.prev_space = false;
    push_paragraph_span(node, "\n".to_string(), plan, ctx, opacity);
    return;
  }

  if matches!(&node.element, Element::Wbr(_)) {
    push_paragraph_span(node, "\u{200B}".to_string(), plan, ctx, opacity);
    return;
  }

  if let Element::Text(s) = &node.element {
    let normalized = normalize_text_for_style(s, &node.style, Some(&mut collapse.prev_space));
    if normalized.is_empty() {
      return;
    }
    let display = match apply_text_transform(&normalized, node.style.text_transform.as_ref()) {
      Some(t) => t,
      None => normalized,
    };
    if style_breaks_all(&node.style) && display.len() > 1 {
      let mut buf = String::with_capacity(display.len() * 2);
      let mut cs = display.chars();
      if let Some(first) = cs.next() {
        buf.push(first);
        for ch in cs {
          buf.push('\u{200B}');
          buf.push(ch);
        }
      }
      push_paragraph_span(node, buf, plan, ctx, opacity);
    } else {
      push_paragraph_span(node, display, plan, ctx, opacity);
    }
    return;
  }

  let leaf_start = plan.spans.len() as u32;
  if let Some(ref pe) = node.marker {
    let pseudo = make_pseudo_node(pe);
    collect_paragraph_spans(&pseudo, plan, ctx, collapse, opacity);
  }
  if let Some(ref pe) = node.before {
    let pseudo = make_pseudo_node(pe);
    collect_paragraph_spans(&pseudo, plan, ctx, collapse, opacity);
  }
  for child in &node.children {
    collect_paragraph_spans(child, plan, ctx, collapse, opacity);
  }
  if let Some(ref pe) = node.after {
    let pseudo = make_pseudo_node(pe);
    collect_paragraph_spans(&pseudo, plan, ctx, collapse, opacity);
  }
  let leaf_end = plan.spans.len() as u32;
  if leaf_end > leaf_start {
    let bg = node.style.background_color.as_ref().and_then(resolve_color);
    let decos = resolve_text_decorations(&node.style);
    if bg.is_some() || !decos.is_empty() {
      let decoration_color = node
        .style
        .color
        .as_ref()
        .and_then(resolve_color)
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
      plan.inline_blocks.push(InlineBlockSpan {
        leaf_range: (leaf_start, leaf_end),
        background: bg,
        decorations: decos,
        decoration_color,
        opacity,
      });
    }
  }
}

/// Build a `LayoutBox` whose only purpose is to paint a solid
/// background fill — used for inline-element backgrounds (`<mark>`)
/// and decoration bars (underline / line-through / overline).
fn make_anon_bg_box(rect: Rect, color: Color, opacity: f32) -> LayoutBox {
  LayoutBox {
    margin_rect: rect,
    border_rect: rect,
    content_rect: rect,
    background: Some(color),
    background_rect: rect,
    background_radii: CornerRadii::zero(),
    border: Insets::zero(),
    border_colors: BorderColors::default(),
    border_styles: BorderStyles::default(),
    border_radius: CornerRadii::zero(),
    kind: BoxKind::Block,
    text_run: None,
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes::visible(),
    resize: Resize::None,
    text_overflow: None,
    transform: None,
    transform_origin: (0.0, 0.0),
    opacity: opacity.clamp(0.0, 1.0),
    pointer_events: PointerEvents::Auto,
    user_select: UserSelect::Auto,
    cursor: Cursor::Auto,
    z_index: None,
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    file_button: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  }
}

fn layout_inline_paragraph(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  container_w: f32,
  text_align: Option<&lui_models::common::css_enums::TextAlign>,
  ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
  layout_profile::count_inline_para(&mut ctx.profiler);
  // 1. Flatten the inline subtree into spans + recorded inline blocks (the elements with bg / decoration whose per-line
  //    bounds we'll need after shaping).
  let mut plan = ParagraphPlan::default();
  let mut collapse = ParagraphCollapseState::default();
  if let Some(ref pe) = node.marker {
    let pseudo = make_pseudo_node(pe);
    collect_paragraph_spans(&pseudo, &mut plan, ctx, &mut collapse, 1.0);
  }
  if let Some(ref pe) = node.before {
    let pseudo = make_pseudo_node(pe);
    collect_paragraph_spans(&pseudo, &mut plan, ctx, &mut collapse, 1.0);
  }
  for child in &node.children {
    collect_paragraph_spans(child, &mut plan, ctx, &mut collapse, 1.0);
  }
  if let Some(ref pe) = node.after {
    let pseudo = make_pseudo_node(pe);
    collect_paragraph_spans(&pseudo, &mut plan, ctx, &mut collapse, 1.0);
  }
  if plan.spans.is_empty() {
    return (Vec::new(), 0.0, 0.0);
  }

  // 2. Hand the paragraph to cosmic-text. Each span's `leaf_id` matches its index in `plan.spans`, which is what
  //    `inline_blocks.leaf_range` indexes into.
  let trim_edges = style_collapses_whitespace(&node.style);
  let paragraph_texts: Vec<&str> = plan
    .spans
    .iter()
    .enumerate()
    .map(|(i, sd)| {
      trim_collapsed_whitespace_edges(&sd.text, trim_edges && i == 0, trim_edges && i + 1 == plan.spans.len())
    })
    .collect();
  if paragraph_texts.iter().all(|text| text.is_empty()) {
    return (Vec::new(), 0.0, 0.0);
  }
  let paragraph_spans: Vec<ParagraphSpan<'_>> = plan
    .spans
    .iter()
    .zip(paragraph_texts.iter())
    .enumerate()
    .map(|(i, (sd, text))| ParagraphSpan {
      text,
      family: &sd.family,
      weight: sd.weight,
      style: sd.style_axis,
      size_px: sd.size_px,
      line_height_px: sd.line_height_px,
      color: sd.color,
      leaf_id: i as u32,
    })
    .collect();
  let para = match {
    layout_profile::count_para_shape(&mut ctx.profiler);
    ctx.text.ctx.shape_paragraph(
      &paragraph_spans,
      if style_wraps_text(&node.style) {
        Some(container_w)
      } else {
        None
      },
    )
  } {
    Some(p) => p,
    None => return (Vec::new(), 0.0, 0.0),
  };

  // 3. Per-line `text-align` shift.
  let line_align_dx: Vec<f32> = para
    .lines
    .iter()
    .map(|line| horizontal_align_offset(text_align, container_w, line.line_width))
    .collect();

  let mut boxes: Vec<LayoutBox> = Vec::new();

  // 4. Inline-element backgrounds (`<mark>` and friends). One anonymous Block per (line × span-in-element-range) so a
  //    span that wraps gets a background bar on each line it occupies.
  for inline in &plan.inline_blocks {
    let Some(bg) = inline.background else {
      continue;
    };
    for leaf_id in inline.leaf_range.0..inline.leaf_range.1 {
      let Some(segs) = para.leaf_segments.get(&leaf_id) else {
        continue;
      };
      for seg in segs {
        let line = &para.lines[seg.line_index];
        let dx = line_align_dx[seg.line_index];
        let r = Rect::new(
          origin_x + seg.x_start + dx,
          origin_y + line.top,
          seg.x_end - seg.x_start,
          line.height,
        );
        if r.w > 0.0 && r.h > 0.0 {
          boxes.push(make_anon_bg_box(r, bg, inline.opacity));
        }
      }
    }
  }

  // 5. Decoration bars. Underline below baseline, line-through through the x-height, overline at line top. Thickness
  //    scales with line ascent so big text gets a beefier line.
  for inline in &plan.inline_blocks {
    if inline.decorations.is_empty() {
      continue;
    }
    for leaf_id in inline.leaf_range.0..inline.leaf_range.1 {
      let Some(segs) = para.leaf_segments.get(&leaf_id) else {
        continue;
      };
      for seg in segs {
        let line = &para.lines[seg.line_index];
        let dx = line_align_dx[seg.line_index];
        let ascent = (line.baseline - line.top).max(1.0);
        let thickness = (ascent / 12.0).max(1.0);
        for deco in &inline.decorations {
          let y = match deco {
            TextDecorationLine::Underline => line.baseline + thickness,
            TextDecorationLine::LineThrough => line.baseline - ascent * 0.30,
            TextDecorationLine::Overline => line.top,
          };
          let r = Rect::new(
            origin_x + seg.x_start + dx,
            origin_y + y,
            seg.x_end - seg.x_start,
            thickness,
          );
          if r.w > 0.0 && r.h > 0.0 {
            boxes.push(make_anon_bg_box(r, inline.decoration_color, inline.opacity));
          }
        }
      }
    }
  }

  // 6. The single `BoxKind::Text` for the whole paragraph. Apply each line's text-align dx to its glyph slice.
  //    Per-glyph colour was baked in by `shape_paragraph` so the paint side just reads `g.color`.
  let mut positioned: Vec<PositionedGlyph> = Vec::with_capacity(para.glyphs.len());
  for (li, line) in para.lines.iter().enumerate() {
    let dx = line_align_dx[li];
    for g in &para.glyphs[line.glyph_range.0..line.glyph_range.1] {
      positioned.push(PositionedGlyph {
        x: g.x + dx,
        y: g.y,
        w: g.w,
        h: g.h,
        uv_min: g.uv_min,
        uv_max: g.uv_max,
        color: g.color,
      });
    }
  }
  let visible_text: String = paragraph_texts.iter().copied().collect();
  let mut line_ranges: Vec<(usize, usize)> = Vec::with_capacity(para.lines.len());
  let mut cursor = 0usize;
  for line in &para.lines {
    let count = line.glyph_range.1.saturating_sub(line.glyph_range.0);
    let start = cursor;
    cursor += count;
    line_ranges.push((start, cursor));
  }

  let run = ShapedRun {
    glyphs: positioned,
    glyph_chars: vec![], // IFC paragraph path: identity fallback
    lines: para
      .lines
      .iter()
      .zip(line_ranges.into_iter())
      .map(|(line, glyph_range)| ShapedLine {
        top: line.top,
        height: line.height,
        glyph_range,
      })
      .collect(),
    byte_boundaries: lui_text::utf8_boundaries(&visible_text),
    text: visible_text,
    width: para.width,
    height: para.height,
    ascent: para.first_line_ascent,
  };

  let r = Rect::new(origin_x, origin_y, para.width, para.height);
  let text_box = LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    background: None,
    background_rect: r,
    background_radii: CornerRadii::zero(),
    border: Insets::zero(),
    border_colors: BorderColors::default(),
    border_styles: BorderStyles::default(),
    border_radius: CornerRadii::zero(),
    kind: BoxKind::Text,
    text_run: Some(run),
    text_color: None,
    text_unselectable: false,
    text_decorations: Vec::new(),
    overflow: OverflowAxes::visible(),
    resize: Resize::None,
    text_overflow: None,
    transform: None,
    transform_origin: (0.0, 0.0),
    opacity: resolved_opacity(&node.style),
    pointer_events: PointerEvents::Auto,
    user_select: resolved_user_select(&node.style),
    cursor: resolved_cursor(&node.style),
    z_index: resolved_z_index(&node.style),
    image: None,
    background_image: None,
    first_line_color: None,
    first_letter_color: None,
    selection_bg: None,
    selection_fg: None,
    accent_color: None,
    lui: LuiProperties::default(),
    lui_popup: None,
    lui_color_picker: None,
    lui_calendar: None,
    file_button: None,
    children: Vec::new(),
    is_fixed: false,
    form_control: None,
  };
  boxes.push(text_box);

  (boxes, para.width, para.height)
}

/// CSS `text-align` → number of pixels to shift each child of the
/// line. `start`/`end` follow `dir: ltr` (no bidi yet). `justify`
/// falls through to `left`.
fn horizontal_align_offset(
  text_align: Option<&lui_models::common::css_enums::TextAlign>,
  container_w: f32,
  line_w: f32,
) -> f32 {
  use lui_models::common::css_enums::TextAlign as Ta;
  let free = (container_w - line_w).max(0.0);
  match text_align {
    Some(Ta::Center) => free * 0.5,
    Some(Ta::Right) | Some(Ta::End) => free,
    // Left, Start, Justify, None — flush to the inline-start edge.
    _ => 0.0,
  }
}

/// Recursively shift every rect on `b` and its descendants by `dx`
/// pixels along the x axis. Used to apply `text-align` after the
/// inline pass has positioned children at the line's left edge.
pub(crate) fn translate_box_x_in_place(b: &mut LayoutBox, dx: f32) {
  b.margin_rect.x += dx;
  b.border_rect.x += dx;
  b.content_rect.x += dx;
  b.background_rect.x += dx;
  if let Some(bgi) = b.background_image.as_mut() {
    for tile in &mut bgi.tiles {
      tile.x += dx;
    }
  }
  for child in &mut b.children {
    translate_box_x_in_place(child, dx);
  }
}

/// Recursively shift every rect on `b` and its descendants by `dy`
/// pixels along the y axis. Used by the inline pass to baseline-
/// align children after laying them all out at the line's top.
pub(crate) fn translate_box_y_in_place(b: &mut LayoutBox, dy: f32) {
  b.margin_rect.y += dy;
  b.border_rect.y += dy;
  b.content_rect.y += dy;
  b.background_rect.y += dy;
  if let Some(bgi) = b.background_image.as_mut() {
    for tile in &mut bgi.tiles {
      tile.y += dy;
    }
  }
  for child in &mut b.children {
    translate_box_y_in_place(child, dy);
  }
}
