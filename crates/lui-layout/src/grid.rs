//! CSS Grid layout — CSS-Grid-1 implementation.
//!
//! Supports: grid-template-columns/rows (px/fr/auto/repeat()/minmax()),
//! gap, auto-placement (row/column/dense), grid-column/row-start/end,
//! span N, grid-auto-rows/columns, align-items/justify-items on cells.

use std::collections::HashMap;

use bumpalo::Bump;
use lui_core::{CssUnit, CssValue, Rect};
use lui_parse::HtmlNode;

use crate::{
  box_tree::LayoutBox, context::LayoutContext, geometry::Point, positioned, sides, sizes, text::TextContext,
};

type LineNames = HashMap<String, usize>;

// ── Track sizing ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum TrackSize {
  Px(f32),
  Fr(f32),
  Auto,
  MinMax(f32, TrackMax),
}

#[derive(Debug, Clone)]
enum TrackMax {
  Px(f32),
  Fr(f32),
  Auto,
}

fn parse_track_list(value: Option<&CssValue>, container_width: f32, gap: f32) -> (Vec<TrackSize>, LineNames) {
  match value {
    Some(CssValue::Unknown(s)) | Some(CssValue::String(s)) => {
      let raw = s.as_ref();
      if raw == "none" || raw.is_empty() {
        return (vec![], LineNames::new());
      }
      parse_track_str(raw, container_width, gap)
    }
    Some(CssValue::Dimension {
      value,
      unit: CssUnit::Px,
    }) => (vec![TrackSize::Px(*value as f32)], LineNames::new()),
    Some(CssValue::Dimension {
      value,
      unit: CssUnit::Fr,
    }) => (vec![TrackSize::Fr(*value as f32)], LineNames::new()),
    Some(CssValue::Number(n)) if *n == 0.0 => (vec![TrackSize::Px(0.0)], LineNames::new()),
    Some(CssValue::Function { function, args }) => {
      (parse_track_function(function, args, container_width, gap), LineNames::new())
    }
    _ => (vec![], LineNames::new()),
  }
}

fn parse_track_function(function: &lui_core::CssFunction, args: &[CssValue], container_width: f32, gap: f32) -> Vec<TrackSize> {
  let name = function.name();
  if name == "repeat" || name.contains("repeat") {
    if args.len() >= 2 {
      let auto_mode = match &args[0] {
        CssValue::Unknown(s) | CssValue::String(s) => match s.as_ref() {
          "auto-fill" => Some(false),
          "auto-fit" => Some(true),
          _ => None,
        },
        _ => None,
      };
      let count = if let Some(_) = auto_mode {
        let track_size = estimate_track_px(&args[1..], container_width);
        if track_size > 0.0 {
          ((container_width + gap) / (track_size + gap)).floor().max(1.0) as usize
        } else {
          1
        }
      } else {
        match &args[0] {
          CssValue::Number(n) => *n as usize,
          _ => 1,
        }
      };
      let mut pattern = Vec::new();
      for arg in &args[1..] {
        match arg {
          CssValue::Dimension {
            value,
            unit: CssUnit::Px,
          } => pattern.push(TrackSize::Px(*value as f32)),
          CssValue::Dimension {
            value,
            unit: CssUnit::Fr,
          } => pattern.push(TrackSize::Fr(*value as f32)),
          CssValue::Percentage(p) => pattern.push(TrackSize::Px(*p as f32 / 100.0 * container_width)),
          CssValue::String(s) | CssValue::Unknown(s) if s.as_ref() == "auto" => pattern.push(TrackSize::Auto),
          _ => {}
        }
      }
      if pattern.is_empty() {
        pattern.push(TrackSize::Fr(1.0));
      }
      let mut result = Vec::with_capacity(count * pattern.len());
      for _ in 0..count {
        result.extend(pattern.iter().cloned());
      }
      return result;
    }
  }
  if name == "minmax" && args.len() >= 2 {
    let min = match &args[0] {
      CssValue::Dimension {
        value,
        unit: CssUnit::Px,
      } => *value as f32,
      CssValue::Number(n) => *n as f32,
      _ => 0.0,
    };
    let max = match &args[1] {
      CssValue::Dimension {
        value,
        unit: CssUnit::Px,
      } => TrackMax::Px(*value as f32),
      CssValue::Dimension {
        value,
        unit: CssUnit::Fr,
      } => TrackMax::Fr(*value as f32),
      _ => TrackMax::Auto,
    };
    return vec![TrackSize::MinMax(min, max)];
  }
  vec![]
}

fn estimate_track_px(args: &[CssValue], container_width: f32) -> f32 {
  let mut total = 0.0_f32;
  for arg in args {
    match arg {
      CssValue::Dimension {
        value,
        unit: CssUnit::Px,
      } => total += *value as f32,
      CssValue::Percentage(p) => total += *p as f32 / 100.0 * container_width,
      _ => {}
    }
  }
  total
}

fn parse_track_str(raw: &str, container_width: f32, gap: f32) -> (Vec<TrackSize>, LineNames) {
  let mut tracks = Vec::new();
  let mut names = LineNames::new();
  let mut chars = raw.chars().peekable();
  let mut buf = String::new();

  while chars.peek().is_some() {
    skip_ws(&mut chars);
    if chars.peek().is_none() {
      break;
    }

    if chars.peek() == Some(&'[') {
      chars.next();
      let mut name_buf = String::new();
      while let Some(&c) = chars.peek() {
        if c == ']' {
          chars.next();
          break;
        }
        name_buf.push(c);
        chars.next();
      }
      let line_idx = tracks.len();
      for n in name_buf.split_whitespace() {
        names.insert(n.to_owned(), line_idx);
      }
      continue;
    }

    buf.clear();
    while let Some(&c) = chars.peek() {
      if c.is_ascii_whitespace() || c == '[' {
        break;
      }
      if c == '(' {
        buf.push(c);
        chars.next();
        let mut depth = 1;
        while let Some(&c2) = chars.peek() {
          buf.push(c2);
          chars.next();
          if c2 == '(' {
            depth += 1;
          }
          if c2 == ')' {
            depth -= 1;
            if depth == 0 {
              break;
            }
          }
        }
        continue;
      }
      buf.push(c);
      chars.next();
    }

    if buf.is_empty() {
      continue;
    }

    if buf.starts_with("repeat(") && buf.ends_with(')') {
      let inner = &buf[7..buf.len() - 1];
      if let Some((count_str, pattern)) = inner.split_once(',') {
        let count_str = count_str.trim();
        let (pattern_tracks, _pattern_names) = parse_track_str(pattern.trim(), container_width, gap);
        let count = if count_str == "auto-fill" || count_str == "auto-fit" {
          let track_px: f32 = pattern_tracks
            .iter()
            .map(|t| match t {
              TrackSize::Px(v) => *v,
              _ => 0.0,
            })
            .sum();
          if track_px > 0.0 {
            ((container_width + gap) / (track_px + gap)).floor().max(1.0) as usize
          } else {
            1
          }
        } else {
          count_str.parse::<usize>().unwrap_or(1)
        };
        for _ in 0..count {
          tracks.extend(pattern_tracks.iter().cloned());
        }
      }
    } else if buf.starts_with("minmax(") && buf.ends_with(')') {
      let inner = &buf[7..buf.len() - 1];
      if let Some((min_s, max_s)) = inner.split_once(',') {
        let min_v = parse_single_size(min_s.trim(), container_width);
        let max_v = parse_single_size(max_s.trim(), container_width);
        let min_px = match &min_v {
          TrackSize::Px(v) => *v,
          _ => 0.0,
        };
        let max_tm = match max_v {
          TrackSize::Px(v) => TrackMax::Px(v),
          TrackSize::Fr(v) => TrackMax::Fr(v),
          TrackSize::Auto => TrackMax::Auto,
          TrackSize::MinMax(_, m) => m,
        };
        tracks.push(TrackSize::MinMax(min_px, max_tm));
      }
    } else {
      tracks.push(parse_single_size(&buf, container_width));
    }
  }
  (tracks, names)
}

fn parse_single_size(token: &str, container_width: f32) -> TrackSize {
  if token == "auto" {
    TrackSize::Auto
  } else if let Some(fr) = token.strip_suffix("fr") {
    fr.parse::<f32>().map(TrackSize::Fr).unwrap_or(TrackSize::Auto)
  } else if let Some(px) = token.strip_suffix("px") {
    px.parse::<f32>().map(TrackSize::Px).unwrap_or(TrackSize::Auto)
  } else if let Some(pct) = token.strip_suffix('%') {
    pct
      .parse::<f32>()
      .map(|v| TrackSize::Px(v / 100.0 * container_width))
      .unwrap_or(TrackSize::Auto)
  } else if let Ok(v) = token.parse::<f32>() {
    TrackSize::Px(v)
  } else {
    TrackSize::Auto
  }
}

fn skip_ws(chars: &mut std::iter::Peekable<std::str::Chars>) {
  while let Some(&c) = chars.peek() {
    if !c.is_ascii_whitespace() {
      break;
    }
    chars.next();
  }
}

fn resolve_tracks(defs: &[TrackSize], available: f32, gap: f32, auto_sizes: &[f32]) -> Vec<f32> {
  if defs.is_empty() {
    return vec![];
  }

  let total_gap = gap * (defs.len() as f32 - 1.0).max(0.0);
  let mut fixed_sum = 0.0_f32;
  let mut fr_sum = 0.0_f32;
  let mut sizes: Vec<f32> = defs
    .iter()
    .enumerate()
    .map(|(i, d)| match d {
      TrackSize::Px(v) => {
        fixed_sum += v;
        *v
      }
      TrackSize::Fr(v) => {
        fr_sum += v;
        0.0
      }
      TrackSize::Auto => {
        let s = auto_sizes.get(i).copied().unwrap_or(0.0);
        fixed_sum += s;
        s
      }
      TrackSize::MinMax(min, max) => match max {
        TrackMax::Fr(v) => {
          fr_sum += v;
          *min
        }
        TrackMax::Px(v) => {
          let s = *v;
          fixed_sum += s;
          s
        }
        TrackMax::Auto => {
          let s = auto_sizes.get(i).copied().unwrap_or(*min).max(*min);
          fixed_sum += s;
          s
        }
      },
    })
    .collect();

  if fr_sum > 0.0 {
    let free = (available - total_gap - fixed_sum).max(0.0);
    for (i, def) in defs.iter().enumerate() {
      let fr_val = match def {
        TrackSize::Fr(f) => Some(*f),
        TrackSize::MinMax(min, TrackMax::Fr(f)) => {
          let share = free * f / fr_sum;
          sizes[i] = share.max(*min);
          None
        }
        _ => None,
      };
      if let Some(f) = fr_val {
        sizes[i] = free * f / fr_sum;
      }
    }
  }
  sizes
}

// ── Grid item placement ───────────────────────────────────────────────

struct AreaPlacement {
  col_start: usize,
  col_end: usize,
  row_start: usize,
  row_end: usize,
}

fn parse_template_areas(
  value: Option<&CssValue>,
  col_names: &mut LineNames,
  row_names: &mut LineNames,
  col_defs: &mut Vec<TrackSize>,
  row_defs: &mut Vec<TrackSize>,
) -> HashMap<String, AreaPlacement> {
  let mut areas: HashMap<String, AreaPlacement> = HashMap::new();
  let raw = match value {
    Some(CssValue::Unknown(s)) | Some(CssValue::String(s)) => s.as_ref(),
    _ => return areas,
  };
  if raw == "none" || raw.is_empty() {
    return areas;
  }

  let rows: Vec<&str> = raw.split('"').filter(|s| !s.trim().is_empty()).collect();

  for (row_idx, row_str) in rows.iter().enumerate() {
    let cells: Vec<&str> = row_str.split_whitespace().collect();
    for (col_idx, &name) in cells.iter().enumerate() {
      if name == "." {
        continue;
      }
      let entry = areas.entry(name.to_owned()).or_insert(AreaPlacement {
        col_start: col_idx,
        col_end: col_idx + 1,
        row_start: row_idx,
        row_end: row_idx + 1,
      });
      if col_idx < entry.col_start {
        entry.col_start = col_idx;
      }
      if col_idx + 1 > entry.col_end {
        entry.col_end = col_idx + 1;
      }
      if row_idx < entry.row_start {
        entry.row_start = row_idx;
      }
      if row_idx + 1 > entry.row_end {
        entry.row_end = row_idx + 1;
      }
    }

    while col_defs.len() < cells.len() {
      col_defs.push(TrackSize::Fr(1.0));
    }
  }

  while row_defs.len() < rows.len() {
    row_defs.push(TrackSize::Auto);
  }

  for (name, area) in &areas {
    col_names.entry(format!("{}-start", name)).or_insert(area.col_start);
    col_names.entry(format!("{}-end", name)).or_insert(area.col_end);
    row_names.entry(format!("{}-start", name)).or_insert(area.row_start);
    row_names.entry(format!("{}-end", name)).or_insert(area.row_end);
  }

  areas
}

fn css_str(v: Option<&CssValue>) -> &str {
  match v {
    Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
    _ => "",
  }
}

struct GridPlacement {
  col_start: usize,
  col_end: usize,
  row_start: usize,
  row_end: usize,
}

fn parse_line_value(v: Option<&CssValue>) -> LinePlacement {
  match v {
    Some(CssValue::Number(n)) => LinePlacement::Line((*n as i32).max(1) as usize),
    Some(CssValue::Dimension { value, .. }) => LinePlacement::Line((*value as i32).max(1) as usize),
    Some(CssValue::Unknown(s)) | Some(CssValue::String(s)) => {
      let s = s.as_ref().trim();
      if s == "auto" || s.is_empty() {
        return LinePlacement::Auto;
      }
      if let Some(rest) = s.strip_prefix("span") {
        let rest = rest.trim();
        if let Ok(n) = rest.parse::<usize>() {
          return LinePlacement::Span(n.max(1));
        }
      }
      if let Ok(n) = s.parse::<i32>() {
        return LinePlacement::Line(n.max(1) as usize);
      }
      LinePlacement::Name(s.to_owned())
    }
    _ => LinePlacement::Auto,
  }
}

#[derive(Debug, Clone)]
enum LinePlacement {
  Auto,
  Line(usize),
  Span(usize),
  Name(String),
}

fn resolve_placement(
  style: &lui_cascade::ComputedStyle,
  auto_col: &mut usize,
  auto_row: &mut usize,
  num_cols: usize,
  num_rows: usize,
  occupied: &mut Vec<Vec<bool>>,
  flow_column: bool,
  dense: bool,
  col_names: &LineNames,
  row_names: &LineNames,
  area_map: &HashMap<String, AreaPlacement>,
) -> GridPlacement {
  let cs = parse_line_value(style.grid_column_start);
  let ce = parse_line_value(style.grid_column_end);
  let rs = parse_line_value(style.grid_row_start);
  let re = parse_line_value(style.grid_row_end);

  let col_span = match (&cs, &ce) {
    (_, LinePlacement::Span(n)) => *n,
    (LinePlacement::Line(s), LinePlacement::Line(e)) if *e > *s => *e - *s,
    _ => 1,
  };
  let row_span = match (&rs, &re) {
    (_, LinePlacement::Span(n)) => *n,
    (LinePlacement::Line(s), LinePlacement::Line(e)) if *e > *s => *e - *s,
    _ => 1,
  };

  let resolve_name_start =
    |name: &str, names: &LineNames, area_map: &HashMap<String, AreaPlacement>, is_col: bool| -> Option<usize> {
      if let Some(area) = area_map.get(name) {
        return Some(if is_col { area.col_start } else { area.row_start });
      }
      let suffixed = format!("{}-start", name);
      names.get(suffixed.as_str()).or_else(|| names.get(name)).copied()
    };
  let resolve_name_end =
    |name: &str, names: &LineNames, area_map: &HashMap<String, AreaPlacement>, is_col: bool| -> Option<usize> {
      if let Some(area) = area_map.get(name) {
        return Some(if is_col { area.col_end } else { area.row_end });
      }
      let suffixed = format!("{}-end", name);
      names.get(suffixed.as_str()).or_else(|| names.get(name)).copied()
    };

  let col_start = match &cs {
    LinePlacement::Line(n) => (*n - 1).min(num_cols.saturating_sub(1)),
    LinePlacement::Name(name) => resolve_name_start(name, col_names, area_map, true).unwrap_or(0),
    _ => {
      if flow_column {
        *auto_col
      } else {
        find_auto_slot(
          auto_col,
          auto_row,
          col_span,
          row_span,
          num_cols,
          num_rows,
          occupied,
          flow_column,
          dense,
        )
      }
    }
  };
  let row_start = match &rs {
    LinePlacement::Line(n) => *n - 1,
    LinePlacement::Name(name) => resolve_name_start(name, row_names, area_map, false).unwrap_or(0),
    _ => {
      if flow_column {
        find_auto_slot(
          auto_col,
          auto_row,
          col_span,
          row_span,
          num_cols,
          num_rows,
          occupied,
          flow_column,
          dense,
        )
      } else {
        *auto_row
      }
    }
  };

  let col_end = match &ce {
    LinePlacement::Line(n) => (*n - 1).max(col_start + 1),
    LinePlacement::Span(n) => col_start + n,
    LinePlacement::Name(name) => resolve_name_end(name, col_names, area_map, true).unwrap_or(col_start + 1),
    LinePlacement::Auto => col_start + col_span,
  };
  let row_end = match &re {
    LinePlacement::Line(n) => (*n - 1).max(row_start + 1),
    LinePlacement::Span(n) => row_start + n,
    LinePlacement::Name(name) => resolve_name_end(name, row_names, area_map, false).unwrap_or(row_start + 1),
    LinePlacement::Auto => row_start + row_span,
  };

  // Mark cells as occupied
  while occupied.len() <= row_end {
    occupied.push(vec![false; num_cols.max(col_end)]);
  }
  for r in row_start..row_end {
    while occupied[r].len() < col_end {
      occupied[r].push(false);
    }
    for c in col_start..col_end {
      occupied[r][c] = true;
    }
  }

  // Advance auto cursor past this item
  if flow_column {
    *auto_row = row_end;
    if *auto_row >= num_rows {
      *auto_row = 0;
      *auto_col += 1;
    }
  } else {
    *auto_col = col_end;
    if *auto_col >= num_cols {
      *auto_col = 0;
      *auto_row += 1;
    }
  }

  GridPlacement {
    col_start,
    col_end,
    row_start,
    row_end,
  }
}

fn find_auto_slot(
  auto_col: &mut usize,
  auto_row: &mut usize,
  col_span: usize,
  row_span: usize,
  num_cols: usize,
  num_rows: usize,
  occupied: &mut Vec<Vec<bool>>,
  flow_column: bool,
  dense: bool,
) -> usize {
  if dense {
    *auto_col = 0;
    *auto_row = 0;
  }
  let max_iter = (num_rows.max(occupied.len()) + row_span + 20) * (num_cols + col_span + 20);
  for _ in 0..max_iter {
    let c = *auto_col;
    let r = *auto_row;
    let fits_cols = if flow_column { true } else { c + col_span <= num_cols };
    let fits_rows = if flow_column { r + row_span <= num_rows } else { true };
    if fits_cols && fits_rows && fits(occupied, r, c, row_span, col_span) {
      return if flow_column { r } else { c };
    }
    if flow_column {
      *auto_row += 1;
      if *auto_row + row_span > num_rows {
        *auto_row = 0;
        *auto_col += 1;
      }
    } else {
      *auto_col += 1;
      if *auto_col + col_span > num_cols {
        *auto_col = 0;
        *auto_row += 1;
      }
    }
  }
  if flow_column { *auto_row } else { *auto_col }
}

fn fits(occupied: &[Vec<bool>], row: usize, col: usize, row_span: usize, col_span: usize) -> bool {
  for r in row..row + row_span {
    if r >= occupied.len() {
      continue;
    }
    for c in col..col + col_span {
      if c >= occupied[r].len() {
        continue;
      }
      if occupied[r][c] {
        return false;
      }
    }
  }
  true
}

// ── Public entry point ────────────────────────────────────────────────

pub fn layout_grid<'a>(
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

  let is_border_box = css_str(b.style.box_sizing) == "border-box";
  let frame_h = border.horizontal() + padding.horizontal();
  let frame_v = border.vertical() + padding.vertical();

  let available = ctx.containing_width - margin.edges.horizontal() - frame_h;
  let w = sizes::resolve_length(b.style.width, ctx.containing_width)
    .map(|v| if is_border_box { (v - frame_h).max(0.0) } else { v })
    .unwrap_or(available.max(0.0));
  let inner_width = w.min(available.max(0.0));
  b.content.width = inner_width;
  b.content.x = pos.x + margin.edges.left + border.left + padding.left;
  b.content.y = pos.y + margin.edges.top + border.top + padding.top;

  let inner_height = sizes::resolve_length(b.style.height, ctx.containing_height)
    .map(|v| if is_border_box { (v - frame_v).max(0.0) } else { v });

  let gap_col = sizes::resolve_length(b.style.column_gap, inner_width).unwrap_or(0.0);
  let gap_row = sizes::resolve_length(b.style.row_gap, inner_width).unwrap_or(0.0);

  let auto_flow = css_str(b.style.grid_auto_flow);
  let flow_column = auto_flow.contains("column");
  let dense = auto_flow.contains("dense");

  let align_items = css_str(b.style.align_items);
  let justify_items = css_str(b.style.justify_items);

  let (mut col_defs, mut col_names) = parse_track_list(b.style.grid_template_columns, inner_width, gap_col);
  let (mut row_defs, mut row_names) = parse_track_list(b.style.grid_template_rows, inner_width, gap_row);

  // Parse grid-template-areas and generate implicit named lines + area map
  let area_map = parse_template_areas(
    b.style.grid_template_areas,
    &mut col_names,
    &mut row_names,
    &mut col_defs,
    &mut row_defs,
  );

  let child_count = b.children.len();
  if child_count == 0 {
    b.content.height = inner_height.unwrap_or(0.0);
    return;
  }

  if col_defs.is_empty() {
    col_defs.push(TrackSize::Fr(1.0));
  }

  let num_cols = col_defs.len();

  let auto_row_size = parse_auto_track_size(b.style.grid_auto_rows);
  let auto_col_size = parse_auto_track_size(b.style.grid_auto_columns);

  // Phase 1: place items
  let taken_children = std::mem::replace(&mut b.children, bumpalo::collections::Vec::new_in(bump));
  let mut placements: Vec<GridPlacement> = Vec::with_capacity(taken_children.len());
  let mut children: Vec<LayoutBox<'a>> = Vec::with_capacity(taken_children.len());
  let mut auto_col = 0_usize;
  let mut auto_row = 0_usize;
  let mut occupied: Vec<Vec<bool>> = Vec::new();

  for child in taken_children {
    if css_str(child.style.display) == "none" {
      continue;
    }
    if positioned::is_out_of_flow(child.style) {
      continue;
    }

    let num_rows_for_placement = row_defs.len().max(1);
    let placement = resolve_placement(
      child.style,
      &mut auto_col,
      &mut auto_row,
      num_cols,
      num_rows_for_placement,
      &mut occupied,
      flow_column,
      dense,
      &col_names,
      &row_names,
      &area_map,
    );
    placements.push(placement);
    children.push(child);
  }

  // Grow implicit columns for column-flow
  let max_col_end = placements.iter().map(|p| p.col_end).max().unwrap_or(num_cols);
  while col_defs.len() < max_col_end {
    col_defs.push(auto_col_size.clone());
  }

  // Compute auto column sizes from item content widths
  let mut col_auto_sizes = vec![0.0_f32; col_defs.len()];
  for (placement, child) in placements.iter().zip(children.iter()) {
    let span = placement.col_end - placement.col_start;
    if span == 1 {
      let c = placement.col_start;
      if c < col_auto_sizes.len() && matches!(col_defs.get(c), Some(TrackSize::Auto)) {
        let w = crate::flex::measure_max_content_width_pub(child, text_ctx);
        col_auto_sizes[c] = col_auto_sizes[c].max(w);
      }
    }
  }

  // Resolve column sizes
  let col_sizes = resolve_tracks(&col_defs, inner_width, gap_col, &col_auto_sizes);

  // Phase 1b: layout items at resolved column widths
  let mut items: Vec<(GridPlacement, LayoutBox<'a>)> = Vec::with_capacity(children.len());
  for (placement, child) in placements.into_iter().zip(children.into_iter()) {
    let item_w: f32 = (placement.col_start..placement.col_end)
      .map(|c| col_sizes.get(c).copied().unwrap_or(0.0))
      .sum::<f32>()
      + gap_col * ((placement.col_end - placement.col_start) as f32 - 1.0).max(0.0);

    let child_ctx = LayoutContext {
      containing_width: item_w,
      ..*ctx
    };
    let laid = crate::engine::layout_node(child, &child_ctx, Point::new(0.0, 0.0), text_ctx, rects, cache, bump);

    items.push((placement, laid));
  }

  // Determine total rows needed
  let max_row_end = items.iter().map(|(p, _)| p.row_end).max().unwrap_or(0);
  let needed_rows = max_row_end.max(row_defs.len());
  while row_defs.len() < needed_rows {
    row_defs.push(auto_row_size.clone());
  }
  let num_rows = row_defs.len();

  // Compute auto row heights from placed items
  let mut row_auto_sizes = vec![0.0_f32; num_rows];
  for (placement, laid) in &items {
    let span = placement.row_end - placement.row_start;
    let h = laid.outer_height() / span as f32;
    for r in placement.row_start..placement.row_end {
      if r < row_auto_sizes.len() {
        row_auto_sizes[r] = row_auto_sizes[r].max(h);
      }
    }
  }

  let mut row_heights = resolve_tracks(&row_defs, inner_height.unwrap_or(0.0), gap_row, &row_auto_sizes);

  // Resolve fr rows with definite height
  let total_row_gap = gap_row * (num_rows as f32 - 1.0).max(0.0);
  if let Some(ih) = inner_height {
    let fixed_h: f32 = row_heights.iter().sum::<f32>();
    let fr_sum: f32 = row_defs
      .iter()
      .map(|d| match d {
        TrackSize::Fr(f) => *f,
        _ => 0.0,
      })
      .sum();
    if fr_sum > 0.0 {
      let free = (ih - total_row_gap - fixed_h).max(0.0);
      for (i, def) in row_defs.iter().enumerate() {
        if let TrackSize::Fr(f) = def {
          if i < row_heights.len() {
            row_heights[i] = free * f / fr_sum;
          }
        }
      }
    }
  }

  // Apply justify-content / align-content distribution
  let justify_content = css_str(b.style.justify_content);
  let align_content = css_str(b.style.align_content);

  let col_positions = distribute_content(&col_sizes, gap_col, inner_width, justify_content);
  let row_positions = distribute_content(&row_heights, gap_row, inner_height.unwrap_or(0.0), align_content);

  // Phase 2: position items
  for (placement, item) in &mut items {
    let cell_x = col_positions.get(placement.col_start).copied().unwrap_or(0.0);
    let cell_y = row_positions.get(placement.row_start).copied().unwrap_or(0.0);

    let cell_w: f32 = (placement.col_start..placement.col_end)
      .map(|c| col_sizes.get(c).copied().unwrap_or(0.0))
      .sum::<f32>()
      + gap_col * ((placement.col_end - placement.col_start) as f32 - 1.0).max(0.0);
    let cell_h: f32 = (placement.row_start..placement.row_end)
      .map(|r| row_heights.get(r).copied().unwrap_or(0.0))
      .sum::<f32>()
      + gap_row * ((placement.row_end - placement.row_start) as f32 - 1.0).max(0.0);

    let item_align = css_str(item.style.align_self);
    let align = if item_align.is_empty() || item_align == "auto" {
      align_items
    } else {
      item_align
    };
    let item_justify = css_str(item.style.justify_self);
    let justify = if item_justify.is_empty() || item_justify == "auto" {
      justify_items
    } else {
      item_justify
    };

    // Grid default is stretch — resize item to fill cell
    let has_explicit_h = item.style.height.is_some()
      && !matches!(css_str(item.style.height), "auto" | "");
    let stretch_h = matches!(align, "" | "normal" | "stretch") && !has_explicit_h;
    if stretch_h && cell_h > 0.0 {
      let target_h = (cell_h - item.margin.vertical() - item.border.vertical() - item.padding.vertical()).max(0.0);
      item.content.height = target_h;
    }

    let has_explicit_w = item.style.width.is_some()
      && !matches!(css_str(item.style.width), "auto" | "");
    let stretch_w = matches!(justify, "" | "normal" | "stretch") && !has_explicit_w;
    if stretch_w && cell_w > 0.0 {
      let target_w = (cell_w - item.margin.horizontal() - item.border.horizontal() - item.padding.horizontal()).max(0.0);
      item.content.width = target_w;
    }

    let item_w = item.outer_width();
    let item_h = item.outer_height();

    let dx_align = match justify {
      "center" => (cell_w - item_w) / 2.0,
      "end" | "flex-end" => cell_w - item_w,
      _ => 0.0,
    };
    let dy_align = match align {
      "center" => (cell_h - item_h) / 2.0,
      "end" | "flex-end" => cell_h - item_h,
      _ => 0.0,
    };

    let target_x = b.content.x + cell_x + dx_align.max(0.0);
    let target_y = b.content.y + cell_y + dy_align.max(0.0);

    let cur_x = item.content.x - item.padding.left - item.border.left - item.margin.left;
    let cur_y = item.content.y - item.padding.top - item.border.top - item.margin.top;
    let dx = target_x - cur_x;
    let dy = target_y - cur_y;
    if dx.abs() > 0.001 || dy.abs() > 0.001 {
      translate_recursive(item, dx, dy);
    }
  }

  b.children = bumpalo::collections::Vec::from_iter_in(items.into_iter().map(|(_, item)| item), bump);

  let total_h: f32 = row_heights.iter().sum::<f32>() + total_row_gap;
  b.content.height = inner_height.unwrap_or(total_h);
}

fn parse_auto_track_size(value: Option<&CssValue>) -> TrackSize {
  match value {
    Some(CssValue::Dimension {
      value,
      unit: CssUnit::Px,
    }) => TrackSize::Px(*value as f32),
    Some(CssValue::Dimension {
      value,
      unit: CssUnit::Fr,
    }) => TrackSize::Fr(*value as f32),
    _ => TrackSize::Auto,
  }
}

fn distribute_content(sizes: &[f32], gap: f32, available: f32, mode: &str) -> Vec<f32> {
  if sizes.is_empty() || available <= 0.0 {
    return compute_positions(sizes, gap);
  }
  let n = sizes.len() as f32;
  let total_track: f32 = sizes.iter().sum();
  let total_gap = gap * (n - 1.0).max(0.0);
  let used = total_track + total_gap;
  let free = (available - used).max(0.0);

  match mode {
    "center" => {
      let offset = free / 2.0;
      let base = compute_positions(sizes, gap);
      base.iter().map(|p| p + offset).collect()
    }
    "end" | "flex-end" => {
      let base = compute_positions(sizes, gap);
      base.iter().map(|p| p + free).collect()
    }
    "space-between" => {
      if sizes.len() < 2 {
        return compute_positions(sizes, gap);
      }
      let between = free / (n - 1.0) + gap;
      let mut positions = Vec::with_capacity(sizes.len());
      let mut cursor = 0.0;
      for (i, s) in sizes.iter().enumerate() {
        positions.push(cursor);
        cursor += s + if i + 1 < sizes.len() { between } else { 0.0 };
      }
      positions
    }
    "space-evenly" => {
      let slot = free / (n + 1.0);
      let mut positions = Vec::with_capacity(sizes.len());
      let mut cursor = slot;
      for (i, s) in sizes.iter().enumerate() {
        positions.push(cursor);
        cursor += s + slot + if i + 1 < sizes.len() { gap } else { 0.0 };
      }
      positions
    }
    "space-around" => {
      let half = free / (2.0 * n);
      let mut positions = Vec::with_capacity(sizes.len());
      let mut cursor = half;
      for (i, s) in sizes.iter().enumerate() {
        positions.push(cursor);
        cursor += s + 2.0 * half + if i + 1 < sizes.len() { gap } else { 0.0 };
      }
      positions
    }
    _ => compute_positions(sizes, gap),
  }
}

fn compute_positions(sizes: &[f32], gap: f32) -> Vec<f32> {
  let mut positions = Vec::with_capacity(sizes.len());
  let mut cursor = 0.0;
  for (i, s) in sizes.iter().enumerate() {
    positions.push(cursor);
    cursor += s + if i + 1 < sizes.len() { gap } else { 0.0 };
  }
  positions
}

fn translate_recursive(b: &mut LayoutBox, dx: f32, dy: f32) {
  b.content.x += dx;
  b.content.y += dy;
  for child in &mut b.children {
    translate_recursive(child, dx, dy);
  }
}
