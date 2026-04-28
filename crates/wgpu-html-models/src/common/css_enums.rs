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

/// CSS Grid: default inline-axis alignment for items inside their
/// grid cell. Mirrors `JustifyContent`'s shape minus the
/// space-* distribution variants (those don't apply to per-item
/// alignment).
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
