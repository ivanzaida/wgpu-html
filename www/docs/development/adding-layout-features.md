---
sidebar_position: 7
---

# Adding Layout Features

This guide covers adding new layout behaviors to the engine.

## Layout Architecture

The layout engine in `crates/wgpu-html-layout/src/lib.rs` is a recursive tree walk. Each node is dispatched based on its `display` value and content type:

```
layout_block(node)
  ├── display: none → skip
  ├── Text leaf → shape text
  ├── Replaced element → intrinsic sizing
  ├── display: flex → layout_flex_children() (flex.rs)
  ├── display: grid → layout_grid_children() (grid.rs)
  ├── display: table → layout_table_children() (table.rs)
  ├── All children inline → layout_inline_block_children()
  └── Block children → recursive layout_block()
```

## Adding a New Display Mode

### 1. Create a New Module

Add a new file, e.g., `crates/wgpu-html-layout/src/my_layout.rs`:

```rust
use super::*;

pub fn layout_my_children(
    node: &CascadedNode,
    parent: &LayoutBox,
    container_rect: Rect,
    ctx: &mut LayoutContext,
) -> Vec<LayoutBox> {
    let style = &node.style;
    
    // 1. Collect child items (skip display: none)
    let items: Vec<&CascadedNode> = node.children.iter()
        .filter(|c| c.style.display != Some(Display::None))
        .collect();
    
    // 2. Compute geometry for each child
    let mut children = Vec::new();
    for item in &items {
        let child = layout_block(item, container_rect, ctx);
        children.push(child);
    }
    
    // 3. Position children in the container
    let mut y = container_rect.y;
    for child in &mut children {
        child.margin_rect.y = y;
        y += child.margin_rect.h;
    }
    
    children
}
```

### 2. Register in Display Dispatch

In `lib.rs`, add a match arm in the layout dispatch section:

```rust
match style.display {
    Some(Display::MyMode) => {
        children = layout_my_children(node, &container, content_rect, ctx);
    }
    // ... existing matches
}
```

### 3. Add to Length Resolution (if needed)

If your layout mode needs new length resolution semantics, extend `crates/wgpu-html-layout/src/length.rs`:

```rust
pub fn resolve_my_length(len: &CssLength, parent: f32, font_size: f32) -> Option<f32> {
    match len {
        CssLength::MyUnit(val) => Some(val * some_factor),
        _ => resolve(len, parent, font_size, root_font_size, viewport),
    }
}
```

## Adding a New Alignment Mode

If adding alignment support (like in flex/grid):

### 1. Define Alignment Types

```rust
pub enum MyAlign {
    Start,
    Center,
    End,
    Stretch,
}
```

### 2. Implement Distribution

```rust
pub fn distribute_items(
    items: &mut [LayoutBox],
    available: f32,
    align: MyAlign,
    gap: f32,
) {
    let total = items.iter().map(|i| i.margin_rect.h).sum::<f32>();
    let remaining = available - total - gap * (items.len().saturating_sub(1)) as f32;
    
    match align {
        MyAlign::Start => { /* items at top */ }
        MyAlign::Center => {
            let offset = remaining * 0.5;
            for item in items { item.margin_rect.y += offset; }
        }
        MyAlign::End => {
            for item in items { item.margin_rect.y += remaining; }
        }
        MyAlign::Stretch => { /* grow items to fill */ }
    }
}
```

## Incremental Layout

For good performance, integrate with the incremental layout system:

```rust
// In relayout_children or layout_incremental:
// If your layout mode has cross-item dependencies (like flex/grid),
// mark the entire container for full re-layout when any child is dirty.
// Otherwise, support shifting siblings without re-layout.

if container_needs_full_relayout {
    return layout_my_children(node, &container, content_rect, ctx);
} else {
    // Recurse and shift
    relayout_children(node, &container, content_rect, ctx);
}
```

## Testing

Add comprehensive tests in `crates/wgpu-html-layout/src/tests.rs`:

```rust
#[test]
fn my_layout_basic_flow() {
    let html = r#"<div style="display: my-mode;">
        <div style="height: 100px;">A</div>
        <div style="height: 100px;">B</div>
    </div>"#;
    let mut tree = parse(html);
    let cascaded = cascade(&tree);
    let layout = layout_with_text(&cascaded, 800.0);
    
    assert_eq!(layout.children[0].margin_rect.y, expected_y);
    assert_eq!(layout.children[1].margin_rect.y, expected_y);
}
```

## Common Pitfalls

- Always handle `display: none` children (exclude from layout)
- Respect `min-width/max-width/min-height/max-height` clamping
- Apply `box-sizing` before computing content rects
- Account for margin, border, and padding when positioning children
- Remember that coordinates are absolute pixels from the viewport origin
- Flex/grid items with explicit `order` must preserve source order for hit-testing
