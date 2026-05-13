//! Incremental layout — skip computation for clean subtrees by cloning
//! cached results from the previous frame.

use bumpalo::Bump;
use rustc_hash::{FxHashMap, FxHashSet};

use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_tree::{BoxKind, LayoutBox, LayoutTree, Overflow, ScrollInfo, StickyInsets};
use crate::context::LayoutContext;
use crate::engine::layout_node;
use crate::geometry::{Point, RectEdges};
use crate::text::TextContext;

#[derive(Debug, Clone)]
pub struct CachedBox {
    pub margin: RectEdges<f32>,
    pub border: RectEdges<f32>,
    pub padding: RectEdges<f32>,
    pub content: Rect,
    pub containing_width: f32,
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

/// Immutable snapshot of a previous frame's layout results.
/// Cheap to keep across frames — never cloned during incremental layout.
pub struct LayoutCache {
    boxes: FxHashMap<*const HtmlNode, CachedBox>,
    old_tree: FxHashMap<*const HtmlNode, OldBoxRef>,
}

/// Per-frame dirty set, built from dirty paths.
pub(crate) struct DirtySet {
    pub(crate) dirty: FxHashSet<*const HtmlNode>,
}

struct OldBoxRef {
    cached: CachedBox,
    child_snapshots: Vec<ChildSnapshot>,
}

struct ChildSnapshot {
    node_ptr: *const HtmlNode,
    style_ptr: *const u8,
    kind: BoxKind,
    cached: CachedBox,
    children: Vec<ChildSnapshot>,
}

impl LayoutCache {
    pub fn empty() -> Self {
        Self {
            boxes: FxHashMap::default(),
            old_tree: FxHashMap::default(),
        }
    }

    pub fn snapshot(tree: &LayoutTree) -> Self {
        let mut boxes = FxHashMap::default();
        let mut old_tree = FxHashMap::default();
        collect_cached(&tree.root, &mut boxes, &mut old_tree, tree.root.content.width);
        Self { boxes, old_tree }
    }

    pub fn from_tree(prev: &LayoutTree) -> Self {
        Self::snapshot(prev)
    }

    fn is_clean(&self, node: &HtmlNode, dirty: &DirtySet) -> bool {
        !dirty.dirty.contains(&(node as *const HtmlNode))
    }

    fn get_tree(&self, node: &HtmlNode) -> Option<&OldBoxRef> {
        self.old_tree.get(&(node as *const HtmlNode))
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }
}

impl DirtySet {
    fn new() -> Self {
        Self { dirty: FxHashSet::default() }
    }

    fn from_paths(doc_root: &HtmlNode, dirty_paths: &[Vec<usize>]) -> Self {
        let mut dirty = FxHashSet::default();
        expand_dirty_set(doc_root, dirty_paths, &mut dirty);
        Self { dirty }
    }

    fn all_dirty() -> Self {
        Self { dirty: FxHashSet::default() }
    }

    fn is_all_dirty(&self) -> bool {
        false
    }
}

// For full layout: everything is dirty, cache lookups always miss.
pub(crate) struct FullDirtyMarker;

/// Combined view passed through layout — either (snapshot + dirty set) or empty.
pub enum CacheView<'a> {
    Full,
    Incremental { cache: &'a LayoutCache, dirty: &'a DirtySet },
}

impl<'a> CacheView<'a> {
    pub fn try_clone<'b>(
        &self,
        b: &mut LayoutBox<'b>,
        ctx: &LayoutContext,
        pos: Point,
        rects: &mut Vec<(&'b HtmlNode, Rect)>,
        bump: &'b Bump,
    ) -> bool {
        match self {
            CacheView::Full => false,
            CacheView::Incremental { cache, dirty } => {
                try_clone_from_cache(b, cache, dirty, ctx, pos, rects, bump)
            }
        }
    }
}

fn snapshot_box(b: &LayoutBox, containing_width: f32) -> CachedBox {
    CachedBox {
        margin: b.margin,
        border: b.border,
        padding: b.padding,
        content: b.content,
        containing_width,
        overflow_x: b.overflow_x,
        overflow_y: b.overflow_y,
        clip: b.clip,
        scroll: b.scroll,
        baseline: b.baseline,
        z_index: b.z_index,
        sticky: b.sticky,
        text_overflow_ellipsis: b.text_overflow_ellipsis,
        text_decoration: b.text_decoration.clone(),
        writing_mode: b.writing_mode.clone(),
        list_marker: b.list_marker.clone(),
    }
}

fn snapshot_children(b: &LayoutBox) -> Vec<ChildSnapshot> {
    let cw = b.content.width;
    b.children.iter().map(|c| ChildSnapshot {
        node_ptr: c.node as *const HtmlNode,
        style_ptr: c.style as *const _ as *const u8,
        kind: c.kind,
        cached: snapshot_box(c, cw),
        children: snapshot_children(c),
    }).collect()
}

fn collect_cached(
    b: &LayoutBox,
    flat: &mut FxHashMap<*const HtmlNode, CachedBox>,
    tree: &mut FxHashMap<*const HtmlNode, OldBoxRef>,
    containing_width: f32,
) {
    let is_anon = matches!(b.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline);
    let ptr = b.node as *const HtmlNode;

    if !is_anon {
        flat.entry(ptr).or_insert_with(|| snapshot_box(b, containing_width));
        tree.entry(ptr).or_insert_with(|| OldBoxRef {
            cached: snapshot_box(b, containing_width),
            child_snapshots: snapshot_children(b),
        });
    }

    let child_cw = b.content.width;
    for child in &b.children {
        collect_cached(child, flat, tree, child_cw);
    }
}

fn expand_dirty_set(node: &HtmlNode, dirty_paths: &[Vec<usize>], set: &mut FxHashSet<*const HtmlNode>) {
    let mut path = Vec::new();
    expand_dirty_walk(node, dirty_paths, &mut path, set);
}

fn expand_dirty_walk(
    node: &HtmlNode,
    dirty_paths: &[Vec<usize>],
    path: &mut Vec<usize>,
    set: &mut FxHashSet<*const HtmlNode>,
) {
    let dominated = dirty_paths.iter().any(|dp| {
        path.starts_with(dp) || dp.starts_with(path)
    });
    if !dominated { return; }
    set.insert(node as *const HtmlNode);
    for (i, child) in node.children.iter().enumerate() {
        path.push(i);
        expand_dirty_walk(child, dirty_paths, path, set);
        path.pop();
    }
}

fn try_clone_from_cache<'a>(
    b: &mut LayoutBox<'a>,
    cache: &LayoutCache,
    dirty: &DirtySet,
    ctx: &LayoutContext,
    pos: Point,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
    bump: &'a Bump,
) -> bool {
    if matches!(b.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline) {
        return false;
    }
    if !cache.is_clean(b.node, dirty) { return false; }
    let old_ref = match cache.get_tree(b.node) {
        Some(r) => r,
        None => return false,
    };
    if (ctx.containing_width - old_ref.cached.containing_width).abs() > 0.5 {
        return false;
    }

    apply_cached(&old_ref.cached, b);
    b.content.x = pos.x + old_ref.cached.margin.left + old_ref.cached.border.left + old_ref.cached.padding.left;
    b.content.y = pos.y + old_ref.cached.margin.top + old_ref.cached.border.top + old_ref.cached.padding.top;
    let dx = b.content.x - old_ref.cached.content.x;
    let dy = b.content.y - old_ref.cached.content.y;

    if let Some(ref mut clip) = b.clip {
        clip.x += dx;
        clip.y += dy;
    }

    if b.children.is_empty() && !old_ref.child_snapshots.is_empty() {
        synthesize_children(b, &old_ref.child_snapshots, dx, dy, bump);
    } else {
        restore_children(&mut b.children, &old_ref.child_snapshots, dx, dy);
    }
    rebuild_rects(b, rects);
    true
}

fn apply_cached(cached: &CachedBox, b: &mut LayoutBox) {
    b.margin = cached.margin;
    b.border = cached.border;
    b.padding = cached.padding;
    b.content.width = cached.content.width;
    b.content.height = cached.content.height;
    b.overflow_x = cached.overflow_x;
    b.overflow_y = cached.overflow_y;
    b.clip = cached.clip;
    b.scroll = cached.scroll;
    b.baseline = cached.baseline;
    b.z_index = cached.z_index;
    b.sticky = cached.sticky;
    b.text_overflow_ellipsis = cached.text_overflow_ellipsis;
    b.text_decoration = cached.text_decoration.clone();
    b.writing_mode = cached.writing_mode.clone();
    b.list_marker = cached.list_marker.clone();
}

/// Create children from snapshots when build_box was skipped for a clean subtree.
/// SAFETY: node_ptr and style_ptr were captured from a LayoutBox whose referents
/// (HtmlNode in the parsed document, ComputedStyle in the cascade arena) outlive
/// the layout tree.
fn synthesize_children<'a>(parent: &mut LayoutBox<'a>, snapshots: &[ChildSnapshot], dx: f32, dy: f32, bump: &'a Bump) {
    parent.children = bumpalo::collections::Vec::from_iter_in(snapshots.iter().map(|snap| {
        let node: &'a HtmlNode = unsafe { &*(snap.node_ptr as *const HtmlNode) };
        let style: &'a lui_cascade::ComputedStyle<'a> = unsafe { &*(snap.style_ptr as *const lui_cascade::ComputedStyle<'a>) };
        let mut child = LayoutBox::new(snap.kind, node, style, bump);
        apply_cached(&snap.cached, &mut child);
        child.content.x = snap.cached.content.x + dx;
        child.content.y = snap.cached.content.y + dy;
        if let Some(ref mut clip) = child.clip {
            clip.x += dx;
            clip.y += dy;
        }
        if !snap.children.is_empty() {
            synthesize_children(&mut child, &snap.children, dx, dy, bump);
        }
        child
    }), bump);
}

fn restore_children(new_children: &mut [LayoutBox], old_snapshots: &[ChildSnapshot], dx: f32, dy: f32) {
    for (new_child, old_snap) in new_children.iter_mut().zip(old_snapshots.iter()) {
        apply_cached(&old_snap.cached, new_child);
        new_child.content.x = old_snap.cached.content.x + dx;
        new_child.content.y = old_snap.cached.content.y + dy;
        if let Some(ref mut clip) = new_child.clip {
            clip.x += dx;
            clip.y += dy;
        }
        restore_children(&mut new_child.children, &old_snap.children, dx, dy);
    }
}

fn rebuild_rects<'a>(b: &LayoutBox<'a>, rects: &mut Vec<(&'a HtmlNode, Rect)>) {
    rects.push((b.node, b.content));
    for child in &b.children {
        rebuild_rects(child, rects);
    }
}

/// Incremental layout using a pre-built cache snapshot (called by `LayoutEngine::layout_dirty`).
pub fn layout_incremental_with<'a>(
    styled: &'a lui_cascade::StyledNode<'a>,
    prev_cache: &LayoutCache,
    dirty_paths: &[Vec<usize>],
    viewport_width: f32,
    viewport_height: f32,
    text_ctx: &mut TextContext,
) -> LayoutTree<'a> {
    if dirty_paths.is_empty() {
        return crate::engine::layout_tree_with(styled, viewport_width, viewport_height, text_ctx);
    }

    let arena_ptr = Box::into_raw(Box::new(Bump::new()));
    let bump: &'a Bump = unsafe { &*arena_ptr };

    let dirty = DirtySet::from_paths(styled.node, dirty_paths);
    let view = CacheView::Incremental { cache: prev_cache, dirty: &dirty };

    let ctx = LayoutContext::new(viewport_width, viewport_height);
    let mut rects = Vec::new();
    let root = crate::box_gen::build_box_incremental(styled, &dirty.dirty, bump);
    let root = layout_node(root, &ctx, Point::new(0.0, 0.0), text_ctx, &mut rects, &view, bump);
    LayoutTree::new(root, rects, arena_ptr)
}

pub fn layout_tree_incremental<'a>(
    styled: &'a lui_cascade::StyledNode<'a>,
    prev: &LayoutTree,
    dirty_paths: &[Vec<usize>],
    viewport_width: f32,
    viewport_height: f32,
) -> LayoutTree<'a> {
    let mut text_ctx = TextContext::new();
    layout_tree_incremental_with(styled, prev, dirty_paths, viewport_width, viewport_height, &mut text_ctx)
}

pub fn layout_tree_incremental_with<'a>(
    styled: &'a lui_cascade::StyledNode<'a>,
    prev: &LayoutTree,
    dirty_paths: &[Vec<usize>],
    viewport_width: f32,
    viewport_height: f32,
    text_ctx: &mut TextContext,
) -> LayoutTree<'a> {
    if dirty_paths.is_empty() {
        return crate::engine::layout_tree_with(styled, viewport_width, viewport_height, text_ctx);
    }

    let arena_ptr = Box::into_raw(Box::new(Bump::new()));
    let bump: &'a Bump = unsafe { &*arena_ptr };

    let cache = LayoutCache::from_tree(prev);
    let dirty = DirtySet::from_paths(styled.node, dirty_paths);
    let view = CacheView::Incremental { cache: &cache, dirty: &dirty };

    let ctx = LayoutContext::new(viewport_width, viewport_height);
    let mut rects = Vec::new();
    let root = crate::box_gen::build_box_incremental(styled, &dirty.dirty, bump);
    let root = layout_node(root, &ctx, Point::new(0.0, 0.0), text_ctx, &mut rects, &view, bump);
    LayoutTree::new(root, rects, arena_ptr)
}
