//! CSS value → string formatters for the devtools styles panel.

use wgpu_html_models::common::css_enums::*;

pub(crate) fn fmt_length(v: &CssLength) -> String {
  match v {
    CssLength::Px(n) => format!("{n}px"),
    CssLength::Percent(n) => format!("{n}%"),
    CssLength::Em(n) => format!("{n}em"),
    CssLength::Rem(n) => format!("{n}rem"),
    CssLength::Vw(n) => format!("{n}vw"),
    CssLength::Vh(n) => format!("{n}vh"),
    CssLength::Vmin(n) => format!("{n}vmin"),
    CssLength::Vmax(n) => format!("{n}vmax"),
    CssLength::Auto => "auto".into(),
    CssLength::Zero => "0".into(),
    CssLength::Calc(_) => "calc(…)".into(),
    CssLength::Min(parts) => format!("min({})", parts.iter().map(fmt_length).collect::<Vec<_>>().join(", ")),
    CssLength::Max(parts) => format!("max({})", parts.iter().map(fmt_length).collect::<Vec<_>>().join(", ")),
    CssLength::Clamp { min, preferred, max } => {
      format!(
        "clamp({}, {}, {})",
        fmt_length(min),
        fmt_length(preferred),
        fmt_length(max)
      )
    }
    CssLength::Raw(s) => s.clone(),
  }
}

pub(crate) fn fmt_color(v: &CssColor) -> String {
  match v {
    CssColor::Named(s) | CssColor::Hex(s) | CssColor::Function(s) => s.clone(),
    CssColor::Rgb(r, g, b) => format!("rgb({r}, {g}, {b})"),
    CssColor::Rgba(r, g, b, a) => format!("rgba({r}, {g}, {b}, {a})"),
    CssColor::Hsl(h, s, l) => format!("hsl({h}, {s}%, {l}%)"),
    CssColor::Hsla(h, s, l, a) => format!("hsla({h}, {s}%, {l}%, {a})"),
    CssColor::Transparent => "transparent".into(),
    CssColor::CurrentColor => "currentColor".into(),
  }
}

pub(crate) fn fmt_image(v: &CssImage) -> String {
  match v {
    CssImage::Url(s) => format!("url({s})"),
    CssImage::Function(s) => s.clone(),
  }
}

pub(crate) fn fmt_display(v: &Display) -> &'static str {
  match v {
    Display::None => "none",
    Display::Block => "block",
    Display::Inline => "inline",
    Display::InlineBlock => "inline-block",
    Display::ListItem => "list-item",
    Display::Flex => "flex",
    Display::InlineFlex => "inline-flex",
    Display::Grid => "grid",
    Display::InlineGrid => "inline-grid",
    Display::Table => "table",
    Display::TableCaption => "table-caption",
    Display::TableHeaderGroup => "table-header-group",
    Display::TableRowGroup => "table-row-group",
    Display::TableFooterGroup => "table-footer-group",
    Display::TableRow => "table-row",
    Display::TableCell => "table-cell",
    Display::TableColumn => "table-column",
    Display::TableColumnGroup => "table-column-group",
    Display::Ruby => "ruby",
    Display::RubyText => "ruby-text",
    Display::Contents => "contents",
  }
}

pub(crate) fn fmt_position(v: &Position) -> &'static str {
  match v {
    Position::Static => "static",
    Position::Relative => "relative",
    Position::Absolute => "absolute",
    Position::Fixed => "fixed",
    Position::Sticky => "sticky",
  }
}

pub(crate) fn fmt_overflow(v: &Overflow) -> &'static str {
  match v {
    Overflow::Visible => "visible",
    Overflow::Hidden => "hidden",
    Overflow::Clip => "clip",
    Overflow::Scroll => "scroll",
    Overflow::Auto => "auto",
  }
}

pub(crate) fn fmt_box_sizing(v: &BoxSizing) -> &'static str {
  match v {
    BoxSizing::ContentBox => "content-box",
    BoxSizing::BorderBox => "border-box",
  }
}

pub(crate) fn fmt_flex_direction(v: &FlexDirection) -> &'static str {
  match v {
    FlexDirection::Row => "row",
    FlexDirection::RowReverse => "row-reverse",
    FlexDirection::Column => "column",
    FlexDirection::ColumnReverse => "column-reverse",
  }
}

pub(crate) fn fmt_flex_wrap(v: &FlexWrap) -> &'static str {
  match v {
    FlexWrap::Nowrap => "nowrap",
    FlexWrap::Wrap => "wrap",
    FlexWrap::WrapReverse => "wrap-reverse",
  }
}

pub(crate) fn fmt_justify_content(v: &JustifyContent) -> &'static str {
  match v {
    JustifyContent::Start => "start",
    JustifyContent::End => "end",
    JustifyContent::Center => "center",
    JustifyContent::FlexStart => "flex-start",
    JustifyContent::FlexEnd => "flex-end",
    JustifyContent::Left => "left",
    JustifyContent::Right => "right",
    JustifyContent::SpaceBetween => "space-between",
    JustifyContent::SpaceAround => "space-around",
    JustifyContent::SpaceEvenly => "space-evenly",
  }
}

pub(crate) fn fmt_align_items(v: &AlignItems) -> &'static str {
  match v {
    AlignItems::Normal => "normal",
    AlignItems::Stretch => "stretch",
    AlignItems::Center => "center",
    AlignItems::Start => "start",
    AlignItems::End => "end",
    AlignItems::FlexStart => "flex-start",
    AlignItems::FlexEnd => "flex-end",
    AlignItems::Baseline => "baseline",
  }
}

pub(crate) fn fmt_align_content(v: &AlignContent) -> &'static str {
  match v {
    AlignContent::Normal => "normal",
    AlignContent::Stretch => "stretch",
    AlignContent::Center => "center",
    AlignContent::Start => "start",
    AlignContent::End => "end",
    AlignContent::FlexStart => "flex-start",
    AlignContent::FlexEnd => "flex-end",
    AlignContent::SpaceBetween => "space-between",
    AlignContent::SpaceAround => "space-around",
    AlignContent::SpaceEvenly => "space-evenly",
  }
}

pub(crate) fn fmt_align_self(v: &AlignSelf) -> &'static str {
  match v {
    AlignSelf::Auto => "auto",
    AlignSelf::Normal => "normal",
    AlignSelf::Stretch => "stretch",
    AlignSelf::Center => "center",
    AlignSelf::Start => "start",
    AlignSelf::End => "end",
    AlignSelf::FlexStart => "flex-start",
    AlignSelf::FlexEnd => "flex-end",
    AlignSelf::Baseline => "baseline",
  }
}

pub(crate) fn fmt_justify_items(v: &JustifyItems) -> &'static str {
  match v {
    JustifyItems::Normal => "normal",
    JustifyItems::Stretch => "stretch",
    JustifyItems::Center => "center",
    JustifyItems::Start => "start",
    JustifyItems::End => "end",
    JustifyItems::FlexStart => "flex-start",
    JustifyItems::FlexEnd => "flex-end",
    JustifyItems::Left => "left",
    JustifyItems::Right => "right",
    JustifyItems::Baseline => "baseline",
  }
}

pub(crate) fn fmt_justify_self(v: &JustifySelf) -> &'static str {
  match v {
    JustifySelf::Auto => "auto",
    JustifySelf::Normal => "normal",
    JustifySelf::Stretch => "stretch",
    JustifySelf::Center => "center",
    JustifySelf::Start => "start",
    JustifySelf::End => "end",
    JustifySelf::FlexStart => "flex-start",
    JustifySelf::FlexEnd => "flex-end",
    JustifySelf::Left => "left",
    JustifySelf::Right => "right",
    JustifySelf::Baseline => "baseline",
  }
}

pub(crate) fn fmt_grid_auto_flow(v: &GridAutoFlow) -> &'static str {
  match v {
    GridAutoFlow::Row => "row",
    GridAutoFlow::Column => "column",
    GridAutoFlow::RowDense => "row dense",
    GridAutoFlow::ColumnDense => "column dense",
  }
}

pub(crate) fn fmt_grid_track(v: &GridTrackSize) -> String {
  match v {
    GridTrackSize::Length(l) => fmt_length(l),
    GridTrackSize::Auto => "auto".into(),
    GridTrackSize::Fr(n) => format!("{n}fr"),
  }
}

pub(crate) fn fmt_grid_tracks(v: &[GridTrackSize]) -> String {
  v.iter().map(fmt_grid_track).collect::<Vec<_>>().join(" ")
}

pub(crate) fn fmt_grid_line(v: &GridLine) -> String {
  match v {
    GridLine::Auto => "auto".into(),
    GridLine::Line(n) => n.to_string(),
    GridLine::Span(n) => format!("span {n}"),
  }
}

pub(crate) fn fmt_font_weight(v: &FontWeight) -> String {
  match v {
    FontWeight::Normal => "normal".into(),
    FontWeight::Bold => "bold".into(),
    FontWeight::Bolder => "bolder".into(),
    FontWeight::Lighter => "lighter".into(),
    FontWeight::Weight(n) => n.to_string(),
  }
}

pub(crate) fn fmt_font_style(v: &FontStyle) -> &'static str {
  match v {
    FontStyle::Normal => "normal",
    FontStyle::Italic => "italic",
    FontStyle::Oblique => "oblique",
  }
}

pub(crate) fn fmt_text_align(v: &TextAlign) -> &'static str {
  match v {
    TextAlign::Left => "left",
    TextAlign::Right => "right",
    TextAlign::Center => "center",
    TextAlign::Justify => "justify",
    TextAlign::Start => "start",
    TextAlign::End => "end",
  }
}

pub(crate) fn fmt_text_transform(v: &TextTransform) -> &'static str {
  match v {
    TextTransform::None => "none",
    TextTransform::Capitalize => "capitalize",
    TextTransform::Uppercase => "uppercase",
    TextTransform::Lowercase => "lowercase",
  }
}

pub(crate) fn fmt_white_space(v: &WhiteSpace) -> &'static str {
  match v {
    WhiteSpace::Normal => "normal",
    WhiteSpace::Nowrap => "nowrap",
    WhiteSpace::Pre => "pre",
    WhiteSpace::PreWrap => "pre-wrap",
    WhiteSpace::PreLine => "pre-line",
    WhiteSpace::BreakSpaces => "break-spaces",
  }
}

pub(crate) fn fmt_visibility(v: &Visibility) -> &'static str {
  match v {
    Visibility::Visible => "visible",
    Visibility::Hidden => "hidden",
    Visibility::Collapse => "collapse",
  }
}

pub(crate) fn fmt_border_style(v: &BorderStyle) -> &'static str {
  match v {
    BorderStyle::None => "none",
    BorderStyle::Hidden => "hidden",
    BorderStyle::Solid => "solid",
    BorderStyle::Dashed => "dashed",
    BorderStyle::Dotted => "dotted",
    BorderStyle::Double => "double",
    BorderStyle::Groove => "groove",
    BorderStyle::Ridge => "ridge",
    BorderStyle::Inset => "inset",
    BorderStyle::Outset => "outset",
  }
}

pub(crate) fn fmt_background_clip(v: &BackgroundClip) -> &'static str {
  match v {
    BackgroundClip::BorderBox => "border-box",
    BackgroundClip::PaddingBox => "padding-box",
    BackgroundClip::ContentBox => "content-box",
  }
}

pub(crate) fn fmt_background_repeat(v: &BackgroundRepeat) -> &'static str {
  match v {
    BackgroundRepeat::Repeat => "repeat",
    BackgroundRepeat::RepeatX => "repeat-x",
    BackgroundRepeat::RepeatY => "repeat-y",
    BackgroundRepeat::NoRepeat => "no-repeat",
    BackgroundRepeat::Space => "space",
    BackgroundRepeat::Round => "round",
  }
}

pub(crate) fn fmt_cursor(v: &Cursor) -> String {
  match v {
    Cursor::Auto => "auto".into(),
    Cursor::Default => "default".into(),
    Cursor::Pointer => "pointer".into(),
    Cursor::Text => "text".into(),
    Cursor::Move => "move".into(),
    Cursor::NotAllowed => "not-allowed".into(),
    Cursor::Grab => "grab".into(),
    Cursor::Grabbing => "grabbing".into(),
    Cursor::Crosshair => "crosshair".into(),
    Cursor::Wait => "wait".into(),
    Cursor::Help => "help".into(),
    Cursor::Progress => "progress".into(),
    Cursor::None => "none".into(),
    Cursor::Resize => "resize".into(),
    Cursor::ColResize => "col-resize".into(),
    Cursor::RowResize => "row-resize".into(),
    Cursor::Raw(s) => s.clone(),
  }
}

pub(crate) fn fmt_pointer_events(v: &PointerEvents) -> &'static str {
  match v {
    PointerEvents::Auto => "auto",
    PointerEvents::None => "none",
  }
}

pub(crate) fn fmt_user_select(v: &UserSelect) -> &'static str {
  match v {
    UserSelect::Auto => "auto",
    UserSelect::None => "none",
    UserSelect::Text => "text",
    UserSelect::All => "all",
  }
}
