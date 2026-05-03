---
title: Layout Engine Overview
---

# Layout Engine Overview

The layout engine (`wgpu-html-layout`) converts a fully-cascaded style tree into a **`LayoutBox`** tree — a positioned, sized, shaped representation of every element in physical pixels. The renderer consumes `LayoutBox` directly; it never re-resolves CSS.

## LayoutBox Tree

```rust
pub struct LayoutBox {
    pub margin_rect: Rect,       // includes margin — sibling stacking
    pub border_rect: Rect,       // paint box — background + border live here
    pub content_rect: Rect,      // child layout area (minus border + padding)

    pub background: Option<Color>,
    pub background_rect: Rect,   // driven by background-clip
    pub background_radii: CornerRadii,

    pub border: Insets,           // per-side thickness
    pub border_colors: BorderColors,
    pub border_styles: BorderStyles,
    pub border_radius: CornerRadii,

    pub kind: BoxKind,            // Block | Text
    pub text_run: Option<ShapedRun>,  // shaped text for text leaves
    pub text_color: Option<Color>,
    pub text_decorations: Vec<TextDecorationLine>,

    pub overflow: OverflowAxes,
    pub opacity: f32,
    pub pointer_events: PointerEvents,
    pub user_select: UserSelect,
    pub cursor: Cursor,
    pub z_index: Option<i32>,

    pub image: Option<ImageData>,
    pub background_image: Option<BackgroundImagePaint>,

    pub children: Vec<LayoutBox>,
}
```

## Mirroring the Source Tree

The `LayoutBox` child structure mirrors the source `Tree` (parser DOM) 1:1. This is a critical invariant: every `Node` in the DOM produces exactly one `LayoutBox`. This makes hit-testing, event dispatch, and DOM manipulation predictable — a `Vec<usize>` path into the layout tree is also a valid path into the element tree.

Visual reordering (e.g. flex `order`) changes box coordinates but never reorders `children`. The hit-test invariant holds regardless of visual order.

## Main Entry Point

```rust
pub fn layout_with_text(
    tree: &CascadedTree,
    text_ctx: &mut TextContext,
    image_cache: &mut ImageCache,
    viewport_w: f32,
    viewport_h: f32,
    scale: f32,
) -> Option<LayoutBox>
```

This is the geometry stage. For each node it:
1. Resolves `display` (block / flex / grid / inline) and dispatches to the correct formatting-context function.
2. Resolves margin, border, padding per side via the CSS shorthand-to-longhand expansion.
3. Shapes text runs against the `TextContext` (cosmic-text shaping + glyph atlas packing).
4. Handles out-of-flow positioning (`absolute`, `fixed`) and relative offsets.
5. Manages images: async fetch/decode, cache TTL, preload queue, animated-frame selection.

## Visual Reordering Without Child Reordering

Flex `order` is the canonical example. Items are stable-sorted by `order` value then by source index, and their coordinates are set accordingly. But `children` retains source order. This means:

```css
.container { display: flex; }
.item-a { order: 2; }
.item-b { order: 1; }
.item-c { order: 3; }
```

The visual order is B (order 1) → A (order 2) → C (order 3), but `LayoutBox::children` remains `[item-a, item-b, item-c]`. Hit-testing correctly maps a screen position to `item-b`'s path even though it's second in the child list.

## Hit Testing

```rust
impl LayoutBox {
    /// Returns the index path to the deepest descendant whose
    /// border_rect contains `point`, or None if outside.
    pub fn hit_path(&self, point: (f32, f32)) -> Option<Vec<usize>>;

    /// Scroll-aware variant — compensates for per-element scroll offsets.
    pub fn hit_path_scrolled(
        &self,
        point: (f32, f32),
        scroll_offsets: &BTreeMap<Vec<usize>, f32>,
    ) -> Option<Vec<usize>>;

    /// Hit-test for text insertion point (glyph-level accuracy).
    pub fn hit_text_cursor_scrolled(
        &self,
        point: (f32, f32),
        scroll_offsets: &BTreeMap<Vec<usize>, f32>,
    ) -> Option<TextCursor>;

    /// Resolve the CSS cursor for a hit path.
    pub fn cursor_at_path(&self, path: &[usize]) -> Cursor;
}
```

## The Three Rectangles

Every `LayoutBox` carries three canonical rectangles:

| Rectangle | Definition | Purpose |
|---|---|---|
| `margin_rect` | `border_rect` expanded by margin on each side | Flow spacing, sibling stacking |
| `border_rect` | The visual border box | Background + border painting area |
| `content_rect` | `border_rect` inset by border + padding | Child layout area |

Be explicit about which rectangle you mean. Geometry assertions in tests use these three names.

## Sub-Pages

- [Block Flow Layout](./block) — vertical stacking, margins, box-sizing
- [Flexbox Layout](./flexbox) — complete CSS Flexbox Level 1
- [CSS Grid Layout](./grid) — grid tracks, fr units, auto-placement
- [Positioned Layout](./positioned) — absolute/relative/fixed
- [Inline Formatting Context](./inline) — line boxes, text wrapping
