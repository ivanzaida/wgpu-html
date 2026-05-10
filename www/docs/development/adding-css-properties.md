---
sidebar_position: 6
---

# Adding CSS Properties

This guide walks through adding support for a new CSS property.

## 1. Add Field to Style Struct

In `crates/wgpu-html-models/src/css/style.rs`, add a field:

```rust
pub struct Style {
    // ... existing fields
    pub new_property: Option<CssLength>,  // or appropriate type
}
```

## 2. Register in style_props! Macro

In `crates/wgpu-html-parser/src/style_props.rs`, add the property:

```rust
style_props! {
    // ... existing properties
    prop "new-property": {
        type: Length,
        field: style.new_property,
        is_inherited: false,
    },
}
```

The `style_props!` macro auto-generates:
- `apply_new_property()` — sets the value
- `clear_new_property()` — resets to `None`
- CSS-wide keyword handling (`inherit`, `initial`, `unset`)
- Cascade merge logic
- Deferred longhand storage (if needed)

## 3. Add to Shorthand (if applicable)

If the property belongs to a shorthand:

```rust
// In apply_css_property, add shorthand expansion:
"shorthand-name" => {
    apply_generic_shorthand(style, key, ...)?;
    // Expand into longhands
    apply_new_property(style, value)?;
}
```

## 4. Handle Inheritance

If the property should inherit from parent to child, add it to the `inherit_into()` function in `crates/wgpu-html-style/src/lib.rs`:

```rust
fn inherit_into(child: &mut Style, parent: &Style) {
    // ... existing inherited properties
    child.new_property = child.new_property.or(parent.new_property);
}
```

## 5. Consume in Layout

In `crates/wgpu-html-layout/src/lib.rs`, read the value:

```rust
let prop_value = style.new_property.unwrap_or_default();
```

## 6. Consume in Paint (if visual)

In `crates/wgpu-html/src/paint.rs`:

```rust
if let Some(value) = style.new_property {
    // emit appropriate display list commands
}
```

## 7. Handle in Var() Resolution

If the property can be set via `var()`, it's already handled — the `style_props!` macro generates `var_properties` tracking. For custom handling, see `resolve_var_references()` in `crates/wgpu-html-parser/src/css_parser.rs`.

## 8. Add Tests

In the relevant crate's tests:

```rust
#[test]
fn new_property_is_parsed() {
    let style = parse_inline_style("new-property: 10px;");
    assert_eq!(style.new_property, Some(CssLength::Px(10.0)));
}

#[test]
fn new_property_affects_layout() {
    let html = r#"<div style="new-property: 10px;">test</div>"#;
    // assert layout changes
}
```

## Common Types

| Type | Use For |
|---|---|
| `CssLength` | Width, height, margin, padding, gaps, sizes |
| `CssColor` / `[f32; 4]` | Colors with alpha |
| `Option<f32>` | Simple numeric values |
| `Option<u32>` | Integer numeric values (z-index, order) |
| `Option<ArcStr>` | Raw strings, identifiers |
| Custom enum | Discrete values (display, position, overflow, etc.) |

## Property Categories

| Category | Where Consumed |
|---|---|
| Layout/sizing | `wgpu-html-layout/src/lib.rs` |
| Flex properties | `wgpu-html-layout/src/flex.rs` |
| Grid properties | `wgpu-html-layout/src/grid.rs` |
| Visual/background | `wgpu-html/src/paint.rs` |
| Text/typography | `wgpu-html-layout/src/lib.rs` (text shaping) |
| Interactive | `wgpu-html/src/interactivity.rs` |
