//! Builder DSL for constructing CSS stylesheets programmatically.
//!
//! Mirrors the `el::` module pattern — each rule is built with a
//! chainable builder, then serialised to a CSS string via `.to_css()`.
//!
//! # Example
//!
//! ```ignore
//! use wgpu_html_ui::style::{self, px, pct};
//! use wgpu_html_models::common::css_enums::*;
//!
//! let css = style::sheet([
//!     style::rule("body")
//!         .display(Display::Flex)
//!         .flex_direction(FlexDirection::Column)
//!         .overflow(Overflow::Hidden),
//!     style::rule(".card")
//!         .padding(px(16))
//!         .border_radius(px(8))
//!         .background_color("#1a1a1a"),
//!     style::rule(".card:hover")
//!         .background_color("#2a2a2a"),
//! ]).to_css();
//! ```

use std::fmt::Write;

use wgpu_html_models::common::css_enums::{
    AlignContent, AlignItems, AlignSelf, BackgroundClip, BackgroundRepeat, BorderStyle, BoxSizing,
    Cursor, Display, FlexDirection, FlexWrap, FontStyle, FontWeight, GridAutoFlow, JustifyContent,
    JustifyItems, JustifySelf, Overflow, PointerEvents, Position, TextAlign, TextTransform,
    UserSelect, Visibility, WhiteSpace,
};

// ── Value types ─────────────────────────────────────────────────────────────

/// A CSS length / size value.
#[derive(Debug, Clone)]
pub enum Val {
    Px(f32),
    Pct(f32),
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

impl Val {
    fn to_css_string(&self) -> String {
        match self {
            Val::Px(v) => format!("{v}px"),
            Val::Pct(v) => format!("{v}%"),
            Val::Em(v) => format!("{v}em"),
            Val::Rem(v) => format!("{v}rem"),
            Val::Vw(v) => format!("{v}vw"),
            Val::Vh(v) => format!("{v}vh"),
            Val::Vmin(v) => format!("{v}vmin"),
            Val::Vmax(v) => format!("{v}vmax"),
            Val::Auto => "auto".into(),
            Val::Zero => "0".into(),
            Val::Raw(s) => s.clone(),
        }
    }
}

/// Convenience: allow `px(18)` from integer.
impl From<i32> for Val {
    fn from(v: i32) -> Self {
        Val::Px(v as f32)
    }
}

/// Convenience: allow `px(18.5)` from float.
impl From<f32> for Val {
    fn from(v: f32) -> Self {
        Val::Px(v)
    }
}

// ── Value constructors ──────────────────────────────────────────────────────

/// Helper trait so `px(16)` and `px(16.0)` both work.
pub trait IntoF32 { fn into_f32(self) -> f32; }
impl IntoF32 for f32 { fn into_f32(self) -> f32 { self } }
impl IntoF32 for i32 { fn into_f32(self) -> f32 { self as f32 } }
impl IntoF32 for u32 { fn into_f32(self) -> f32 { self as f32 } }

pub fn px(v: impl IntoF32) -> Val { Val::Px(v.into_f32()) }
pub fn pct(v: impl IntoF32) -> Val { Val::Pct(v.into_f32()) }
pub fn em(v: impl IntoF32) -> Val { Val::Em(v.into_f32()) }
pub fn rem(v: impl IntoF32) -> Val { Val::Rem(v.into_f32()) }
pub fn vw(v: impl IntoF32) -> Val { Val::Vw(v.into_f32()) }
pub fn vh(v: impl IntoF32) -> Val { Val::Vh(v.into_f32()) }
pub fn vmin(v: impl IntoF32) -> Val { Val::Vmin(v.into_f32()) }
pub fn vmax(v: impl IntoF32) -> Val { Val::Vmax(v.into_f32()) }
pub fn auto() -> Val { Val::Auto }
pub fn raw(s: impl Into<String>) -> Val { Val::Raw(s.into()) }

// ── Stylesheet ──────────────────────────────────────────────────────────────

/// A collection of CSS rules.
pub struct Stylesheet {
    rules: Vec<Rule>,
}

impl Stylesheet {
    /// An empty stylesheet (no rules).
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    /// Returns true if there are no rules.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Serialize to a CSS string.
    pub fn to_css(&self) -> String {
        let mut out = String::new();
        for rule in &self.rules {
            rule.write_css(&mut out);
            out.push('\n');
        }
        out
    }

    /// Serialize to CSS with all class selectors prefixed.
    ///
    /// Every `.classname` in a selector becomes `.{prefix}-classname`.
    /// Use with [`Ctx::scoped`] in view() to match.
    pub fn to_css_scoped(&self, prefix: &str) -> String {
        let mut out = String::new();
        for rule in &self.rules {
            rule.write_css_scoped(&mut out, prefix);
            out.push('\n');
        }
        out
    }
}

/// Create a stylesheet from an iterator of rules.
pub fn sheet(rules: impl IntoIterator<Item = Rule>) -> Stylesheet {
    Stylesheet {
        rules: rules.into_iter().collect(),
    }
}

// ── Rule ────────────────────────────────────────────────────────────────────

/// A single CSS rule: selector + declarations.
pub struct Rule {
    selector: String,
    decls: Vec<(String, String)>,
}

/// Create a rule with the given selector.
pub fn rule(selector: impl Into<String>) -> Rule {
    Rule {
        selector: selector.into(),
        decls: Vec::new(),
    }
}

impl Rule {
    fn write_css(&self, out: &mut String) {
        let _ = write!(out, "{} {{\n", self.selector);
        for (prop, val) in &self.decls {
            let _ = write!(out, "    {}: {};\n", prop, val);
        }
        out.push_str("}\n");
    }

    fn write_css_scoped(&self, out: &mut String, prefix: &str) {
        let scoped_sel = scope_selector(&self.selector, prefix);
        let _ = write!(out, "{} {{\n", scoped_sel);
        for (prop, val) in &self.decls {
            let _ = write!(out, "    {}: {};\n", prop, val);
        }
        out.push_str("}\n");
    }

    /// Escape hatch: set any property with a raw string value.
    pub fn prop(mut self, property: impl Into<String>, value: impl Into<String>) -> Self {
        self.decls.push((property.into(), value.into()));
        self
    }

    // ── Layout ──────────────────────────────────────────────────────────

    pub fn display(self, v: Display) -> Self { self.prop("display", css_display(v)) }
    pub fn position(self, v: Position) -> Self { self.prop("position", css_position(v)) }
    pub fn width(self, v: Val) -> Self { self.prop("width", v.to_css_string()) }
    pub fn height(self, v: Val) -> Self { self.prop("height", v.to_css_string()) }
    pub fn min_width(self, v: Val) -> Self { self.prop("min-width", v.to_css_string()) }
    pub fn min_height(self, v: Val) -> Self { self.prop("min-height", v.to_css_string()) }
    pub fn max_width(self, v: Val) -> Self { self.prop("max-width", v.to_css_string()) }
    pub fn max_height(self, v: Val) -> Self { self.prop("max-height", v.to_css_string()) }
    pub fn top(self, v: Val) -> Self { self.prop("top", v.to_css_string()) }
    pub fn right(self, v: Val) -> Self { self.prop("right", v.to_css_string()) }
    pub fn bottom(self, v: Val) -> Self { self.prop("bottom", v.to_css_string()) }
    pub fn left(self, v: Val) -> Self { self.prop("left", v.to_css_string()) }

    // ── Spacing ─────────────────────────────────────────────────────────

    pub fn margin(self, v: Val) -> Self { self.prop("margin", v.to_css_string()) }
    pub fn margin_top(self, v: Val) -> Self { self.prop("margin-top", v.to_css_string()) }
    pub fn margin_right(self, v: Val) -> Self { self.prop("margin-right", v.to_css_string()) }
    pub fn margin_bottom(self, v: Val) -> Self { self.prop("margin-bottom", v.to_css_string()) }
    pub fn margin_left(self, v: Val) -> Self { self.prop("margin-left", v.to_css_string()) }
    pub fn padding(self, v: Val) -> Self { self.prop("padding", v.to_css_string()) }
    pub fn padding_top(self, v: Val) -> Self { self.prop("padding-top", v.to_css_string()) }
    pub fn padding_right(self, v: Val) -> Self { self.prop("padding-right", v.to_css_string()) }
    pub fn padding_bottom(self, v: Val) -> Self { self.prop("padding-bottom", v.to_css_string()) }
    pub fn padding_left(self, v: Val) -> Self { self.prop("padding-left", v.to_css_string()) }

    /// Shorthand: `padding: top right bottom left;`
    pub fn padding_trbl(self, t: Val, r: Val, l: Val, b: Val) -> Self {
        self.prop("padding", format!("{} {} {} {}",
            t.to_css_string(), r.to_css_string(), b.to_css_string(), l.to_css_string()))
    }

    /// Shorthand: `padding: vertical horizontal;`
    pub fn padding_vh(self, vertical: Val, horizontal: Val) -> Self {
        self.prop("padding", format!("{} {}",
            vertical.to_css_string(), horizontal.to_css_string()))
    }

    /// Shorthand: `margin: vertical horizontal;`
    pub fn margin_vh(self, vertical: Val, horizontal: Val) -> Self {
        self.prop("margin", format!("{} {}",
            vertical.to_css_string(), horizontal.to_css_string()))
    }

    // ── Flex ────────────────────────────────────────────────────────────

    pub fn flex_direction(self, v: FlexDirection) -> Self { self.prop("flex-direction", css_flex_direction(v)) }
    pub fn flex_wrap(self, v: FlexWrap) -> Self { self.prop("flex-wrap", css_flex_wrap(v)) }
    pub fn flex_grow(self, v: f32) -> Self { self.prop("flex-grow", v.to_string()) }
    pub fn flex_shrink(self, v: f32) -> Self { self.prop("flex-shrink", v.to_string()) }
    pub fn flex_basis(self, v: Val) -> Self { self.prop("flex-basis", v.to_css_string()) }
    pub fn justify_content(self, v: JustifyContent) -> Self { self.prop("justify-content", css_justify_content(v)) }
    pub fn align_items(self, v: AlignItems) -> Self { self.prop("align-items", css_align_items(v)) }
    pub fn align_content(self, v: AlignContent) -> Self { self.prop("align-content", css_align_content(v)) }
    pub fn align_self(self, v: AlignSelf) -> Self { self.prop("align-self", css_align_self(v)) }
    pub fn gap(self, v: Val) -> Self { self.prop("gap", v.to_css_string()) }
    pub fn row_gap(self, v: Val) -> Self { self.prop("row-gap", v.to_css_string()) }
    pub fn column_gap(self, v: Val) -> Self { self.prop("column-gap", v.to_css_string()) }
    pub fn order(self, v: i32) -> Self { self.prop("order", v.to_string()) }

    // ── Grid ────────────────────────────────────────────────────────────

    pub fn grid_template_columns(self, v: impl Into<String>) -> Self { self.prop("grid-template-columns", v.into()) }
    pub fn grid_template_rows(self, v: impl Into<String>) -> Self { self.prop("grid-template-rows", v.into()) }
    pub fn grid_auto_flow(self, v: GridAutoFlow) -> Self { self.prop("grid-auto-flow", css_grid_auto_flow(v)) }
    pub fn grid_column(self, v: impl Into<String>) -> Self { self.prop("grid-column", v.into()) }
    pub fn grid_row(self, v: impl Into<String>) -> Self { self.prop("grid-row", v.into()) }
    pub fn justify_items(self, v: JustifyItems) -> Self { self.prop("justify-items", css_justify_items(v)) }
    pub fn justify_self(self, v: JustifySelf) -> Self { self.prop("justify-self", css_justify_self(v)) }

    // ── Text ────────────────────────────────────────────────────────────

    pub fn font_family(self, v: impl Into<String>) -> Self { self.prop("font-family", v.into()) }
    pub fn font_size(self, v: Val) -> Self { self.prop("font-size", v.to_css_string()) }
    pub fn font_weight(self, v: FontWeight) -> Self { self.prop("font-weight", css_font_weight(v)) }
    pub fn font_style(self, v: FontStyle) -> Self { self.prop("font-style", css_font_style(v)) }
    pub fn line_height(self, v: Val) -> Self { self.prop("line-height", v.to_css_string()) }
    pub fn letter_spacing(self, v: Val) -> Self { self.prop("letter-spacing", v.to_css_string()) }
    pub fn text_align(self, v: TextAlign) -> Self { self.prop("text-align", css_text_align(v)) }
    pub fn text_transform(self, v: TextTransform) -> Self { self.prop("text-transform", css_text_transform(v)) }
    pub fn text_decoration(self, v: impl Into<String>) -> Self { self.prop("text-decoration", v.into()) }
    pub fn white_space(self, v: WhiteSpace) -> Self { self.prop("white-space", css_white_space(v)) }

    // ── Visual ──────────────────────────────────────────────────────────

    pub fn color(self, v: impl Into<String>) -> Self { self.prop("color", v.into()) }
    pub fn background(self, v: impl Into<String>) -> Self { self.prop("background", v.into()) }
    pub fn background_color(self, v: impl Into<String>) -> Self { self.prop("background-color", v.into()) }
    pub fn background_clip(self, v: BackgroundClip) -> Self { self.prop("background-clip", css_background_clip(v)) }
    pub fn background_repeat(self, v: BackgroundRepeat) -> Self { self.prop("background-repeat", css_background_repeat(v)) }
    pub fn border(self, v: impl Into<String>) -> Self { self.prop("border", v.into()) }
    pub fn border_top(self, v: impl Into<String>) -> Self { self.prop("border-top", v.into()) }
    pub fn border_right(self, v: impl Into<String>) -> Self { self.prop("border-right", v.into()) }
    pub fn border_bottom(self, v: impl Into<String>) -> Self { self.prop("border-bottom", v.into()) }
    pub fn border_left(self, v: impl Into<String>) -> Self { self.prop("border-left", v.into()) }
    pub fn border_style(self, v: BorderStyle) -> Self { self.prop("border-style", css_border_style(v)) }
    pub fn border_radius(self, v: Val) -> Self { self.prop("border-radius", v.to_css_string()) }
    pub fn opacity(self, v: f32) -> Self { self.prop("opacity", v.to_string()) }
    pub fn overflow(self, v: Overflow) -> Self { self.prop("overflow", css_overflow(v)) }
    pub fn overflow_x(self, v: Overflow) -> Self { self.prop("overflow-x", css_overflow(v)) }
    pub fn overflow_y(self, v: Overflow) -> Self { self.prop("overflow-y", css_overflow(v)) }
    pub fn box_shadow(self, v: impl Into<String>) -> Self { self.prop("box-shadow", v.into()) }

    // ── Misc ────────────────────────────────────────────────────────────

    pub fn box_sizing(self, v: BoxSizing) -> Self { self.prop("box-sizing", css_box_sizing(v)) }
    pub fn cursor(self, v: Cursor) -> Self { self.prop("cursor", css_cursor(v)) }
    pub fn pointer_events(self, v: PointerEvents) -> Self { self.prop("pointer-events", css_pointer_events(v)) }
    pub fn user_select(self, v: UserSelect) -> Self { self.prop("user-select", css_user_select(v)) }
    pub fn visibility(self, v: Visibility) -> Self { self.prop("visibility", css_visibility(v)) }
    pub fn z_index(self, v: i32) -> Self { self.prop("z-index", v.to_string()) }
}

// ── Enum → CSS string conversions ───────────────────────────────────────────

fn css_display(v: Display) -> &'static str {
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

fn css_position(v: Position) -> &'static str {
    match v {
        Position::Static => "static",
        Position::Relative => "relative",
        Position::Absolute => "absolute",
        Position::Fixed => "fixed",
        Position::Sticky => "sticky",
    }
}

fn css_flex_direction(v: FlexDirection) -> &'static str {
    match v {
        FlexDirection::Row => "row",
        FlexDirection::RowReverse => "row-reverse",
        FlexDirection::Column => "column",
        FlexDirection::ColumnReverse => "column-reverse",
    }
}

fn css_flex_wrap(v: FlexWrap) -> &'static str {
    match v {
        FlexWrap::Nowrap => "nowrap",
        FlexWrap::Wrap => "wrap",
        FlexWrap::WrapReverse => "wrap-reverse",
    }
}

fn css_justify_content(v: JustifyContent) -> &'static str {
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

fn css_align_items(v: AlignItems) -> &'static str {
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

fn css_align_content(v: AlignContent) -> &'static str {
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

fn css_align_self(v: AlignSelf) -> &'static str {
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

fn css_justify_items(v: JustifyItems) -> &'static str {
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

fn css_justify_self(v: JustifySelf) -> &'static str {
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

fn css_grid_auto_flow(v: GridAutoFlow) -> &'static str {
    match v {
        GridAutoFlow::Row => "row",
        GridAutoFlow::Column => "column",
        GridAutoFlow::RowDense => "row dense",
        GridAutoFlow::ColumnDense => "column dense",
    }
}

fn css_font_weight(v: FontWeight) -> String {
    match v {
        FontWeight::Normal => "normal".into(),
        FontWeight::Bold => "bold".into(),
        FontWeight::Bolder => "bolder".into(),
        FontWeight::Lighter => "lighter".into(),
        FontWeight::Weight(n) => n.to_string(),
    }
}

fn css_font_style(v: FontStyle) -> &'static str {
    match v {
        FontStyle::Normal => "normal",
        FontStyle::Italic => "italic",
        FontStyle::Oblique => "oblique",
    }
}

fn css_text_align(v: TextAlign) -> &'static str {
    match v {
        TextAlign::Left => "left",
        TextAlign::Right => "right",
        TextAlign::Center => "center",
        TextAlign::Justify => "justify",
        TextAlign::Start => "start",
        TextAlign::End => "end",
    }
}

fn css_text_transform(v: TextTransform) -> &'static str {
    match v {
        TextTransform::None => "none",
        TextTransform::Capitalize => "capitalize",
        TextTransform::Uppercase => "uppercase",
        TextTransform::Lowercase => "lowercase",
    }
}

fn css_white_space(v: WhiteSpace) -> &'static str {
    match v {
        WhiteSpace::Normal => "normal",
        WhiteSpace::Nowrap => "nowrap",
        WhiteSpace::Pre => "pre",
        WhiteSpace::PreWrap => "pre-wrap",
        WhiteSpace::PreLine => "pre-line",
        WhiteSpace::BreakSpaces => "break-spaces",
    }
}

fn css_overflow(v: Overflow) -> &'static str {
    match v {
        Overflow::Visible => "visible",
        Overflow::Hidden => "hidden",
        Overflow::Clip => "clip",
        Overflow::Scroll => "scroll",
        Overflow::Auto => "auto",
    }
}

fn css_background_clip(v: BackgroundClip) -> &'static str {
    match v {
        BackgroundClip::BorderBox => "border-box",
        BackgroundClip::PaddingBox => "padding-box",
        BackgroundClip::ContentBox => "content-box",
    }
}

fn css_background_repeat(v: BackgroundRepeat) -> &'static str {
    match v {
        BackgroundRepeat::Repeat => "repeat",
        BackgroundRepeat::RepeatX => "repeat-x",
        BackgroundRepeat::RepeatY => "repeat-y",
        BackgroundRepeat::NoRepeat => "no-repeat",
        BackgroundRepeat::Space => "space",
        BackgroundRepeat::Round => "round",
    }
}

fn css_border_style(v: BorderStyle) -> &'static str {
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

fn css_box_sizing(v: BoxSizing) -> &'static str {
    match v {
        BoxSizing::ContentBox => "content-box",
        BoxSizing::BorderBox => "border-box",
    }
}

fn css_cursor(v: Cursor) -> String {
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
        Cursor::Raw(s) => s,
    }
}

fn css_pointer_events(v: PointerEvents) -> &'static str {
    match v {
        PointerEvents::Auto => "auto",
        PointerEvents::None => "none",
    }
}

fn css_user_select(v: UserSelect) -> &'static str {
    match v {
        UserSelect::Auto => "auto",
        UserSelect::None => "none",
        UserSelect::Text => "text",
        UserSelect::All => "all",
    }
}

fn css_visibility(v: Visibility) -> &'static str {
    match v {
        Visibility::Visible => "visible",
        Visibility::Hidden => "hidden",
        Visibility::Collapse => "collapse",
    }
}

// ── Scoping helpers ─────────────────────────────────────────────────────────

/// Prefix all class selectors in a selector string.
///
/// `.tree-row:hover` → `.{prefix}-tree-row:hover`
/// `.main .item` → `.{prefix}-main .{prefix}-item`
/// `body` → `body` (no dot → no change)
fn scope_selector(selector: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(selector.len() + 32);
    let mut chars = selector.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '.' {
            // Check it's a class selector (not part of a decimal number)
            // by looking at what follows: must be a letter, underscore, or hyphen
            if chars
                .peek()
                .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-')
            {
                out.push('.');
                out.push_str(prefix);
                out.push('-');
            } else {
                out.push('.');
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Build the scoped class name for use in `el::div().class(...)`.
///
/// `scoped_class("dt", "tree-row")` → `"dt-tree-row"`
pub fn scoped_class(prefix: &str, class: &str) -> String {
    format!("{prefix}-{class}")
}

/// Build multiple scoped class names separated by spaces.
///
/// `scoped_classes("dt", &["tree-row", "selected"])` → `"dt-tree-row dt-selected"`
pub fn scoped_classes(prefix: &str, classes: &[&str]) -> String {
    classes
        .iter()
        .map(|c| format!("{prefix}-{c}"))
        .collect::<Vec<_>>()
        .join(" ")
}
