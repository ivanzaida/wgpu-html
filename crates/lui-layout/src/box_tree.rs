//! Layout box types and the box tree.

use bumpalo::Bump;
use lui_cascade::ComputedStyle;
use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::geometry::{RectEdges, Size};

/// The type of box, determining which layout algorithm applies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxKind {
    /// Block-level box — stacks children vertically.
    Block,
    /// Inline-level box — flows children horizontally with line breaking.
    Inline,
    /// Inline-block — inline outside, block inside.
    InlineBlock,
    /// Flex container.
    FlexContainer,
    /// Inline flex container — inline outside, flex inside.
    InlineFlex,
    /// Grid container.
    GridContainer,
    /// Inline grid container — inline outside, grid inside.
    InlineGrid,
    /// Absolute/fixed positioned — removed from flow.
    Absolute,
    /// Table wrapper / table / table-row / table-cell.
    Table, TableRow, TableCell,
    /// Table row group (<thead>, <tbody>, <tfoot>).
    TableRowGroup,
    /// Table caption (<caption>).
    TableCaption,
    /// Table column group (<colgroup>).
    TableColumnGroup,
    /// Table column (<col>).
    TableColumn,
    /// Anonymous box created for text runs between block siblings.
    AnonymousBlock,
    /// Anonymous box for inline text content.
    AnonymousInline,
    /// Root box.
    Root,
    /// List-item marker box.
    ListItem,
}

/// Overflow behavior for a box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
    Clip,
}

/// Scroll container state — present when overflow is scroll/auto/hidden.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollInfo {
    pub scroll_width: f32,
    pub scroll_height: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub scrollbar_width: f32,
}

impl ScrollInfo {
    pub fn max_scroll_x(&self, content_width: f32) -> f32 {
        (self.scroll_width - content_width).max(0.0)
    }

    pub fn max_scroll_y(&self, content_height: f32) -> f32 {
        (self.scroll_height - content_height).max(0.0)
    }
}

/// A box in the layout tree. One LayoutBox per CSS box.
#[derive(Debug)]
pub struct LayoutBox<'a> {
    pub kind: BoxKind,
    pub node: &'a HtmlNode,
    pub style: &'a ComputedStyle<'a>,
    pub margin: RectEdges<f32>,
    pub border: RectEdges<f32>,
    pub padding: RectEdges<f32>,
    pub content: Rect,
    pub intrinsic: Option<Size>,
    pub children: bumpalo::collections::Vec<'a, LayoutBox<'a>>,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
    pub clip: Option<Rect>,
    pub scroll: Option<ScrollInfo>,
    pub baseline: Option<f32>,
    pub z_index: Option<i32>,
    pub sticky: Option<StickyInsets>,
    pub text_overflow_ellipsis: bool,
    pub text_decoration: Option<String>,
    pub writing_mode: Option<String>,
    pub list_marker: Option<String>,
}

/// Sticky positioning thresholds. Values are the distance from the scroll
/// container edge at which the element starts sticking.
#[derive(Debug, Clone, Copy, Default)]
pub struct StickyInsets {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
}

impl<'a> LayoutBox<'a> {
    pub fn new(kind: BoxKind, node: &'a HtmlNode, style: &'a ComputedStyle<'a>, bump: &'a Bump) -> Self {
        Self { kind, node, style, margin: RectEdges::default(),
            border: RectEdges::default(), padding: RectEdges::default(),
            content: Rect::default(), intrinsic: None,
            children: bumpalo::collections::Vec::new_in(bump),
            overflow_x: Overflow::Visible, overflow_y: Overflow::Visible,
            clip: None, scroll: None, baseline: None,
            z_index: None, sticky: None, text_overflow_ellipsis: false,
            text_decoration: None, writing_mode: None, list_marker: None }
    }

    pub fn padding_rect(&self) -> Rect {
        Rect::new(
            self.content.x - self.padding.left,
            self.content.y - self.padding.top,
            self.content.width + self.padding.horizontal(),
            self.content.height + self.padding.vertical(),
        )
    }

    /// Total width consumed: margin + border + padding + content.
    pub fn outer_width(&self) -> f32 {
        self.margin.horizontal() + self.border.horizontal() + self.padding.horizontal() + self.content.width
    }

    /// Total height consumed.
    pub fn outer_height(&self) -> f32 {
        self.margin.vertical() + self.border.vertical() + self.padding.vertical() + self.content.height
    }

    /// Border box (content + padding + border).
    pub fn border_rect(&self) -> Rect {
        Rect::new(
            self.content.x - self.border.left - self.padding.left,
            self.content.y - self.border.top - self.padding.top,
            self.content.width + self.border.horizontal() + self.padding.horizontal(),
            self.content.height + self.border.vertical() + self.padding.vertical(),
        )
    }

    /// True if this box is a scroll container.
    pub fn is_scroll_container(&self) -> bool {
        self.scroll.is_some()
    }

    /// Set scroll position, clamped to valid range.
    /// Returns true if the position actually changed.
    pub fn set_scroll(&mut self, x: f32, y: f32) -> bool {
        let Some(ref mut info) = self.scroll else { return false; };
        let max_x = info.max_scroll_x(self.content.width);
        let max_y = info.max_scroll_y(self.content.height);
        let new_x = x.clamp(0.0, max_x);
        let new_y = y.clamp(0.0, max_y);
        let changed = (new_x - info.scroll_x).abs() > 0.001
            || (new_y - info.scroll_y).abs() > 0.001;
        info.scroll_x = new_x;
        info.scroll_y = new_y;
        changed
    }

    /// Scroll by a delta, clamped. Returns true if position changed.
    pub fn scroll_by(&mut self, dx: f32, dy: f32) -> bool {
        let Some(ref info) = self.scroll else { return false; };
        let x = info.scroll_x + dx;
        let y = info.scroll_y + dy;
        self.set_scroll(x, y)
    }

    /// Get the visible rect for a child, accounting for this box's scroll offset.
    /// Returns the child's position in the viewport coordinate space.
    pub fn child_visible_rect(&self, child_content: Rect) -> Rect {
        if let Some(ref info) = self.scroll {
            Rect::new(
                child_content.x - info.scroll_x,
                child_content.y - info.scroll_y,
                child_content.width,
                child_content.height,
            )
        } else {
            child_content
        }
    }

    /// Find a descendant scroll container by node pointer.
    pub fn find_scroll_container_mut(&mut self, node: &HtmlNode) -> Option<&mut LayoutBox<'a>> {
        let ptr = node as *const HtmlNode;
        if self.node as *const _ == ptr && self.is_scroll_container() {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_scroll_container_mut(node) {
                return Some(found);
            }
        }
        None
    }

    /// Compute the scroll offset that would make `target_rect` visible
    /// within this scroll container. Returns `(scroll_x, scroll_y)`.
    pub fn scroll_to_reveal(&self, target: Rect) -> Option<(f32, f32)> {
        let info = self.scroll.as_ref()?;
        let mut sx = info.scroll_x;
        let mut sy = info.scroll_y;

        let view_left = self.content.x + sx;
        let view_top = self.content.y + sy;
        let view_right = view_left + self.content.width;
        let view_bottom = view_top + self.content.height;

        // Horizontal
        if target.x < view_left {
            sx -= view_left - target.x;
        } else if target.x + target.width > view_right {
            sx += (target.x + target.width) - view_right;
        }

        // Vertical
        if target.y < view_top {
            sy -= view_top - target.y;
        } else if target.y + target.height > view_bottom {
            sy += (target.y + target.height) - view_bottom;
        }

        let max_x = info.max_scroll_x(self.content.width);
        let max_y = info.max_scroll_y(self.content.height);
        Some((sx.clamp(0.0, max_x), sy.clamp(0.0, max_y)))
    }
}

/// The full layout tree, plus a node → content-rect map for fast lookup.
///
/// Owns a bump arena (`Bump`) that backs all `LayoutBox::children` vecs.
/// The arena is heap-allocated; the root box borrows from it via
/// `ManuallyDrop` so we can control drop order (root first, then arena).
pub struct LayoutTree<'a> {
    pub root: std::mem::ManuallyDrop<LayoutBox<'a>>,
    pub rects: Vec<(&'a HtmlNode, Rect)>,
    arena: *mut Bump,
}

// SAFETY: The raw pointer is only used for drop; no concurrent access.
unsafe impl Send for LayoutTree<'_> {}
unsafe impl Sync for LayoutTree<'_> {}

impl std::fmt::Debug for LayoutTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutTree")
            .field("root", &*self.root)
            .field("rects", &self.rects)
            .finish()
    }
}

impl<'a> Drop for LayoutTree<'a> {
    fn drop(&mut self) {
        // Drop the root (and its bumpalo children) first, while the arena is still alive.
        // SAFETY: ManuallyDrop::drop is safe here because we only call it once (in Drop).
        unsafe { std::mem::ManuallyDrop::drop(&mut self.root); }
        // Now free the arena.
        // SAFETY: arena was allocated with Box::into_raw and is valid.
        unsafe { drop(Box::from_raw(self.arena)); }
    }
}

impl<'a> LayoutTree<'a> {
    /// Create a new LayoutTree that takes ownership of the arena.
    ///
    /// SAFETY: `arena_ptr` must have been obtained from `Box::into_raw(Box::new(Bump::new()))`,
    /// and `root` must have been built using a reference to that same arena.
    pub(crate) fn new(root: LayoutBox<'a>, rects: Vec<(&'a HtmlNode, Rect)>, arena_ptr: *mut Bump) -> Self {
        Self {
            root: std::mem::ManuallyDrop::new(root),
            rects,
            arena: arena_ptr,
        }
    }

    /// Test-only constructor that allocates a fresh arena.
    /// The root box does **not** need to originate from this arena;
    /// the arena will simply be freed on drop.
    pub fn new_for_test(root: LayoutBox<'a>, rects: Vec<(&'a HtmlNode, Rect)>) -> Self {
        let arena_ptr = Box::into_raw(Box::new(Bump::new()));
        Self {
            root: std::mem::ManuallyDrop::new(root),
            rects,
            arena: arena_ptr,
        }
    }

    pub fn find_rect(&self, node: &HtmlNode) -> Option<Rect> {
        let ptr = node as *const HtmlNode;
        self.rects.iter().find(|(n, _)| *n as *const _ == ptr).map(|(_, r)| *r)
    }

    /// Hit-test: find the deepest node at the given point,
    /// accounting for scroll offsets and clip rects.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<&'a HtmlNode> {
        hit_test_box(&self.root, x, y, 0.0, 0.0)
    }

    /// Set scroll position on a scroll container identified by node.
    pub fn set_scroll(&mut self, node: &HtmlNode, x: f32, y: f32) -> bool {
        if let Some(b) = self.root.find_scroll_container_mut(node) {
            b.set_scroll(x, y)
        } else {
            false
        }
    }

    /// Scroll a container by delta. Returns true if position changed.
    pub fn scroll_by(&mut self, node: &HtmlNode, dx: f32, dy: f32) -> bool {
        if let Some(b) = self.root.find_scroll_container_mut(node) {
            b.scroll_by(dx, dy)
        } else {
            false
        }
    }
}

fn hit_test_box<'a>(
    b: &LayoutBox<'a>,
    x: f32,
    y: f32,
    scroll_offset_x: f32,
    scroll_offset_y: f32,
) -> Option<&'a HtmlNode> {
    let adjusted_x = x + scroll_offset_x;
    let adjusted_y = y + scroll_offset_y;

    let br = b.border_rect();
    if adjusted_x < br.x || adjusted_x > br.x + br.width
        || adjusted_y < br.y || adjusted_y > br.y + br.height
    {
        return None;
    }

    // If this box clips, check if point is inside clip rect
    if let Some(clip) = b.clip {
        let clip_x = clip.x - scroll_offset_x;
        let clip_y = clip.y - scroll_offset_y;
        if x < clip_x || x > clip_x + clip.width
            || y < clip_y || y > clip_y + clip.height
        {
            return None;
        }
    }

    // Accumulate scroll offset for children
    let child_sx = scroll_offset_x + b.scroll.map(|s| s.scroll_x).unwrap_or(0.0);
    let child_sy = scroll_offset_y + b.scroll.map(|s| s.scroll_y).unwrap_or(0.0);

    // Check children deepest-first (last child paints on top)
    for child in b.children.iter().rev() {
        if let Some(hit) = hit_test_box(child, x, y, child_sx, child_sy) {
            return Some(hit);
        }
    }

    Some(b.node)
}
