---
sidebar_position: 1
---

# Public API

The main facade is `wgpu_html`. Most applications only need this crate plus a driver.

## Pipeline Functions

```rust
// Full pipeline: cascade → layout → paint (text shaping included)
pub fn paint_tree_with_text(
    tree: &Tree, text_ctx: &mut TextContext, image_cache: &mut ImageCache,
    viewport_w: f32, viewport_h: f32, scale: f32, viewport_scroll_y: f32,
) -> DisplayList;

// Returns both DisplayList and LayoutBox
pub fn paint_tree_returning_layout(...) -> (DisplayList, Option<LayoutBox>);

// With PipelineTimings
pub fn paint_tree_returning_layout_profiled(...) -> (DisplayList, Option<LayoutBox>, PipelineTimings);

// Cached pipeline — skips cascade/layout when inputs unchanged
pub fn paint_tree_cached<'c>(..., cache: &'c mut PipelineCache) -> (DisplayList, Option<&'c LayoutBox>, PipelineTimings);

// Layout only (no paint)
pub fn compute_layout(...) -> Option<LayoutBox>;
pub fn compute_layout_profiled(...) -> (Option<LayoutBox>, PipelineTimings);
```

## Pipeline Cache

```rust
pub struct PipelineCache;
impl PipelineCache {
    pub fn new() -> Self;
    pub fn invalidate(&mut self);
    pub fn layout(&self) -> Option<&LayoutBox>;
    pub fn viewport(&self) -> (f32, f32);
    pub paint_only_pseudo_rules: bool;
}

pub enum PipelineAction { FullPipeline, PartialCascade, LayoutOnly, PatchFormControls, RepaintOnly }
pub fn classify_frame(tree: &Tree, cache: &PipelineCache, ...) -> PipelineAction;
```

## Interactivity

```rust
pub fn pointer_move(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32)) -> bool;
pub fn pointer_move_with_cursor(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32)) -> (bool, Cursor);
pub fn mouse_down(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool;
pub fn mouse_down_with_click_count(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton, click_count: u8) -> bool;
pub fn mouse_up(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool;
```

## Text Selection

```rust
pub fn select_all_text(tree: &mut Tree, layout: &LayoutBox) -> bool;
pub fn selected_text(tree: &Tree, layout: &LayoutBox) -> Option<String>;
pub fn select_word_at_cursor(tree: &mut Tree, layout: &LayoutBox, cursor: &TextCursor) -> bool;
pub fn select_line_at_cursor(tree: &mut Tree, layout: &LayoutBox, cursor: &TextCursor) -> bool;
```

## Screenshots

```rust
pub fn screenshot_node_to(
    tree: &Tree, text_ctx: &mut TextContext, image_cache: &mut ImageCache,
    renderer: &mut Renderer, layout_path: &[usize],
    viewport_w: f32, viewport_h: f32, scale: f32,
    out_path: impl AsRef<Path>,
) -> Result<(), NodeScreenshotError>;
```

## Scroll Utilities

```rust
pub fn clamp_scroll_y(scroll_y: f32, layout: &LayoutBox, viewport_h: f32) -> f32;
pub fn max_scroll_y(layout: &LayoutBox, viewport_h: f32) -> f32;
pub fn scroll_element_at(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), dx: f32, dy: f32) -> bool;
pub fn deepest_scrollable_path_at(layout: &LayoutBox, pos: (f32, f32)) -> Option<Vec<usize>>;
```

## Tree API

```rust
impl Tree {
    pub fn register_font(&mut self, face: FontFace) -> FontHandle;
    pub fn register_system_fonts(&mut self, family: &str) -> usize;
    pub fn register_custom_element(&mut self, tag: ArcStr, factory: impl Fn(&Node) -> Node + Send + Sync + 'static);
    pub fn resolve_custom_elements(&mut self);

    // DOM queries
    pub fn get_element_by_id(&mut self, id: &str) -> Option<&mut Node>;
    pub fn get_elements_by_class_name(&self, class: &str) -> Vec<&Node>;
    pub fn query_selector(&self, sel: &str) -> Option<&Node>;
    pub fn query_selector_all(&self, sel: &str) -> Vec<&Node>;

    // Mutations
    pub fn insert_node(&mut self, parent_path: &[usize], index: usize, node: Node) -> bool;
    pub fn append_node(&mut self, parent_path: &[usize], node: Node) -> Option<usize>;
    pub fn remove_node(&mut self, path: &[usize]) -> Option<Node>;
}
```

## Events (dispatch module)

```rust
pub fn dispatch_pointer_move(tree: &mut Tree, target_path: &[usize], pos: (f32, f32), cursor: Option<TextCursor>) -> bool;
pub fn dispatch_mouse_down(tree: &mut Tree, target_path: &[usize], pos: (f32, f32), button: MouseButton, cursor: Option<TextCursor>) -> bool;
pub fn dispatch_mouse_up(tree: &mut Tree, target_path: &[usize], pos: (f32, f32), button: MouseButton, cursor: Option<TextCursor>) -> bool;
pub fn focus(tree: &mut Tree, target_path: &[usize]) -> bool;
pub fn blur(tree: &mut Tree) -> bool;
pub fn focus_next(tree: &mut Tree) -> bool;
pub fn key_down(tree: &mut Tree, key: &str, code: &str, repeat: bool) -> bool;
pub fn key_up(tree: &mut Tree, key: &str, code: &str) -> bool;
pub fn wheel_event(tree: &mut Tree, x: f32, y: f32, dx: f64, dy: f64, mode: WheelDeltaMode) -> bool;
pub fn text_input(tree: &mut Tree, text: &str);
```
