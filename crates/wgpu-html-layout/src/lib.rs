//! Block layout.
//!
//! Walks a `CascadedTree` (one Style per node, already cascaded) and
//! produces a `LayoutBox` tree positioned in physical pixels. The renderer
//! consumes the result directly — it never re-resolves CSS.
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
use wgpu_html_models::common::css_enums::{BoxSizing, CssLength, Display};
use wgpu_html_style::{CascadedNode, CascadedTree};
use wgpu_html_tree::Element;

mod color;
mod flex;
mod length;

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
    /// Per-side border thickness, in physical pixels.
    pub border: Insets,
    /// Resolved border color (used for all four sides). `None` falls
    /// back to the foreground color, which we don't track yet, so paint
    /// skips the border in that case.
    pub border_color: Option<Color>,
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

/// Lay the cascaded tree out into a viewport of `viewport_w × viewport_h`
/// physical pixels. The returned root box's `margin_rect` covers the viewport.
pub fn layout(tree: &CascadedTree, viewport_w: f32, viewport_h: f32) -> Option<LayoutBox> {
    let root = tree.root.as_ref()?;
    let mut ctx = Ctx {
        viewport_w,
        viewport_h,
    };
    Some(layout_block(root, 0.0, 0.0, viewport_w, viewport_h, &mut ctx))
}

pub(crate) struct Ctx {
    pub viewport_w: f32,
    pub viewport_h: f32,
}

// ---------------------------------------------------------------------------
// Block layout
// ---------------------------------------------------------------------------

/// Re-exposed under a stable name so submodules (like `flex`) can call
/// the block layout entry point without exposing it to the public API.
pub(crate) fn layout_block_at(
    node: &CascadedNode,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    container_h: f32,
    ctx: &mut Ctx,
) -> LayoutBox {
    layout_block(node, origin_x, origin_y, container_w, container_h, ctx)
}

fn layout_block(
    node: &CascadedNode,
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
            border: Insets::zero(),
            border_color: None,
            kind: BoxKind::Text,
            children: Vec::new(),
        };
    }

    let style = &node.style;

    let margin = resolve_insets_margin(style, container_w, ctx);
    let border_width = length::resolve(style.border_width.as_ref(), container_w, ctx).unwrap_or(0.0);
    let border = Insets {
        top: border_width,
        right: border_width,
        bottom: border_width,
        left: border_width,
    };
    let padding = resolve_insets_padding(style, container_w, ctx);

    let box_sizing = style.box_sizing.clone().unwrap_or(BoxSizing::ContentBox);

    // Inner width: explicit `width` or fill the parent.
    // - `content-box` (CSS default): `width` is the content-box width.
    // - `border-box`: `width` is the border-box width, so we subtract
    //   horizontal border + padding to get the content-box width.
    let frame_w = margin.horizontal() + border.horizontal() + padding.horizontal();
    let inner_width = match length::resolve(style.width.as_ref(), container_w, ctx) {
        Some(specified) => match box_sizing {
            BoxSizing::ContentBox => specified,
            BoxSizing::BorderBox => {
                (specified - border.horizontal() - padding.horizontal()).max(0.0)
            }
        },
        None => (container_w - frame_w).max(0.0),
    };

    // Lay out children inside the content box, dispatching on display.
    let content_x = origin_x + margin.left + border.left + padding.left;
    let content_y_top = origin_y + margin.top + border.top + padding.top;

    // Pre-resolve an explicit height (used for `align-items: stretch`).
    let inner_height_explicit = length::resolve(style.height.as_ref(), container_h, ctx)
        .map(|specified| match box_sizing {
            BoxSizing::ContentBox => specified,
            BoxSizing::BorderBox => {
                (specified - border.vertical() - padding.vertical()).max(0.0)
            }
        });

    let display = style.display.clone().unwrap_or(Display::Block);
    let (children, content_h_from_children) = match display {
        Display::Flex | Display::InlineFlex => {
            let (kids, _content_w_used, content_h_used) = flex::layout_flex_children(
                node,
                style,
                content_x,
                content_y_top,
                inner_width,
                inner_height_explicit,
                ctx,
            );
            (kids, content_h_used)
        }
        _ => {
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
            (children, cursor)
        }
    };

    let inner_height = inner_height_explicit.unwrap_or(content_h_from_children);

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
    let border_color = style.border_color.as_ref().and_then(resolve_color);

    LayoutBox {
        margin_rect,
        border_rect,
        content_rect,
        background,
        border,
        border_color,
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
