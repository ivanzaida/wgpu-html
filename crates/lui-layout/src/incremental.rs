//! Incremental layout — skip computation for clean subtrees by cloning
//! cached results from the previous frame.

use rustc_hash::{FxHashMap, FxHashSet};

use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_gen::build_box;
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

pub struct LayoutCache {
    boxes: FxHashMap<*const HtmlNode, CachedBox>,
    old_tree: FxHashMap<*const HtmlNode, OldBoxRef>,
    dirty: FxHashSet<*const HtmlNode>,
}

struct OldBoxRef {
    cached: CachedBox,
    child_snapshots: Vec<ChildSnapshot>,
}

struct ChildSnapshot {
    node_ptr: *const HtmlNode,
    kind: BoxKind,
    cached: CachedBox,
    children: Vec<ChildSnapshot>,
}

impl LayoutCache {
    pub fn empty() -> Self {
        Self {
            boxes: FxHashMap::default(),
            old_tree: FxHashMap::default(),
            dirty: FxHashSet::default(),
        }
    }

    pub fn from_tree(prev: &LayoutTree, dirty_paths: &[Vec<usize>], doc_root: &HtmlNode) -> Self {
        let mut boxes = FxHashMap::default();
        let mut old_tree = FxHashMap::default();
        collect_cached(&prev.root, &mut boxes, &mut old_tree, LayoutContext::new(0.0, 0.0).containing_width);
        let mut dirty = FxHashSet::default();
        expand_dirty_set(doc_root, dirty_paths, &mut dirty);
        Self { boxes, old_tree, dirty }
    }

    pub fn is_clean(&self, node: &HtmlNode) -> bool {
        !self.dirty.contains(&(node as *const HtmlNode))
    }

    pub fn get(&self, node: &HtmlNode) -> Option<&CachedBox> {
        self.boxes.get(&(node as *const HtmlNode))
    }

    pub fn get_tree(&self, node: &HtmlNode) -> Option<&OldBoxRef> {
        self.old_tree.get(&(node as *const HtmlNode))
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

pub fn try_clone_from_cache<'a>(
    b: &mut LayoutBox<'a>,
    cache: &LayoutCache,
    ctx: &LayoutContext,
    pos: Point,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) -> bool {
    if matches!(b.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline) {
        return false;
    }
    if !cache.is_clean(b.node) { return false; }
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

    restore_children(&mut b.children, &old_ref.child_snapshots, dx, dy);
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

pub fn layout_tree_incremental<'a>(
    styled: &'a lui_cascade::StyledNode<'a>,
    prev: &LayoutTree,
    dirty_paths: &[Vec<usize>],
    viewport_width: f32,
    viewport_height: f32,
) -> LayoutTree<'a> {
    if (viewport_width - prev.root.content.width).abs() > 0.5
        || dirty_paths.is_empty()
    {
        return crate::engine::layout_tree(styled, viewport_width, viewport_height);
    }

    let doc_root = styled.node;
    let cache = LayoutCache::from_tree(prev, dirty_paths, doc_root);

    let ctx = LayoutContext::new(viewport_width, viewport_height);
    let mut text_ctx = TextContext::new();
    let mut rects = Vec::new();
    let root = build_box(styled);
    let root = layout_node(root, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects, &cache);
    LayoutTree { root, rects }
}
