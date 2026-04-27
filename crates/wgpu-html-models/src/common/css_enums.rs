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
  Raw(String),
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
}

#[derive(Debug, Clone)]
pub enum Display {
  None,
  Block,
  Inline,
  InlineBlock,
  Flex,
  InlineFlex,
  Grid,
  InlineGrid,
  Contents,
}

#[derive(Debug, Clone)]
pub enum Position {
  Static,
  Relative,
  Absolute,
  Fixed,
  Sticky,
}

#[derive(Debug, Clone)]
pub enum BackgroundRepeat {
  Repeat,
  RepeatX,
  RepeatY,
  NoRepeat,
  Space,
  Round,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum FontWeight {
  Normal,
  Bold,
  Bolder,
  Lighter,
  Weight(u16),
}

#[derive(Debug, Clone)]
pub enum FontStyle {
  Normal,
  Italic,
  Oblique,
}

#[derive(Debug, Clone)]
pub enum TextAlign {
  Left,
  Right,
  Center,
  Justify,
  Start,
  End,
}

#[derive(Debug, Clone)]
pub enum TextTransform {
  None,
  Capitalize,
  Uppercase,
  Lowercase,
}

#[derive(Debug, Clone)]
pub enum WhiteSpace {
  Normal,
  Nowrap,
  Pre,
  PreWrap,
  PreLine,
  BreakSpaces,
}

#[derive(Debug, Clone)]
pub enum Overflow {
  Visible,
  Hidden,
  Clip,
  Scroll,
  Auto,
}

#[derive(Debug, Clone)]
pub enum Visibility {
  Visible,
  Hidden,
  Collapse,
}

#[derive(Debug, Clone)]
pub enum FlexDirection {
  Row,
  RowReverse,
  Column,
  ColumnReverse,
}

#[derive(Debug, Clone)]
pub enum FlexWrap {
  Nowrap,
  Wrap,
  WrapReverse,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
  Raw(String),
}

#[derive(Debug, Clone)]
pub enum PointerEvents {
  Auto,
  None,
}

#[derive(Debug, Clone)]
pub enum UserSelect {
  Auto,
  None,
  Text,
  All,
}

#[derive(Debug, Clone)]
pub enum BoxSizing {
  ContentBox,
  BorderBox,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundClip {
  BorderBox,
  PaddingBox,
  ContentBox,
}
