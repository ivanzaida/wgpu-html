---
title: Overflow Properties
---

# Overflow Properties

wgpu-html implements CSS overflow handling with per-axis independent clipping, rectangular scissor clips, rounded SDF (Signed Distance Field) clipping, and a clip stack with rectangle intersection for nested overflow containers.

## `overflow`, `overflow-x`, `overflow-y`

Controls what happens when content overflows its containing box:

```css
overflow: visible;     /* content is not clipped (default) */
overflow: hidden;      /* content is clipped, no scrollbars */
overflow: scroll;      /* content is clipped, scrollable area */
overflow: auto;        /* like scroll, but only when needed */
```

### Per-Axis Control

```css
overflow-x: hidden;    /* clip horizontally */
overflow-y: scroll;    /* allow vertical scrolling */
```

The `overflow` shorthand sets both axes simultaneously. The per-axis properties override specific axes.

## How Overflow Clipping Works

The renderer implements two clipping mechanisms that stack:

### 1. Rectangular Scissor Clip

When `overflow: hidden | scroll | auto` is set, a **scissor rectangle** is pushed onto the clip stack for that element's padding box. All subsequent draw commands (quads, glyphs, images) are constrained to this rectangle by the GPU hardware scissor test.

```
Element with overflow: hidden
┌──── padding box ────┐
│ ████████████████████ │  ← content draws normally inside
│ ████████████████████ │
│ ████████████████░░░░ │  ← clipped at scissor edge
└──────────────────────┘
     ░░░░░░░░░░░░░░░░     ← clipped by scissor
```

### 2. Rounded SDF Clipping

When `border-radius` is combined with `overflow: hidden`, the SDF (Signed Distance Field) shader performs per-pixel rounded clipping. Each fragment's distance from the rounded rectangle is evaluated, and fragments outside the radius are discarded.

```
overflow: hidden + border-radius: 12px
╭──────────────╮
│ ██████████████ │  ← clips to rounded corners
│ ██████████████ │
│ ██████████████ │
╰──────────────╯
```

### Clip Stack with Rectangle Intersection

Nested overflow containers build a **clip stack**. Each new scissor is the **intersection** of its parent's clip rectangle and its own padding box:

```css
.outer {
  overflow: hidden;
  width: 300px;
  height: 200px;
}

.inner {
  overflow-y: auto;
  width: 400px;   /* wider than parent */
  height: 150px;
}
```

```
Parent clip rect: [0, 0, 300, 200]
Child clip rect:  parent ∩ child_padding = [0, 0, 300, 150]
→ Child content beyond 300px width is doubly clipped
```

The clip stack is pushed/popped per element during display list building. Each `push_clip` command records the rectangle and clips subsequent draw commands against the cumulative scissor.

## Overflow Values in Detail

### `visible` (Default)

No clipping. Content overflows the box and is still rendered:

```css
.box { overflow: visible; }
```

### `hidden`

Content is clipped to the padding box. No scroll mechanism is provided:

```css
.clipped {
  overflow: hidden;
  width: 200px;
  height: 100px;
}
```

### `scroll`

Content is clipped to the padding box. The element becomes a scroll container with a scrollbar (if content overflows). The scroll position is tracked in `InteractionState::scroll_offsets_y`:

```css
.scrollable {
  overflow-y: scroll;
  height: 300px;
}
```

### `auto`

Behaves like `scroll`, but scrollbars only appear when content actually overflows:

```css
.auto-scroll {
  overflow: auto;
  max-height: 500px;
}
```

## Scroll Interaction

Scroll containers are interactive:
- Mouse wheel events scroll the deepest scrollable container at the cursor position
- Per-element scroll offsets are stored in `BTreeMap<Vec<usize>, f32>` (path → scroll_y)
- Scrollbar paint: 10px track width, drag-to-scroll thumb
- `Wheel` events scroll viewport and nested containers but are not forwarded to element `on_event` callbacks

```rust
// Scroll API
use wgpu_html::scroll;

let geometry = scroll::scrollbar_geometry(&layout_root, path);
let y_offset = scroll::scroll_y_from_thumb_top(&geometry, thumb_top);
scroll::translate_display_list_y(&mut display_list, y_offset);
```

## `overflow` + `border-radius` Interaction

When both are set, clipping respects the rounded corners:

```css
.card {
  overflow: hidden;
  border-radius: 12px;
}
```

The SDF quad shader uses the corner radii from the `LayoutBox` to evaluate per-pixel coverage. Content outside the rounded rectangle is discarded, creating smooth anti-aliased rounded clipping.

> **Note:** There was a known bug where `overflow: auto` elements with no painted children (like `<textarea>` with UA default `overflow: auto`) could cause clip index shifting that made subsequent glyphs invisible. This was fixed by remapping `clip_index` values after empty clip ranges were dropped during `finalize()`. See `wgpu-html-renderer/src/paint.rs` for the fix.

## Code Examples

### Scrolling Panel

```css
.scroll-panel {
  overflow-y: auto;
  height: 400px;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 16px;
}
```

### Horizontal Scroll Container

```css
.h-scroll {
  overflow-x: auto;
  white-space: pre;
  width: 100%;
}
```

### Rounded Image Container

```css
.avatar {
  overflow: hidden;
  border-radius: 50%;
  width: 64px;
  height: 64px;
}

.avatar img {
  width: 100%;
  height: 100%;
}
```

### Nested Overflow

```css
.page {
  overflow-y: auto;
  height: 100vh;
}

.sidebar {
  overflow-y: auto;
  width: 250px;
  height: 100%;
}

.content-area {
  overflow-y: auto;
  flex: 1;
}
```

### Clipped Dropdown Menu

```css
.dropdown {
  overflow: hidden;
  border-radius: 6px;
  border: 1px solid #ddd;
  background-color: white;
}

.dropdown-item {
  padding: 8px 16px;
  cursor: pointer;
}

.dropdown-item:hover {
  background-color: #f0f0f0;
}
```
