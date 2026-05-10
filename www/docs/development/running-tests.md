---
sidebar_position: 2
---

# Running Tests

## Full Test Suite

```bash
cargo test --workspace
```

## Targeted Test Loops

```bash
# Parser tests (HTML tokenizer, CSS parser)
cargo test -p lui-parser

# Style/cascade tests
cargo test -p lui-style

# Layout tests (block, flex, grid, IFC, positioning)
cargo test -p lui-layout

# Paint and facade tests
cargo test -p lui

# Renderer tests
cargo test -p lui-renderer

# Tree/event tests
cargo test -p lui-tree

# Component framework tests
cargo test -p lui-ui
```

## Test Patterns

Tests follow a consistent pattern across crates:

### Layout Tests

```rust
#[test]
fn my_feature_test() {
    let html = r#"<div style="margin: 0;">content</div>"#;
    let mut tree = parse(html);
    let cascaded = cascade(&tree);
    let layout = layout_with_text(&cascaded, 800.0);

    // Assert on geometry
    assert_eq!(layout.content_rect.w, 800.0);
}
```

### Paint Tests

```rust
#[test]
fn my_paint_test() {
    // Build synthetic layout, paint, assert on display list
    let display_list = paint_tree_returning_layout(&layout, &cascaded, 800, 600);
    assert!(!display_list.quads.is_empty());
}
```

## Conventions

- Neutralize UA defaults with `body { margin: 0; }` unless testing UA stylesheet behavior
- Use the three canonical rectangles for geometry assertions: `margin_rect`, `border_rect`, `content_rect`
- Inline HTML/CSS in Rust unit tests, not external fixtures (following existing patterns in `tests.rs`)

## Running with Specific Flags

```bash
# Run only ignored (slow) tests
cargo test --workspace -- --ignored

# Run with stdout shown (for println! debugging)
cargo test -p lui-layout -- --nocapture

# Run a specific test
cargo test -p lui-layout flex_grow
```
