---
title: HTML Markup Language
---

# HTML Markup Language

wgpu-html uses **standard HTML5** as its markup language. There is no custom markup, no template DSL, and no intermediate representation. You author plain HTML5 documents and the engine parses them directly into a typed DOM tree.

## How HTML Is Parsed

The parsing pipeline has two stages:

1. **Tokenizer** — The raw HTML string is scanned character-by-character and emitted as a flat sequence of tokens: open tags (with attributes), close tags, self-closing tags, text content, comments, and DOCTYPE declarations.

2. **Tree builder** — Tokens are consumed and assembled into a `Tree<Node<Element>>`. Open tags push onto a stack, children are attached, and close tags pop the stack. The result is a hierarchical document tree where every node is one of ~100 typed `Element` variants or raw `Text`.

```rust
use wgpu_html_parser::parse;

let tree = parse(r#"
<!DOCTYPE html>
<html>
  <body>
    <h1 class="title">Hello, wgpu-html!</h1>
    <p>This is a <strong>Rust</strong> renderer.</p>
  </body>
</html>
"#);
```

## Key Differences From a Full Browser

wgpu-html is a **renderer**, not a browser. It optimizes for GPU-accelerated rendering over pristine HTML5 spec compliance:

- **No HTML5 insertion-mode state machine** — No `<table>` foster-parenting, no `</br>`→`<br>` quirk, no scripting insertion modes.
- **Unknown tags are dropped** with their entire subtree. Only the ~100 recognized element types survive.
- **Comments and DOCTYPE are tokenized but discarded** at tree-building time.
- **Whitespace-only text between tags is dropped** — only meaningful text content is preserved.
- **No `<![CDATA[]]>`**, no foreign content inner nodes (SVG/MathML tokenized but not parsed inline).
- **No `<script>` execution** — `<script>` and `<noscript>` are parsed but entirely ignored. JavaScript is permanently out of scope.

## How CSS and HTML Interact

CSS styling flows into the DOM through three channels recognized during parsing:

### 1. Inline `style` Attribute

```html
<div style="color: red; font-size: 18px; display: flex;">
  Styled inline
</div>
```

The `style` attribute value is parsed by the CSS declaration parser (`parse_inline_style`) and stored directly on the element. Inline styles sit at the top of the cascade — they always beat author stylesheets.

### 2. `<style>` Blocks

```html
<head>
  <style>
    .card {
      background-color: #f0f0f0;
      border-radius: 8px;
      padding: 16px;
    }
  </style>
</head>
```

Stylesheet text from `<style>` elements is parsed by the stylesheet parser and fed to the cascade as author-level rules. Selectors are evaluated against the full tree using specificity ordering.

### 3. `id` and `class` Attributes

```html
<div id="main-content" class="card highlighted">
```

- `id` — Enables `#id` selector matching. Uniqueness is not enforced but first-match semantics apply.
- `class` — A whitespace-separated token list enabling multi-class matching. An element with `class="card highlighted"` matches both `.card` and `.highlighted`.

The cascade engine matches these against selector patterns (tag, `#id`, `.class`, `*`, descendant combinator) to determine which rules apply to which elements.

### 4. Programmatic Custom Properties

Custom properties (`--my-var: value`) can be set programmatically on any node:

```rust
tree.root.as_mut().unwrap().set_custom_property("--theme-bg", "#1a1a2e");
```

These flow through the cascade and are available via `var(--theme-bg)` in CSS values throughout the document.

## DOM-style Queries

The tree exposes a browser-like API for walking and querying the DOM. These operate on the parsed tree **before** style cascade and layout:

```rust
// By ID
if let Some(btn) = tree.get_element_by_id("submit-btn") {
    btn.on_click = Some(Arc::new(|ev| println!("clicked!")));
}

// By class
let cards = tree.get_elements_by_class_name("card");
for card in &cards {
    // Walk each card node
}

// CSS selector query
if let Some(target) = tree.query_selector(".sidebar > .active") {
    // Matched element
}
```

## See Also

- [Element Index](./element-index) — Complete reference table of all ~100 supported elements
- [Elements](./elements) — Detailed per-element documentation with attribute specs
- [Parsing](./parsing) — Deep dive into tokenizer and tree builder internals
- [DOM API](./dom-api) — Full DOM-style API reference
