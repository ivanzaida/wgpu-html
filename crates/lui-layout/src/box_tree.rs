//! Layout box types and the box tree.

use bumpalo::Bump;
use lui_cascade::{ComputedStyle, ScrollbarPseudoStyles};
use lui_core::{CssValue, Rect, resolve_scrollbar_inset, resolve_scrollbar_min_thumb_size};
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
  Table,
  TableRow,
  TableCell,
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
  pub scrollbar_pseudo: Option<&'a ScrollbarPseudoStyles<'a>>,
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
  pub fn margin_rect(&self) -> Rect {
    let br = self.border_rect();
    Rect::new(
      br.x - self.margin.left,
      br.y - self.margin.top,
      br.width + self.margin.horizontal(),
      br.height + self.margin.vertical(),
    )
  }

  pub fn new(kind: BoxKind, node: &'a HtmlNode, style: &'a ComputedStyle<'a>, bump: &'a Bump) -> Self {
    Self {
      kind,
      node,
      style,
      margin: RectEdges::default(),
      border: RectEdges::default(),
      padding: RectEdges::default(),
      content: Rect::default(),
      intrinsic: None,
      children: bumpalo::collections::Vec::new_in(bump),
      overflow_x: Overflow::Visible,
      overflow_y: Overflow::Visible,
      clip: None,
      scroll: None,
      baseline: None,
      z_index: None,
      sticky: None,
      text_overflow_ellipsis: false,
      text_decoration: None,
      writing_mode: None,
      list_marker: None,
      scrollbar_pseudo: None,
    }
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
    let Some(ref mut info) = self.scroll else {
      return false;
    };
    let max_x = info.max_scroll_x(self.content.width);
    let max_y = info.max_scroll_y(self.content.height);
    let new_x = x.clamp(0.0, max_x);
    let new_y = y.clamp(0.0, max_y);
    let changed = (new_x - info.scroll_x).abs() > 0.001 || (new_y - info.scroll_y).abs() > 0.001;
    info.scroll_x = new_x;
    info.scroll_y = new_y;
    changed
  }

  /// Scroll by a delta, clamped. Returns true if position changed.
  pub fn scroll_by(&mut self, dx: f32, dy: f32) -> bool {
    let Some(ref info) = self.scroll else {
      return false;
    };
    let x = info.scroll_x + dx;
    let y = info.scroll_y + dy;
    self.set_scroll(x, y)
  }

  /// Scroll by delta, consuming what this container can absorb.
  /// Returns the unconsumed (remaining) delta.
  pub fn scroll_by_consuming(&mut self, dx: f32, dy: f32) -> (f32, f32) {
    let Some(ref mut info) = self.scroll else {
      return (dx, dy);
    };
    let max_x = info.max_scroll_x(self.content.width);
    let max_y = info.max_scroll_y(self.content.height);
    let new_x = (info.scroll_x + dx).clamp(0.0, max_x);
    let new_y = (info.scroll_y + dy).clamp(0.0, max_y);
    let consumed_x = new_x - info.scroll_x;
    let consumed_y = new_y - info.scroll_y;
    info.scroll_x = new_x;
    info.scroll_y = new_y;
    (dx - consumed_x, dy - consumed_y)
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
    unsafe {
      std::mem::ManuallyDrop::drop(&mut self.root);
    }
    // Now free the arena.
    // SAFETY: arena was allocated with Box::into_raw and is valid.
    unsafe {
      drop(Box::from_raw(self.arena));
    }
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

  pub fn set_scroll_at_path(&mut self, path: &[usize], x: f32, y: f32) -> bool {
    box_at_path_mut(&mut self.root, path).is_some_and(|b| b.set_scroll(x, y))
  }

  pub fn scroll_by_at_path(&mut self, path: &[usize], dx: f32, dy: f32) -> bool {
    box_at_path_mut(&mut self.root, path).is_some_and(|b| b.scroll_by(dx, dy))
  }

  pub fn deepest_scrollable_path_at(&self, x: f32, y: f32) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    deepest_scrollable_path_at_box(&self.root, x, y, 0.0, 0.0, &mut path)
  }

  /// Try to scroll at `start_path`, then chain unconsumed delta up through
  /// ancestor scroll containers. Returns which containers changed and any
  /// remaining delta that no container could absorb.
  pub fn scroll_chain(
    &mut self,
    start_path: &[usize],
    dx: f32,
    dy: f32,
  ) -> ScrollChainResult {
    let mut remaining = (dx, dy);
    let mut changed = Vec::new();

    try_scroll_at(&mut self.root, start_path, &mut remaining, &mut changed);

    for len in (0..start_path.len()).rev() {
      if remaining.0.abs() < 0.001 && remaining.1.abs() < 0.001 {
        break;
      }
      try_scroll_at(&mut self.root, &start_path[..len], &mut remaining, &mut changed);
    }

    ScrollChainResult {
      changed,
      remaining_x: remaining.0,
      remaining_y: remaining.1,
    }
  }

  pub fn viewport_scroll_bounds(&self, viewport_width: f32, viewport_height: f32) -> (f32, f32) {
    let right = document_extent_right(&self.root, 0);
    let bottom = document_extent_bottom(&self.root, 0);
    ((right - viewport_width).max(0.0), (bottom - viewport_height).max(0.0))
  }

  pub fn cursor_at(&self, x: f32, y: f32) -> &str {
    cursor_at_box(&self.root, x, y, 0.0, 0.0).unwrap_or("auto")
  }
}

pub struct ScrollChainResult {
  pub changed: Vec<(Vec<usize>, ScrollInfo)>,
  pub remaining_x: f32,
  pub remaining_y: f32,
}

fn try_scroll_at(
  root: &mut LayoutBox<'_>,
  path: &[usize],
  remaining: &mut (f32, f32),
  changed: &mut Vec<(Vec<usize>, ScrollInfo)>,
) {
  let Some(b) = box_at_path_mut(root, path) else {
    return;
  };
  if !b.is_scroll_container() {
    return;
  }
  let old_x = b.scroll.map(|s| s.scroll_x).unwrap_or(0.0);
  let old_y = b.scroll.map(|s| s.scroll_y).unwrap_or(0.0);
  let rem = b.scroll_by_consuming(remaining.0, remaining.1);
  if let Some(info) = b.scroll {
    if (info.scroll_x - old_x).abs() > 0.001 || (info.scroll_y - old_y).abs() > 0.001 {
      changed.push((path.to_vec(), info));
    }
  }
  *remaining = rem;
}

fn box_at_path_mut<'a, 'b>(mut current: &'b mut LayoutBox<'a>, path: &[usize]) -> Option<&'b mut LayoutBox<'a>> {
  for &idx in path {
    current = current.children.get_mut(idx)?;
  }
  Some(current)
}

fn is_pointer_events_none(v: Option<&CssValue>) -> bool {
  matches!(v, Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) if s.as_ref() == "none")
}

fn css_cursor_str<'a>(v: Option<&'a CssValue>) -> &'a str {
  match v {
    Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
    _ => "auto",
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
  if adjusted_x < br.x || adjusted_x > br.x + br.width || adjusted_y < br.y || adjusted_y > br.y + br.height {
    return None;
  }

  if let Some(clip) = b.clip {
    let clip_x = clip.x - scroll_offset_x;
    let clip_y = clip.y - scroll_offset_y;
    if x < clip_x || x > clip_x + clip.width || y < clip_y || y > clip_y + clip.height {
      return None;
    }
  }

  let child_sx = scroll_offset_x + b.scroll.map(|s| s.scroll_x).unwrap_or(0.0);
  let child_sy = scroll_offset_y + b.scroll.map(|s| s.scroll_y).unwrap_or(0.0);

  for child in b.children.iter().rev() {
    if let Some(hit) = hit_test_box(child, x, y, child_sx, child_sy) {
      return Some(hit);
    }
  }

  if is_pointer_events_none(b.style.pointer_events) {
    return None;
  }

  Some(b.node)
}

fn cursor_at_box<'a>(
  b: &LayoutBox<'a>,
  x: f32,
  y: f32,
  scroll_offset_x: f32,
  scroll_offset_y: f32,
) -> Option<&'a str> {
  let adjusted_x = x + scroll_offset_x;
  let adjusted_y = y + scroll_offset_y;

  let br = b.border_rect();
  if adjusted_x < br.x || adjusted_x > br.x + br.width || adjusted_y < br.y || adjusted_y > br.y + br.height {
    return None;
  }

  if let Some(clip) = b.clip {
    let clip_x = clip.x - scroll_offset_x;
    let clip_y = clip.y - scroll_offset_y;
    if x < clip_x || x > clip_x + clip.width || y < clip_y || y > clip_y + clip.height {
      return None;
    }
  }

  let child_sx = scroll_offset_x + b.scroll.map(|s| s.scroll_x).unwrap_or(0.0);
  let child_sy = scroll_offset_y + b.scroll.map(|s| s.scroll_y).unwrap_or(0.0);

  for child in b.children.iter().rev() {
    if let Some(c) = cursor_at_box(child, x, y, child_sx, child_sy) {
      return Some(c);
    }
  }

  if is_pointer_events_none(b.style.pointer_events) {
    return None;
  }

  Some(css_cursor_str(b.style.cursor))
}

fn deepest_scrollable_path_at_box(
  b: &LayoutBox<'_>,
  x: f32,
  y: f32,
  scroll_offset_x: f32,
  scroll_offset_y: f32,
  path: &mut Vec<usize>,
) -> Option<Vec<usize>> {
  let adjusted_x = x + scroll_offset_x;
  let adjusted_y = y + scroll_offset_y;

  let br = b.border_rect();
  if adjusted_x < br.x || adjusted_x > br.x + br.width || adjusted_y < br.y || adjusted_y > br.y + br.height {
    return None;
  }

  if let Some(clip) = b.clip {
    let clip_x = clip.x - scroll_offset_x;
    let clip_y = clip.y - scroll_offset_y;
    if x < clip_x || x > clip_x + clip.width || y < clip_y || y > clip_y + clip.height {
      return None;
    }
  }

  let child_sx = scroll_offset_x + b.scroll.map(|s| s.scroll_x).unwrap_or(0.0);
  let child_sy = scroll_offset_y + b.scroll.map(|s| s.scroll_y).unwrap_or(0.0);

  for idx in (0..b.children.len()).rev() {
    path.push(idx);
    if let Some(found) = deepest_scrollable_path_at_box(&b.children[idx], x, y, child_sx, child_sy, path) {
      return Some(found);
    }
    path.pop();
  }

  if b.is_scroll_container() {
    return Some(path.clone());
  }

  None
}

fn document_extent_right(b: &LayoutBox<'_>, depth: usize) -> f32 {
  let mr = b.margin_rect();
  b.children
    .iter()
    .map(|child| {
      if child.is_scroll_container() && depth > 1 {
        let child_mr = child.margin_rect();
        child_mr.x + child_mr.width
      } else {
        document_extent_right(child, depth + 1)
      }
    })
    .fold(mr.x + mr.width, f32::max)
}

fn document_extent_bottom(b: &LayoutBox<'_>, depth: usize) -> f32 {
  let mr = b.margin_rect();
  b.children
    .iter()
    .map(|child| {
      if child.is_scroll_container() && depth > 1 {
        let child_mr = child.margin_rect();
        child_mr.y + child_mr.height
      } else {
        document_extent_bottom(child, depth + 1)
      }
    })
    .fold(mr.y + mr.height, f32::max)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollbarAxis {
  Vertical,
  Horizontal,
}

#[derive(Debug, Clone)]
pub struct ScrollbarHit {
  pub path: Vec<usize>,
  pub axis: ScrollbarAxis,
  pub on_thumb: bool,
  pub grab_offset: f32,
  pub track_start: f32,
  pub track_length: f32,
  pub thumb_length: f32,
  pub max_scroll: f32,
}

impl<'a> LayoutTree<'a> {
  pub fn scrollbar_hit_test(&self, x: f32, y: f32) -> Option<ScrollbarHit> {
    let mut path = Vec::new();
    scrollbar_hit_test_box(&self.root, x, y, 0.0, 0.0, &mut path)
  }
}

fn scrollbar_hit_test_box(
  b: &LayoutBox<'_>,
  x: f32,
  y: f32,
  scroll_offset_x: f32,
  scroll_offset_y: f32,
  path: &mut Vec<usize>,
) -> Option<ScrollbarHit> {
  let ax = x + scroll_offset_x;
  let ay = y + scroll_offset_y;

  let br = b.border_rect();
  if ax < br.x || ax > br.x + br.width || ay < br.y || ay > br.y + br.height {
    return None;
  }

  if let Some(clip) = b.clip {
    let cx = clip.x - scroll_offset_x;
    let cy = clip.y - scroll_offset_y;
    if x < cx || x > cx + clip.width || y < cy || y > cy + clip.height {
      return None;
    }
  }

  let child_sx = scroll_offset_x + b.scroll.map(|s| s.scroll_x).unwrap_or(0.0);
  let child_sy = scroll_offset_y + b.scroll.map(|s| s.scroll_y).unwrap_or(0.0);

  for idx in (0..b.children.len()).rev() {
    path.push(idx);
    if let Some(hit) = scrollbar_hit_test_box(&b.children[idx], x, y, child_sx, child_sy, path) {
      return Some(hit);
    }
    path.pop();
  }

  if let Some(scroll) = &b.scroll {
    if let Some(hit) = test_scrollbar_area(b, scroll, ax, ay, path) {
      return Some(hit);
    }
  }

  None
}

fn test_scrollbar_area(
  b: &LayoutBox<'_>,
  scroll: &ScrollInfo,
  doc_x: f32,
  doc_y: f32,
  path: &[usize],
) -> Option<ScrollbarHit> {
  let bar_w = scroll.scrollbar_width;
  if bar_w <= 0.0 { return None; }

  let (inset, min_thumb) = if let Some(ps) = &b.scrollbar_pseudo {
    (
      resolve_scrollbar_inset(ps.scrollbar.scrollbar_inset),
      resolve_scrollbar_min_thumb_size(ps.scrollbar.scrollbar_min_thumb_size),
    )
  } else {
    ([0.0_f32; 4], 20.0_f32)
  };

  let pad = b.padding_rect();
  let has_v = matches!(b.overflow_y, Overflow::Scroll | Overflow::Auto) && scroll.scroll_height > b.content.height;
  let has_h = matches!(b.overflow_x, Overflow::Scroll | Overflow::Auto) && scroll.scroll_width > b.content.width;

  if has_v {
    let track_x = pad.x + pad.width - bar_w + inset[1];
    let track_y = pad.y + inset[0];
    let track_w = bar_w - inset[1] - inset[3];
    let track_h = pad.height - inset[0] - inset[2] - if has_h { bar_w } else { 0.0 };

    if doc_x >= track_x && doc_x <= track_x + track_w && doc_y >= track_y && doc_y <= track_y + track_h {
      let max_scroll = (scroll.scroll_height - b.content.height).max(0.0);
      let ratio = b.content.height / scroll.scroll_height;
      let thumb_h = (track_h * ratio).max(min_thumb).min(track_h);
      let thumb_y = if max_scroll > 0.0 {
        track_y + (track_h - thumb_h) * (scroll.scroll_y / max_scroll)
      } else {
        track_y
      };
      let on_thumb = doc_y >= thumb_y && doc_y <= thumb_y + thumb_h;
      return Some(ScrollbarHit {
        path: path.to_vec(),
        axis: ScrollbarAxis::Vertical,
        on_thumb,
        grab_offset: if on_thumb { doc_y - thumb_y } else { thumb_h * 0.5 },
        track_start: track_y,
        track_length: track_h,
        thumb_length: thumb_h,
        max_scroll,
      });
    }
  }

  if has_h {
    let track_x = pad.x + inset[3];
    let track_y = pad.y + pad.height - bar_w + inset[2];
    let track_w = pad.width - inset[1] - inset[3] - if has_v { bar_w } else { 0.0 };
    let track_h = bar_w - inset[0] - inset[2];

    if doc_x >= track_x && doc_x <= track_x + track_w && doc_y >= track_y && doc_y <= track_y + track_h {
      let max_scroll = (scroll.scroll_width - b.content.width).max(0.0);
      let ratio = b.content.width / scroll.scroll_width;
      let thumb_w = (track_w * ratio).max(min_thumb).min(track_w);
      let thumb_x = if max_scroll > 0.0 {
        track_x + (track_w - thumb_w) * (scroll.scroll_x / max_scroll)
      } else {
        track_x
      };
      let on_thumb = doc_x >= thumb_x && doc_x <= thumb_x + thumb_w;
      return Some(ScrollbarHit {
        path: path.to_vec(),
        axis: ScrollbarAxis::Horizontal,
        on_thumb,
        grab_offset: if on_thumb { doc_x - thumb_x } else { thumb_w * 0.5 },
        track_start: track_x,
        track_length: track_w,
        thumb_length: thumb_w,
        max_scroll,
      });
    }
  }

  None
}
