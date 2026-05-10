//! Block layout: the primary formatting-context dispatcher. Walks the
//! cascaded node tree and produces a [`LayoutBox`] tree.  Delegates to
//! flex, grid, table, and inline paths as appropriate.

use lui_models::common::css_enums::{
    BoxSizing, CssColor, CssLength, Display, Position, Resize,
};
use lui_style::CascadedNode;
use lui_tree::Element;

use crate::{
    background::resolve_background_image,
    box_model::{
        clamp_axis, clamp_corner_radii, compute_background_box, is_auto_margin,
        resolve_border_widths, resolve_insets_margin, resolve_insets_padding,
    },
    color::{self, resolve_foreground, resolve_with_current},
    flex,
    form_controls::{
        compute_placeholder_run, compute_value_run, form_control_default_line_height,
        form_control_info, has_native_appearance, shape_input_text,
    },
    grid,
    incremental::{
        file_button_from_pseudo, lui_calendar_from_pseudo, lui_color_from_pseudo,
        lui_popup_from_pseudo, resolve_lui_properties,
    },
    inline::{
        all_children_inline_level, effective_children, layout_inline_block_children,
    },
    layout_profile,
    length,
    positioned::{
        apply_relative_position, effective_overflow, establishes_containing_block,
        is_out_of_flow_position, layout_out_of_flow_block, resolved_cursor,
        resolved_opacity, resolved_pointer_events, resolved_user_select, resolved_z_index,
    },
    svg,
    table,
    text_shaping::{empty_box, font_size_px, line_height_px, make_text_leaf},
    types::*,
    Ctx,
};

// ---------------------------------------------------------------------------
// Block overrides
// ---------------------------------------------------------------------------

/// Optional caller-supplied content-box overrides for the recursive
/// block layout. Used by the flex layer to drive an item to a
/// pre-computed main / cross extent without mutating its style.
///
/// When a field is `Some`, that axis is sized exactly to the value
/// (already in *content-box* pixels — `box-sizing` and `min-*` /
/// `max-*` clamping have already been applied by the caller).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BlockOverrides {
  pub width: Option<f32>,
  pub height: Option<f32>,
  pub ignore_style_width: bool,
  pub ignore_style_height: bool,
}

// ---------------------------------------------------------------------------
// Entry helpers
// ---------------------------------------------------------------------------

/// Lay out one node as a block at `(origin_x, origin_y)` inside the
/// given container, with optional content-box overrides. Used by the
/// flex layer to drive an item to a precomputed main / cross extent.
pub(crate) fn layout_block_at_with(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  container_w: f32,
  container_h: f32,
  overrides: BlockOverrides,
  ctx: &mut Ctx,
) -> LayoutBox {
  layout_block(
    node,
    origin_x,
    origin_y,
    container_w,
    container_h,
    Rect::new(origin_x, origin_y, container_w, container_h),
    overrides,
    ctx,
  )
}

// ---------------------------------------------------------------------------
// Main block layout
// ---------------------------------------------------------------------------

pub(crate) fn layout_block(
  node: &CascadedNode,
  origin_x: f32,
  origin_y: f32,
  container_w: f32,
  container_h: f32,
  containing_block: Rect,
  overrides: BlockOverrides,
  ctx: &mut Ctx,
) -> LayoutBox {
  layout_profile::count_block(&mut ctx.profiler);

  // `display: none` removes the element and its subtree from the
  // box tree entirely. Returning a zero-sized box means the parent
  // contributes nothing for this child — no painting, no
  // descendants, no advance.
  if matches!(node.style.display, Some(Display::None)) {
    return empty_box(origin_x, origin_y);
  }

  // Text leaves: shape with the first registered font (T3
  // simplification — proper font-family inheritance lands in T4).
  // If no font is registered, the run is `None` and the box has
  // zero size, matching pre-text behaviour.
  if let Element::Text(s) = &node.element {
    // Block-flow text leaf — uses the parent's content width as
    // the soft-wrap budget so paragraphs that are *direct* text
    // children of a block (rare, but legal) wrap rather than
    // overflow.
    let (box_, _w, _h, _ascent) = make_text_leaf(s, &node.style, origin_x, origin_y, Some(container_w), true, ctx);
    return box_;
  }

  // <img> replaced element: load the image and use its intrinsic
  // dimensions (or the HTML width/height attributes) as the
  // content size. CSS width/height override below as usual.
  let img_data = if let Element::Img(img) = &node.element {
    ctx.images.load_image(img)
  } else {
    None
  };
  let (html_img_width, html_img_height) = match &node.element {
    Element::Img(img) => (img.width.map(|v| v as f32), img.height.map(|v| v as f32)),
    // For <svg>, use the element's own width/height attrs as intrinsic size.
    Element::Svg(_) => {
      let (w, h) = svg::svg_intrinsic_css_size(match &node.element {
        Element::Svg(s) => s,
        _ => unreachable!(),
      });
      (w, h)
    }
    _ => (None, None),
  };
  // SVG serialisation string, used after the size is known.
  let svg_xml = if matches!(&node.element, Element::Svg(_)) {
    Some(svg::serialize_svg_node(node))
  } else {
    None
  };

  let style = &node.style;

  let mut margin = resolve_insets_margin(style, container_w, ctx);
  let mut border = resolve_border_widths(style, container_w, ctx);
  let mut padding = resolve_insets_padding(style, container_w, ctx);

  // Native-appearance form controls (checkbox, radio, range) suppress
  // author CSS border/padding — they draw their own visuals.
  if has_native_appearance(node) {
    border = Insets::zero();
    padding = Insets::zero();
  }

  let box_sizing = style.box_sizing.clone().unwrap_or(BoxSizing::ContentBox);

  // Inner width: caller-supplied override, then explicit `width`,
  // then fill the parent. Min/max clamping is applied to the
  // cascade-derived size; overrides are taken at face value (the
  // flex algorithm has already clamped).
  let frame_w = margin.horizontal() + border.horizontal() + padding.horizontal();
  let inner_width = match overrides.width {
    Some(w) => w,
    None => {
      let style_width = if overrides.ignore_style_width {
        None
      } else {
        style.width.as_ref()
      };
      let base = match length::resolve(style_width, container_w, ctx) {
        Some(specified) => match box_sizing {
          BoxSizing::ContentBox => specified,
          BoxSizing::BorderBox => (specified - border.horizontal() - padding.horizontal()).max(0.0),
        },
        None => {
          // Replaced elements (<img>) use HTML width first,
          // then decoded intrinsic width when no CSS width is
          // specified.
          if let Some(w) = html_img_width {
            w * ctx.scale
          } else if let Some(ref id) = img_data {
            id.width as f32 * ctx.scale
          } else if has_native_appearance(node) {
            14.0 * ctx.scale
          } else {
            (container_w - frame_w).max(0.0)
          }
        }
      };
      clamp_axis(
        base,
        style.min_width.as_ref(),
        style.max_width.as_ref(),
        container_w,
        border.horizontal() + padding.horizontal(),
        box_sizing.clone(),
        ctx,
      )
    }
  };

  // Auto horizontal margins on a block with an explicit width center
  // (or push) the block within its container, matching the standard
  // CSS `margin: 0 auto` idiom. Only kicks in when there's a
  // non-zero `width` and free space remains.
  //
  // Skipped when `overrides` is non-default — that signals the call
  // came from the flex layer, which handles its own auto-margin
  // redistribution before placing items. Running the block-level
  // pass on top would double-consume free space and push flex items
  // off the line.
  let from_flex = overrides.width.is_some() || overrides.height.is_some();
  let auto_left = is_auto_margin(&style.margin_left, &style.margin);
  let auto_right = is_auto_margin(&style.margin_right, &style.margin);
  let has_explicit_width = style.width.is_some();
  if !from_flex && has_explicit_width && (auto_left || auto_right) {
    let used = margin.horizontal() + border.horizontal() + padding.horizontal() + inner_width;
    let free = (container_w - used).max(0.0);
    match (auto_left, auto_right) {
      (true, true) => {
        let half = free * 0.5;
        margin.left += half;
        margin.right += half;
      }
      (true, false) => margin.left += free,
      (false, true) => margin.right += free,
      (false, false) => {}
    }
  }

  // Lay out children inside the content box, dispatching on display.
  let content_x = origin_x + margin.left + border.left + padding.left;
  let content_y_top = origin_y + margin.top + border.top + padding.top;

  // Pre-resolve an explicit height (used for `align-items: stretch`
  // and as the override target). Caller-supplied override wins; then
  // an explicit `height` style; otherwise unknown until children lay
  // out. Min/max clamping happens after content height is known
  // (when no explicit height) so it can extend a too-short block.
  let inner_height_explicit = match overrides.height {
    Some(h) => Some(h),
    None => {
      let style_height = if overrides.ignore_style_height {
        None
      } else {
        style.height.as_ref()
      };
      // Replaced elements use intrinsic height when no CSS
      // height is specified. HTML height wins over the decoded
      // intrinsic height, but does not force a CPU resize.
      let css_h = length::resolve(style_height, container_h, ctx);
      let effective_h = css_h
        .or_else(|| html_img_height.map(|h| h * ctx.scale))
        .or_else(|| img_data.as_ref().map(|id| id.height as f32 * ctx.scale))
        .or_else(|| if has_native_appearance(node) { Some(14.0 * ctx.scale) } else { None });
      effective_h.map(|specified| {
        let raw = match box_sizing {
          BoxSizing::ContentBox => specified,
          BoxSizing::BorderBox => (specified - border.vertical() - padding.vertical()).max(0.0),
        };
        clamp_axis(
          raw,
          style.min_height.as_ref(),
          style.max_height.as_ref(),
          container_h,
          border.vertical() + padding.vertical(),
          box_sizing.clone(),
          ctx,
        )
      })
    }
  };

  let display = style.display.clone().unwrap_or(Display::Block);
  let child_containing_block = if establishes_containing_block(style) {
    Rect::new(
      origin_x + margin.left + border.left,
      origin_y + margin.top + border.top,
      padding.horizontal() + inner_width,
      padding.vertical() + inner_height_explicit.unwrap_or(container_h),
    )
  } else {
    containing_block
  };
  // <svg> is treated as a replaced element: its children (<path>, <circle>, …)
  // were already serialised by serialize_svg_node() and will be rasterised;
  // they must not be recursively laid out as block/inline content.
  let (children, content_h_from_children) = if matches!(&node.element, Element::Svg(_)) {
    (Vec::new(), 0.0_f32)
  } else {
    match display {
      Display::Flex | Display::InlineFlex => {
        layout_profile::count_flex(&mut ctx.profiler);
        let (kids, _content_w_used, content_h_used) = flex::layout_flex_children(
          node,
          style,
          content_x,
          content_y_top,
          inner_width,
          inner_height_explicit,
          ctx,
        );
        (kids, content_h_used)
      }
      Display::Grid | Display::InlineGrid => {
        layout_profile::count_grid(&mut ctx.profiler);
        let (kids, _content_w_used, content_h_used) = grid::layout_grid_children(
          node,
          style,
          content_x,
          content_y_top,
          inner_width,
          inner_height_explicit,
          ctx,
        );
        (kids, content_h_used)
      }
      Display::Table => {
        layout_profile::count_table(&mut ctx.profiler);
        let (kids, _content_w_used, content_h_used) = table::layout_table_children(
          node,
          style,
          content_x,
          content_y_top,
          inner_width,
          inner_height_explicit,
          ctx,
        );
        (kids, content_h_used)
      }
      _ => {
        // Inline formatting context: when every child of this
        // block is inline-level (text, <strong>, <em>, …), pack
        // them onto a single line box at the parent's content
        // origin. Otherwise fall back to the block flow that
        // stacks children vertically.
        if all_children_inline_level(node) {
          let (kids, _w_used, h_used) = layout_inline_block_children(node, content_x, content_y_top, inner_width, ctx);
          (kids, h_used)
        } else {
          let effective = effective_children(node);
          let mut children = Vec::with_capacity(effective.len());
          let mut cursor = 0.0_f32;
          for child in &effective {
            let child_position = child.style.position.clone().unwrap_or(Position::Static);
            let mut child_box = if is_out_of_flow_position(child_position.clone()) {
              layout_out_of_flow_block(
                child,
                content_x,
                content_y_top + cursor,
                inner_width,
                container_h,
                child_containing_block,
                ctx,
              )
            } else {
              layout_block(
                child,
                content_x,
                content_y_top + cursor,
                inner_width,
                container_h,
                child_containing_block,
                BlockOverrides::default(),
                ctx,
              )
            };
            if matches!(child_position, Position::Relative | Position::Sticky) {
              apply_relative_position(&mut child_box, &child.style, inner_width, container_h, ctx);
            }
            if !is_out_of_flow_position(child_position) {
              cursor += child_box.margin_rect.h;
            }
            children.push(child_box);
          }
          (children, cursor)
        }
      }
    } // end of else branch for non-SVG children
  }; // end of (children, content_h_from_children)

  // Final inner height: explicit / override wins; otherwise content
  // size, then clamped by min/max (so a too-short content can be
  // extended by `min-height` and a too-tall content by `max-height`).
  let mut inner_height = match inner_height_explicit {
    Some(h) => h,
    None => clamp_axis(
      content_h_from_children,
      style.min_height.as_ref(),
      style.max_height.as_ref(),
      container_h,
      border.vertical() + padding.vertical(),
      box_sizing.clone(),
      ctx,
    ),
  };

  // Empty form controls (`<input>`, `<textarea>`, `<select>`,
  // `<button>` with no children) collapse to `inner_height = 0`
  // because they have nothing to measure. Browsers give them a
  // default content height of one line of the cascaded font, so
  // the placeholder text run we attach below has room to render
  // and the input visually matches typed content height.
  if inner_height_explicit.is_none() && form_control_default_line_height(node) {
    let font_size = font_size_px(style).unwrap_or(16.0);
    let line_h = line_height_px(style, font_size) * ctx.scale;
    if inner_height < line_h {
      inner_height = line_h;
    }
  }

  // Compose the rects.
  let border_rect = Rect::new(
    origin_x + margin.left,
    origin_y + margin.top,
    border.horizontal() + padding.horizontal() + inner_width,
    border.vertical() + padding.vertical() + inner_height,
  );
  let content_rect = Rect::new(content_x, content_y_top, inner_width, inner_height);
  let margin_rect = Rect::new(
    origin_x,
    origin_y,
    margin.horizontal() + border_rect.w,
    margin.vertical() + border_rect.h,
  );

  let fg = resolve_foreground(style.color.as_ref(), color::BLACK);
  let background = style.background_color.as_ref().and_then(|c| resolve_with_current(c, fg));
  let accent_color = style.accent_color.as_ref().and_then(|c| resolve_with_current(c, fg));
  let lui = resolve_lui_properties(&style.custom_properties, fg);
  let resolve_border = |c: &CssColor| resolve_with_current(c, fg);
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
    // Vertical resolves against the box height when known, else
    // viewport height (Ctx). When the v field is unset, fall back
    // to the same value as h (CSS default).
    let v_px = match v {
      Some(_) => length::resolve(v, container_h, ctx).unwrap_or(0.0).max(0.0),
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

  let background_image = resolve_background_image(style, background_rect, ctx.images);

  // For <svg> nodes, rasterise the serialised SVG at the final
  // content-box pixel dimensions. We prefer img_data (if somehow
  // set) over svg rasterization so normal <img> is unaffected.
  let svg_img_data = svg_xml.and_then(|xml| {
    // content_rect is already in physical pixels (CSS px × ctx.scale).
    let w = content_rect.w.round() as u32;
    let h = content_rect.h.round() as u32;
    svg::make_svg_image_data(&xml, w.max(1), h.max(1))
  });
  let effective_image = img_data.or(svg_img_data);

  // For form controls without a value/content, shape the
  // `placeholder` attribute as the box's text run so the empty
  // input shows the hint text (HTML's `:placeholder-shown`
  // behaviour). Painted with `color` reduced to ~50% opacity, the
  // browser default `::placeholder` styling.
  // Value takes priority: if the field has a non-empty value, shape
  // that instead of the placeholder.

  let (value_run, value_color) = compute_value_run(node, content_rect, ctx);
  let (placeholder_run, placeholder_color) = if value_run.is_some() {
    (value_run, value_color)
  } else {
    compute_placeholder_run(node, content_rect, ctx)
  };

  let fc = form_control_info(node);
  let text_color = placeholder_color.or_else(|| if fc.is_some() { Some(fg) } else { None });

  let mut lb = LayoutBox {
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
    overflow: effective_overflow(style),
    resize: style.resize.unwrap_or(Resize::None),
    text_overflow: style.text_overflow,
    opacity: resolved_opacity(style),
    pointer_events: resolved_pointer_events(style),
    user_select: resolved_user_select(style),
    cursor: resolved_cursor(style),
    z_index: resolved_z_index(style),
    image: effective_image,
    background_image,
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
  };

  if matches!(lb.form_control.as_ref(), Some(FormControlInfo { kind: FormControlKind::File { .. } })) {
    let btn_label = ctx.locale.file_browse_label();
    let (btn_run, _) = shape_input_text(btn_label, false, content_rect, node, ctx);
    let fb = lb.file_button.get_or_insert_with(FileButtonStyle::default);
    fb.text_run = btn_run;
  }

  lb
}
