---
sidebar_position: 5
---

# Adding HTML Elements

This guide walks through adding support for a new HTML element.

## 1. Add Element Struct

In `crates/wgpu-html-models/src/html/`, create a new file (e.g., `figure.rs`):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Figure {
    pub base: ElementBase,
    // element-specific attributes here
}

impl Figure {
    pub fn attrs(&self) -> &ElementBase { &self.base }
    pub fn attrs_mut(&mut self) -> &mut ElementBase { &mut self.base }
}
```

## 2. Register in Element Enum

Add the variant to `Element` in `crates/wgpu-html-models/src/common/html_enums.rs`:

```rust
pub enum Element {
    // ... existing variants
    Figure(Figure),
}
```

## 3. Add Parser Mapping

In `crates/wgpu-html-parser/src/attr_parser.rs`, add the tag name mapping:

```rust
match tag {
    // ... existing matches
    "figure" => build_element::<Figure>(tag, attrs, child_attrs),
}
```

Also add the tag to the void element list (`is_void_element`) and auto-close rules in the tree builder if needed.

## 4. Add UA Stylesheet Rules

In `crates/wgpu-html-style/src/ua.css`, add default styling:

```css
figure {
    display: block;
    margin: 16px 40px;
}
```

## 5. Handle in Layout (if needed)

If the element needs special layout behavior, add a match arm in `crates/wgpu-html-layout/src/lib.rs`:

```rust
match &node.element {
    Element::Figure(fig) => {
        // custom layout logic
    }
    _ => {}
}
```

## 6. Handle in Paint (if needed)

For native-appearance elements (like form controls), add a paint handler in `crates/wgpu-html/src/paint.rs`:

```rust
match &element {
    Element::Figure(_) => {
        // custom paint logic
    }
    _ => {}
}
```

## 7. Add Tests

Add a test in the relevant crate's `tests.rs`:

```rust
#[test]
fn figure_renders_as_block() {
    let html = r#"<figure>content</figure>"#;
    let mut tree = parse(html);
    let cascaded = cascade(&tree);
    let layout = layout_with_text(&cascaded, 800.0);
    // assert layout behavior
}
```

## Checklist

- [ ] Element struct with `ElementBase` in `models/src/html/`
- [ ] `Element` enum variant registered
- [ ] Parser mapping in `attr_parser.rs`
- [ ] UA stylesheet defaults in `ua.css`
- [ ] Layout handling if needed in `layout/src/lib.rs`
- [ ] Paint handling if needed in `wgpu-html/src/paint.rs`
- [ ] Unit tests
- [ ] Demo HTML page exercising the element
