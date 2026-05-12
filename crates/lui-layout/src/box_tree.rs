//! Layout box types and the box tree.

use lui_html_parser::{HtmlNode, Rect};

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
    /// Grid container.
    GridContainer,
    /// Absolute/fixed positioned — removed from flow.
    Absolute,
    /// Table wrapper / table / table-row / table-cell.
    Table, TableRow, TableCell,
    /// Anonymous box created for text runs between block siblings.
    AnonymousBlock,
    /// Anonymous box for inline text content.
    AnonymousInline,
    /// Root box.
    Root,
    /// List-item marker box.
    ListItem,
}

/// A box in the layout tree. One LayoutBox per CSS box.
#[derive(Debug)]
pub struct LayoutBox<'a> {
    pub kind: BoxKind,
    /// The source HTML node (for text content, element type).
    pub node: &'a HtmlNode,
    /// Computed geometry after layout.
    pub margin: RectEdges<f32>,
    pub border: RectEdges<f32>,
    pub padding: RectEdges<f32>,
    /// Content box position and size (relative to containing block).
    pub content: Rect,
    /// Intrinsic size hints computed before final layout.
    pub intrinsic: Option<Size>,
    /// Children boxes.
    pub children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    pub fn new(kind: BoxKind, node: &'a HtmlNode) -> Self {
        Self {
            kind,
            node,
            margin: RectEdges::default(),
            border: RectEdges::default(),
            padding: RectEdges::default(),
            content: Rect::default(),
            intrinsic: None,
            children: Vec::new(),
        }
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
}

/// The full layout tree, plus a node → content-rect map for fast lookup.
#[derive(Debug)]
pub struct LayoutTree<'a> {
    pub root: LayoutBox<'a>,
    pub rects: Vec<(*const HtmlNode, Rect)>,
}

impl<'a> LayoutTree<'a> {
    pub fn find_rect(&self, node: &HtmlNode) -> Option<Rect> {
        let ptr = node as *const HtmlNode;
        self.rects.iter().find(|(n, _)| *n == ptr).map(|(_, r)| *r)
    }
}
