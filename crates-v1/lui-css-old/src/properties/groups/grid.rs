use super::{property_group::PropertyGroup, warn_none};
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style, values::*};

pub struct GridGroup;

impl PropertyGroup for GridGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      let (p, v) = (&*decl.property, &*decl.value);
      match p {
        "grid-template-columns" => {
          let list = parse_grid_track_list(v);
          if !list.is_empty() {
            style.grid_template_columns = Some(list);
          }
        }
        "grid-template-rows" => {
          let list = parse_grid_track_list(v);
          if !list.is_empty() {
            style.grid_template_rows = Some(list);
          }
        }
        "grid-auto-columns" => style.grid_auto_columns = parse_grid_track_size(v),
        "grid-auto-rows" => style.grid_auto_rows = parse_grid_track_size(v),
        "grid-auto-flow" => style.grid_auto_flow = warn_none(p, v, parse_grid_auto_flow(v)),
        "grid-column" => apply_grid_axis_shorthand(v, style, GridAxis::Column),
        "grid-column-start" => style.grid_column_start = parse_grid_line(v),
        "grid-column-end" => style.grid_column_end = parse_grid_line(v),
        "grid-row" => apply_grid_axis_shorthand(v, style, GridAxis::Row),
        "grid-row-start" => style.grid_row_start = parse_grid_line(v),
        "grid-row-end" => style.grid_row_end = parse_grid_line(v),
        "justify-items" => style.justify_items = warn_none(p, v, parse_justify_items(v)),
        "justify-self" => style.justify_self = warn_none(p, v, parse_justify_self(v)),
        "grid" => apply_grid_shorthand(style, v),
        "grid-template" => apply_grid_template_shorthand(style, v),
        "grid-area" => apply_grid_area_shorthand(v, style),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "grid-template-columns",
      "grid-template-rows",
      "grid-auto-columns",
      "grid-auto-rows",
      "grid-auto-flow",
      "grid-column",
      "grid-column-start",
      "grid-column-end",
      "grid-row",
      "grid-row-start",
      "grid-row-end",
      "justify-items",
      "justify-self",
      "grid",
      "grid-template",
      "grid-area",
    ]
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum GridAxis {
  Column,
  Row,
}

/// Expand `grid-column` / `grid-row` shorthand into the start / end
/// longhands. Accepts:
/// - `<line>` -> start=line, end=auto
/// - `<line> / <line>` -> start, end
/// - `span <n> / <line>` (and the reverse), etc.
fn apply_grid_axis_shorthand(value: &str, style: &mut Style, axis: GridAxis) {
  // Round-trip the raw value for cascade introspection.
  match axis {
    GridAxis::Column => style.grid_column = Some(ArcStr::from(value)),
    GridAxis::Row => style.grid_row = Some(ArcStr::from(value)),
  }
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return;
  }
  let parts: Vec<&str> = trimmed.split('/').map(|p| p.trim()).collect();
  let (start_tok, end_tok) = match parts.len() {
    1 => (parts[0], "auto"),
    _ => (parts[0], parts[1]),
  };
  let start = parse_grid_line(start_tok).unwrap_or(GridLine::Auto);
  let end = parse_grid_line(end_tok).unwrap_or(GridLine::Auto);
  match axis {
    GridAxis::Column => {
      style.grid_column_start = Some(start);
      style.grid_column_end = Some(end);
    }
    GridAxis::Row => {
      style.grid_row_start = Some(start);
      style.grid_row_end = Some(end);
    }
  }
}

fn apply_grid_template_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "grid-template");
  if let Some((rows, cols)) = split_once_top_level(value, '/') {
    let rows_list = parse_grid_track_list(rows.trim());
    if !rows_list.is_empty() {
      style.grid_template_rows = Some(rows_list);
    }
    let cols_list = parse_grid_track_list(cols.trim());
    if !cols_list.is_empty() {
      style.grid_template_columns = Some(cols_list);
    }
  } else {
    let rows_list = parse_grid_track_list(value);
    if !rows_list.is_empty() {
      style.grid_template_rows = Some(rows_list);
    }
  }
  set_deferred(style, "grid-template-areas", value);
}

fn apply_grid_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "grid");
  if let Some((template, auto)) = split_once_top_level(value, '/') {
    apply_grid_template_shorthand(style, template.trim());
    set_deferred(style, "grid-auto-flow", auto.trim());
    set_deferred(style, "grid-auto-rows", auto.trim());
    set_deferred(style, "grid-auto-columns", auto.trim());
  } else {
    apply_grid_template_shorthand(style, value);
  }
}

fn apply_grid_area_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &["grid-row-start", "grid-column-start", "grid-row-end", "grid-column-end"],
  );
  let parts: Vec<&str> = value.split('/').map(str::trim).filter(|p| !p.is_empty()).collect();
  match parts.as_slice() {
    [a] => {
      let line = parse_grid_line(a);
      style.grid_row_start = line.clone();
      style.grid_column_start = line.clone();
      style.grid_row_end = line.clone();
      style.grid_column_end = line;
    }
    [a, b] => {
      style.grid_row_start = parse_grid_line(a);
      style.grid_column_start = parse_grid_line(b);
      style.grid_row_end = parse_grid_line(a);
      style.grid_column_end = parse_grid_line(b);
    }
    [a, b, c] => {
      style.grid_row_start = parse_grid_line(a);
      style.grid_column_start = parse_grid_line(b);
      style.grid_row_end = parse_grid_line(c);
      style.grid_column_end = parse_grid_line(b);
    }
    [a, b, c, d] => {
      style.grid_row_start = parse_grid_line(a);
      style.grid_column_start = parse_grid_line(b);
      style.grid_row_end = parse_grid_line(c);
      style.grid_column_end = parse_grid_line(d);
    }
    _ => {}
  }
}

fn split_once_top_level<'a>(value: &'a str, delim: char) -> Option<(&'a str, &'a str)> {
  let mut depth = 0i32;
  for (idx, ch) in value.char_indices() {
    match ch {
      '(' => depth += 1,
      ')' => depth -= 1,
      _ if ch == delim && depth == 0 => {
        return Some((&value[..idx], &value[idx + ch.len_utf8()..]));
      }
      _ => {}
    }
  }
  None
}
