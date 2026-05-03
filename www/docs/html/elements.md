---
id: html/elements
title: Element Reference
---

# Element Reference

This page provides detailed documentation for element attribute parsing, global attributes, and the most significant element types in wgpu-html.

## Attribute Parsing

Every recognized HTML element is parsed into a Rust struct in `wgpu-html-models`. Each struct carries typed fields for the element's specific attributes plus the shared global attribute set. The attribute parser (`attr_parser::parse_element`) maps raw `(name, value)` token pairs to struct fields:

```rust
// wgpu-html-models/src/html/input.rs
pub struct Input {
    // Global attributes
    pub id: Option<String>,
    pub class: Option<String>,
    pub style: Option<String>,
    pub title: Option<String>,
    pub lang: Option<String>,
    pub dir: Option<HtmlDirection>,
    pub hidden: Option<bool>,
    pub tabindex: Option<i32>,
    pub accesskey: Option<String>,
    pub contenteditable: Option<String>,
    pub draggable: Option<String>,
    pub spellcheck: Option<String>,
    pub translate: Option<String>,
    pub role: Option<String>,
    pub aria_attrs: HashMap<String, String>,
    pub data_attrs: HashMap<String, String>,
    // Element-specific attributes
    pub r#type: Option<InputType>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub placeholder: Option<String>,
    pub disabled: Option<bool>,
    pub readonly: Option<bool>,
    pub required: Option<bool>,
    pub checked: Option<bool>,
    pub multiple: Option<bool>,
    pub autofocus: Option<bool>,
    pub min: Option<String>,
    pub max: Option<String>,
    pub step: Option<String>,
    pub pattern: Option<String>,
}
```

Boolean attributes (`disabled`, `readonly`, `required`, `checked`, `selected`, `multiple`, `autofocus`) are stored as `Option<bool>` where `Some(true)` means present and the query API returns empty-string for attribute presence checks.

## Global Attributes

These attributes are parsed on **every** element (except `Text`). Access them via `Element::attr()` or the typed fields on each element struct.

```rust
let node = tree.get_element_by_id("my-div").unwrap();
println!("id: {:?}", node.element.attr("id"));
println!("class: {:?}", node.element.attr("class"));
println!("tabindex: {:?}", node.element.attr("tabindex"));
println!("hidden: {:?}", node.element.attr("hidden"));
```

### `id`
Used for `#id` CSS selectors and DOM lookup (`get_element_by_id`). Uniqueness is not enforced, but first-match depth-first search determines which node is returned.

```html
<div id="sidebar">...</div>
```

### `class`
Whitespace-separated token list. Used in `.class` CSS selectors and DOM queries.

```html
<div class="card active highlighted">...</div>
```

In CSS: `.card { ... }`, `.active { ... }`, `.highlighted { ... }` all match. In DOM queries: `get_elements_by_class_name("card")` finds this element.

### `style`
Inline CSS declarations. Parsed into a `Style` struct and applied at the highest cascade priority (above author stylesheets).

```html
<div style="color: red; display: flex; gap: 12px;">...</div>
```

### `data-*`
Custom data attributes stored in a `HashMap<String, String>`. Example:

```html
<div data-user-id="42" data-role="admin">...</div>
```

Query: `element.attr("data-user-id")` returns `Some("42")`.

### `aria-*`
ARIA accessibility attributes stored in a `HashMap<String, String>`. Example:

```html
<button aria-label="Close dialog" aria-pressed="false">X</button>
```

Query: `element.attr("aria-label")` returns `Some("Close dialog")`.

### `hidden`
Present → `hidden: Some(true)` → UA cascade sets `display: none`. The element is parsed but excluded from layout.

### `tabindex`
Controls keyboard focusability:
- `tabindex="0"` → Element participates in sequential Tab navigation
- `tabindex="-1"` → Focusable via scripts but skipped in Tab traversal
- `tabindex` absent on most elements → Not keyboard-focusable

Some elements are implicitly focusable even without `tabindex`: `<button>`, `<a href>`, `<input>` (non-hidden), `<textarea>`, `<select>`, `<summary>`.

## Special Element Types

### `<a>` — Anchor / Hyperlink

```rust
// wgpu-html-models/src/html/a.rs
pub struct A {
    pub href: Option<String>,
    pub r#type: Option<String>,
    // ... global attributes
}
```

The `href` attribute determines focusability — an `<a>` with `href` set can receive keyboard focus. Without `href`, it behaves as an inline anchor.

```html
<a href="https://example.com" id="learn-more">Learn More</a>
```

```rust
// Rust: Install a click handler
if let Some(link) = tree.get_element_by_id("learn-more") {
    link.on_click = Some(Arc::new(|ev| {
        println!("Link clicked at ({}, {})", ev.pos.0, ev.pos.1);
    }));
}
```

### `<img>` — Image

```rust
pub struct Img {
    pub src: Option<String>,
    pub alt: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    // ... global attributes
}
```

Image loading is **asynchronous** and handled by the layout crate, not the renderer:

- **Protocols:** HTTP(S), file paths, data URIs
- **Formats:** PNG, JPEG, GIF (animated), WebP (animated)
- **Caching:** Two-level cache with configurable TTL (`tree.asset_cache_ttl`) and byte-budget eviction
- **Preloading:** `tree.preload_asset("logo.png")` queues URLs for early fetch
- **Asset root:** `tree.set_asset_root("/path/to/assets")` resolves relative paths

```html
<img src="hero.png" alt="Site hero" width="800" height="400">
```

```rust
tree.set_asset_root("./assets");
tree.preload_asset("hero.png");
```

### `<input>` — Form Input

```rust
pub struct Input {
    pub r#type: Option<InputType>,  // 22 variants
    pub name: Option<String>,
    pub value: Option<String>,
    pub placeholder: Option<String>,
    pub disabled: Option<bool>,
    pub readonly: Option<bool>,
    pub required: Option<bool>,
    pub checked: Option<bool>,
    pub multiple: Option<bool>,
    pub autofocus: Option<bool>,
    pub min: Option<String>,
    pub max: Option<String>,
    pub step: Option<String>,
    pub pattern: Option<String>,
    // ... global attributes
}
```

The `type` field is an enum of 22 variants:

```
Text, Password, Email, Url, Tel, Search, Number, Range,
Date, Time, DatetimeLocal, Month, Week, Color,
Checkbox, Radio, File, Hidden, Image, Submit, Reset, Button
```

Key behaviors:

- `type="hidden"` — No placeholder rendering, excluded from layout
- `type="checkbox"` / `type="radio"` — `checked` parsed but click-to-toggle not yet implemented
- `type="text"` and most others — Full text editing with caret, selection, arrow-key navigation, Home/End, Shift-select, and clipboard
- `placeholder` — Rendered in gray (color × 0.5 alpha) when `value` is empty; single-line vertically centered, clipped at right padding
- `disabled` — Parsed and cascade respects it; `:disabled` available in query engine but not in cascade matching

```html
<form>
  <label for="username">Username:</label>
  <input type="text" id="username" name="username" placeholder="Enter username">

  <label for="email">Email:</label>
  <input type="email" id="email" name="email" required>

  <input type="checkbox" id="agree" name="agree" checked>
  <label for="agree">I agree to terms</label>
</form>
```

### `<textarea>` — Multi-line Text Input

```rust
pub struct Textarea {
    pub name: Option<String>,
    pub placeholder: Option<String>,
    pub disabled: Option<bool>,
    pub readonly: Option<bool>,
    pub required: Option<bool>,
    pub autofocus: Option<bool>,
    // content is the text between <textarea> and </textarea>
}
```

`<textarea>` is a raw-text element — the tokenizer captures everything between `<textarea>` and `</textarea>` as a single text child. Full text editing with line breaks, caret navigation, and Shift-select.

```html
<textarea name="bio" placeholder="Tell us about yourself..."
          rows="4" cols="50">Default content</textarea>
```

### `<button>` — Button

```rust
pub struct Button {
    pub r#type: Option<ButtonType>,  // Button, Submit, Reset
    pub name: Option<String>,
    pub value: Option<String>,
    pub disabled: Option<bool>,
    pub autofocus: Option<bool>,
    // ... global attributes + children
}
```

Buttons are focusable by default. Click handlers are installed programmatically:

```rust
if let Some(btn) = tree.get_element_by_id("my-btn") {
    btn.on_click = Some(Arc::new(|ev| {
        println!("Button clicked!");
    }));
}
```

### `<select>` / `<option>` / `<optgroup>`

```rust
pub struct Select {
    pub name: Option<String>,
    pub disabled: Option<bool>,
    pub required: Option<bool>,
    pub multiple: Option<bool>,
    pub autofocus: Option<bool>,
}

pub struct OptionElement {
    pub value: Option<String>,
    pub selected: Option<bool>,
    pub disabled: Option<bool>,
}

pub struct Optgroup {
    pub disabled: Option<bool>,
}
```

Attributes are fully parsed. Dropdown menu rendering is not yet implemented.

### `<svg>` — Inline SVG

SVG elements are rasterized via `resvg` and uploaded as a GPU texture. Inline SVG path data is captured but not styled by CSS — SVG content is rendered as a single raster image.

```html
<svg width="100" height="100" viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="40" fill="red" />
</svg>
```

### Form Controls and Interactivity

Form controls that support text editing (`<input>`, `<textarea>`) receive a full text-editing implementation:

- **Insert/edit/delete** at caret position
- **Arrow-key navigation** (left, right, up, down), Home/End
- **Shift-select** text ranges
- **Single/double/triple click** → place caret / select word / select line
- **Copy/Cut** via `arboard` clipboard (Ctrl+C)
- **Paste** not yet implemented
- **`<input type="submit">` and `<button type="submit">`** do not yet fire form submission events
- **`<input type="checkbox">` / `<input type="radio">`** — `checked` is parsed but pointer clicks do not toggle the state

## Code Examples

### Building a Simple Form

```html
<!DOCTYPE html>
<html>
  <body>
    <form id="login-form">
      <div style="display: flex; flex-direction: column; gap: 12px; max-width: 300px;">
        <label for="user">Username</label>
        <input type="text" id="user" name="user" placeholder="Enter username">

        <label for="pass">Password</label>
        <input type="password" id="pass" name="pass" placeholder="Enter password">

        <div style="display: flex; gap: 8px;">
          <button type="submit" id="login-btn">Log In</button>
          <button type="reset">Clear</button>
        </div>
      </div>
    </form>
  </body>
</html>
```

### Wiring Callbacks in Rust

```rust
use wgpu_html_parser::parse;
use std::sync::Arc;

let mut tree = parse(include_str!("form.html"));

tree.get_element_by_id("login-btn").map(|btn| {
    btn.on_click = Some(Arc::new(|ev| {
        let user = tree.get_element_by_id("user")
            .and_then(|n| n.element.attr("value"));
        let pass = tree.get_element_by_id("pass")
            .and_then(|n| n.element.attr("value"));
        eprintln!("Login attempt: {:?} / {:?}", user, pass);
    }));
});

tree.get_element_by_id("user").map(|input| {
    // Focus the username field on startup
    wgpu_html_tree::focus(&mut tree, /* path to input */);
});
```

## See Also

- [Element Index](./element-index) — Full table of all 98 supported elements
- [DOM API](./dom-api) — Query and traversal API documentation
- [HTML Overview](./overview) — How HTML and CSS interact
