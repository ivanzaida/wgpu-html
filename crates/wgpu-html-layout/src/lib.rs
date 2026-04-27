//! Block layout.
//!
//! Walks a `Tree` and produces a `LayoutBox` tree positioned in physical
//! pixels. The renderer (or paint pass) consumes this directly — it never
//! re-resolves CSS lengths.
//!
//! Scope (M4):
//! - Block formatting context only: every element stacks vertically inside
//!   its parent's content box.
//! - Margin and padding (per-side or shorthand) are honoured.
//! - Width auto-fills the parent's content width; height auto-fits content.
//! - Borders are not drawn yet (treated as zero); inline / flex / floats
//!   come in later milestones.
//! - Text nodes contribute zero height; M5 brings real text layout.

use wgpu_html_models::Style;
use wgpu_html_models::common::css_enums::CssLength;
use wgpu_html_parser::parse_inline_style;
use wgpu_html_tree::{Element, Node, Tree};

mod color;
mod length;
mod style_attr;

pub use color::{Color, resolve_color};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Axis-aligned rectangle in physical pixels, top-left origin.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

/// Per-side insets in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Insets {
    pub const fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

/// One laid-out box. Coordinates are absolute (already translated for
/// every parent on the path); the paint pass just reads them.
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Includes margin. Used for sibling stacking.
    pub margin_rect: Rect,
    /// The rect a background / border would paint (margin excluded).
    pub border_rect: Rect,
    /// The rect children are laid out into (margin + border + padding excluded).
    pub content_rect: Rect,
    /// Resolved background color, if any.
    pub background: Option<Color>,
    pub kind: BoxKind,
    pub children: Vec<LayoutBox>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxKind {
    /// A regular block element.
    Block,
    /// A text leaf. Contains no laid-out content yet.
    Text,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Lay the tree out into a viewport of `viewport_w × viewport_h` physical
/// pixels. The returned root box's `margin_rect` covers the viewport.
pub fn layout(tree: &Tree, viewport_w: f32, viewport_h: f32) -> Option<LayoutBox> {
    let root = tree.root.as_ref()?;
    let mut ctx = Ctx {
        viewport_w,
        viewport_h,
    };
    Some(layout_block(root, 0.0, 0.0, viewport_w, viewport_h, &mut ctx))
}

struct Ctx {
    viewport_w: f32,
    viewport_h: f32,
}

// ---------------------------------------------------------------------------
// Block layout
// ---------------------------------------------------------------------------

fn layout_block(
    node: &Node,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    container_h: f32,
    ctx: &mut Ctx,
) -> LayoutBox {
    // Text leaves: no styling, zero size — placeholder until M5.
    if let Element::Text(_) = &node.element {
        return LayoutBox {
            margin_rect: Rect::new(origin_x, origin_y, 0.0, 0.0),
            border_rect: Rect::new(origin_x, origin_y, 0.0, 0.0),
            content_rect: Rect::new(origin_x, origin_y, 0.0, 0.0),
            background: None,
            kind: BoxKind::Text,
            children: Vec::new(),
        };
    }

    let style = style_attr::element_style_attr(&node.element)
        .map(parse_inline_style)
        .unwrap_or_default();

    let margin = resolve_insets_margin(&style, container_w, ctx);
    let border = Insets::zero(); // borders deferred (M7)
    let padding = resolve_insets_padding(&style, container_w, ctx);

    // Inner width: explicit `width` or fill the parent.
    let frame_h = margin.horizontal() + border.horizontal() + padding.horizontal();
    let inner_width = length::resolve(style.width.as_ref(), container_w, ctx)
        .unwrap_or((container_w - frame_h).max(0.0));

    // Lay out children top-down inside the content box.
    let content_x = origin_x + margin.left + border.left + padding.left;
    let content_y_top = origin_y + margin.top + border.top + padding.top;

    let mut children = Vec::with_capacity(node.children.len());
    let mut cursor = 0.0_f32;
    for child in &node.children {
        let child_box = layout_block(
            child,
            content_x,
            content_y_top + cursor,
            inner_width,
            container_h,
            ctx,
        );
        cursor += child_box.margin_rect.h;
        children.push(child_box);
    }
    let content_h_from_children = cursor;

    // Inner height: explicit `height` or fit content.
    let inner_height = length::resolve(style.height.as_ref(), container_h, ctx)
        .unwrap_or(content_h_from_children);

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

    let background = style.background_color.as_ref().and_then(resolve_color);

    LayoutBox {
        margin_rect,
        border_rect,
        content_rect,
        background,
        kind: BoxKind::Block,
        children,
    }
}

fn resolve_insets_margin(style: &Style, container_w: f32, ctx: &mut Ctx) -> Insets {
    Insets {
        top: side(&style.margin_top, &style.margin, container_w, ctx),
        right: side(&style.margin_right, &style.margin, container_w, ctx),
        bottom: side(&style.margin_bottom, &style.margin, container_w, ctx),
        left: side(&style.margin_left, &style.margin, container_w, ctx),
    }
}

fn resolve_insets_padding(style: &Style, container_w: f32, ctx: &mut Ctx) -> Insets {
    Insets {
        top: side(&style.padding_top, &style.padding, container_w, ctx),
        right: side(&style.padding_right, &style.padding, container_w, ctx),
        bottom: side(&style.padding_bottom, &style.padding, container_w, ctx),
        left: side(&style.padding_left, &style.padding, container_w, ctx),
    }
}

fn side(
    specific: &Option<CssLength>,
    shorthand: &Option<CssLength>,
    container_w: f32,
    ctx: &mut Ctx,
) -> f32 {
    length::resolve(specific.as_ref(), container_w, ctx)
        .or_else(|| length::resolve(shorthand.as_ref(), container_w, ctx))
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests;
