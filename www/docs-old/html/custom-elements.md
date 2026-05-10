---
title: Custom Elements
---

# Custom Elements

wgpu-html supports HTML custom elements — any tag whose name contains a hyphen (per the [HTML spec](https://html.spec.whatwg.org/multipage/custom-elements.html#valid-custom-element-name)).

```html
<my-card class="featured" theme="dark">
  <h2>Title</h2>
  <p>Content inside a custom element.</p>
</my-card>
```

Custom elements are first-class citizens in the DOM, cascade, layout, and paint pipeline. They behave like standard HTML elements with no built-in semantics.

## Parsing Rules

The parser recognizes a tag as a custom element when its name contains at least one hyphen (`-`). Tags without a hyphen that aren't recognized HTML elements are still discarded.

| Tag | Parsed as |
|---|---|
| `<my-card>` | `Element::CustomElement` |
| `<app-header>` | `Element::CustomElement` |
| `<x-button>` | `Element::CustomElement` |
| `<mycard>` | Discarded (no hyphen, not a known element) |
| `<div>` | `Element::Div` (standard element) |

Children of custom elements are preserved normally:

```html
<my-layout>
  <div class="sidebar">...</div>
  <div class="content">...</div>
</my-layout>
```

## Attributes

Custom elements support all [global HTML attributes](/docs/html/elements#global-attributes):

```html
<my-card id="card-1" class="featured" style="padding: 16px;"
         data-category="news" aria-label="News card"
         tabindex="0" hidden>
  ...
</my-card>
```

Any attribute that is not a global attribute is stored as a **custom attribute** and accessible via `element.attr()`:

```html
<my-card theme="dark" variant="outlined" size="large">
```

```rust
// Query custom attributes
let theme = node.element.attr("theme");   // Some("dark")
let variant = node.element.attr("variant"); // Some("outlined")
```

## CSS Styling

Custom elements work with all CSS selectors — tag, class, ID, attribute, pseudo-class:

```html
<style>
  my-card {
    display: block;
    padding: 16px;
    border: 1px solid #ccc;
    border-radius: 8px;
  }

  my-card.featured {
    border-color: gold;
  }

  my-card[theme="dark"] {
    background-color: #1a1a2e;
    color: #eee;
  }

  my-card:hover {
    box-shadow: 0 2px 8px rgba(0,0,0,0.15);
  }
</style>
```

### Default Display

Custom elements default to **inline** display (matching browser behavior for unknown elements). Set `display: block` in CSS to make them behave as block containers:

```css
my-card {
  display: block;
}
```

## Rust Component DSL

The `el` module provides a `custom()` constructor:

```rust
use wgpu_html_ui::el;

// Create a custom element
let card = el::custom("my-card")
    .id("card-1")
    .class("featured")
    .style("padding: 16px")
    .data("category", "news")
    .children([
        el::h2().text("Title"),
        el::p().text("Card content."),
    ]);
```

Use `.configure()` to access element-specific fields:

```rust
use wgpu_html_models::CustomElement;

let card = el::custom("my-card")
    .configure(|e: &mut CustomElement| {
        e.custom_attrs.insert("theme".into(), "dark".into());
        e.custom_attrs.insert("variant".into(), "outlined".into());
    });
```

## DOM Queries

Custom elements are queryable with the standard DOM API:

```rust
// By tag name
let cards = tree.query_selector_all("my-card");

// By class
let featured = tree.query_selector("my-card.featured");

// By attribute
let dark_cards = tree.query_selector_all("my-card[theme='dark']");

// By ID
let card = tree.get_element_by_id("card-1");
```

## Data Model

Custom elements are represented by `Element::CustomElement(CustomElement)`:

```rust
// wgpu-html-models/src/html/custom_element.rs
pub struct CustomElement {
    pub tag_name: ArcStr,
    // Global attributes (same as every other element)
    pub id: Option<ArcStr>,
    pub class: Option<ArcStr>,
    pub style: Option<ArcStr>,
    pub title: Option<ArcStr>,
    pub lang: Option<ArcStr>,
    pub dir: Option<HtmlDirection>,
    pub hidden: Option<bool>,
    pub tabindex: Option<i32>,
    // ... other global attrs, aria_attrs, data_attrs
    pub custom_attrs: HashMap<ArcStr, ArcStr>,
}
```

The `tag_name` field stores the original tag name. The `custom_attrs` map holds any non-global attributes.

## Component Factories

Register a factory function on `Tree` to automatically expand custom element tags into full DOM subtrees. The factory receives the original node (with all its attributes and children) and returns a replacement.

### Registration

```rust
use wgpu_html_tree::{Tree, Node, Element};
use wgpu_html_models::Div;

let mut tree = wgpu_html_parser::parse(r#"
  <body>
    <my-card theme="dark" class="featured">
      <h2>Hello</h2>
      <p>Card content</p>
    </my-card>
  </body>
"#);

tree.register_custom_element("my-card", |node: &Node| {
    // Read attributes from the original custom element
    let theme = node.element.attr("theme")
        .unwrap_or_default();

    // Build a replacement subtree
    let mut wrapper = Node::new(Div::default());

    // Transfer class from the custom element
    if let Some(class) = node.element.attr("class") {
        if let Element::Div(ref mut div) = wrapper.element {
            div.class = Some(class.into());
        }
    }

    // Apply theme-dependent inline style
    if let Element::Div(ref mut div) = wrapper.element {
        let bg = if theme == "dark" { "#1a1a2e" } else { "#ffffff" };
        div.style = Some(
            format!("padding:16px;border-radius:8px;background:{bg}").into()
        );
    }

    // Pass children through
    wrapper.children = node.children.clone();
    wrapper
});

// Resolve all registered custom elements (single pass)
tree.resolve_custom_elements();
```

After `resolve_custom_elements()`, every `<my-card>` in the tree is replaced by the factory's output. The original attributes, children, and event handlers are available to the factory via the `&Node` argument.

### When to Call

Call `resolve_custom_elements()` once after:
1. Parsing HTML (`wgpu_html_parser::parse(...)`)
2. Registering all factories (`tree.register_custom_element(...)`)

```rust
let mut tree = wgpu_html_parser::parse(html);
tree.register_custom_element("my-card", card_factory);
tree.register_custom_element("app-header", header_factory);
tree.resolve_custom_elements(); // replaces all matching nodes
// Now render as usual
```

Resolution is a single pass — factory output is not re-scanned for further custom elements.

## Limitations

- **No Shadow DOM** — custom elements share the same flat DOM and cascade as all other elements
- **No lifecycle callbacks** — there is no `connectedCallback` / `disconnectedCallback` equivalent at the DOM level (use the [Component trait](/docs/component-framework/component-trait) for lifecycle hooks)
- **No `:defined` pseudo-class** — all custom elements are always considered defined
- **Single-pass resolution** — if a factory produces output containing other custom elements, those are not resolved automatically (call `resolve_custom_elements()` again if needed)
