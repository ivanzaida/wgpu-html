---
sidebar_position: 4
---

# Error Types

## Renderer Errors

```rust
pub enum ScreenshotError {
    SurfaceError(wgpu::SurfaceError),
    BufferMapError,
    ImageEncodingError,
}
```

```rust
pub enum NodeScreenshotError {
    NoLayout,
    NodeNotFound(Vec<usize>),
    EmptyRect,
    Render(ScreenshotError),
}
```

## Frame Outcome

```rust
pub enum FrameOutcome {
    Presented,       // Frame rendered successfully
    Reconfigure,     // Surface lost, must call resize() and retry
    Skipped,         // Nothing to draw (empty display list)
}
```

## Pipeline Errors

The pipeline functions use `Option` returns rather than `Result` for expected states:

- `compute_layout()` returns `None` when the tree is empty
- `paint_tree_returning_layout()` returns `Option<LayoutBox>` — `None` if no content
- `screenshot_node_to()` returns `NodeScreenshotError` for missing or empty nodes

## Parser Behavior

The HTML parser is lenient. Invalid HTML is recovered gracefully:
- Unknown tags drop their entire subtree silently
- Mismatched tags follow auto-close rules
- Invalid CSS properties are silently ignored (not parsed)
- CSS parse errors don't prevent the rest of the stylesheet from working

## Layout Behavior

- `display: none` children are excluded from layout (no error)
- Unsupported CSS values fall back to defaults (e.g., unknown `display` value → `block`)
- Missing fonts fall back to the first registered font
- Images that fail to load are silently hidden
- Gradients with invalid parameters degrade to a solid 1×1 transparent pixel

## Error Handling Pattern

For custom integrations, wrap the pipeline:

```rust
match lui::compute_layout(&tree, &mut text_ctx, &mut image_cache, vw, vh, scale) {
    Some(layout) => { /* render */ }
    None => { /* tree is empty, show placeholder */ }
}
```
