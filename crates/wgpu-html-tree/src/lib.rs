//! Element tree.
//!
//! `Tree` is the root container. It holds a single `Node`. Each `Node` pairs
//! an `Element` (one of the HTML element model structs from
//! `wgpu-html-models`, or raw text) with its child nodes.
//!
//! Models stay pure data. Composition lives here.

use std::{collections::HashMap, ops::Range, time::Duration};

use wgpu_html_models as m;

mod dispatch;
mod events;
mod focus;
mod fonts;
mod profiler;
pub mod query;
mod system_fonts;
pub mod text_edit;
pub mod tree_hook;

pub use dispatch::{
  blur, clipboard_event, dispatch_mouse_down, dispatch_mouse_up, dispatch_pointer_leave, dispatch_pointer_move, focus,
  focus_next, key_down, key_up, resize_event, scroll_event, select_event, selectionchange_event, text_input,
  wheel_event,
};
pub use events::{
  EditCursor, EventCallback, HtmlEvent, HtmlEventType, InteractionSnapshot, InteractionState, Modifier, Modifiers,
  MouseButton, MouseCallback, MouseEvent, SelectionColors, TextCursor, TextSelection,
};
pub use focus::{
  focusable_paths, is_focusable, is_keyboard_focusable, keyboard_focusable_paths, next_in_order, prev_in_order,
};
pub use fonts::{FontFace, FontHandle, FontRegistry, FontStyleAxis};
pub use profiler::{ProfileEntry, Profiler};
pub use query::{Combinator, ComplexSelector, CompoundSelector, SelectorList};
pub use system_fonts::{SystemFontVariant, register_system_fonts, system_font_variants};
pub use tree_hook::{
  TreeHook, TreeHookHandle, TreeHookResponse, TreeLifecycleEvent, TreeLifecyclePhase, TreeLifecycleStage,
  TreeRenderEvent, TreeRenderViewport,
};

#[derive(Debug, Clone, Default)]
pub struct Tree {
  pub root: Option<Node>,
  /// Fonts available to this document. Populated by the host before
  /// layout / paint; consulted by the cascade and the text crate.
  /// See `docs/text.md` §3.
  pub fonts: FontRegistry,
  /// Live interaction state (hover / active / focus / pointer
  /// position / text selection / scroll offsets). Mutated by the
  /// dispatchers in `crate::dispatch` (re-exported as `tree.focus(…)`,
  /// `tree.key_down(…)`, `tree.dispatch_mouse_down(…)`, etc.); the
  /// cascade and paint passes read it but never write.
  pub interaction: InteractionState,
  /// Optional document-local override for the host DPI scale.
  /// When unset, integrations should use the window/system scale
  /// factor they receive from the host toolkit. When set, this
  /// value is used instead, after clamping invalid/non-positive
  /// inputs back to the host scale.
  pub dpi_scale_override: Option<f32>,
  /// How long to keep decoded images in memory after their last use
  /// before reclaiming. `None` leaves the engine default in place
  /// (5 minutes); `Some(d)` overrides it on the next layout pass.
  /// The setting is applied process-wide — successive trees with
  /// different values "win" in last-applied order. Set to a small
  /// value to aggressively free memory on memory-constrained hosts;
  /// raise it to keep images warm across navigations.
  pub asset_cache_ttl: Option<Duration>,
  /// URLs (or local file paths) the host wants pre-fetched into the
  /// image cache before they're referenced from the DOM. The layout
  /// pass walks this list once per pass and dispatches any
  /// not-yet-known URL to the worker pool. Calls are idempotent —
  /// already-cached URLs are skipped — so it's safe to populate
  /// once at startup via [`Tree::preload_asset`] and forget about
  /// it.
  pub preload_queue: Vec<String>,
  /// Stylesheet sources resolved by the host for
  /// `<link rel="stylesheet" href="...">` elements. The engine does
  /// not fetch CSS by itself; integrations can register local,
  /// embedded, or already-fetched stylesheets here, keyed by the
  /// exact `href` used in the document.
  pub linked_stylesheets: HashMap<String, String>,
  /// Host hooks registered on this document. Integration crates emit through
  /// `Tree::emit_*` methods so hook dispatch stays owned by this crate.
  pub hooks: Vec<TreeHookHandle>,
  /// Monotonically increasing counter, bumped whenever the DOM
  /// structure or content changes (custom properties, form control
  /// values, etc.). The pipeline cache compares this against its
  /// stored value to detect mutations that require re-cascade + relayout.
  pub generation: u64,
  /// Bumped only when stylesheets or element selectors (tag / id /
  /// class) change.  Stays stable across inline-style-only edits
  /// so that [`wgpu_html::PipelineAction::LayoutOnly`] can skip the
  /// full CSS cascade.
  pub cascade_generation: u64,
  /// Optional per-frame profiler. When `Some`, the cascade → layout
  /// → paint pipeline records each stage's wall-clock duration.
  /// Cleared at the start of every frame.
  pub profiler: Option<Profiler>,
  /// Base directory for resolving relative asset paths (images,
  /// stylesheets, fonts). When set, `<img src="logo.png">` resolves
  /// to `{asset_root}/logo.png`. Set via [`Tree::set_asset_root`].
  pub asset_root: Option<std::path::PathBuf>,
}

impl Tree {
  pub fn new(root: Node) -> Self {
    Self {
      root: Some(root),
      fonts: FontRegistry::new(),
      interaction: InteractionState::default(),
      dpi_scale_override: None,
      asset_cache_ttl: None,
      preload_queue: Vec::new(),
      linked_stylesheets: HashMap::new(),
      hooks: Vec::new(),
      generation: 0,
      cascade_generation: 0,
      profiler: None,
      asset_root: None,
    }
  }

  /// Override this tree's CSS-pixel to physical-pixel scale.
  /// Pass `None` to return to the host/window scale factor.
  pub fn set_dpi_scale_override(&mut self, scale: Option<f32>) {
    self.dpi_scale_override = scale.filter(|s| s.is_finite() && *s > 0.0);
  }

  /// Resolve the scale this tree should use for layout and paint.
  /// The host scale normally comes from `window.scale_factor()` or
  /// the embedding toolkit's equivalent.
  pub fn effective_dpi_scale(&self, host_scale: f32) -> f32 {
    self
      .dpi_scale_override
      .filter(|s| s.is_finite() && *s > 0.0)
      .or_else(|| (host_scale.is_finite() && host_scale > 0.0).then_some(host_scale))
      .unwrap_or(1.0)
  }

  /// Set a CSS custom property on the document root. Shorthand for
  /// `tree.root.set_custom_property(name, value)`.
  pub fn set_custom_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
    if let Some(root) = &mut self.root {
      root.set_custom_property(name, value);
      self.generation += 1;
    }
  }

  /// Remove a programmatic custom property from the document root.
  pub fn remove_custom_property(&mut self, name: &str) -> Option<String> {
    let v = self.root.as_mut()?.remove_custom_property(name);
    if v.is_some() {
      self.generation += 1;
    }
    v
  }

  /// Register a font face with this document and return its handle.
  /// Re-registering a face with the same `(family, weight, style)`
  /// overrides the previous one (later registration wins on ties
  /// during matching).
  pub fn register_font(&mut self, face: FontFace) -> FontHandle {
    self.fonts.register(face)
  }

  /// Queue an image URL (or local filesystem path) for pre-loading.
  /// The next call to `paint_tree*` / `compute_layout` will dispatch
  /// the URL to the image-fetch worker pool if it's not already in
  /// the cache, so the first frame that actually needs the image
  /// doesn't wait. Duplicates are de-duped — calling this with the
  /// same URL twice is a no-op the second time.
  ///
  /// Typical usage at startup:
  /// ```ignore
  /// tree.preload_asset("https://example.com/hero.png");
  /// tree.preload_asset("assets/icons/menu.png");
  /// ```
  pub fn preload_asset(&mut self, src: impl Into<String>) {
    let s = src.into();
    if s.is_empty() || self.preload_queue.iter().any(|u| u == &s) {
      return;
    }
    self.preload_queue.push(s);
  }

  /// Register CSS text for a document stylesheet link.
  ///
  /// This resolves links by exact `href` string. Relative paths,
  /// filesystem lookup, package embeds, and network fetching remain
  /// host responsibilities so the core renderer stays deterministic.
  pub fn register_linked_stylesheet(&mut self, href: impl Into<String>, css: impl Into<String>) {
    let href = href.into();
    if href.trim().is_empty() {
      return;
    }
    self.linked_stylesheets.insert(href, css.into());
    self.generation += 1;
    self.cascade_generation += 1;
  }

  /// Remove a previously registered linked stylesheet.
  pub fn remove_linked_stylesheet(&mut self, href: &str) -> Option<String> {
    let removed = self.linked_stylesheets.remove(href);
    if removed.is_some() {
      self.generation += 1;
      self.cascade_generation += 1;
    }
    removed
  }

  /// Serialize the tree to an HTML string. Linked stylesheets are
  /// emitted as `<style>` blocks inside `<head>`. Useful for
  /// debugging layout in a real browser.
  pub fn to_html(&self) -> String {
    let mut buf = String::with_capacity(8192);
    buf.push_str("<!DOCTYPE html>\n");
    if let Some(root) = &self.root {
      root.write_html_into(&mut buf, &self.linked_stylesheets, false);
    }
    buf
  }

  /// Serialise a single node to its outer HTML (including the
  /// node itself and all descendants). Returns a complete HTML
  /// fragment — no `<!DOCTYPE>` prefix.
  pub fn node_to_html(&self, path: &[usize]) -> Option<String> {
    let node = self.root.as_ref()?.at_path(path)?;
    let mut buf = String::with_capacity(1024);
    node.write_html_into(&mut buf, &self.linked_stylesheets, false);
    Some(buf)
  }

  /// Set the base directory for resolving relative asset paths.
  ///
  /// After this call, relative `src` / `href` references in `<img>`,
  /// `<link>`, fonts, etc. resolve against this directory.
  pub fn set_asset_root(&mut self, path: impl Into<std::path::PathBuf>) {
    self.asset_root = Some(path.into());
  }

  /// Resolve a relative path against the asset root. Returns a
  /// borrowed reference when no transformation is needed (path is
  /// already absolute, is a URL, or no asset root is set), avoiding
  /// allocation in the common case.
  pub fn resolve_asset_path<'a>(&self, relative: &'a str) -> std::borrow::Cow<'a, str> {
    use std::borrow::Cow;
    if relative.is_empty() {
      return Cow::Borrowed(relative);
    }
    let path = std::path::Path::new(relative);
    if path.is_absolute() || relative.starts_with("http://") || relative.starts_with("https://") {
      return Cow::Borrowed(relative);
    }
    match &self.asset_root {
      Some(root) => Cow::Owned(root.join(relative).to_string_lossy().into_owned()),
      None => Cow::Borrowed(relative),
    }
  }

  /// Return an immutable reference to the currently focused element,
  /// or `None` if nothing is focused or the focus path is stale.
  ///
  /// Useful for reading the focused form control's value without
  /// walking the path manually.
  pub fn active_element(&self) -> Option<&Node> {
    let path = self.interaction.focus_path.as_deref()?;
    self.root.as_ref()?.at_path(path)
  }

  /// Return a mutable reference to the currently focused element,
  /// or `None` if nothing is focused or the focus path is stale.
  pub fn active_element_mut(&mut self) -> Option<&mut Node> {
    let path = self.interaction.focus_path.clone()?;
    self.root.as_mut()?.at_path_mut(&path)
  }

  /// Find the first descendant whose `id` attribute equals `id`,
  /// document-order. Returns `None` if no element matches or the
  /// tree is empty.
  ///
  /// ```ignore
  /// if let Some(el) = tree.get_element_by_id("submit") {
  ///     el.on_click.push(std::sync::Arc::new(|ev| {
  ///         eprintln!("clicked at {:?}", ev.pos);
  ///     }));
  /// }
  /// ```
  pub fn get_element_by_id(&mut self, id: &str) -> Option<&mut Node> {
    self.root.as_mut()?.find_by_id_mut(id)
  }

  pub fn get_element_by_class_name(&self, class_name: &str) -> Option<&Node> {
    self.root.as_ref()?.get_element_by_class_name(class_name)
  }

  pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<&Node> {
    self
      .root
      .as_ref()
      .map(|r| r.get_elements_by_class_name(class_name))
      .unwrap_or_default()
  }

  pub fn get_element_by_name(&self, name: &str) -> Option<&Node> {
    self.root.as_ref()?.get_element_by_name(name)
  }

  pub fn get_elements_by_name(&self, name: &str) -> Vec<&Node> {
    self
      .root
      .as_ref()
      .map(|r| r.get_elements_by_name(name))
      .unwrap_or_default()
  }

  pub fn get_element_by_tag_name(&self, tag_name: &str) -> Option<&Node> {
    self.root.as_ref()?.get_element_by_tag_name(tag_name)
  }

  pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<&Node> {
    self
      .root
      .as_ref()
      .map(|r| r.get_elements_by_tag_name(tag_name))
      .unwrap_or_default()
  }

  /// Return paths to every element whose `class` attribute contains
  /// `class_name` as a whitespace-separated token. Document order.
  pub fn find_elements_by_class_name(&self, class_name: &str) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if let Some(root) = self.root.as_ref() {
      collect_class_name_paths(root, class_name, &mut vec![], &mut out);
    }
    out
  }

  /// Return paths to every element whose `name` attribute equals
  /// `name` (case-insensitive, but the value is stored as-is).
  /// Document order.
  pub fn find_elements_by_name(&self, name: &str) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if let Some(root) = self.root.as_ref() {
      collect_name_paths(root, name, &mut vec![], &mut out);
    }
    out
  }

  /// Return paths to every element whose tag name equals
  /// `tag_name` (case-insensitive). Document order.
  pub fn find_elements_by_tag_name(&self, tag_name: &str) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if let Some(root) = self.root.as_ref() {
      collect_tag_name_paths(root, tag_name, &mut vec![], &mut out);
    }
    out
  }

  /// Clone the child nodes held by a `<template id="...">`.
  ///
  /// Template contents remain inert while they stay inside the
  /// template. Cloning returns detached nodes that callers can
  /// insert into the live tree with [`Tree::insert_template_content`]
  /// or regular `Node` mutation APIs.
  pub fn clone_template_content_by_id(&self, template_id: &str) -> Option<Vec<Node>> {
    let node = self.root.as_ref()?.find_by_id(template_id)?;
    matches!(node.element, Element::Template(_)).then(|| node.children.clone())
  }

  /// Clone a template's content and insert it into the node at
  /// `parent_path` before `index`.
  ///
  /// Returns the inserted child index range on success. `index` may
  /// equal the current child count to append; larger indices fail
  /// without mutating the tree.
  pub fn insert_template_content(
    &mut self,
    template_id: &str,
    parent_path: &[usize],
    index: usize,
  ) -> Option<Range<usize>> {
    let content = self.clone_template_content_by_id(template_id)?;
    let count = content.len();
    let parent = self.root.as_mut()?.at_path_mut(parent_path)?;
    if index > parent.children.len() {
      return None;
    }
    parent.children.splice(index..index, content);
    if count > 0 {
      self.generation += 1;
    }
    Some(index..index + count)
  }

  /// Clone a template's content and append it to the first element
  /// with `parent_id`.
  pub fn append_template_content_to_id(&mut self, template_id: &str, parent_id: &str) -> Option<Range<usize>> {
    let content = self.clone_template_content_by_id(template_id)?;
    let count = content.len();
    let parent = self.root.as_mut()?.find_by_id_mut(parent_id)?;
    let start = parent.children.len();
    parent.children.extend(content);
    if count > 0 {
      self.generation += 1;
    }
    Some(start..start + count)
  }

  /// Insert a node into the tree at `parent_path[index]`.
  ///
  /// Bumps `generation` and fires `on_element_added` hooks.
  /// Returns `true` on success.
  pub fn insert_node(&mut self, parent_path: &[usize], index: usize, node: Node) -> bool {
    let node_for_hook = node.clone();
    let Some(parent) = self.root.as_mut().and_then(|r| r.at_path_mut(parent_path)) else {
      return false;
    };
    if index > parent.children.len() {
      return false;
    }
    parent.children.insert(index, node);
    self.generation += 1;
    self.emit_element_added(&node_for_hook);
    true
  }

  /// Append a node to the children of the node at `parent_path`.
  ///
  /// Bumps `generation` and fires `on_element_added` hooks.
  /// Returns the index of the appended child.
  pub fn append_node(&mut self, parent_path: &[usize], node: Node) -> Option<usize> {
    let node_for_hook = node.clone();
    let parent = self.root.as_mut()?.at_path_mut(parent_path)?;
    let index = parent.children.len();
    parent.children.push(node);
    self.generation += 1;
    self.emit_element_added(&node_for_hook);
    Some(index)
  }

  /// Remove the node at `path` from the tree.
  ///
  /// Bumps `generation` and fires `on_element_removed` hooks.
  /// Returns the removed node on success.
  pub fn remove_node(&mut self, path: &[usize]) -> Option<Node> {
    if path.is_empty() {
      return None;
    }
    let parent_path = &path[..path.len() - 1];
    let child_index = *path.last()?;
    let parent = self.root.as_mut()?.at_path_mut(parent_path)?;
    if child_index >= parent.children.len() {
      return None;
    }
    let removed = parent.children.remove(child_index);
    self.generation += 1;
    self.emit_element_removed(&removed);
    Some(removed)
  }

  /// Override the colors used when painting selected text.
  pub fn set_selection_colors(&mut self, background: [f32; 4], foreground: [f32; 4]) {
    self.interaction.selection_colors = SelectionColors { background, foreground };
  }

  /// Clear any active text selection and exit selection-drag mode.
  pub fn clear_selection(&mut self) {
    self.interaction.selection = None;
    self.interaction.selecting_text = false;
  }

  /// Return the current cursor position in document space, or
  /// `None` if the pointer is outside the window / surface.
  pub fn cursor_position(&self) -> Option<(f32, f32)> {
    self.interaction.pointer_pos
  }

  /// Return the deepest hovered element, or `None` if nothing is
  /// hovered or the hover path is stale.
  pub fn hovered_element(&self) -> Option<&Node> {
    let path = self.interaction.hover_path.as_deref()?;
    self.root.as_ref()?.at_path(path)
  }

  /// Check whether the element at `path` is in the current hover
  /// chain (i.e. `path` is a prefix of the deepest hover path).
  pub fn is_hovered(&self, path: &[usize]) -> bool {
    self
      .interaction
      .hover_path
      .as_deref()
      .is_some_and(|hp| hp.len() >= path.len() && &hp[..path.len()] == path)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeRect {
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
}

#[derive(Clone)]
pub struct Node {
  pub element: Element,
  pub children: Vec<Node>,
  /// CSS custom properties set programmatically on this node.
  /// Behaves as if declared in an inline `style` attribute — the
  /// cascade sees them after author/inline layers, and they
  /// inherit to descendants just like CSS-declared custom
  /// properties. Keys include the `--` prefix (e.g. `"--color"`).
  pub custom_properties: HashMap<String, String>,
  /// Fires when a primary-button press *and* the matching release
  /// both land inside this node's subtree. Bubbles target → root.
  /// Multiple handlers accumulate — calling `.on_click()` again
  /// adds a listener rather than replacing the previous one.
  pub on_click: Vec<MouseCallback>,
  /// Fires on every primary-button press, target → root.
  /// Multiple handlers accumulate (like `addEventListener`).
  pub on_mouse_down: Vec<MouseCallback>,
  /// Fires on every primary-button release, target → root.
  pub on_mouse_up: Vec<MouseCallback>,
  /// Fires on every pointer move while over this node's subtree.
  /// Bubbles target → root.
  pub on_mouse_move: Vec<MouseCallback>,
  /// Fires when the pointer enters this node's subtree (root-first
  /// across the entered chain). No bubbling beyond the entered set.
  pub on_mouse_enter: Vec<MouseCallback>,
  /// Fires when the pointer leaves this node's subtree
  /// (deepest-first across the left chain).
  pub on_mouse_leave: Vec<MouseCallback>,
  /// Fires on `keydown` dispatched to this node. Bubbles target → root.
  /// Receives the full `HtmlEvent::Keyboard` variant.
  pub on_keydown: Vec<EventCallback>,
  /// Fires on `keyup` dispatched to this node. Bubbles target → root.
  pub on_keyup: Vec<EventCallback>,
  /// Fires on `focus` (does not bubble). Target only.
  pub on_focus: Vec<EventCallback>,
  /// Fires on `blur` (does not bubble). Target only.
  pub on_blur: Vec<EventCallback>,
  /// Fires on `focusin` (bubbles target → root).
  pub on_focusin: Vec<EventCallback>,
  /// Fires on `focusout` (bubbles target → root).
  pub on_focusout: Vec<EventCallback>,
  /// Fires on `input` dispatched to this node. Bubbles target → root.
  pub on_input: Vec<EventCallback>,
  /// Fires on `beforeinput` before text mutation. Bubbles target → root.
  /// `cancelable: true` — calling `ev.prevent_default()` skips the edit.
  pub on_beforeinput: Vec<EventCallback>,
  /// Fires on `change` dispatched to this node. Bubbles target → root.
  pub on_change: Vec<EventCallback>,
  /// Fires on `wheel` dispatched to this node. Bubbles target → root.
  pub on_wheel: Vec<EventCallback>,
  /// Fires on `dblclick` (two clicks within 300 ms on same element).
  /// Bubbles target → root.
  pub on_dblclick: Vec<MouseCallback>,
  /// Fires on `contextmenu` (secondary-button release). Bubbles target → root.
  pub on_contextmenu: Vec<MouseCallback>,
  /// Fires on `auxclick` (middle-button release). Bubbles target → root.
  pub on_auxclick: Vec<MouseCallback>,
  /// Fires on `dragstart` (mousedown + ≥5 px movement on draggable element).
  /// Does not bubble — fires on the source element only.
  pub on_dragstart: Vec<MouseCallback>,
  /// Fires on `dragend` when the drag operation ends (mouseup after dragstart).
  /// Does not bubble — fires on the source element only.
  pub on_dragend: Vec<MouseCallback>,
  /// Fires on `drop` when the dragged element is released over a target.
  /// Does not bubble — fires on the drop target element only.
  pub on_drop: Vec<MouseCallback>,
  /// Fires on `drag` on the source element during pointer moves while dragging.
  /// Does not bubble — fires on the source only.
  pub on_drag: Vec<MouseCallback>,
  /// Fires on `dragover` on the element under the pointer while dragging.
  /// Does not bubble — fires on the hovered target only.
  pub on_dragover: Vec<MouseCallback>,
  /// Fires on `dragenter` when a drag enters this element (root-first,
  /// like `mouseenter`). Does not bubble.
  pub on_dragenter: Vec<MouseCallback>,
  /// Fires on `dragleave` when a drag leaves this element (deepest-first,
  /// like `mouseleave`). Does not bubble.
  pub on_dragleave: Vec<MouseCallback>,
  /// Whether this element can be dragged. When `true`, a primary-button
  /// mousedown followed by ≥5 px movement fires `dragstart`.
  pub draggable: bool,
  /// Fires on `copy` dispatched to this node. Bubbles target → root.
  pub on_copy: Vec<EventCallback>,
  /// Fires on `cut` dispatched to this node. Bubbles target → root.
  pub on_cut: Vec<EventCallback>,
  /// Fires on `paste` dispatched to this node. Bubbles target → root.
  pub on_paste: Vec<EventCallback>,
  /// Fires on `scroll` when this element's scroll position changes.
  /// Does not bubble (per DOM spec). Target only.
  pub on_scroll: Vec<EventCallback>,
  /// Fires on `select` when text selection changes in an
  /// `<input>` or `<textarea>`. Does not bubble. Target only.
  pub on_select: Vec<EventCallback>,
  /// General-purpose handler that receives the full [`HtmlEvent`] for any
  /// event dispatched to this node, fired *after* the type-specific slot
  /// (e.g. `on_click`). Use this for keyboard, focus, wheel, or any event
  /// without a dedicated slot. Multiple handlers accumulate.
  pub on_event: Vec<EventCallback>,
  /// Computed layout rectangle for this node (content-box).
  /// Populated by the layout pass; `None` if not yet laid out.
  pub rect: Option<NodeRect>,
  /// Raw HTML attributes as parsed from the source (name, value pairs).
  /// Includes all attributes — standard, non-standard, and custom.
  pub raw_attrs: Vec<(String, String)>,
}

impl std::fmt::Debug for Node {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // The callback slots can't be Debug-printed; just note whether
    // each is wired so the tree's structure stays inspectable.
    f.debug_struct("Node")
      .field("element", &self.element)
      .field("children", &self.children)
      .field("custom_properties", &self.custom_properties)
      .field("on_click", &format!("{} handlers", self.on_click.len()))
      .field("on_mouse_down", &format!("{} handlers", self.on_mouse_down.len()))
      .field("on_mouse_up", &format!("{} handlers", self.on_mouse_up.len()))
      .field("on_mouse_move", &format!("{} handlers", self.on_mouse_move.len()))
      .field("on_mouse_enter", &format!("{} handlers", self.on_mouse_enter.len()))
      .field("on_mouse_leave", &format!("{} handlers", self.on_mouse_leave.len()))
      .field("on_keydown", &format!("{} handlers", self.on_keydown.len()))
      .field("on_keyup", &format!("{} handlers", self.on_keyup.len()))
      .field("on_focus", &format!("{} handlers", self.on_focus.len()))
      .field("on_blur", &format!("{} handlers", self.on_blur.len()))
      .field("on_focusin", &format!("{} handlers", self.on_focusin.len()))
      .field("on_focusout", &format!("{} handlers", self.on_focusout.len()))
      .field("on_input", &format!("{} handlers", self.on_input.len()))
      .field("on_beforeinput", &format!("{} handlers", self.on_beforeinput.len()))
      .field("on_change", &format!("{} handlers", self.on_change.len()))
      .field("on_wheel", &format!("{} handlers", self.on_wheel.len()))
      .field("on_dblclick", &format!("{} handlers", self.on_dblclick.len()))
      .field("on_contextmenu", &format!("{} handlers", self.on_contextmenu.len()))
      .field("on_auxclick", &format!("{} handlers", self.on_auxclick.len()))
      .field("on_dragstart", &format!("{} handlers", self.on_dragstart.len()))
      .field("on_dragend", &format!("{} handlers", self.on_dragend.len()))
      .field("on_drop", &format!("{} handlers", self.on_drop.len()))
      .field("on_drag", &format!("{} handlers", self.on_drag.len()))
      .field("on_dragover", &format!("{} handlers", self.on_dragover.len()))
      .field("on_dragenter", &format!("{} handlers", self.on_dragenter.len()))
      .field("on_dragleave", &format!("{} handlers", self.on_dragleave.len()))
      .field("draggable", &self.draggable)
      .field("on_copy", &format!("{} handlers", self.on_copy.len()))
      .field("on_cut", &format!("{} handlers", self.on_cut.len()))
      .field("on_paste", &format!("{} handlers", self.on_paste.len()))
      .field("on_scroll", &format!("{} handlers", self.on_scroll.len()))
      .field("on_select", &format!("{} handlers", self.on_select.len()))
      .field("on_event", &format!("{} handlers", self.on_event.len()))
      .finish()
  }
}

impl Node {
  pub fn new(element: impl Into<Element>) -> Self {
    Self {
      element: element.into(),
      children: Vec::new(),
      custom_properties: HashMap::new(),
      on_click: Vec::new(),
      on_mouse_down: Vec::new(),
      on_mouse_up: Vec::new(),
      on_mouse_move: Vec::new(),
      on_mouse_enter: Vec::new(),
      on_mouse_leave: Vec::new(),
      on_keydown: Vec::new(),
      on_keyup: Vec::new(),
      on_focus: Vec::new(),
      on_blur: Vec::new(),
      on_focusin: Vec::new(),
      on_focusout: Vec::new(),
      on_input: Vec::new(),
      on_beforeinput: Vec::new(),
      on_change: Vec::new(),
      on_wheel: Vec::new(),
      on_dblclick: Vec::new(),
      on_contextmenu: Vec::new(),
      on_auxclick: Vec::new(),
      on_dragstart: Vec::new(),
      on_dragend: Vec::new(),
      on_drop: Vec::new(),
      on_drag: Vec::new(),
      on_dragover: Vec::new(),
      on_dragenter: Vec::new(),
      on_dragleave: Vec::new(),
      draggable: false,
      on_copy: Vec::new(),
      on_cut: Vec::new(),
      on_paste: Vec::new(),
      on_scroll: Vec::new(),
      on_select: Vec::new(),
      on_event: Vec::new(),
      rect: None,
    }
  }

  pub fn with_children(mut self, children: Vec<Node>) -> Self {
    self.children = children;
    self
  }

  pub fn push(&mut self, child: Node) -> &mut Self {
    self.children.push(child);
    self
  }

  /// Set a CSS custom property on this node. Behaves as if declared
  /// in an inline `style` attribute — inherits to descendants and
  /// is available via `var(--name)` in CSS values.
  ///
  /// `name` must include the `--` prefix (e.g. `"--theme-color"`).
  pub fn set_custom_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
    self.custom_properties.insert(name.into(), value.into());
  }

  /// Remove a previously set programmatic custom property.
  pub fn remove_custom_property(&mut self, name: &str) -> Option<String> {
    self.custom_properties.remove(name)
  }

  /// Read a programmatic custom property set on this node (does NOT
  /// walk the cascade or ancestors).
  pub fn custom_property(&self, name: &str) -> Option<&str> {
    self.custom_properties.get(name).map(|s| s.as_str())
  }

  /// Depth-first search for a descendant (or `self`) whose `id`
  /// attribute equals `id`. Document order; first match wins.
  pub fn find_by_id(&self, id: &str) -> Option<&Node> {
    if self.element.id() == Some(id) {
      return Some(self);
    }
    for child in &self.children {
      if let Some(found) = child.find_by_id(id) {
        return Some(found);
      }
    }
    None
  }

  pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Node> {
    if self.element.id() == Some(id) {
      return Some(self);
    }
    for child in &mut self.children {
      if let Some(found) = child.find_by_id_mut(id) {
        return Some(found);
      }
    }
    None
  }

  /// Depth-first search for a descendant (or `self`) whose `class`
  /// attribute contains `class_name` as a whitespace-separated
  /// token. Document order; first match wins.
  pub fn get_element_by_class_name(&self, class_name: &str) -> Option<&Node> {
    if self
      .element
      .class()
      .is_some_and(|c| c.split_ascii_whitespace().any(|t| t == class_name))
    {
      return Some(self);
    }
    for child in &self.children {
      if let Some(found) = child.get_element_by_class_name(class_name) {
        return Some(found);
      }
    }
    None
  }

  pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<&Node> {
    let mut out = Vec::new();
    collect_class_name_nodes(self, class_name, &mut out);
    out
  }

  /// Depth-first search for a descendant (or `self`) whose `name`
  /// attribute equals `name`. Document order; first match wins.
  pub fn get_element_by_name(&self, name: &str) -> Option<&Node> {
    if self.element.attr("name").as_deref() == Some(name) {
      return Some(self);
    }
    for child in &self.children {
      if let Some(found) = child.get_element_by_name(name) {
        return Some(found);
      }
    }
    None
  }

  pub fn get_elements_by_name(&self, name: &str) -> Vec<&Node> {
    let mut out = Vec::new();
    collect_name_nodes(self, name, &mut out);
    out
  }

  /// Depth-first search for a descendant (or `self`) whose tag
  /// name matches `tag_name` (case-insensitive). Document order;
  /// first match wins.
  pub fn get_element_by_tag_name(&self, tag_name: &str) -> Option<&Node> {
    if self.element.tag_name().eq_ignore_ascii_case(tag_name) {
      return Some(self);
    }
    for child in &self.children {
      if let Some(found) = child.get_element_by_tag_name(tag_name) {
        return Some(found);
      }
    }
    None
  }

  pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<&Node> {
    let mut out = Vec::new();
    collect_tag_name_nodes(self, tag_name, &mut out);
    out
  }

  /// Walk a child-index path from this node to a descendant. An empty
  /// path returns `Some(self)`. Returns `None` if any index is out of
  /// bounds.
  /// Walk a child-index path and return an immutable reference to
  /// the node at the end. Returns `None` for out-of-bounds indices.
  pub fn at_path(&self, path: &[usize]) -> Option<&Node> {
    let mut cursor: &Node = self;
    for &i in path {
      cursor = cursor.children.get(i)?;
    }
    Some(cursor)
  }

  pub fn at_path_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
    let mut cursor: &mut Node = self;
    for &i in path {
      cursor = cursor.children.get_mut(i)?;
    }
    Some(cursor)
  }

  /// Walk a child-index path and collect every node visited, ordered
  /// deepest descendant → root. The first element is the deepest hit;
  /// the last is `self`.
  ///
  /// Soundness: the returned `&mut` references all alias into nested
  /// subtrees of the same borrow — each is an ancestor of the next.
  /// Two of them must never be dereferenced concurrently. Walking the
  /// chain one step at a time (event bubbling, etc.) is fine.
  pub fn ancestry_at_path_mut(&mut self, path: &[usize]) -> Vec<&mut Node> {
    let mut out: Vec<&mut Node> = Vec::with_capacity(path.len() + 1);
    // SAFETY: every pointer is derived from `self`'s exclusive
    // borrow and points at a strict subtree of the previous one.
    // We rely on the documented contract that callers do not
    // access two of the returned references simultaneously.
    unsafe {
      let mut cursor: *mut Node = self as *mut Node;
      out.push(&mut *cursor);
      for &i in path {
        let children: *mut Vec<Node> = &raw mut (*cursor).children;
        if i >= (*children).len() {
          break;
        }
        cursor = (*children).as_mut_ptr().add(i);
        out.push(&mut *cursor);
      }
    }
    out.reverse();
    out
  }
}

/// One variant per HTML element kind, plus raw text.
#[derive(Debug, Clone)]
pub enum Element {
  Text(String),

  Html(m::Html),
  Head(m::Head),
  Body(m::Body),
  Title(m::Title),
  Meta(m::Meta),
  Link(m::Link),
  StyleElement(m::StyleElement),
  Script(m::Script),
  Noscript(m::Noscript),

  H1(m::H1),
  H2(m::H2),
  H3(m::H3),
  H4(m::H4),
  H5(m::H5),
  H6(m::H6),
  P(m::P),
  Br(m::Br),
  Hr(m::Hr),
  Pre(m::Pre),
  Blockquote(m::Blockquote),
  Address(m::Address),

  Span(m::Span),
  A(m::A),
  Strong(m::Strong),
  B(m::B),
  Em(m::Em),
  I(m::I),
  U(m::U),
  S(m::S),
  Small(m::Small),
  Mark(m::Mark),
  Code(m::Code),
  Kbd(m::Kbd),
  Samp(m::Samp),
  Var(m::Var),
  Abbr(m::Abbr),
  Cite(m::Cite),
  Dfn(m::Dfn),
  Sub(m::Sub),
  Sup(m::Sup),
  Time(m::Time),

  Ul(m::Ul),
  Ol(m::Ol),
  Li(m::Li),
  Dl(m::Dl),
  Dt(m::Dt),
  Dd(m::Dd),

  Header(m::Header),
  Nav(m::Nav),
  Main(m::Main),
  Section(m::Section),
  Article(m::Article),
  Aside(m::Aside),
  Footer(m::Footer),
  Div(m::Div),

  Img(m::Img),
  Picture(m::Picture),
  Source(m::Source),
  Video(m::Video),
  Audio(m::Audio),
  Track(m::Track),
  Iframe(m::Iframe),
  Canvas(m::Canvas),
  Svg(m::Svg),
  SvgPath(m::SvgPath),

  Table(m::Table),
  Caption(m::Caption),
  Thead(m::Thead),
  Tbody(m::Tbody),
  Tfoot(m::Tfoot),
  Tr(m::Tr),
  Th(m::Th),
  Td(m::Td),
  Colgroup(m::Colgroup),
  Col(m::Col),

  Form(m::Form),
  Label(m::Label),
  Input(m::Input),
  Textarea(m::Textarea),
  Button(m::Button),
  Select(m::Select),
  OptionElement(m::OptionElement),
  Optgroup(m::Optgroup),
  Fieldset(m::Fieldset),
  Legend(m::Legend),
  Datalist(m::Datalist),
  Output(m::Output),
  Progress(m::Progress),
  Meter(m::Meter),

  Details(m::Details),
  Summary(m::Summary),
  Dialog(m::Dialog),
  Template(m::Template),
  Slot(m::Slot),
  Del(m::Del),
  Ins(m::Ins),
  Bdi(m::Bdi),
  Bdo(m::Bdo),
  Wbr(m::Wbr),
  Data(m::Data),
  Ruby(m::Ruby),
  Rt(m::Rt),
  Rp(m::Rp),
}

/// Generate `From<T> for Element` impls so `Node::new(Div::default())` works.
macro_rules! element_from {
    ($($variant:ident => $ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for Element {
                #[inline]
                fn from(v: $ty) -> Self { Element::$variant(v) }
            }
        )*
    };
}

impl From<String> for Element {
  fn from(s: String) -> Self {
    Element::Text(s)
  }
}
impl From<&str> for Element {
  fn from(s: &str) -> Self {
    Element::Text(s.to_owned())
  }
}

element_from! {
    Html => m::Html, Head => m::Head, Body => m::Body, Title => m::Title,
    Meta => m::Meta, Link => m::Link, StyleElement => m::StyleElement,
    Script => m::Script, Noscript => m::Noscript,

    H1 => m::H1, H2 => m::H2, H3 => m::H3, H4 => m::H4, H5 => m::H5, H6 => m::H6,
    P => m::P, Br => m::Br, Hr => m::Hr, Pre => m::Pre,
    Blockquote => m::Blockquote, Address => m::Address,

    Span => m::Span, A => m::A, Strong => m::Strong, B => m::B, Em => m::Em,
    I => m::I, U => m::U, S => m::S, Small => m::Small, Mark => m::Mark,
    Code => m::Code, Kbd => m::Kbd, Samp => m::Samp, Var => m::Var,
    Abbr => m::Abbr, Cite => m::Cite, Dfn => m::Dfn, Sub => m::Sub, Sup => m::Sup,
    Time => m::Time,

    Ul => m::Ul, Ol => m::Ol, Li => m::Li, Dl => m::Dl, Dt => m::Dt, Dd => m::Dd,

    Header => m::Header, Nav => m::Nav, Main => m::Main, Section => m::Section,
    Article => m::Article, Aside => m::Aside, Footer => m::Footer, Div => m::Div,

    Img => m::Img, Picture => m::Picture, Source => m::Source, Video => m::Video,
    Audio => m::Audio, Track => m::Track, Iframe => m::Iframe, Canvas => m::Canvas,
    Svg => m::Svg, SvgPath => m::SvgPath,

    Table => m::Table, Caption => m::Caption, Thead => m::Thead, Tbody => m::Tbody,
    Tfoot => m::Tfoot, Tr => m::Tr, Th => m::Th, Td => m::Td,
    Colgroup => m::Colgroup, Col => m::Col,

    Form => m::Form, Label => m::Label, Input => m::Input, Textarea => m::Textarea,
    Button => m::Button, Select => m::Select, OptionElement => m::OptionElement,
    Optgroup => m::Optgroup, Fieldset => m::Fieldset, Legend => m::Legend,
    Datalist => m::Datalist, Output => m::Output, Progress => m::Progress,
    Meter => m::Meter,

    Details => m::Details, Summary => m::Summary, Dialog => m::Dialog,
    Template => m::Template, Slot => m::Slot, Del => m::Del, Ins => m::Ins,
    Bdi => m::Bdi, Bdo => m::Bdo, Wbr => m::Wbr, Data => m::Data,
    Ruby => m::Ruby, Rt => m::Rt, Rp => m::Rp,
}

/// Same variant list used for any "do this for every element" dispatch.
/// `Text` is excluded — it has no attributes.
macro_rules! all_element_variants {
  ($cb:ident) => {
    $cb!(
      Html,
      Head,
      Body,
      Title,
      Meta,
      Link,
      StyleElement,
      Script,
      Noscript,
      H1,
      H2,
      H3,
      H4,
      H5,
      H6,
      P,
      Br,
      Hr,
      Pre,
      Blockquote,
      Address,
      Span,
      A,
      Strong,
      B,
      Em,
      I,
      U,
      S,
      Small,
      Mark,
      Code,
      Kbd,
      Samp,
      Var,
      Abbr,
      Cite,
      Dfn,
      Sub,
      Sup,
      Time,
      Ul,
      Ol,
      Li,
      Dl,
      Dt,
      Dd,
      Header,
      Nav,
      Main,
      Section,
      Article,
      Aside,
      Footer,
      Div,
      Img,
      Picture,
      Source,
      Video,
      Audio,
      Track,
      Iframe,
      Canvas,
      Svg,
      SvgPath,
      Table,
      Caption,
      Thead,
      Tbody,
      Tfoot,
      Tr,
      Th,
      Td,
      Colgroup,
      Col,
      Form,
      Label,
      Input,
      Textarea,
      Button,
      Select,
      OptionElement,
      Optgroup,
      Fieldset,
      Legend,
      Datalist,
      Output,
      Progress,
      Meter,
      Details,
      Summary,
      Dialog,
      Template,
      Slot,
      Del,
      Ins,
      Bdi,
      Bdo,
      Wbr,
      Data,
      Ruby,
      Rt,
      Rp,
    )
  };
}

impl Element {
  /// `id` HTML attribute on this element, if set. `Text` has no
  /// attributes and returns `None`.
  pub fn id(&self) -> Option<&str> {
    macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.id.as_deref(),)*
                }
            };
        }
    all_element_variants!(arms)
  }

  /// `tabindex` HTML attribute on this element, if set. `Text`
  /// returns `None`.
  pub fn tabindex(&self) -> Option<i32> {
    macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.tabindex,)*
                }
            };
        }
    all_element_variants!(arms)
  }

  /// `class` HTML attribute on this element, if set. `Text`
  /// returns `None`. The returned string is the raw attribute
  /// value — split on ASCII whitespace to enumerate classes.
  pub fn class(&self) -> Option<&str> {
    macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.class.as_deref(),)*
                }
            };
        }
    all_element_variants!(arms)
  }

  /// Look up an HTML attribute by name (case-insensitive).
  /// Returns `None` if the element doesn't carry that attribute
  /// or the slot is empty.
  ///
  /// Coverage is the subset that actually shows up in selectors:
  /// the global attributes (`id`, `class`, `title`, `lang`,
  /// `tabindex`, `hidden`, `style`), `data-*` / `aria-*` entries,
  /// and the most common per-element attributes (`type`, `name`,
  /// `value`, `placeholder`, `href`, `src`, `alt`, `for`,
  /// `content`, plus boolean form attributes `disabled`,
  /// `readonly`, `required`, `checked`, `selected`, `multiple`,
  /// `autofocus`). Boolean attributes return `Some(String::new())`
  /// when present so they participate in `[attr]` presence
  /// filters, and `[attr=""]` matches them — same shape as the
  /// browser's reflection of HTML boolean attributes.
  ///
  /// `Text` nodes have no attributes and always return `None`.
  pub fn attr(&self, name: &str) -> Option<String> {
    let lname = name.to_ascii_lowercase();

    if let Some(v) = self.global_attr(&lname) {
      return Some(v);
    }
    if let Some(suffix) = lname.strip_prefix("data-") {
      return self.data_attr(suffix);
    }
    if let Some(suffix) = lname.strip_prefix("aria-") {
      return self.aria_attr(suffix);
    }
    self.specific_attr(&lname)
  }

  fn global_attr(&self, name: &str) -> Option<String> {
    macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => match name {
                        "id" => e.id.clone(),
                        "class" => e.class.clone(),
                        "title" => e.title.clone(),
                        "lang" => e.lang.clone(),
                        "dir" => e.dir.as_ref().map(|d| {
                            use wgpu_html_models::common::html_enums::HtmlDirection;
                            match d {
                                HtmlDirection::Ltr => "ltr",
                                HtmlDirection::Rtl => "rtl",
                                HtmlDirection::Auto => "auto",
                            }.to_owned()
                        }),
                        "tabindex" => e.tabindex.map(|t| t.to_string()),
                        "hidden" => match e.hidden { Some(true) => Some(String::new()), _ => None },
                        "style" => e.style.clone(),
                        _ => None,
                    },)*
                }
            };
        }
    all_element_variants!(arms)
  }

  fn data_attr(&self, suffix: &str) -> Option<String> {
    macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.data_attrs.get(suffix).cloned(),)*
                }
            };
        }
    all_element_variants!(arms)
  }

  fn aria_attr(&self, suffix: &str) -> Option<String> {
    macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.aria_attrs.get(suffix).cloned(),)*
                }
            };
        }
    all_element_variants!(arms)
  }

  fn specific_attr(&self, name: &str) -> Option<String> {
    // Boolean attribute helper: `Some(true)` becomes the empty
    // string (`[attr]` presence test), anything else `None`.
    fn flag(b: Option<bool>) -> Option<String> {
      match b {
        Some(true) => Some(String::new()),
        _ => None,
      }
    }
    match (name, self) {
      // type
      ("type", Element::Input(e)) => e.r#type.as_ref().map(|t| {
        use m::common::html_enums::InputType::*;
        match t {
          Button => "button",
          Checkbox => "checkbox",
          Color => "color",
          Date => "date",
          DatetimeLocal => "datetime-local",
          Email => "email",
          File => "file",
          Hidden => "hidden",
          Image => "image",
          Month => "month",
          Number => "number",
          Password => "password",
          Radio => "radio",
          Range => "range",
          Reset => "reset",
          Search => "search",
          Submit => "submit",
          Tel => "tel",
          Text => "text",
          Time => "time",
          Url => "url",
          Week => "week",
        }
        .to_owned()
      }),
      ("type", Element::Button(e)) => e.r#type.as_ref().map(|t| {
        use m::common::html_enums::ButtonType::*;
        match t {
          Button => "button",
          Submit => "submit",
          Reset => "reset",
        }
        .to_owned()
      }),
      ("type", Element::Source(e)) => e.r#type.clone(),
      ("type", Element::Script(e)) => e.r#type.clone(),
      ("type", Element::StyleElement(e)) => e.r#type.clone(),
      ("type", Element::Link(e)) => e.r#type.clone(),
      ("type", Element::A(e)) => e.r#type.clone(),

      // name
      ("name", Element::Input(e)) => e.name.clone(),
      ("name", Element::Textarea(e)) => e.name.clone(),
      ("name", Element::Select(e)) => e.name.clone(),
      ("name", Element::Button(e)) => e.name.clone(),
      ("name", Element::Output(e)) => e.name.clone(),
      ("name", Element::Form(e)) => e.name.clone(),
      ("name", Element::Iframe(e)) => e.name.clone(),
      ("name", Element::Slot(e)) => e.name.clone(),
      ("name", Element::Meta(e)) => e.name.clone(),
      ("name", Element::Fieldset(e)) => e.name.clone(),
      ("name", Element::Details(e)) => e.name.clone(),

      // value
      ("value", Element::Input(e)) => e.value.clone(),
      ("value", Element::Button(e)) => e.value.clone(),
      ("value", Element::OptionElement(e)) => e.value.clone(),
      ("value", Element::Data(e)) => e.value.clone(),
      ("value", Element::Progress(e)) => e.value.map(|v| v.to_string()),
      ("value", Element::Meter(e)) => e.value.map(|v| v.to_string()),
      ("value", Element::Li(e)) => e.value.map(|v| v.to_string()),

      // content (meta)
      ("content", Element::Meta(e)) => e.content.clone(),

      // href
      ("href", Element::A(e)) => e.href.clone(),
      ("href", Element::Link(e)) => e.href.clone(),

      // src
      ("src", Element::Img(e)) => e.src.clone(),
      ("src", Element::Iframe(e)) => e.src.clone(),
      ("src", Element::Source(e)) => e.src.clone(),
      ("src", Element::Video(e)) => e.src.clone(),
      ("src", Element::Audio(e)) => e.src.clone(),
      ("src", Element::Track(e)) => e.src.clone(),
      ("src", Element::Script(e)) => e.src.clone(),

      // alt
      ("alt", Element::Img(e)) => e.alt.clone(),

      // for
      ("for", Element::Label(e)) => e.r#for.clone(),
      ("for", Element::Output(e)) => e.r#for.as_ref().map(|v| v.join(" ")),

      // placeholder
      ("placeholder", Element::Input(e)) => e.placeholder.clone(),
      ("placeholder", Element::Textarea(e)) => e.placeholder.clone(),

      // booleans
      ("disabled", Element::Input(e)) => flag(e.disabled),
      ("disabled", Element::Textarea(e)) => flag(e.disabled),
      ("disabled", Element::Select(e)) => flag(e.disabled),
      ("disabled", Element::Button(e)) => flag(e.disabled),
      ("disabled", Element::Optgroup(e)) => flag(e.disabled),
      ("disabled", Element::OptionElement(e)) => flag(e.disabled),
      ("disabled", Element::Fieldset(e)) => flag(e.disabled),

      ("readonly", Element::Input(e)) => flag(e.readonly),
      ("readonly", Element::Textarea(e)) => flag(e.readonly),

      ("required", Element::Input(e)) => flag(e.required),
      ("required", Element::Textarea(e)) => flag(e.required),
      ("required", Element::Select(e)) => flag(e.required),

      ("checked", Element::Input(e)) => flag(e.checked),
      ("selected", Element::OptionElement(e)) => flag(e.selected),

      ("multiple", Element::Input(e)) => flag(e.multiple),
      ("multiple", Element::Select(e)) => flag(e.multiple),

      ("autofocus", Element::Input(e)) => flag(e.autofocus),
      ("autofocus", Element::Textarea(e)) => flag(e.autofocus),
      ("autofocus", Element::Select(e)) => flag(e.autofocus),
      ("autofocus", Element::Button(e)) => flag(e.autofocus),

      ("open", Element::Dialog(e)) => flag(e.open),
      ("open", Element::Details(e)) => flag(e.open),

      _ => None,
    }
  }

  /// Lowercase HTML tag name for this element (e.g. `"div"`,
  /// `"option"`, `"style"`). `Text` returns `"#text"`.
  pub fn tag_name(&self) -> &'static str {
    match self {
      Element::Text(_) => "#text",
      Element::StyleElement(_) => "style",
      Element::OptionElement(_) => "option",
      Element::Html(_) => "html",
      Element::Head(_) => "head",
      Element::Body(_) => "body",
      Element::Title(_) => "title",
      Element::Meta(_) => "meta",
      Element::Link(_) => "link",
      Element::Script(_) => "script",
      Element::Noscript(_) => "noscript",
      Element::H1(_) => "h1",
      Element::H2(_) => "h2",
      Element::H3(_) => "h3",
      Element::H4(_) => "h4",
      Element::H5(_) => "h5",
      Element::H6(_) => "h6",
      Element::P(_) => "p",
      Element::Br(_) => "br",
      Element::Hr(_) => "hr",
      Element::Pre(_) => "pre",
      Element::Blockquote(_) => "blockquote",
      Element::Address(_) => "address",
      Element::Span(_) => "span",
      Element::A(_) => "a",
      Element::Strong(_) => "strong",
      Element::B(_) => "b",
      Element::Em(_) => "em",
      Element::I(_) => "i",
      Element::U(_) => "u",
      Element::S(_) => "s",
      Element::Small(_) => "small",
      Element::Mark(_) => "mark",
      Element::Code(_) => "code",
      Element::Kbd(_) => "kbd",
      Element::Samp(_) => "samp",
      Element::Var(_) => "var",
      Element::Abbr(_) => "abbr",
      Element::Cite(_) => "cite",
      Element::Dfn(_) => "dfn",
      Element::Sub(_) => "sub",
      Element::Sup(_) => "sup",
      Element::Time(_) => "time",
      Element::Ul(_) => "ul",
      Element::Ol(_) => "ol",
      Element::Li(_) => "li",
      Element::Dl(_) => "dl",
      Element::Dt(_) => "dt",
      Element::Dd(_) => "dd",
      Element::Header(_) => "header",
      Element::Nav(_) => "nav",
      Element::Main(_) => "main",
      Element::Section(_) => "section",
      Element::Article(_) => "article",
      Element::Aside(_) => "aside",
      Element::Footer(_) => "footer",
      Element::Div(_) => "div",
      Element::Img(_) => "img",
      Element::Picture(_) => "picture",
      Element::Source(_) => "source",
      Element::Video(_) => "video",
      Element::Audio(_) => "audio",
      Element::Track(_) => "track",
      Element::Iframe(_) => "iframe",
      Element::Canvas(_) => "canvas",
      Element::Svg(_) => "svg",
      Element::SvgPath(_) => "path",
      Element::Table(_) => "table",
      Element::Caption(_) => "caption",
      Element::Thead(_) => "thead",
      Element::Tbody(_) => "tbody",
      Element::Tfoot(_) => "tfoot",
      Element::Tr(_) => "tr",
      Element::Th(_) => "th",
      Element::Td(_) => "td",
      Element::Colgroup(_) => "colgroup",
      Element::Col(_) => "col",
      Element::Form(_) => "form",
      Element::Label(_) => "label",
      Element::Input(_) => "input",
      Element::Textarea(_) => "textarea",
      Element::Button(_) => "button",
      Element::Select(_) => "select",
      Element::Optgroup(_) => "optgroup",
      Element::Fieldset(_) => "fieldset",
      Element::Legend(_) => "legend",
      Element::Datalist(_) => "datalist",
      Element::Output(_) => "output",
      Element::Progress(_) => "progress",
      Element::Meter(_) => "meter",
      Element::Details(_) => "details",
      Element::Summary(_) => "summary",
      Element::Dialog(_) => "dialog",
      Element::Template(_) => "template",
      Element::Slot(_) => "slot",
      Element::Del(_) => "del",
      Element::Ins(_) => "ins",
      Element::Bdi(_) => "bdi",
      Element::Bdo(_) => "bdo",
      Element::Wbr(_) => "wbr",
      Element::Data(_) => "data",
      Element::Ruby(_) => "ruby",
      Element::Rt(_) => "rt",
      Element::Rp(_) => "rp",
    }
  }
}

// ── HTML serialisation ──────────────────────────────────────────────────────

/// Void elements that must not have a closing tag.
const VOID_ELEMENTS: &[&str] = &[
  "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr",
];

/// Attributes worth serialising (global + common specific).
const ATTRS_TO_TRY: &[&str] = &[
  "id",
  "class",
  "style",
  "href",
  "src",
  "alt",
  "type",
  "name",
  "value",
  "placeholder",
  "for",
  "rel",
  "content",
  "disabled",
  "readonly",
  "required",
  "checked",
  "selected",
  "multiple",
  "autofocus",
  "width",
  "height",
  "action",
  "method",
  "target",
  "colspan",
  "rowspan",
  "role",
];

impl Node {
  /// Serialise this node and all descendants to an HTML string.
  pub fn to_html(&self) -> String {
    let mut buf = String::with_capacity(4096);
    self.write_html_into(&mut buf, &Default::default(), false);
    buf
  }

  fn write_html_into(&self, buf: &mut String, stylesheets: &std::collections::HashMap<String, String>, raw_text: bool) {
    use std::fmt::Write;

    match &self.element {
      Element::Text(s) => {
        if raw_text {
          buf.push_str(s);
        } else {
          for ch in s.chars() {
            match ch {
              '&' => buf.push_str("&amp;"),
              '<' => buf.push_str("&lt;"),
              '>' => buf.push_str("&gt;"),
              _ => buf.push(ch),
            }
          }
        }
        return;
      }
      _ => {}
    }

    let tag = self.element.tag_name();

    buf.push('<');
    buf.push_str(tag);

    for &attr_name in ATTRS_TO_TRY {
      if let Some(val) = self.element.attr(attr_name) {
        if val.is_empty() {
          let _ = write!(buf, " {attr_name}");
        } else {
          let _ = write!(buf, " {attr_name}=\"{}\"", html_escape_attr(&val));
        }
      }
    }

    write_map_attrs(buf, &self.element, "data-");
    write_map_attrs(buf, &self.element, "aria-");

    buf.push('>');

    if tag == "head" {
      for (_href, css) in stylesheets {
        buf.push_str("\n<style>\n");
        buf.push_str(css);
        buf.push_str("\n</style>\n");
      }
    }

    let children_raw = matches!(tag, "style" | "script");
    for child in &self.children {
      child.write_html_into(buf, stylesheets, children_raw);
    }

    if !VOID_ELEMENTS.contains(&tag) {
      let _ = write!(buf, "</{tag}>");
    }
  }
}

fn html_escape_attr(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for ch in s.chars() {
    match ch {
      '"' => out.push_str("&quot;"),
      '&' => out.push_str("&amp;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      _ => out.push(ch),
    }
  }
  out
}

fn write_map_attrs(buf: &mut String, element: &Element, prefix: &str) {
  use std::fmt::Write;
  macro_rules! arms {
    ($($v:ident),* $(,)?) => {
      match element {
        Element::Text(_) => {},
        $(Element::$v(e) => {
          let map = if prefix == "data-" { &e.data_attrs } else { &e.aria_attrs };
          let mut keys: Vec<_> = map.keys().collect();
          keys.sort();
          for key in keys {
            if let Some(val) = map.get(key) {
              let _ = write!(buf, " {prefix}{key}=\"{}\"", html_escape_attr(val));
            }
          }
        },)*
      }
    };
  }
  all_element_variants!(arms);
}

// ── Path-collection helpers for find_elements_by_* ─────────────

fn collect_class_name_paths(node: &Node, class_name: &str, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
  if node
    .element
    .class()
    .is_some_and(|c| c.split_ascii_whitespace().any(|t| t == class_name))
  {
    out.push(path.clone());
  }
  for (i, child) in node.children.iter().enumerate() {
    path.push(i);
    collect_class_name_paths(child, class_name, path, out);
    path.pop();
  }
}

fn collect_name_paths(node: &Node, name: &str, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
  if node.element.attr("name").as_deref() == Some(name) {
    out.push(path.clone());
  }
  for (i, child) in node.children.iter().enumerate() {
    path.push(i);
    collect_name_paths(child, name, path, out);
    path.pop();
  }
}

fn collect_tag_name_paths(node: &Node, tag_name: &str, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
  if node.element.tag_name().eq_ignore_ascii_case(tag_name) {
    out.push(path.clone());
  }
  for (i, child) in node.children.iter().enumerate() {
    path.push(i);
    collect_tag_name_paths(child, tag_name, path, out);
    path.pop();
  }
}

fn collect_class_name_nodes<'a>(node: &'a Node, class_name: &str, out: &mut Vec<&'a Node>) {
  if node
    .element
    .class()
    .is_some_and(|c| c.split_ascii_whitespace().any(|t| t == class_name))
  {
    out.push(node);
  }
  for child in &node.children {
    collect_class_name_nodes(child, class_name, out);
  }
}

fn collect_name_nodes<'a>(node: &'a Node, name: &str, out: &mut Vec<&'a Node>) {
  if node.element.attr("name").as_deref() == Some(name) {
    out.push(node);
  }
  for child in &node.children {
    collect_name_nodes(child, name, out);
  }
}

fn collect_tag_name_nodes<'a>(node: &'a Node, tag_name: &str, out: &mut Vec<&'a Node>) {
  if node.element.tag_name().eq_ignore_ascii_case(tag_name) {
    out.push(node);
  }
  for child in &node.children {
    collect_tag_name_nodes(child, tag_name, out);
  }
}

/// Stable hash of the tree's selector-relevant surface: tag name,
/// id, class, and child structure — everything but inline styles
/// and text content.
///
/// Two trees with the same fingerprint produce identical cascade
/// results, so [`wgpu_html::PipelineAction::LayoutOnly`] can skip
/// re-cascading.
pub fn node_selector_fingerprint(node: &Node) -> u64 {
  use std::hash::Hasher;
  let mut h = std::collections::hash_map::DefaultHasher::new();
  selector_fingerprint_impl(node, &mut h);
  h.finish()
}

fn selector_fingerprint_impl(node: &Node, h: &mut std::collections::hash_map::DefaultHasher) {
  use std::hash::Hash;
  node.element.tag_name().hash(h);
  if let Some(id) = node.element.id() {
    id.hash(h);
  }
  if let Some(cls) = node.element.class() {
    cls.hash(h);
  }
  for child in &node.children {
    selector_fingerprint_impl(child, h);
  }
}
