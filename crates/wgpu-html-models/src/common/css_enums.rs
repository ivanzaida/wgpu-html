use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub enum CssLength {
  Px(f32),
  Percent(f32),
  Em(f32),
  Rem(f32),
  Vw(f32),
  Vh(f32),
  Vmin(f32),
  Vmax(f32),
  Auto,
  Zero,
  Calc(Box<CssMathExpr>),
  Min(Vec<CssLength>),
  Max(Vec<CssLength>),
  Clamp {
    min: Box<CssLength>,
    preferred: Box<CssLength>,
    max: Box<CssLength>,
  },
  Raw(String),
}

#[derive(Debug, Clone)]
pub enum CssMathExpr {
  Length(CssLength),
  Number(f32),
  Add(Box<CssMathExpr>, Box<CssMathExpr>),
  Sub(Box<CssMathExpr>, Box<CssMathExpr>),
  Mul(Box<CssMathExpr>, Box<CssMathExpr>),
  Div(Box<CssMathExpr>, Box<CssMathExpr>),
  Function(CssNumericFunction, Vec<CssMathExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssNumericFunction {
  Sin,
  Cos,
  Tan,
  Asin,
  Acos,
  Atan,
  Atan2,
  Pow,
  Sqrt,
  Hypot,
  Log,
  Exp,
  Abs,
  Sign,
  Mod,
  Rem,
  Round,
}

#[derive(Debug, Clone)]
pub enum CssColor {
  Named(String),
  Hex(String),
  Rgb(u8, u8, u8),
  Rgba(u8, u8, u8, f32),
  Hsl(f32, f32, f32),
  Hsla(f32, f32, f32, f32),
  Transparent,
  CurrentColor,
  Function(String),
}

#[derive(Debug, Clone)]
pub enum CssImage {
  Url(String),
  Function(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
  None,
  Block,
  Inline,
  InlineBlock,
  ListItem,
  Flex,
  InlineFlex,
  Grid,
  InlineGrid,
  Table,
  TableCaption,
  TableHeaderGroup,
  TableRowGroup,
  TableFooterGroup,
  TableRow,
  TableCell,
  TableColumn,
  TableColumnGroup,
  Ruby,
  RubyText,
  Contents,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
  Static,
  Relative,
  Absolute,
  Fixed,
  Sticky,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundRepeat {
  Repeat,
  RepeatX,
  RepeatY,
  NoRepeat,
  Space,
  Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
  None,
  Hidden,
  Solid,
  Dashed,
  Dotted,
  Double,
  Groove,
  Ridge,
  Inset,
  Outset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
  Normal,
  Bold,
  Bolder,
  Lighter,
  Weight(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
  Normal,
  Italic,
  Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
  Left,
  Right,
  Center,
  Justify,
  Start,
  End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTransform {
  None,
  Capitalize,
  Uppercase,
  Lowercase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteSpace {
  Normal,
  Nowrap,
  Pre,
  PreWrap,
  PreLine,
  BreakSpaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
  Visible,
  Hidden,
  Clip,
  Scroll,
  Auto,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollbarWidth {
  Auto,
  Thin,
  None,
  Px(f32),
}

#[derive(Debug, Clone)]
pub enum ScrollbarColor {
  Auto,
  Custom { thumb: CssColor, track: CssColor },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
  Visible,
  Hidden,
  Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
  Row,
  RowReverse,
  Column,
  ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
  Nowrap,
  Wrap,
  WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
  Start,
  End,
  Center,
  FlexStart,
  FlexEnd,
  Left,
  Right,
  SpaceBetween,
  SpaceAround,
  SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
  Normal,
  Stretch,
  Center,
  Start,
  End,
  FlexStart,
  FlexEnd,
  Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
  Normal,
  Stretch,
  Center,
  Start,
  End,
  FlexStart,
  FlexEnd,
  SpaceBetween,
  SpaceAround,
  SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignSelf {
  /// Defer to the parent flex container's `align-items`.
  Auto,
  Normal,
  Stretch,
  Center,
  Start,
  End,
  FlexStart,
  FlexEnd,
  Baseline,
}

/// CSS Grid: default inline-axis alignment for items inside their
/// grid cell. Mirrors `JustifyContent`'s shape minus the
/// space-* distribution variants (those don't apply to per-item
/// alignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyItems {
  Normal,
  Stretch,
  Center,
  Start,
  End,
  FlexStart,
  FlexEnd,
  Left,
  Right,
  Baseline,
}

/// CSS Grid: per-item override of `justify-items`. Shares the same
/// shape but adds the `Auto` variant that means "defer to parent's
/// `justify-items`".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifySelf {
  Auto,
  Normal,
  Stretch,
  Center,
  Start,
  End,
  FlexStart,
  FlexEnd,
  Left,
  Right,
  Baseline,
}

/// `grid-auto-flow` direction. `RowDense` / `ColumnDense` accept
/// the value through the cascade for fidelity but currently lay
/// out the same as the non-dense variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridAutoFlow {
  Row,
  Column,
  RowDense,
  ColumnDense,
}

/// One entry in `grid-template-columns` / `grid-template-rows` (or
/// the implicit-track sizes). Kept distinct from `CssLength`
/// because the `<flex>` (`fr`) unit is only legal in track-sizing
/// contexts.
#[derive(Debug, Clone)]
pub enum GridTrackSize {
  /// A length / percent / em / etc. — anything `CssLength` can
  /// represent.
  Length(CssLength),
  /// `auto` — track sized to the max-content of items spanning it
  /// (or the implicit `grid-auto-rows` / `-columns` size).
  Auto,
  /// `<flex>` (1fr, 2fr, …). Stored as the raw factor.
  Fr(f32),
}

/// One end of a `grid-row` / `grid-column` placement.
///
/// We deliberately don't model named lines yet — those need a
/// stylesheet-wide line-name registry that doesn't exist in the
/// engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridLine {
  /// `auto` — the value is decided by the auto-placement algorithm.
  Auto,
  /// 1-based explicit line number, matching CSS conventions
  /// (`grid-column: 1 / 3` covers columns 1 and 2).
  Line(i32),
  /// `span <N>` — extends `N` tracks from the resolved opposite
  /// edge. Stored as a positive integer; `0` is invalid in CSS so
  /// we never produce one.
  Span(u32),
}

#[derive(Debug, Clone)]
pub enum Cursor {
  Auto,
  Default,
  Pointer,
  Text,
  Move,
  NotAllowed,
  Grab,
  Grabbing,
  Crosshair,
  Wait,
  Help,
  Progress,
  None,
  Resize,
  ColResize,
  RowResize,
  Raw(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerEvents {
  Auto,
  None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserSelect {
  Auto,
  None,
  Text,
  All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxSizing {
  ContentBox,
  BorderBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundClip {
  BorderBox,
  PaddingBox,
  ContentBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resize {
  None,
  Both,
  Horizontal,
  Vertical,
}

impl fmt::Display for CssLength {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CssLength::Px(value) => write!(f, "{value}px"),
      CssLength::Percent(value) => write!(f, "{value}%"),
      CssLength::Em(value) => write!(f, "{value}em"),
      CssLength::Rem(value) => write!(f, "{value}rem"),
      CssLength::Vw(value) => write!(f, "{value}vw"),
      CssLength::Vh(value) => write!(f, "{value}vh"),
      CssLength::Vmin(value) => write!(f, "{value}vmin"),
      CssLength::Vmax(value) => write!(f, "{value}vmax"),
      CssLength::Auto => f.write_str("auto"),
      CssLength::Zero => f.write_str("0"),
      CssLength::Calc(expr) => write!(f, "calc({expr})"),
      CssLength::Min(parts) => write_joined(f, "min(", parts, ", ", ")"),
      CssLength::Max(parts) => write_joined(f, "max(", parts, ", ", ")"),
      CssLength::Clamp { min, preferred, max } => {
        write!(f, "clamp({min}, {preferred}, {max})")
      }
      CssLength::Raw(value) => f.write_str(value),
    }
  }
}

impl fmt::Display for CssMathExpr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CssMathExpr::Length(value) => write!(f, "{value}"),
      CssMathExpr::Number(value) => write!(f, "{value}"),
      CssMathExpr::Add(left, right) => write!(f, "{left} + {right}"),
      CssMathExpr::Sub(left, right) => write!(f, "{left} - {right}"),
      CssMathExpr::Mul(left, right) => write!(f, "{left} * {right}"),
      CssMathExpr::Div(left, right) => write!(f, "{left} / {right}"),
      CssMathExpr::Function(name, args) => write_joined(f, &format!("{name}("), args, ", ", ")"),
    }
  }
}

impl fmt::Display for CssColor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CssColor::Named(value) | CssColor::Hex(value) | CssColor::Function(value) => f.write_str(value),
      CssColor::Rgb(r, g, b) => write!(f, "rgb({r}, {g}, {b})"),
      CssColor::Rgba(r, g, b, a) => write!(f, "rgba({r}, {g}, {b}, {a})"),
      CssColor::Hsl(h, s, l) => write!(f, "hsl({h}, {s}%, {l}%)"),
      CssColor::Hsla(h, s, l, a) => write!(f, "hsla({h}, {s}%, {l}%, {a})"),
      CssColor::Transparent => f.write_str("transparent"),
      CssColor::CurrentColor => f.write_str("currentColor"),
    }
  }
}

impl fmt::Display for CssImage {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      CssImage::Url(value) => write!(f, "url({value})"),
      CssImage::Function(value) => f.write_str(value),
    }
  }
}

macro_rules! css_keyword_enum {
  ($ty:ty { $($variant:path => $css:literal),+ $(,)? }) => {
    impl $ty {
      pub fn as_css_str(&self) -> &'static str {
        match self {
          $($variant => $css),+
        }
      }
    }

    impl fmt::Display for $ty {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_css_str())
      }
    }

    impl FromStr for $ty {
      type Err = ();

      fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();
        $(
          if value.eq_ignore_ascii_case($css) {
            return Ok($variant);
          }
        )+
        Err(())
      }
    }
  };
}

css_keyword_enum!(CssNumericFunction {
  CssNumericFunction::Sin => "sin",
  CssNumericFunction::Cos => "cos",
  CssNumericFunction::Tan => "tan",
  CssNumericFunction::Asin => "asin",
  CssNumericFunction::Acos => "acos",
  CssNumericFunction::Atan => "atan",
  CssNumericFunction::Atan2 => "atan2",
  CssNumericFunction::Pow => "pow",
  CssNumericFunction::Sqrt => "sqrt",
  CssNumericFunction::Hypot => "hypot",
  CssNumericFunction::Log => "log",
  CssNumericFunction::Exp => "exp",
  CssNumericFunction::Abs => "abs",
  CssNumericFunction::Sign => "sign",
  CssNumericFunction::Mod => "mod",
  CssNumericFunction::Rem => "rem",
  CssNumericFunction::Round => "round",
});

css_keyword_enum!(Display {
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
});

css_keyword_enum!(Position {
  Position::Static => "static",
  Position::Relative => "relative",
  Position::Absolute => "absolute",
  Position::Fixed => "fixed",
  Position::Sticky => "sticky",
});

css_keyword_enum!(BackgroundRepeat {
  BackgroundRepeat::Repeat => "repeat",
  BackgroundRepeat::RepeatX => "repeat-x",
  BackgroundRepeat::RepeatY => "repeat-y",
  BackgroundRepeat::NoRepeat => "no-repeat",
  BackgroundRepeat::Space => "space",
  BackgroundRepeat::Round => "round",
});

css_keyword_enum!(BorderStyle {
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
});

impl fmt::Display for FontWeight {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      FontWeight::Normal => f.write_str("normal"),
      FontWeight::Bold => f.write_str("bold"),
      FontWeight::Bolder => f.write_str("bolder"),
      FontWeight::Lighter => f.write_str("lighter"),
      FontWeight::Weight(value) => write!(f, "{value}"),
    }
  }
}

impl FromStr for FontWeight {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    match value.trim().to_ascii_lowercase().as_str() {
      "normal" => Ok(FontWeight::Normal),
      "bold" => Ok(FontWeight::Bold),
      "bolder" => Ok(FontWeight::Bolder),
      "lighter" => Ok(FontWeight::Lighter),
      other => other.parse::<u16>().map(FontWeight::Weight).map_err(|_| ()),
    }
  }
}

css_keyword_enum!(FontStyle {
  FontStyle::Normal => "normal",
  FontStyle::Italic => "italic",
  FontStyle::Oblique => "oblique",
});

css_keyword_enum!(TextAlign {
  TextAlign::Left => "left",
  TextAlign::Right => "right",
  TextAlign::Center => "center",
  TextAlign::Justify => "justify",
  TextAlign::Start => "start",
  TextAlign::End => "end",
});

css_keyword_enum!(TextTransform {
  TextTransform::None => "none",
  TextTransform::Capitalize => "capitalize",
  TextTransform::Uppercase => "uppercase",
  TextTransform::Lowercase => "lowercase",
});

css_keyword_enum!(WhiteSpace {
  WhiteSpace::Normal => "normal",
  WhiteSpace::Nowrap => "nowrap",
  WhiteSpace::Pre => "pre",
  WhiteSpace::PreWrap => "pre-wrap",
  WhiteSpace::PreLine => "pre-line",
  WhiteSpace::BreakSpaces => "break-spaces",
});

css_keyword_enum!(Overflow {
  Overflow::Visible => "visible",
  Overflow::Hidden => "hidden",
  Overflow::Clip => "clip",
  Overflow::Scroll => "scroll",
  Overflow::Auto => "auto",
});

impl fmt::Display for ScrollbarWidth {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ScrollbarWidth::Auto => f.write_str("auto"),
      ScrollbarWidth::Thin => f.write_str("thin"),
      ScrollbarWidth::None => f.write_str("none"),
      ScrollbarWidth::Px(value) => write!(f, "{value}px"),
    }
  }
}

impl FromStr for ScrollbarWidth {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let value = value.trim();
    match value.to_ascii_lowercase().as_str() {
      "auto" => Ok(ScrollbarWidth::Auto),
      "thin" => Ok(ScrollbarWidth::Thin),
      "none" => Ok(ScrollbarWidth::None),
      other => other
        .strip_suffix("px")
        .and_then(|value| value.trim().parse::<f32>().ok())
        .map(ScrollbarWidth::Px)
        .ok_or(()),
    }
  }
}

css_keyword_enum!(Visibility {
  Visibility::Visible => "visible",
  Visibility::Hidden => "hidden",
  Visibility::Collapse => "collapse",
});

css_keyword_enum!(FlexDirection {
  FlexDirection::Row => "row",
  FlexDirection::RowReverse => "row-reverse",
  FlexDirection::Column => "column",
  FlexDirection::ColumnReverse => "column-reverse",
});

css_keyword_enum!(FlexWrap {
  FlexWrap::Nowrap => "nowrap",
  FlexWrap::Wrap => "wrap",
  FlexWrap::WrapReverse => "wrap-reverse",
});

css_keyword_enum!(JustifyContent {
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
});

css_keyword_enum!(AlignItems {
  AlignItems::Normal => "normal",
  AlignItems::Stretch => "stretch",
  AlignItems::Center => "center",
  AlignItems::Start => "start",
  AlignItems::End => "end",
  AlignItems::FlexStart => "flex-start",
  AlignItems::FlexEnd => "flex-end",
  AlignItems::Baseline => "baseline",
});

css_keyword_enum!(AlignContent {
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
});

css_keyword_enum!(AlignSelf {
  AlignSelf::Auto => "auto",
  AlignSelf::Normal => "normal",
  AlignSelf::Stretch => "stretch",
  AlignSelf::Center => "center",
  AlignSelf::Start => "start",
  AlignSelf::End => "end",
  AlignSelf::FlexStart => "flex-start",
  AlignSelf::FlexEnd => "flex-end",
  AlignSelf::Baseline => "baseline",
});

css_keyword_enum!(JustifyItems {
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
});

css_keyword_enum!(JustifySelf {
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
});

impl GridAutoFlow {
  pub fn as_css_str(&self) -> &'static str {
    match self {
      GridAutoFlow::Row => "row",
      GridAutoFlow::Column => "column",
      GridAutoFlow::RowDense => "row dense",
      GridAutoFlow::ColumnDense => "column dense",
    }
  }
}

impl fmt::Display for GridAutoFlow {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.as_css_str())
  }
}

impl FromStr for GridAutoFlow {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
      "row" => Ok(GridAutoFlow::Row),
      "column" => Ok(GridAutoFlow::Column),
      "dense" | "row dense" | "dense row" => Ok(GridAutoFlow::RowDense),
      "column dense" | "dense column" => Ok(GridAutoFlow::ColumnDense),
      _ => Err(()),
    }
  }
}

impl fmt::Display for GridLine {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      GridLine::Auto => f.write_str("auto"),
      GridLine::Line(value) => write!(f, "{value}"),
      GridLine::Span(value) => write!(f, "span {value}"),
    }
  }
}

impl FromStr for GridLine {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("auto") {
      return Ok(GridLine::Auto);
    }
    let tokens: Vec<&str> = value.split_whitespace().collect();
    if tokens.len() == 2 && tokens[0].eq_ignore_ascii_case("span") {
      return tokens[1]
        .parse::<u32>()
        .ok()
        .filter(|value| *value >= 1)
        .map(GridLine::Span)
        .ok_or(());
    }
    if tokens.len() == 1 {
      return tokens[0]
        .parse::<i32>()
        .ok()
        .filter(|value| *value != 0)
        .map(GridLine::Line)
        .ok_or(());
    }
    Err(())
  }
}

impl fmt::Display for GridTrackSize {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      GridTrackSize::Length(value) => write!(f, "{value}"),
      GridTrackSize::Auto => f.write_str("auto"),
      GridTrackSize::Fr(value) => write!(f, "{value}fr"),
    }
  }
}

impl fmt::Display for Cursor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Cursor::Auto => f.write_str("auto"),
      Cursor::Default => f.write_str("default"),
      Cursor::Pointer => f.write_str("pointer"),
      Cursor::Text => f.write_str("text"),
      Cursor::Move => f.write_str("move"),
      Cursor::NotAllowed => f.write_str("not-allowed"),
      Cursor::Grab => f.write_str("grab"),
      Cursor::Grabbing => f.write_str("grabbing"),
      Cursor::Crosshair => f.write_str("crosshair"),
      Cursor::Wait => f.write_str("wait"),
      Cursor::Help => f.write_str("help"),
      Cursor::Progress => f.write_str("progress"),
      Cursor::None => f.write_str("none"),
      Cursor::Resize => f.write_str("resize"),
      Cursor::ColResize => f.write_str("col-resize"),
      Cursor::RowResize => f.write_str("row-resize"),
      Cursor::Raw(value) => f.write_str(value),
    }
  }
}

impl FromStr for Cursor {
  type Err = ();

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    let trimmed = value.trim();
    match trimmed.to_ascii_lowercase().as_str() {
      "auto" => Ok(Cursor::Auto),
      "default" => Ok(Cursor::Default),
      "pointer" => Ok(Cursor::Pointer),
      "text" => Ok(Cursor::Text),
      "move" => Ok(Cursor::Move),
      "not-allowed" => Ok(Cursor::NotAllowed),
      "grab" => Ok(Cursor::Grab),
      "grabbing" => Ok(Cursor::Grabbing),
      "crosshair" => Ok(Cursor::Crosshair),
      "wait" => Ok(Cursor::Wait),
      "help" => Ok(Cursor::Help),
      "progress" => Ok(Cursor::Progress),
      "none" => Ok(Cursor::None),
      "resize" => Ok(Cursor::Resize),
      "col-resize" => Ok(Cursor::ColResize),
      "row-resize" => Ok(Cursor::RowResize),
      _ => Ok(Cursor::Raw(trimmed.to_string())),
    }
  }
}

css_keyword_enum!(PointerEvents {
  PointerEvents::Auto => "auto",
  PointerEvents::None => "none",
});

css_keyword_enum!(UserSelect {
  UserSelect::Auto => "auto",
  UserSelect::None => "none",
  UserSelect::Text => "text",
  UserSelect::All => "all",
});

css_keyword_enum!(BoxSizing {
  BoxSizing::ContentBox => "content-box",
  BoxSizing::BorderBox => "border-box",
});

css_keyword_enum!(BackgroundClip {
  BackgroundClip::BorderBox => "border-box",
  BackgroundClip::PaddingBox => "padding-box",
  BackgroundClip::ContentBox => "content-box",
});

css_keyword_enum!(Resize {
  Resize::None => "none",
  Resize::Both => "both",
  Resize::Horizontal => "horizontal",
  Resize::Vertical => "vertical",
});

fn write_joined<T: fmt::Display>(
  f: &mut fmt::Formatter<'_>,
  prefix: &str,
  values: &[T],
  separator: &str,
  suffix: &str,
) -> fmt::Result {
  f.write_str(prefix)?;
  for (index, value) in values.iter().enumerate() {
    if index > 0 {
      f.write_str(separator)?;
    }
    write!(f, "{value}")?;
  }
  f.write_str(suffix)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn css_keyword_enums_round_trip_kebab_case() {
    let parsed: JustifyContent = "space-between".parse().unwrap();
    assert_eq!(parsed, JustifyContent::SpaceBetween);
    assert_eq!(parsed.to_string(), "space-between");

    let parsed: FlexDirection = "ROW-REVERSE".parse().unwrap();
    assert_eq!(parsed, FlexDirection::RowReverse);
    assert_eq!(parsed.to_string(), "row-reverse");

    let parsed: Display = " inline-grid ".parse().unwrap();
    assert_eq!(parsed, Display::InlineGrid);
    assert_eq!(parsed.to_string(), "inline-grid");
  }

  #[test]
  fn value_backed_enums_round_trip() {
    let weight: FontWeight = "700".parse().unwrap();
    assert_eq!(weight, FontWeight::Weight(700));
    assert_eq!(weight.to_string(), "700");

    let width: ScrollbarWidth = "4px".parse().unwrap();
    assert_eq!(width, ScrollbarWidth::Px(4.0));
    assert_eq!(width.to_string(), "4px");

    let line: GridLine = "span 3".parse().unwrap();
    assert_eq!(line, GridLine::Span(3));
    assert_eq!(line.to_string(), "span 3");
  }

  #[test]
  fn grid_auto_flow_accepts_dense_order_variants() {
    assert_eq!("dense".parse::<GridAutoFlow>().unwrap(), GridAutoFlow::RowDense);
    assert_eq!(
      "dense column".parse::<GridAutoFlow>().unwrap(),
      GridAutoFlow::ColumnDense
    );
    assert_eq!(GridAutoFlow::ColumnDense.to_string(), "column dense");
  }

  #[test]
  fn css_value_types_format_as_css() {
    assert_eq!(CssLength::Px(12.0).to_string(), "12px");
    assert_eq!(CssColor::Rgba(1, 2, 3, 0.5).to_string(), "rgba(1, 2, 3, 0.5)");
    assert_eq!(CssImage::Url("a.png".into()).to_string(), "url(a.png)");
    assert_eq!(GridTrackSize::Fr(2.0).to_string(), "2fr");
  }
}
