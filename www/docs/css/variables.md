---
title: Custom Properties (CSS Variables)
---

# Custom Properties (CSS Variables)

wgpu-html fully implements CSS custom properties (CSS variables) with `--*` declaration syntax, `var()` usage, inheritance, recursive substitution, cycle detection, and a programmatic API for runtime manipulation.

## `--custom-property` Syntax

Custom properties are defined using the `--` prefix:

```css
:root {
  --primary-color: #3498db;
  --spacing-unit: 8px;
  --border-radius: 6px;
  --font-stack: "Inter", system-ui, sans-serif;
}

.button {
  background-color: var(--primary-color);
  padding: var(--spacing-unit) calc(var(--spacing-unit) * 2);
  border-radius: var(--border-radius);
  font-family: var(--font-stack);
}
```

Custom properties can be defined:
- In any stylesheet rule (class, ID, tag, `*`)
- In inline `style="--color: red;"` attributes
- Programmatically via `Node::set_custom_property()`

### Engine Vendor Properties (`--lui-*`)

wgpu-html reserves the `--lui-` prefix for vendor-specific form control styling. Unlike plain custom properties, `--lui-*` values are **parsed through the CSS engine** into typed values (colors, lengths, border shorthands, etc.) — they support the same syntax as their regular CSS counterparts.

#### Range slider

| Property | Type | Effect |
|---|---|---|
| `--lui-track-color` | `<color>` | Unfilled track background |
| `--lui-thumb-color` | `<color>` | Thumb fill |

#### Popup (shared by color picker and date picker)

| Property | Type | Effect |
|---|---|---|
| `--lui-popup-width` | `<length>` | Popup width |
| `--lui-popup-height` | `<length>` | Popup height |
| `--lui-popup-background` | `<color>` | Popup background |
| `--lui-popup-color` | `<color>` | Popup text color |
| `--lui-popup-border` | `<border-shorthand>` | Popup border (width + style + color) |
| `--lui-popup-border-width` | `<length>` | All four sides |
| `--lui-popup-border-style` | `<border-style>` | All four sides |
| `--lui-popup-border-color` | `<color>` | All four sides |
| `--lui-popup-border-top-width` | `<length>` | Single side (also `-right`, `-bottom`, `-left`) |
| `--lui-popup-border-top-style` | `<border-style>` | Single side |
| `--lui-popup-border-top-color` | `<color>` | Single side |
| `--lui-popup-border-radius` | `<length>` | Corner radius |
| `--lui-popup-font-size` | `<length>` | Text font size |
| `--lui-popup-font-family` | `<string>` | Text font family |
| `--lui-popup-font-weight` | `<font-weight>` | Text font weight |

#### Color picker

| Property | Type | Effect |
|---|---|---|
| `--lui-color-canvas-width` | `<length>` | Saturation/value canvas width |
| `--lui-color-canvas-height` | `<length>` | Saturation/value canvas height |
| `--lui-color-range-height` | `<length>` | Hue/alpha slider bar height |
| `--lui-color-range-border-radius` | `<length>` | Slider bar corner radius |
| `--lui-color-thumb-width` | `<length>` | Slider thumb knob width |
| `--lui-color-thumb-height` | `<length>` | Slider thumb knob height |
| `--lui-color-thumb-color` | `<color>` | Slider thumb and canvas crosshair color |
| `--lui-color-input-height` | `<length>` | Text input field height |
| `--lui-color-input-background` | `<color>` | Text input field background |
| `--lui-color-input-border-color` | `<color>` | Text input field border color |
| `--lui-color-input-border-width` | `<length>` | Text input field border width |
| `--lui-color-input-border-radius` | `<length>` | Text input field corner radius |
| `--lui-color-input-font-size` | `<length>` | Text input field font size |

See [CSS Property Index — Form Control Styling](./property-index#form-control-styling) for details and examples.

## `var()` Usage

The `var()` function references a custom property:

```css
.element {
  color: var(--text-color);
  margin: var(--space, 16px);   /* with fallback */
}
```

### Fallback Values

A second argument to `var()` provides a fallback when the custom property is not defined:

```css
color: var(--accent, blue);
padding: var(--custom-gap, 12px);
font-family: var(--heading-font, sans-serif);
```

The fallback can itself contain `var()` references (nested):

```css
color: var(--brand-color, var(--fallback-color, black));
```

## Inheritance of Custom Properties

Custom properties **always inherit** from parent to child. This is a key difference from regular CSS properties:

```css
.container {
  --accent: #e74c3c;
}

.container .child {
  /* --accent is inherited from .container */
  color: var(--accent);  /* #e74c3c */
}
```

Values propagate through the cascade just like regular inheritable properties:
1. The cascade resolves explicit `--*` declarations for each element
2. After cascade, unset custom properties are bulk-cloned from the parent
3. Programmatic custom properties from `Node::custom_properties` are injected after inheritance, overriding any inherited value

## Recursive Variable Substitution

Variables can reference other variables, and the engine resolves chains recursively:

```css
:root {
  --hue: 200;
  --saturation: 80%;
  --lightness: 50%;
  --accent: hsl(var(--hue), var(--saturation), var(--lightness));
  --button-bg: var(--accent);
}

.button {
  background-color: var(--button-bg);
  /* Resolves: hsl(200, 80%, 50%) → #1a8cff */
}
```

The resolution happens in two phases:
1. **Phase 1** — resolve `var()` inside custom property values (e.g., `--a: var(--b)` chains collapse)
2. **Phase 2** — resolve `var()` in regular property declarations, re-parsing the substituted value through `apply_css_property()`

## Cycle Detection

Circular variable references are detected and handled gracefully:

```css
:root {
  --a: var(--b);
  --b: var(--a);
  /* Cycle! Both evaluate to empty (guaranteed-invalid) */
}
```

The resolver tracks a `resolving: HashSet<String>` during substitution. When a variable name already exists in the set, a cycle is detected and:
- The `var()` evaluates to the fallback value (if provided)
- If no fallback, the `var()` evaluates to nothing (guaranteed-invalid per CSS spec)

```css
color: var(--a, red);  /* cycle + fallback → "red" */
color: var(--a);        /* cycle + no fallback → "" → no color set */
```

## Programmatic API

Custom properties can be manipulated from Rust code at runtime:

### `Node::set_custom_property()`

```rust
use wgpu_html_tree::Tree;

let mut tree = parse(html);
let node = tree.get_element_by_id("my-element").unwrap();

// Set a custom property
tree.node_ref_mut(node).set_custom_property("--accent", "#ff6600");

// The value participates in var() resolution during the next cascade
```

### `Node::remove_custom_property()`

```rust
tree.node_ref_mut(node).remove_custom_property("--accent");
// The property reverts to its inherited or default value
```

### `Node::custom_properties`

The `custom_properties: HashMap<String, String>` field on `Node` stores programmatic custom properties. These are injected into the cascaded `Style` after inheritance, overriding any stylesheet-defined or inherited values:

```rust
// Direct access
node.custom_properties.insert("--brand-color".into(), "#2c3e50".into());
```

### How Programmatic Properties Flow Through Cascade

In `cascade_node()` (and `re_cascade_dirty()`):

```rust
// Inject programmatic custom properties from the Node.
for (prop, value) in &node.custom_properties {
    style.custom_properties.insert(prop.clone(), value.clone());
}
```

Then `resolve_var_references()` runs to substitute all `var()` references using the final custom property map.

## Late Re-Parse Through `apply_css_property()`

When a property value contains `var()`, it cannot be fully parsed at declaration time. Instead, it's stored in `Style::var_properties` as a raw string:

```rust
// During parsing:
if value_contains_var(value) {
    style.var_properties.insert(property.to_owned(), value.to_owned());
    return;  // defer parsing
}
```

After the cascade and inheritance are complete, `resolve_var_references()` processes all deferred properties:

```rust
pub fn resolve_var_references(style: &mut Style) {
    // Phase 1: resolve var() in custom property values
    // Phase 2: resolve var() in regular property declarations
    let pending: Vec<(String, String)> = style.var_properties.drain().collect();
    for (prop, raw_value) in pending {
        let resolved = substitute_vars(&raw_value, &style.custom_properties, &mut resolving);
        if !resolved.is_empty() {
            apply_css_property(style, &prop, &resolved);
        }
    }
}
```

This means `var()` can be used with any supported CSS property:

```css
width: var(--sidebar-width);
margin: var(--spacing);
color: var(--text-color);
display: var(--layout-mode);  /* "flex" → display: flex */
```

## Code Examples

### Design Tokens Pattern

```css
:root {
  --color-primary: #3498db;
  --color-primary-dark: #2980b9;
  --color-danger: #e74c3c;
  --color-success: #2ecc71;
  --color-text: #2c3e50;
  --color-text-muted: #7f8c8d;
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 16px;
}

.btn {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-sm);
  font-size: 14px;
}

.btn-primary {
  background-color: var(--color-primary);
  color: white;
}

.btn-primary:hover {
  background-color: var(--color-primary-dark);
}

.btn-danger {
  background-color: var(--color-danger);
  color: white;
}
```

### Theme Switching via Programmatic API

```rust
fn apply_theme(tree: &mut Tree, theme: &Theme) {
    if let Some(root) = tree.root_node_ref_mut() {
        for (name, value) in theme.variables() {
            root.set_custom_property(name, value);
        }
    }
}

// Dark theme
let dark = Theme::new()
    .var("--bg", "#1a1a2e")
    .var("--text", "#e0e0e0")
    .var("--accent", "#3498db");
apply_theme(&mut tree, &dark);
```

### Computed Values with var()

```css
:root {
  --grid-columns: 3;
  --card-gap: 24px;
  --sidebar-width: 250px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(var(--grid-columns), 1fr);
  gap: var(--card-gap);
}

.with-sidebar {
  display: grid;
  grid-template-columns: var(--sidebar-width) 1fr;
  gap: var(--card-gap);
}
```

### var() Fallback Chain

```css
.card {
  background-color: var(--card-bg, var(--surface-color, white));
  border-color: var(--card-border, var(--border-color, #ddd));
  box-shadow: var(--card-shadow, var(--shadow-sm, none));
}
```
