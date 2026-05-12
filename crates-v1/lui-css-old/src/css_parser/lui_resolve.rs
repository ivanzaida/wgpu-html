use std::collections::HashMap;

use super::{border::parse_border_pieces, parse_border_style, parse_css_color, parse_css_length, parse_font_weight};
use crate::{
  style::{LuiCalendarStyle, LuiColorPickerStyle, LuiPopupStyle},
  values::ArcStr,
};

// ---------------------------------------------------------------------------
// --lui-popup-* / --lui-color-* vendor property dispatch
// ---------------------------------------------------------------------------

/// Resolve `--lui-popup-*` custom properties into a typed struct.
pub fn resolve_lui_popup_style(props: &HashMap<ArcStr, ArcStr>) -> LuiPopupStyle {
  let mut p = LuiPopupStyle::default();
  for (key, value) in props {
    let Some(sub) = key.strip_prefix("--lui-popup-") else {
      continue;
    };
    match sub {
      "width" => p.width = parse_css_length(value),
      "height" => p.height = parse_css_length(value),
      "background" | "background-color" => p.background_color = parse_css_color(value),
      "color" => p.color = parse_css_color(value),
      "border" => {
        let (w, s, c) = parse_border_pieces(value);
        if let Some(w) = w {
          p.border_top_width = Some(w.clone());
          p.border_right_width = Some(w.clone());
          p.border_bottom_width = Some(w.clone());
          p.border_left_width = Some(w);
        }
        if let Some(s) = s {
          p.border_top_style = Some(s.clone());
          p.border_right_style = Some(s.clone());
          p.border_bottom_style = Some(s.clone());
          p.border_left_style = Some(s);
        }
        if let Some(c) = c {
          p.border_top_color = Some(c.clone());
          p.border_right_color = Some(c.clone());
          p.border_bottom_color = Some(c.clone());
          p.border_left_color = Some(c);
        }
      }
      "border-width" => {
        let w = parse_css_length(value);
        p.border_top_width = w.clone();
        p.border_right_width = w.clone();
        p.border_bottom_width = w.clone();
        p.border_left_width = w;
      }
      "border-style" => {
        let s = parse_border_style(value);
        p.border_top_style = s.clone();
        p.border_right_style = s.clone();
        p.border_bottom_style = s.clone();
        p.border_left_style = s;
      }
      "border-color" => {
        let c = parse_css_color(value);
        p.border_top_color = c.clone();
        p.border_right_color = c.clone();
        p.border_bottom_color = c.clone();
        p.border_left_color = c;
      }
      "border-top-width" => p.border_top_width = parse_css_length(value),
      "border-right-width" => p.border_right_width = parse_css_length(value),
      "border-bottom-width" => p.border_bottom_width = parse_css_length(value),
      "border-left-width" => p.border_left_width = parse_css_length(value),
      "border-top-style" => p.border_top_style = parse_border_style(value),
      "border-right-style" => p.border_right_style = parse_border_style(value),
      "border-bottom-style" => p.border_bottom_style = parse_border_style(value),
      "border-left-style" => p.border_left_style = parse_border_style(value),
      "border-top-color" => p.border_top_color = parse_css_color(value),
      "border-right-color" => p.border_right_color = parse_css_color(value),
      "border-bottom-color" => p.border_bottom_color = parse_css_color(value),
      "border-left-color" => p.border_left_color = parse_css_color(value),
      "border-radius" => p.border_radius = parse_css_length(value),
      "font-size" => p.font_size = parse_css_length(value),
      "font-family" => p.font_family = Some(ArcStr::from(value.as_ref())),
      "font-weight" => p.font_weight = parse_font_weight(value),
      _ => {}
    }
  }
  p
}

/// Resolve `--lui-color-*` custom properties into a typed struct.
pub fn resolve_lui_color_picker_style(props: &HashMap<ArcStr, ArcStr>) -> LuiColorPickerStyle {
  let mut p = LuiColorPickerStyle::default();
  for (key, value) in props {
    let Some(sub) = key.strip_prefix("--lui-color-") else {
      continue;
    };
    match sub {
      "canvas-width" => p.canvas_width = parse_css_length(value),
      "canvas-height" => p.canvas_height = parse_css_length(value),
      "range-height" => p.range_height = parse_css_length(value),
      "range-border-radius" => p.range_border_radius = parse_css_length(value),
      "thumb-width" => p.thumb_width = parse_css_length(value),
      "thumb-height" => p.thumb_height = parse_css_length(value),
      "thumb-color" => p.thumb_color = parse_css_color(value),
      "input-height" => p.input_height = parse_css_length(value),
      "input-background" => p.input_background = parse_css_color(value),
      "input-border-color" => p.input_border_color = parse_css_color(value),
      "input-border-width" => p.input_border_width = parse_css_length(value),
      "input-border-radius" => p.input_border_radius = parse_css_length(value),
      "input-font-size" => p.input_font_size = parse_css_length(value),
      _ => {}
    }
  }
  p
}

/// Resolve `--lui-calendar-*` custom properties into a typed struct.
pub fn resolve_lui_calendar_style(props: &HashMap<ArcStr, ArcStr>) -> LuiCalendarStyle {
  let mut p = LuiCalendarStyle::default();
  for (key, value) in props {
    let Some(sub) = key.strip_prefix("--lui-calendar-") else {
      continue;
    };
    match sub {
      "padding" => p.padding = parse_css_length(value),
      "gap" => p.gap = parse_css_length(value),
      "cell-size" => p.cell_size = parse_css_length(value),
      "cell-radius" => p.cell_radius = parse_css_length(value),
      "selected-bg" | "selected-background" => p.selected_bg = parse_css_color(value),
      "selected-color" => p.selected_color = parse_css_color(value),
      "today-color" => p.today_color = parse_css_color(value),
      "today-width" => p.today_width = parse_css_length(value),
      "dim" | "dim-color" => p.dim = parse_css_color(value),
      "nav-size" => p.nav_size = parse_css_length(value),
      "header-font-size" => p.header_font_size = parse_css_length(value),
      "header-font-weight" => p.header_font_weight = parse_font_weight(value),
      "weekday-font-size" => p.weekday_font_size = parse_css_length(value),
      "day-font-size" => p.day_font_size = parse_css_length(value),
      "time-width" => p.time_width = parse_css_length(value),
      "time-height" => p.time_height = parse_css_length(value),
      "time-background" => p.time_background = parse_css_color(value),
      "time-border-color" => p.time_border_color = parse_css_color(value),
      "time-border-width" => p.time_border_width = parse_css_length(value),
      "time-border-radius" => p.time_border_radius = parse_css_length(value),
      "time-font-size" => p.time_font_size = parse_css_length(value),
      "reset-height" => p.reset_height = parse_css_length(value),
      "reset-background" => p.reset_background = parse_css_color(value),
      "reset-color" => p.reset_color = parse_css_color(value),
      "reset-border-color" => p.reset_border_color = parse_css_color(value),
      "reset-border-width" => p.reset_border_width = parse_css_length(value),
      "reset-border-radius" => p.reset_border_radius = parse_css_length(value),
      "reset-font-size" => p.reset_font_size = parse_css_length(value),
      _ => {}
    }
  }
  p
}
