//! Flex formatting context — CSS-Flexbox-1 §9.
//!
//! Supports: flex-direction, flex-wrap, flex-grow/shrink/basis,
//! justify-content, align-items, align-self, align-content,
//! order, gap/row-gap/column-gap, auto margins, min/max clamping.

use bumpalo::Bump;
use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_tree::{Overflow, ScrollInfo};

use crate::{
  box_tree::{BoxKind, LayoutBox},
  context::LayoutContext,
  geometry::Point,
  sides, sizes,
  text::TextContext,
};

const EPS: f32 = 0.001;

// ── CSS value extraction helpers ──────────────────────────────────────

fn css_str<'a>(v: Option<&'a lui_core::CssValue>) -> &'a str {
  match v {
    Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(),
    _ => "",
  }
}

fn css_f32(v: Option<&lui_core::CssValue>) -> Option<f32> {
  match v {
    Some(lui_core::CssValue::Number(n)) => Some(*n as f32),
    _ => None,
  }
}

fn css_i32(v: Option<&lui_core::CssValue>) -> Option<i32> {
  match v {
    Some(lui_core::CssValue::Number(n)) => Some(*n as i32),
    _ => None,
  }
}

fn is_auto(v: Option<&lui_core::CssValue>) -> bool {
  matches!(css_str(v), "auto")
}

fn is_whitespace_only_anon(b: &LayoutBox) -> bool {
  if !matches!(b.kind, BoxKind::AnonymousBlock) {
    return false;
  }
  b.children.iter().all(|child| {
    if let lui_core::HtmlElement::Text(ref content) = *child.node.element() {
      content.chars().all(|c| c.is_ascii_whitespace())
    } else {
      false
    }
  })
}

// ── Public entry point ────────────────────────────────────────────────

pub fn layout_flex<'a>(
  b: &mut LayoutBox<'a>,
  ctx: &LayoutContext,
  pos: Point,
  text_ctx: &mut TextContext,
  rects: &mut Vec<(&'a HtmlNode, Rect)>,
  cache: &crate::incremental::CacheView,
  bump: &'a Bump,
) {
  let margin = sides::resolve_margin_against(b.style, ctx.containing_width);
  let border = sides::resolve_border(b.style);
  let padding = sides::resolve_padding_against(b.style, ctx.containing_width);

  b.margin = margin.edges;
  b.border = border;
  b.padding = padding;

  let available = ctx.containing_width - margin.edges.horizontal() - border.horizontal() - padding.horizontal();
  let w = sizes::resolve_length(b.style.width, ctx.containing_width).unwrap_or(available.max(0.0));
  let inner_width = w.min(available.max(0.0));
  b.content.width = inner_width;

  let has_auto_left = margin.auto_mask & (1 << 3) != 0;
  let has_auto_right = margin.auto_mask & (1 << 1) != 0;
  let has_explicit_w = sizes::resolve_length(b.style.width, ctx.containing_width).is_some();
  if has_explicit_w && has_auto_left && has_auto_right {
    let free = (ctx.containing_width - b.content.width - border.horizontal() - padding.horizontal()).max(0.0);
    b.margin.left = free / 2.0;
    b.margin.right = free / 2.0;
  } else if has_explicit_w && has_auto_left {
    let free =
      (ctx.containing_width - b.content.width - border.horizontal() - padding.horizontal() - margin.edges.right)
        .max(0.0);
    b.margin.left = free;
  } else if has_explicit_w && has_auto_right {
    let free =
      (ctx.containing_width - b.content.width - border.horizontal() - padding.horizontal() - margin.edges.left)
        .max(0.0);
    b.margin.right = free;
  }

  b.content.x = pos.x + b.margin.left + border.left + padding.left;
  b.content.y = pos.y + margin.edges.top + border.top + padding.top;

  let inner_height = sizes::resolve_length(b.style.height, ctx.containing_height);

  let direction = css_str(b.style.flex_direction);
  let wrap_str = css_str(b.style.flex_wrap);
  let justify = css_str(b.style.justify_content);
  let align_items_str = css_str(b.style.align_items);
  let align_content_str = css_str(b.style.align_content);

  let is_row = !matches!(direction, "column" | "column-reverse");
  let is_dir_reverse = matches!(direction, "row-reverse" | "column-reverse");
  let is_wrap = matches!(wrap_str, "wrap" | "wrap-reverse");
  let is_wrap_reverse = matches!(wrap_str, "wrap-reverse");

  let main_axis_size = if is_row { Some(inner_width) } else { inner_height };
  let cross_axis_size = if is_row { inner_height } else { Some(inner_width) };

  let gap_main = if is_row {
    sizes::resolve_length(b.style.column_gap, inner_width)
  } else {
    sizes::resolve_length(b.style.row_gap, inner_width)
  }
  .unwrap_or(0.0);
  let gap_cross = if is_row {
    sizes::resolve_length(b.style.row_gap, inner_width)
  } else {
    sizes::resolve_length(b.style.column_gap, inner_width)
  }
  .unwrap_or(0.0);

  // Phase 1: build flex items (filter out-of-flow children and whitespace-only anon items)
  let mut items: Vec<FlexItem<'a>> = Vec::with_capacity(b.children.len());
  let mut out_of_flow: Vec<LayoutBox<'a>> = Vec::new();
  let taken_children = std::mem::replace(&mut b.children, bumpalo::collections::Vec::new_in(bump));
  for (idx, child) in taken_children.into_iter().enumerate() {
    if css_str(child.style.display) == "none" {
      continue;
    }
    if crate::positioned::is_out_of_flow(child.style) {
      out_of_flow.push(child);
      continue;
    }
    if is_whitespace_only_anon(&child) {
      continue;
    }
    let item = build_item(child, idx, is_row, main_axis_size, cross_axis_size, ctx, text_ctx);
    items.push(item);
  }
  if items.is_empty() {
    b.content.height = inner_height.unwrap_or(0.0);
    return;
  }
  items.sort_by_key(|i| (i.order, i.source_index));

  // Phase 2: collect into lines
  let mut lines: Vec<Vec<usize>> = Vec::new();
  if !is_wrap || main_axis_size.is_none() {
    lines.push((0..items.len()).collect());
  } else {
    let main_size = main_axis_size.unwrap();
    let mut current: Vec<usize> = Vec::new();
    let mut running = 0.0_f32;
    for (i, item) in items.iter().enumerate() {
      let outer = item.hypothetical_outer_main();
      let prospective = if current.is_empty() {
        outer
      } else {
        running + gap_main + outer
      };
      if !current.is_empty() && prospective > main_size + EPS {
        lines.push(std::mem::take(&mut current));
        running = outer;
        current.push(i);
      } else {
        running = prospective;
        current.push(i);
      }
    }
    if !current.is_empty() {
      lines.push(current);
    }
  }

  // Phase 3: resolve flexible lengths
  if let Some(main_size) = main_axis_size {
    for line in &lines {
      resolve_flexible_lengths(&mut items, line, main_size, gap_main);
    }
  }

  // Phase 4: per-item layout at resolved main size
  let child_ctx = LayoutContext {
    containing_width: inner_width,
    containing_height: inner_height.unwrap_or(ctx.containing_height),
    ..*ctx
  };
  for item in &mut items {
    let main = if item.collapsed {
      item.hypothetical_main
    } else {
      item.resolved_main
    };
    let (item_w, item_h) = if is_row {
      (Some(main), None)
    } else {
      let align = resolve_align_self(item.align_self, align_items_str);
      let cross_w = if align != "stretch" && !item.has_explicit_cross_size {
        let border_box_w = measure_max_content_width(&item.box_, text_ctx);
        let content_w = (border_box_w - item.frame_cross).max(0.0);
        Some(content_w)
      } else {
        None
      };
      (cross_w, Some(main))
    };
    layout_flex_item(
      &mut item.box_,
      &child_ctx,
      is_row,
      item_w,
      item_h,
      text_ctx,
      rects,
      cache,
      bump,
    );
    item.measured_cross_inner = if is_row {
      item.box_.content.height
    } else {
      item.box_.content.width
    };
    if item.collapsed {
      if is_row {
        item.box_.content.width = 0.0;
      } else {
        item.box_.content.height = 0.0;
      }
    }
  }

  // Phase 4.5: compute baselines after layout
  if is_row {
    for item in &mut items {
      item.first_baseline = find_first_baseline(&item.box_).map(|bl| bl + item.box_.padding.top + item.box_.border.top);
    }
  }

  // Phase 5: line cross sizes (including baseline contributions)
  let mut line_baselines: Vec<f32> = Vec::new();
  let mut line_cross_sizes: Vec<f32> = Vec::with_capacity(lines.len());
  for line in &lines {
    let mut max_cross = 0.0_f32;
    let mut max_above_bl = 0.0_f32;
    let mut max_below_bl = 0.0_f32;
    let mut has_baseline = false;
    for &i in line {
      max_cross = max_cross.max(items[i].outer_cross());
      let align = resolve_align_self(items[i].align_self, align_items_str);
      if align == "baseline" {
        if let Some(bl) = items[i].first_baseline {
          has_baseline = true;
          let above = bl + items[i].margin_cross_start;
          let below = items[i].outer_cross() - above;
          max_above_bl = max_above_bl.max(above);
          max_below_bl = max_below_bl.max(below);
        }
      }
    }
    if has_baseline {
      max_cross = max_cross.max(max_above_bl + max_below_bl);
    }
    line_baselines.push(max_above_bl);
    line_cross_sizes.push(max_cross);
  }

  let single_line = lines.len() == 1;
  let mut cross_start_offset = 0.0_f32;
  let mut cross_between = 0.0_f32;

  if single_line {
    if let Some(c) = cross_axis_size {
      line_cross_sizes[0] = c;
    }
  } else {
    // Phase 6: align-content for multi-line
    let total_lines_cross: f32 = line_cross_sizes.iter().sum::<f32>() + gap_cross * (lines.len() as f32 - 1.0).max(0.0);
    let cross_box = cross_axis_size.unwrap_or(total_lines_cross);
    let lines_free = (cross_box - total_lines_cross).max(0.0);
    let (start, between, stretch) = align_content_distribution(
      align_content_str,
      lines_free,
      lines.len() as f32,
      cross_axis_size.is_some(),
    );
    cross_start_offset = start;
    cross_between = between;
    if stretch > 0.0 {
      for s in &mut line_cross_sizes {
        *s += stretch;
      }
    }
  }

  let cross_box = cross_axis_size
    .unwrap_or_else(|| line_cross_sizes.iter().sum::<f32>() + gap_cross * (lines.len() as f32 - 1.0).max(0.0));

  let mut line_cross_positions: Vec<f32> = Vec::with_capacity(lines.len());
  {
    let mut cursor = cross_start_offset;
    for s in &line_cross_sizes {
      line_cross_positions.push(cursor);
      cursor += s + cross_between + gap_cross;
    }
  }
  if is_wrap_reverse {
    for (idx, p) in line_cross_positions.iter_mut().enumerate() {
      *p = (cross_box - *p - line_cross_sizes[idx]).max(0.0);
    }
  }

  // Phase 7: main-axis distribution + cross alignment + translation
  for (line_idx, line) in lines.iter().enumerate() {
    let line_cross_size = line_cross_sizes[line_idx];
    let line_cross_pos = line_cross_positions[line_idx];

    let total_main: f32 =
      line.iter().map(|&i| items[i].outer_main()).sum::<f32>() + gap_main * (line.len() as f32 - 1.0).max(0.0);
    let mut free_main = main_axis_size.map(|m| (m - total_main).max(0.0)).unwrap_or(0.0);

    // Auto main margins absorb free space first
    let auto_count: usize = line
      .iter()
      .map(|&i| items[i].auto_main_start as usize + items[i].auto_main_end as usize)
      .sum();
    let auto_each = if auto_count > 0 && free_main > 0.0 {
      let each = free_main / auto_count as f32;
      free_main = 0.0;
      each
    } else {
      0.0
    };

    let (start_main, between_extra) = distribution(justify, free_main, line.len() as f32);
    let mut cursor_main = start_main;

    for &i in line {
      let item = &mut items[i];
      let outer_main = item.outer_main();
      let auto_pre = if item.auto_main_start { auto_each } else { 0.0 };
      let auto_post = if item.auto_main_end { auto_each } else { 0.0 };

      let item_main_pos = cursor_main + auto_pre;
      cursor_main = item_main_pos + outer_main + auto_post + gap_main + between_extra;

      // Cross alignment
      let mut item_cross_pos = line_cross_pos;
      let line_free_cross = (line_cross_size - item.outer_cross()).max(0.0);
      let align = resolve_align_self(item.align_self, align_items_str);
      let stretched =
        align == "stretch" && !item.has_explicit_cross_size && !item.auto_cross_start && !item.auto_cross_end;

      if item.auto_cross_start || item.auto_cross_end {
        let ac = item.auto_cross_start as u32 + item.auto_cross_end as u32;
        if ac > 0 && line_free_cross > 0.0 {
          if item.auto_cross_start {
            item_cross_pos += line_free_cross / ac as f32;
          }
        }
      } else if align == "baseline" && is_row {
        if let Some(bl) = item.first_baseline {
          let line_bl = line_baselines[line_idx];
          item_cross_pos += line_bl - bl - item.margin_cross_start;
        }
      } else if !stretched {
        match align {
          "flex-end" | "end" => item_cross_pos += line_free_cross,
          "center" => item_cross_pos += line_free_cross * 0.5,
          _ => {}
        }
      }

      // Stretch: re-layout with line cross size
      if stretched {
        let stretch_target = (line_cross_size - item.margin_cross_outer_known()).max(0.0);
        let already_correct = (item.measured_cross_inner - stretch_target).abs() < 0.5;
        if !already_correct {
          let (sw, sh) = if is_row {
            (Some(item.resolved_main), Some(stretch_target))
          } else {
            (Some(stretch_target), Some(item.resolved_main))
          };
          layout_flex_item(&mut item.box_, &child_ctx, is_row, sw, sh, text_ctx, rects, cache, bump);
        }
      }

      // Translate to final position
      let (fx, fy) = if is_row {
        (b.content.x + item_main_pos, b.content.y + item_cross_pos)
      } else {
        (b.content.x + item_cross_pos, b.content.y + item_main_pos)
      };
      translate_box(&mut item.box_, fx, fy);
    }
  }

  // Phase 8: row-reverse / column-reverse
  if is_dir_reverse {
    if let Some(main_size) = main_axis_size {
      for item in &mut items {
        if is_row {
          let cur =
            item.box_.content.x - item.box_.padding.left - item.box_.border.left - item.box_.margin.left - b.content.x;
          let new_x = b.content.x + (main_size - cur - item.box_.outer_width()).max(0.0);
          let dx =
            new_x - (item.box_.content.x - item.box_.padding.left - item.box_.border.left - item.box_.margin.left);
          translate_box_delta(&mut item.box_, dx, 0.0);
        } else {
          let cur =
            item.box_.content.y - item.box_.padding.top - item.box_.border.top - item.box_.margin.top - b.content.y;
          let new_y = b.content.y + (main_size - cur - item.box_.outer_height()).max(0.0);
          let dy = new_y - (item.box_.content.y - item.box_.padding.top - item.box_.border.top - item.box_.margin.top);
          translate_box_delta(&mut item.box_, 0.0, dy);
        }
      }
    }
  }

  // Phase 9: restore source order
  items.sort_by_key(|i| i.source_index);
  let used_main = lines
    .iter()
    .map(|line| {
      line.iter().map(|&i| items[i].outer_main()).sum::<f32>() + gap_main * (line.len() as f32 - 1.0).max(0.0)
    })
    .fold(0.0_f32, f32::max);
  let used_cross = line_cross_sizes.iter().sum::<f32>() + gap_cross * (lines.len() as f32 - 1.0).max(0.0);

  let (content_w, content_h) = if is_row {
    (used_main, used_cross)
  } else {
    (used_cross, used_main)
  };
  b.content.width = sizes::resolve_length(b.style.width, ctx.containing_width).unwrap_or_else(|| {
    if matches!(b.kind, BoxKind::InlineFlex) {
      content_w
    } else {
      inner_width
    }
  });
  b.content.height = inner_height.unwrap_or(content_h);

  // Layout out-of-flow children against the flex container's padding box
  let containing_block = Rect::new(
    b.content.x - padding.left,
    b.content.y - padding.top,
    b.content.width + padding.horizontal(),
    b.content.height + padding.vertical(),
  );
  b.children = bumpalo::collections::Vec::from_iter_in(items.into_iter().map(|i| i.box_), bump);
  for child in b.children.iter_mut() {
    crate::positioned::apply_relative_offset(child, b.content.width, b.content.height);
  }
  for mut oof in out_of_flow {
    let static_pos = Point::new(b.content.x, b.content.y);
    crate::positioned::layout_out_of_flow(
      &mut oof,
      ctx,
      static_pos,
      containing_block,
      text_ctx,
      rects,
      cache,
      bump,
    );
    rects.push((oof.node, oof.content));
    b.children.push(oof);
  }

  finalize_flex_item_scroll(b);
}

// ── FlexItem ──────────────────────────────────────────────────────────

struct FlexItem<'a> {
  box_: LayoutBox<'a>,
  source_index: usize,
  order: i32,
  flex_grow: f32,
  flex_shrink: f32,
  base_size: f32,
  hypothetical_main: f32,
  resolved_main: f32,
  main_min: f32,
  main_max: f32,
  frame_main: f32,
  frame_cross: f32,
  margin_main_start: f32,
  margin_main_end: f32,
  margin_cross_start: f32,
  margin_cross_end: f32,
  auto_main_start: bool,
  auto_main_end: bool,
  auto_cross_start: bool,
  auto_cross_end: bool,
  has_explicit_cross_size: bool,
  align_self: &'a str,
  measured_cross_inner: f32,
  collapsed: bool,
  first_baseline: Option<f32>,
}

impl FlexItem<'_> {
  fn outer_main(&self) -> f32 {
    if self.collapsed {
      return 0.0;
    }
    self.resolved_main + self.frame_main + self.margin_main_start + self.margin_main_end
  }
  fn outer_cross(&self) -> f32 {
    self.measured_cross_inner + self.frame_cross + self.margin_cross_start + self.margin_cross_end
  }
  fn margin_cross_outer_known(&self) -> f32 {
    self.frame_cross + self.margin_cross_start + self.margin_cross_end
  }
  fn hypothetical_outer_main(&self) -> f32 {
    if self.collapsed {
      return 0.0;
    }
    self.hypothetical_main + self.frame_main + self.margin_main_start + self.margin_main_end
  }
}

fn build_item<'a>(
  child: LayoutBox<'a>,
  source_index: usize,
  is_row: bool,
  parent_main: Option<f32>,
  parent_cross: Option<f32>,
  ctx: &LayoutContext,
  text_ctx: &mut TextContext,
) -> FlexItem<'a> {
  let style = child.style;
  let order = css_i32(style.order).unwrap_or(0);
  let flex_grow = css_f32(style.flex_grow).unwrap_or(0.0).max(0.0);
  let flex_shrink = css_f32(style.flex_shrink).unwrap_or(1.0).max(0.0);

  let containing = parent_main.unwrap_or(ctx.containing_width);
  let margin = sides::resolve_margin_against(style, containing);
  let border = sides::resolve_border(style);
  let padding = sides::resolve_padding_against(style, containing);

  let frame_h = border.horizontal() + padding.horizontal();
  let frame_v = border.vertical() + padding.vertical();
  let (frame_main, frame_cross) = if is_row { (frame_h, frame_v) } else { (frame_v, frame_h) };

  let (margin_main_start, margin_main_end) = if is_row {
    (margin.edges.left, margin.edges.right)
  } else {
    (margin.edges.top, margin.edges.bottom)
  };
  let (margin_cross_start, margin_cross_end) = if is_row {
    (margin.edges.top, margin.edges.bottom)
  } else {
    (margin.edges.left, margin.edges.right)
  };
  // auto_mask bits: 0=top, 1=right, 2=bottom, 3=left
  let auto_top = margin.auto_mask & (1 << 0) != 0;
  let auto_right = margin.auto_mask & (1 << 1) != 0;
  let auto_bottom = margin.auto_mask & (1 << 2) != 0;
  let auto_left = margin.auto_mask & (1 << 3) != 0;
  let (auto_main_start, auto_main_end) = if is_row {
    (auto_left, auto_right)
  } else {
    (auto_top, auto_bottom)
  };
  let (auto_cross_start, auto_cross_end) = if is_row {
    (auto_top, auto_bottom)
  } else {
    (auto_left, auto_right)
  };

  // Flex basis — measure_max/min_content_width returns border-box,
  // but base_size must be content-box (frame is added separately).
  let main_prop = if is_row { style.width } else { style.height };
  let basis_keyword = css_str(style.flex_basis);
  let basis = match basis_keyword {
    "content" | "max-content" => Some(if is_row {
      (measure_max_content_width(&child, text_ctx) - frame_main).max(0.0)
    } else {
      (measure_max_content_height(&child, text_ctx) - frame_main).max(0.0)
    }),
    "min-content" => Some(if is_row {
      (measure_min_content_width(&child, text_ctx) - frame_main).max(0.0)
    } else {
      (measure_min_content_height(&child, text_ctx) - frame_main).max(0.0)
    }),
    _ => sizes::resolve_length(style.flex_basis, containing).or_else(|| sizes::resolve_length(main_prop, containing)),
  };
  let base_size = basis
    .unwrap_or_else(|| {
      if is_row {
        (measure_max_content_width(&child, text_ctx) - frame_main).max(0.0)
      } else {
        (measure_max_content_height(&child, text_ctx) - frame_main).max(0.0)
      }
    })
    .max(0.0);

  let (min_prop, max_prop) = if is_row {
    (style.min_width, style.max_width)
  } else {
    (style.min_height, style.max_height)
  };
  let main_min = sizes::resolve_length(min_prop, containing).unwrap_or_else(|| {
    let content_min = if is_row {
      (measure_min_content_width(&child, text_ctx) - frame_main).max(0.0)
    } else {
      (measure_min_content_height(&child, text_ctx) - frame_main).max(0.0)
    };
    content_min.min(base_size)
  });
  let main_max = sizes::resolve_length(max_prop, containing).unwrap_or(f32::INFINITY);

  let hypothetical_main = base_size.clamp(main_min, main_max);

  let cross_prop = if is_row { style.height } else { style.width };
  let is_pct_cross = matches!(cross_prop, Some(lui_core::CssValue::Percentage(_)));
  let has_explicit_cross_size =
    cross_prop.is_some() && !is_auto(cross_prop) && !(is_pct_cross && parent_cross.is_none());

  let align_self_str = css_str(style.align_self);
  let collapsed = css_str(style.visibility) == "collapse";

  FlexItem {
    box_: child,
    source_index,
    order,
    flex_grow,
    flex_shrink,
    base_size,
    hypothetical_main,
    resolved_main: hypothetical_main,
    main_min,
    main_max,
    frame_main,
    frame_cross,
    margin_main_start,
    margin_main_end,
    margin_cross_start,
    margin_cross_end,
    auto_main_start,
    auto_main_end,
    auto_cross_start,
    auto_cross_end,
    has_explicit_cross_size,
    align_self: align_self_str,
    measured_cross_inner: 0.0,
    collapsed,
    first_baseline: None,
  }
}

// ── Per-item layout ───────────────────────────────────────────────────

fn layout_flex_item<'a>(
  b: &mut LayoutBox<'a>,
  ctx: &LayoutContext,
  _is_row: bool,
  override_w: Option<f32>,
  override_h: Option<f32>,
  text_ctx: &mut TextContext,
  rects: &mut Vec<(&'a HtmlNode, Rect)>,
  cache: &crate::incremental::CacheView,
  bump: &'a Bump,
) {
  let margin = sides::resolve_margin_against(b.style, ctx.containing_width);
  let border = sides::resolve_border(b.style);
  let padding = sides::resolve_padding_against(b.style, ctx.containing_width);
  b.margin = margin.edges;
  b.border = border;
  b.padding = padding;

  let available = ctx.containing_width - margin.edges.horizontal() - border.horizontal() - padding.horizontal();
  let w = override_w
    .or_else(|| sizes::resolve_length(b.style.width, ctx.containing_width))
    .unwrap_or(available.max(0.0));
  b.content.width = w;
  b.content.x = margin.edges.left + border.left + padding.left;
  b.content.y = margin.edges.top + border.top + padding.top;

  let resolved_h = override_h
    .or_else(|| sizes::resolve_length(b.style.height, ctx.containing_height));
  let child_ctx = LayoutContext {
    containing_width: b.content.width,
    containing_height: resolved_h.unwrap_or(ctx.containing_height),
    ..*ctx
  };
  let origin = Point::new(b.content.x, b.content.y);

  // For flex/grid items that are themselves containers, set their width
  // from the override before dispatching to their own layout algorithm.
  if let Some(w) = override_w {
    b.content.width = w;
  }

  match b.kind {
    BoxKind::FlexContainer | BoxKind::InlineFlex => {
      let outer = override_w.map(|w| w + b.border.horizontal() + b.padding.horizontal() + b.margin.horizontal());
      let flex_ctx = LayoutContext {
        containing_width: outer.unwrap_or(child_ctx.containing_width),
        ..child_ctx
      };
      crate::flex::layout_flex(b, &flex_ctx, origin, text_ctx, rects, cache, bump);
      if let Some(w) = override_w {
        b.content.width = w;
      }
      if let Some(h) = override_h {
        b.content.height = h;
      }
    }
    BoxKind::GridContainer | BoxKind::InlineGrid => {
      let outer = override_w.map(|w| w + b.border.horizontal() + b.padding.horizontal() + b.margin.horizontal());
      let grid_ctx = LayoutContext {
        containing_width: outer.unwrap_or(child_ctx.containing_width),
        ..child_ctx
      };
      crate::grid::layout_grid(b, &grid_ctx, origin, text_ctx, rects, cache, bump);
      if let Some(w) = override_w {
        b.content.width = w;
      }
      if let Some(h) = override_h {
        b.content.height = h;
      }
    }
    BoxKind::Table => {
      let outer = override_w.map(|w| w + b.border.horizontal() + b.padding.horizontal() + b.margin.horizontal());
      let table_ctx = LayoutContext {
        containing_width: outer.unwrap_or(child_ctx.containing_width),
        ..child_ctx
      };
      crate::table::layout_table(b, &table_ctx, origin, text_ctx, rects, cache, bump);
      if let Some(w) = override_w {
        b.content.width = w;
      }
      if let Some(h) = override_h {
        b.content.height = h;
      }
    }
    _ => {
      let mut cursor_y = b.content.y;
      for child in b.children.iter_mut() {
        let placeholder = LayoutBox::new(BoxKind::Block, child.node, child.style, bump);
        let old = std::mem::replace(child, placeholder);
        let result = crate::engine::layout_node(
          old,
          &child_ctx,
          Point::new(b.content.x, cursor_y),
          text_ctx,
          rects,
          cache,
          bump,
        );
        *child = result;
        cursor_y += child.outer_height();
      }
      let content_h = (cursor_y - b.content.y).max(0.0);
      b.content.height = override_h
        .or_else(|| sizes::resolve_length(b.style.height, ctx.containing_height))
        .unwrap_or(content_h);

      finalize_flex_item_scroll(b);
    }
  }
}

// ── Translation helpers ───────────────────────────────────────────────

fn finalize_flex_item_scroll(b: &mut LayoutBox) {
  let ov_x = crate::block::parse_overflow_value(b.style.overflow_x);
  let ov_y = crate::block::parse_overflow_value(b.style.overflow_y);
  b.overflow_x = ov_x;
  b.overflow_y = ov_y;

  let is_scroll = !matches!(ov_x, Overflow::Visible) || !matches!(ov_y, Overflow::Visible);
  if !is_scroll {
    return;
  }

  let mut max_right = 0.0_f32;
  let mut max_bottom = 0.0_f32;
  for child in &b.children {
    let cr = child.content.x + child.content.width + child.padding.right + child.border.right + child.margin.right;
    let cb = child.content.y + child.content.height + child.padding.bottom + child.border.bottom + child.margin.bottom;
    max_right = max_right.max(cr);
    max_bottom = max_bottom.max(cb);
  }

  let scroll_width = (max_right - b.content.x).max(b.content.width);
  let scroll_height = (max_bottom - b.content.y).max(b.content.height);
  let overflows_y = scroll_height > b.content.height + 0.5;
  let overflows_x = scroll_width > b.content.width + 0.5;
  let show_y = matches!(ov_y, Overflow::Scroll) || (ov_y == Overflow::Auto && overflows_y);
  let show_x = matches!(ov_x, Overflow::Scroll) || (ov_x == Overflow::Auto && overflows_x);

  let scrollbar_w = lui_core::resolve_scrollbar_width(b.style.scrollbar_width);
  let sb_w = if show_y || show_x { scrollbar_w } else { 0.0 };

  b.scroll = Some(ScrollInfo {
    scroll_width,
    scroll_height,
    scroll_x: 0.0,
    scroll_y: 0.0,
    scrollbar_width: sb_w,
  });
  b.clip = Some(b.padding_rect());
}

fn translate_box(b: &mut LayoutBox, target_x: f32, target_y: f32) {
  let cur_x = b.content.x - b.padding.left - b.border.left - b.margin.left;
  let cur_y = b.content.y - b.padding.top - b.border.top - b.margin.top;
  let dx = target_x - cur_x;
  let dy = target_y - cur_y;
  if dx.abs() > EPS || dy.abs() > EPS {
    translate_box_delta(b, dx, dy);
  }
}

fn translate_box_delta(b: &mut LayoutBox, dx: f32, dy: f32) {
  b.content.x += dx;
  b.content.y += dy;
  if let Some(ref mut clip) = b.clip {
    clip.x += dx;
    clip.y += dy;
  }
  for child in &mut b.children {
    translate_box_delta(child, dx, dy);
  }
}

// ── Flexible lengths (CSS-Flex-1 §9.7) ───────────────────────────────

fn resolve_flexible_lengths(items: &mut [FlexItem<'_>], line: &[usize], main_axis_size: f32, gap_main: f32) {
  if line.is_empty() {
    return;
  }

  for &i in line {
    items[i].resolved_main = if items[i].collapsed {
      0.0
    } else {
      items[i].hypothetical_main
    };
  }

  let initial_outer: f32 = line.iter().map(|&i| items[i].hypothetical_outer_main()).sum::<f32>()
    + gap_main * (line.len() as f32 - 1.0).max(0.0);
  let initial_free = main_axis_size - initial_outer;
  if initial_free.abs() <= EPS {
    return;
  }
  let growing = initial_free > 0.0;

  let mut frozen = vec![false; line.len()];
  for (k, &i) in line.iter().enumerate() {
    if items[i].collapsed {
      frozen[k] = true;
      continue;
    }
    let factor = if growing {
      items[i].flex_grow
    } else {
      items[i].flex_shrink
    };
    if factor <= 0.0 || (!growing && items[i].base_size <= 0.0) {
      frozen[k] = true;
    }
  }
  if frozen.iter().all(|&f| f) {
    return;
  }

  let frame_outer = |it: &FlexItem| -> f32 { it.frame_main + it.margin_main_start + it.margin_main_end };
  let gap_total = gap_main * (line.len() as f32 - 1.0).max(0.0);

  for _ in 0..(line.len() + 1) {
    let mut consumed = gap_total;
    let mut sum_factor = 0.0_f32;
    let mut sum_scaled_shrink = 0.0_f32;
    for (k, &i) in line.iter().enumerate() {
      let it = &items[i];
      if frozen[k] {
        consumed += it.resolved_main + frame_outer(it);
      } else {
        consumed += it.hypothetical_main + frame_outer(it);
        if growing {
          sum_factor += it.flex_grow;
        } else {
          sum_scaled_shrink += it.flex_shrink * it.base_size;
        }
      }
    }
    let free = main_axis_size - consumed;

    let denom = if growing && sum_factor < 1.0 {
      1.0
    } else {
      sum_factor.max(0.0)
    };
    for (k, &i) in line.iter().enumerate() {
      if frozen[k] {
        continue;
      }
      let it = &mut items[i];
      let new = if growing {
        if denom <= 0.0 {
          it.hypothetical_main
        } else {
          it.hypothetical_main + (free * it.flex_grow / denom).max(0.0)
        }
      } else if sum_scaled_shrink <= 0.0 {
        it.hypothetical_main
      } else {
        let ratio = (it.flex_shrink * it.base_size) / sum_scaled_shrink;
        (it.hypothetical_main + free * ratio).max(0.0)
      };
      it.resolved_main = new;
    }

    let mut total_violation = 0.0_f32;
    let mut any_violated = false;
    let mut violated = vec![0i8; line.len()];
    for (k, &i) in line.iter().enumerate() {
      if frozen[k] {
        continue;
      }
      let it = &mut items[i];
      let clamped = it.resolved_main.clamp(it.main_min, it.main_max).max(0.0);
      let diff = clamped - it.resolved_main;
      if diff.abs() > EPS {
        any_violated = true;
        total_violation += diff;
        violated[k] = if diff > 0.0 { 1 } else { -1 };
      }
      it.resolved_main = clamped;
    }

    if !any_violated {
      break;
    }
    let direction = if total_violation.abs() <= EPS {
      0
    } else if total_violation > 0.0 {
      1
    } else {
      -1
    };
    for k in 0..line.len() {
      if !frozen[k] && violated[k] != 0 && (direction == 0 || violated[k] == direction) {
        frozen[k] = true;
      }
    }
    if frozen.iter().all(|&f| f) {
      break;
    }
  }
}

// ── justify-content ───────────────────────────────────────────────────

fn distribution(justify: &str, free: f32, n: f32) -> (f32, f32) {
  if free <= 0.0 {
    return (0.0, 0.0);
  }
  match justify {
    "flex-start" | "start" | "left" | "" => (0.0, 0.0),
    "flex-end" | "end" | "right" => (free, 0.0),
    "center" => (free * 0.5, 0.0),
    "space-between" => {
      if n > 1.0 {
        (0.0, free / (n - 1.0))
      } else {
        (0.0, 0.0)
      }
    }
    "space-around" => {
      if n > 0.0 {
        let s = free / n;
        (s * 0.5, s)
      } else {
        (0.0, 0.0)
      }
    }
    "space-evenly" => {
      if n > 0.0 {
        let s = free / (n + 1.0);
        (s, s)
      } else {
        (0.0, 0.0)
      }
    }
    _ => (0.0, 0.0),
  }
}

// ── align-content ─────────────────────────────────────────────────────

fn align_content_distribution(align: &str, free: f32, n_lines: f32, definite: bool) -> (f32, f32, f32) {
  if !definite || n_lines <= 1.0 || free <= 0.0 {
    return (0.0, 0.0, 0.0);
  }
  match align {
    "" | "normal" | "stretch" => (0.0, 0.0, free / n_lines),
    "flex-start" | "start" => (0.0, 0.0, 0.0),
    "flex-end" | "end" => (free, 0.0, 0.0),
    "center" => (free * 0.5, 0.0, 0.0),
    "space-between" => {
      if n_lines > 1.0 {
        (0.0, free / (n_lines - 1.0), 0.0)
      } else {
        (0.0, 0.0, 0.0)
      }
    }
    "space-around" => {
      let s = free / n_lines;
      (s * 0.5, s, 0.0)
    }
    "space-evenly" => {
      let s = free / (n_lines + 1.0);
      (s, s, 0.0)
    }
    _ => (0.0, 0.0, 0.0),
  }
}

// ── align-self resolution ─────────────────────────────────────────────

fn resolve_align_self<'a>(item_align: &'a str, parent_align: &'a str) -> &'a str {
  if item_align.is_empty() || item_align == "auto" {
    if parent_align.is_empty() {
      "stretch"
    } else {
      parent_align
    }
  } else {
    item_align
  }
}

// ── Intrinsic (max-content) measurement ──────────────────────────────

fn find_first_baseline(box_: &LayoutBox) -> Option<f32> {
  if box_.baseline.is_some() {
    return box_.baseline;
  }
  for child in &box_.children {
    if let Some(child_bl) = find_first_baseline(child) {
      let child_top = child.content.y - child.padding.top - child.border.top;
      let parent_top = box_.content.y;
      return Some(child_bl + (child_top - parent_top));
    }
  }
  None
}

pub fn measure_max_content_width_pub(box_: &LayoutBox, text_ctx: &mut TextContext) -> f32 {
  measure_max_content_width(box_, text_ctx)
}

fn measure_max_content_width(box_: &LayoutBox, text_ctx: &mut TextContext) -> f32 {
  if let lui_core::HtmlElement::Text(ref content) = *box_.node.element() {
    let style = crate::text::text_style_from_cascade(box_.style);
    let ws = css_str(box_.style.white_space);
    let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
      std::borrow::Cow::Owned(crate::flow::collapse_whitespace(content))
    } else {
      std::borrow::Cow::Borrowed(content.as_ref())
    };
    let run = text_ctx.shape(&text, &style);
    return run.width;
  }

  let frame = if matches!(box_.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline) {
    0.0
  } else {
    let border = sides::resolve_border(box_.style);
    let padding = sides::resolve_padding(box_.style);
    border.horizontal() + padding.horizontal()
  };

  if let Some(w) = sizes::resolve_length(box_.style.width, 0.0) {
    return w + frame;
  }

  if matches!(box_.kind, BoxKind::FlexContainer | BoxKind::InlineFlex) {
    let direction = css_str(box_.style.flex_direction);
    let is_row = !matches!(direction, "column" | "column-reverse");
    if is_row {
      let gap = sizes::resolve_length(box_.style.column_gap, 0.0).unwrap_or(0.0);
      let mut total = 0.0_f32;
      let mut count = 0u32;
      for child in &box_.children {
        if css_str(child.style.display) == "none" {
          continue;
        }
        total += measure_max_content_width(child, text_ctx);
        count += 1;
      }
      if count > 1 {
        total += gap * (count - 1) as f32;
      }
      return total + frame;
    }
  }

  let mut inline_run = 0.0_f32;
  let mut max_block = 0.0_f32;

  for child in &box_.children {
    let child_w = measure_max_content_width(child, text_ctx);
    match child.kind {
      BoxKind::Block | BoxKind::FlexContainer | BoxKind::GridContainer => {
        max_block = max_block.max(inline_run);
        inline_run = 0.0;
        max_block = max_block.max(child_w);
      }
      _ => {
        inline_run += child_w;
      }
    }
  }
  max_block = max_block.max(inline_run);
  max_block + frame
}

fn measure_min_content_width(box_: &LayoutBox, text_ctx: &mut TextContext) -> f32 {
  if let lui_core::HtmlElement::Text(ref content) = *box_.node.element() {
    let style = crate::text::text_style_from_cascade(box_.style);
    let ws = css_str(box_.style.white_space);
    let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
      std::borrow::Cow::Owned(crate::flow::collapse_whitespace(content))
    } else {
      std::borrow::Cow::Borrowed(content.as_ref())
    };
    let mut max_word = 0.0_f32;
    for word in text.split_whitespace() {
      let run = text_ctx.shape(word, &style);
      max_word = max_word.max(run.width);
    }
    return max_word;
  }

  let frame = if matches!(box_.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline) {
    0.0
  } else {
    let border = sides::resolve_border(box_.style);
    let padding = sides::resolve_padding(box_.style);
    border.horizontal() + padding.horizontal()
  };

  let mut max_child = 0.0_f32;
  for child in &box_.children {
    max_child = max_child.max(measure_min_content_width(child, text_ctx));
  }
  max_child + frame
}

fn measure_min_content_height(box_: &LayoutBox, text_ctx: &mut TextContext) -> f32 {
  if let lui_core::HtmlElement::Text(ref content) = *box_.node.element() {
    let style = crate::text::text_style_from_cascade(box_.style);
    let ws = css_str(box_.style.white_space);
    let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
      std::borrow::Cow::Owned(crate::flow::collapse_whitespace(content))
    } else {
      std::borrow::Cow::Borrowed(content.as_ref())
    };
    if text.is_empty() {
      return 0.0;
    }
    let min_w = measure_min_content_width(box_, text_ctx);
    if min_w > 0.0 {
      let lines = text_ctx.break_into_lines(&text, &style, min_w);
      if !lines.is_empty() {
        return lines.iter().map(|l| l.height).sum();
      }
    }
    let run = text_ctx.shape(&text, &style);
    return run.height;
  }

  let border = sides::resolve_border(box_.style);
  let padding = sides::resolve_padding(box_.style);
  let frame = border.vertical() + padding.vertical();

  // Row flex: cross-axis height = max of children (no wrap info available).
  // Column flex: main-axis height = sum of children + gaps.
  if matches!(box_.kind, BoxKind::FlexContainer | BoxKind::InlineFlex) {
    let direction = css_str(box_.style.flex_direction);
    let is_row = !matches!(direction, "column" | "column-reverse");
    if is_row {
      let mut max_h = 0.0_f32;
      for child in &box_.children {
        if css_str(child.style.display) == "none" {
          continue;
        }
        max_h = max_h.max(measure_min_content_height(child, text_ctx));
      }
      return max_h + frame;
    } else {
      let gap = sizes::resolve_length(box_.style.row_gap, 0.0).unwrap_or(0.0);
      let mut total = 0.0_f32;
      let mut count = 0u32;
      for child in &box_.children {
        if css_str(child.style.display) == "none" {
          continue;
        }
        total += measure_min_content_height(child, text_ctx);
        count += 1;
      }
      if count > 1 {
        total += gap * (count - 1) as f32;
      }
      return total + frame;
    }
  }

  let mut block_sum = 0.0_f32;
  let mut max_inline = 0.0_f32;
  for child in &box_.children {
    let child_h = measure_min_content_height(child, text_ctx);
    match child.kind {
      BoxKind::Block
      | BoxKind::FlexContainer
      | BoxKind::GridContainer
      | BoxKind::Table
      | BoxKind::TableRow
      | BoxKind::TableRowGroup
      | BoxKind::TableCaption
      | BoxKind::AnonymousBlock
      | BoxKind::ListItem => {
        block_sum += child_h;
      }
      _ => {
        max_inline = max_inline.max(child_h);
      }
    }
  }
  block_sum + max_inline + frame
}

fn measure_max_content_height(box_: &LayoutBox, text_ctx: &mut TextContext) -> f32 {
  if let lui_core::HtmlElement::Text(ref content) = *box_.node.element() {
    let style = crate::text::text_style_from_cascade(box_.style);
    let ws = css_str(box_.style.white_space);
    let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
      std::borrow::Cow::Owned(crate::flow::collapse_whitespace(content))
    } else {
      std::borrow::Cow::Borrowed(content.as_ref())
    };
    if text.is_empty() {
      return 0.0;
    }
    let run = text_ctx.shape(&text, &style);
    return run.height;
  }

  let border = sides::resolve_border(box_.style);
  let padding = sides::resolve_padding(box_.style);
  let frame = border.vertical() + padding.vertical();

  if let Some(h) = sizes::resolve_length(box_.style.height, 0.0) {
    return h + frame;
  }

  if matches!(box_.kind, BoxKind::FlexContainer | BoxKind::InlineFlex) {
    let direction = css_str(box_.style.flex_direction);
    let is_row = !matches!(direction, "column" | "column-reverse");
    if is_row {
      let mut max_h = 0.0_f32;
      for child in &box_.children {
        if css_str(child.style.display) == "none" {
          continue;
        }
        max_h = max_h.max(measure_max_content_height(child, text_ctx));
      }
      return max_h + frame;
    } else {
      let gap = sizes::resolve_length(box_.style.row_gap, 0.0).unwrap_or(0.0);
      let mut total = 0.0_f32;
      let mut count = 0u32;
      for child in &box_.children {
        if css_str(child.style.display) == "none" {
          continue;
        }
        total += measure_max_content_height(child, text_ctx);
        count += 1;
      }
      if count > 1 {
        total += gap * (count - 1) as f32;
      }
      return total + frame;
    }
  }

  let mut block_sum = 0.0_f32;
  let mut max_inline = 0.0_f32;

  for child in &box_.children {
    let child_h = measure_max_content_height(child, text_ctx);
    match child.kind {
      BoxKind::Block
      | BoxKind::FlexContainer
      | BoxKind::GridContainer
      | BoxKind::Table
      | BoxKind::TableRow
      | BoxKind::TableRowGroup
      | BoxKind::TableCaption
      | BoxKind::AnonymousBlock
      | BoxKind::ListItem => {
        block_sum += child_h;
      }
      _ => {
        max_inline = max_inline.max(child_h);
      }
    }
  }
  block_sum + max_inline + frame
}
