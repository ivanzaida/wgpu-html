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
use wgpu_html_models::common::css_enums::{BorderStyle, BoxSizing, CssLength, Display};
use wgpu_html_style::{CascadedNode, CascadedTree};
use wgpu_html_tree::{Element, Node, Tree};

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

    /// Half-open hit test: a point on the top/left edge is inside,
    /// a point on the bottom/right edge is not.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
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

/// One corner's radius. `h` is the horizontal extent, `v` the vertical.
/// Equal components describe a circular corner; otherwise an ellipse.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radius {
    pub h: f32,
    pub v: f32,
}

impl Radius {
    pub const fn zero() -> Self {
        Self { h: 0.0, v: 0.0 }
    }
    pub const fn circle(r: f32) -> Self {
        Self { h: r, v: r }
    }
    pub fn is_zero(self) -> bool {
        self.h <= 0.0 && self.v <= 0.0
    }
}

/// Per-corner elliptical radii in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadii {
    pub top_left: Radius,
    pub top_right: Radius,
    pub bottom_right: Radius,
    pub bottom_left: Radius,
}

impl CornerRadii {
    pub const fn zero() -> Self {
        Self {
            top_left: Radius::zero(),
            top_right: Radius::zero(),
            bottom_right: Radius::zero(),
            bottom_left: Radius::zero(),
        }
    }
    pub fn any_nonzero(&self) -> bool {
        !self.top_left.is_zero()
            || !self.top_right.is_zero()
            || !self.bottom_right.is_zero()
            || !self.bottom_left.is_zero()
    }
}

/// Per-side border colors, top / right / bottom / left.
#[derive(Debug, Clone, Copy, Default)]
pub struct BorderColors {
    pub top: Option<Color>,
    pub right: Option<Color>,
    pub bottom: Option<Color>,
    pub left: Option<Color>,
}

impl BorderColors {
    pub fn any(&self) -> bool {
        self.top.is_some() || self.right.is_some() || self.bottom.is_some() || self.left.is_some()
    }
}

/// Per-side border styles. `None` means the property was unset; paint
/// treats that as `solid` for sides that have a width and a colour
/// (matching the convention `border: <w> <c>` implies "solid" in
/// CSS source — we already inflate the shorthand to `solid` in the
/// parser for that case, but per-side longhand can leave it unset).
#[derive(Debug, Clone, Default)]
pub struct BorderStyles {
    pub top: Option<BorderStyle>,
    pub right: Option<BorderStyle>,
    pub bottom: Option<BorderStyle>,
    pub left: Option<BorderStyle>,
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
    /// Rectangle the background fills. Driven by `background-clip`:
    /// `border-box` (default) → equal to `border_rect`; `padding-box` →
    /// border-rect inset by border thickness; `content-box` → equal to
    /// `content_rect`.
    pub background_rect: Rect,
    /// Per-corner radii to use when painting the background — already
    /// reduced from the outer radii to match `background_rect`.
    pub background_radii: CornerRadii,
    /// Per-side border thickness, in physical pixels.
    pub border: Insets,
    /// Per-side border color. `None` for a side means that edge is
    /// skipped during paint (we don't track foreground color yet, so
    /// there's no spec-default fallback).
    pub border_colors: BorderColors,
    /// Per-side border style. Currently honoured for `solid`, `dashed`,
    /// `dotted`, `none`, `hidden`. Other values render as solid.
    pub border_styles: BorderStyles,
    /// Per-corner radii. Currently parsed and laid out but **not yet
    /// rendered** (paint emits straight-edge quads).
    pub border_radius: CornerRadii,
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

impl LayoutBox {
    /// Index path from `self` to the deepest descendant whose
    /// `border_rect` contains `point`. An empty path means `self` is the
    /// deepest match. `None` if the point is outside `self`.
    ///
    /// The layout tree mirrors the source element tree 1:1, so this
    /// path can be applied to a [`Tree`] / [`Node`] to navigate to the
    /// corresponding element.
    pub fn hit_path(&self, point: (f32, f32)) -> Option<Vec<usize>> {
        let (x, y) = point;
        if !self.border_rect.contains(x, y) {
            return None;
        }
        let mut path: Vec<usize> = Vec::new();
        collect_hit_path(self, x, y, &mut path);
        Some(path)
    }

    /// Hit-test the layout at `point` and return a mutable reference
    /// to the matching element node in `tree`. Use this to read or
    /// modify the source element (style, text, attributes, etc.).
    /// Returns `None` if the point is outside `self` or the tree has
    /// no root.
    ///
    /// `tree` must be the same tree this layout was produced from; we
    /// rely on the layout's child structure mirroring the element
    /// tree's child structure 1:1.
    ///
    /// On overlap, children are walked last-to-first so the topmost
    /// (last-painted) hit wins.
    pub fn find_element_from_point<'a>(
        &self,
        tree: &'a mut Tree,
        point: (f32, f32),
    ) -> Option<&'a mut Node> {
        let path = self.hit_path(point)?;
        tree.root.as_mut()?.at_path_mut(&path)
    }

    /// Like [`Self::find_element_from_point`] but returns the entire
    /// ancestor chain of element nodes from the deepest hit up to (and
    /// including) the tree root. Empty when the point is outside or
    /// the tree has no root.
    ///
    /// Soundness inherits from [`Node::ancestry_at_path_mut`]: the
    /// returned `&mut` references alias into nested subtrees of the
    /// same borrow, so two of them must never be dereferenced at the
    /// same time.
    pub fn find_elements_from_point<'a>(
        &self,
        tree: &'a mut Tree,
        point: (f32, f32),
    ) -> Vec<&'a mut Node> {
        let Some(path) = self.hit_path(point) else {
            return Vec::new();
        };
        let Some(root) = tree.root.as_mut() else {
            return Vec::new();
        };
        root.ancestry_at_path_mut(&path)
    }
}

fn collect_hit_path(b: &LayoutBox, x: f32, y: f32, path: &mut Vec<usize>) {
    for (i, child) in b.children.iter().enumerate().rev() {
        if child.border_rect.contains(x, y) {
            path.push(i);
            collect_hit_path(child, x, y, path);
            return;
        }
    }
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
            background_rect: Rect::new(origin_x, origin_y, 0.0, 0.0),
            background_radii: CornerRadii::zero(),
            border: Insets::zero(),
            border_colors: BorderColors::default(),
            border_styles: BorderStyles::default(),
            border_radius: CornerRadii::zero(),
            kind: BoxKind::Text,
            children: Vec::new(),
        };
    }

    let style = &node.style;

    let margin = resolve_insets_margin(style, container_w, ctx);
    let border = Insets {
        top: length::resolve(style.border_top_width.as_ref(), container_w, ctx).unwrap_or(0.0),
        right: length::resolve(style.border_right_width.as_ref(), container_w, ctx).unwrap_or(0.0),
        bottom: length::resolve(style.border_bottom_width.as_ref(), container_w, ctx)
            .unwrap_or(0.0),
        left: length::resolve(style.border_left_width.as_ref(), container_w, ctx).unwrap_or(0.0),
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
    let border_colors = BorderColors {
        top: style.border_top_color.as_ref().and_then(resolve_color),
        right: style.border_right_color.as_ref().and_then(resolve_color),
        bottom: style.border_bottom_color.as_ref().and_then(resolve_color),
        left: style.border_left_color.as_ref().and_then(resolve_color),
    };
    let border_styles = BorderStyles {
        top: style.border_top_style.clone(),
        right: style.border_right_style.clone(),
        bottom: style.border_bottom_style.clone(),
        left: style.border_left_style.clone(),
    };
    let resolve_corner = |h: Option<&CssLength>, v: Option<&CssLength>, ctx: &mut Ctx| -> Radius {
        let h_px = length::resolve(h, container_w, ctx).unwrap_or(0.0).max(0.0);
        // Vertical resolves against the box height when known, else
        // viewport height (Ctx). When the v field is unset, fall back
        // to the same value as h (CSS default).
        let v_px = match v {
            Some(_) => length::resolve(v, container_h, ctx).unwrap_or(0.0).max(0.0),
            None => h_px,
        };
        Radius { h: h_px, v: v_px }
    };
    let mut border_radius = CornerRadii {
        top_left: resolve_corner(
            style.border_top_left_radius.as_ref(),
            style.border_top_left_radius_v.as_ref(),
            ctx,
        ),
        top_right: resolve_corner(
            style.border_top_right_radius.as_ref(),
            style.border_top_right_radius_v.as_ref(),
            ctx,
        ),
        bottom_right: resolve_corner(
            style.border_bottom_right_radius.as_ref(),
            style.border_bottom_right_radius_v.as_ref(),
            ctx,
        ),
        bottom_left: resolve_corner(
            style.border_bottom_left_radius.as_ref(),
            style.border_bottom_left_radius_v.as_ref(),
            ctx,
        ),
    };
    clamp_corner_radii(&mut border_radius, border_rect.w, border_rect.h);

    let (background_rect, background_radii) = compute_background_box(
        style,
        border_rect,
        content_rect,
        border,
        padding,
        &border_radius,
    );

    LayoutBox {
        margin_rect,
        border_rect,
        content_rect,
        background,
        background_rect,
        background_radii,
        border,
        border_colors,
        border_styles,
        border_radius,
        kind: BoxKind::Block,
        children,
    }
}

/// Pick the rectangle and corner radii that the background fills, based
/// on `background-clip`. The default `border-box` keeps the outer
/// rectangle and radii. `padding-box` shrinks by the border thickness;
/// `content-box` shrinks by border + padding. Inner radii are reduced
/// in step so the curvature stays concentric with the outer edge.
fn compute_background_box(
    style: &Style,
    border_rect: Rect,
    content_rect: Rect,
    border: Insets,
    padding: Insets,
    radii: &CornerRadii,
) -> (Rect, CornerRadii) {
    use wgpu_html_models::common::css_enums::BackgroundClip;
    match style.background_clip.clone().unwrap_or(BackgroundClip::BorderBox) {
        BackgroundClip::BorderBox => (border_rect, radii.clone()),
        BackgroundClip::PaddingBox => {
            let inset_top = border.top;
            let inset_right = border.right;
            let inset_bottom = border.bottom;
            let inset_left = border.left;
            let r = inset_radii(
                radii,
                inset_top,
                inset_right,
                inset_bottom,
                inset_left,
            );
            let rect = Rect::new(
                border_rect.x + inset_left,
                border_rect.y + inset_top,
                (border_rect.w - inset_left - inset_right).max(0.0),
                (border_rect.h - inset_top - inset_bottom).max(0.0),
            );
            (rect, r)
        }
        BackgroundClip::ContentBox => {
            let inset_top = border.top + padding.top;
            let inset_right = border.right + padding.right;
            let inset_bottom = border.bottom + padding.bottom;
            let inset_left = border.left + padding.left;
            let r = inset_radii(
                radii,
                inset_top,
                inset_right,
                inset_bottom,
                inset_left,
            );
            (content_rect, r)
        }
    }
}

/// Reduce each corner's radius by the matching adjacent insets,
/// clamped at zero. The horizontal component shrinks by the inset of
/// the side it shares an x-edge with; the vertical component shrinks
/// by the inset of its y-edge. A tight border eats into the curvature
/// until the inner edge is straight.
fn inset_radii(r: &CornerRadii, top: f32, right: f32, bottom: f32, left: f32) -> CornerRadii {
    let shrink = |corner: Radius, dh: f32, dv: f32| Radius {
        h: (corner.h - dh).max(0.0),
        v: (corner.v - dv).max(0.0),
    };
    CornerRadii {
        top_left: shrink(r.top_left, left, top),
        top_right: shrink(r.top_right, right, top),
        bottom_right: shrink(r.bottom_right, right, bottom),
        bottom_left: shrink(r.bottom_left, left, bottom),
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

/// Per CSS 3 border-radius spec: when the sum of radii on any side
/// exceeds that side's length, *all* radii are scaled down by the
/// same factor (the smallest of the per-side ratios) so adjacent
/// corners no longer overlap. The horizontal and vertical components
/// are checked independently — overflow on either axis triggers a
/// uniform scale of every corner on every axis.
fn clamp_corner_radii(r: &mut CornerRadii, width: f32, height: f32) {
    let mut scale: f32 = 1.0;
    let limit = |edge_len: f32, sum: f32, scale: &mut f32| {
        if sum > 0.0 && edge_len > 0.0 && sum > edge_len {
            *scale = scale.min(edge_len / sum);
        }
    };
    // Horizontal axis: the h-components on each horizontal edge.
    limit(width, r.top_left.h + r.top_right.h, &mut scale);
    limit(width, r.bottom_left.h + r.bottom_right.h, &mut scale);
    // Vertical axis: the v-components on each vertical edge.
    limit(height, r.top_left.v + r.bottom_left.v, &mut scale);
    limit(height, r.top_right.v + r.bottom_right.v, &mut scale);
    if scale < 1.0 {
        for c in [
            &mut r.top_left,
            &mut r.top_right,
            &mut r.bottom_right,
            &mut r.bottom_left,
        ] {
            c.h *= scale;
            c.v *= scale;
        }
    }
}

#[cfg(test)]
mod tests;
