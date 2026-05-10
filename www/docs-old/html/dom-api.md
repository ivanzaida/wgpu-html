---
title: DOM-style API
---

# DOM-style API

lui exposes a browser-inspired DOM API on the `Tree` and `Node` types. All query methods are synchronous and walk the in-memory tree.

## Tree-level Query Methods

All methods on `Tree` operate from the document root:

### `get_element_by_id(id: &str) -> Option<&mut Node>`

Depth-first search for the first element with matching `id` attribute. Returns a mutable reference so callbacks and mutations can be attached.

```rust
if let Some(btn) = tree.get_element_by_id("submit-btn") {
    btn.on_click = Some(Arc::new(|ev| {
        println!("Submit clicked!");
    }));
}
```

### `get_element_by_class_name(class_name: &str) -> Option<&Node>`

First match by class token (whitespace-separated matching).

```rust
if let Some(card) = tree.get_element_by_class_name("card") {
    println!("First card found");
}
```

### `get_elements_by_class_name(class_name: &str) -> Vec<&Node>`

All matching elements, document order.

```rust
let all_active = tree.get_elements_by_class_name("active");
eprintln!("{} active elements", all_active.len());
```

### `get_element_by_name(name: &str) -> Option<&Node>`

First element whose `name` attribute matches. Useful for form controls.

```rust
if let Some(username_input) = tree.get_element_by_name("username") {
    // Found the input with name="username"
}
```

### `get_elements_by_name(name: &str) -> Vec<&Node>`

All elements with matching `name` attribute (case-sensitive).

```rust
let radios = tree.get_elements_by_name("color");
// All radio buttons with name="color"
```

### `get_element_by_tag_name(tag_name: &str) -> Option<&Node>`

First element matching the tag name (case-insensitive).

```rust
if let Some(form) = tree.get_element_by_tag_name("form") {
    // First <form> in the document
}
```

### `get_elements_by_tag_name(tag_name: &str) -> Vec<&Node>`

All elements matching the tag name.

```rust
let all_divs = tree.get_elements_by_tag_name("div");
```

### Path-returning Variants

The `find_elements_by_*` family returns `Vec<Vec<usize>>` — child-index paths instead of references:

```rust
let paths: Vec<Vec<usize>> = tree.find_elements_by_class_name("card");
for path in &paths {
    if let Some(node) = tree.root.as_ref()?.at_path(path) {
        println!("Card at path {:?}: {:?}", path, node.element.tag_name());
    }
}
```

Available:
- `find_elements_by_class_name(name) -> Vec<Vec<usize>>`
- `find_elements_by_name(name) -> Vec<Vec<usize>>`
- `find_elements_by_tag_name(name) -> Vec<Vec<usize>>`

Paths are useful for passing to `at_path()`, `at_path_mut()`, `node_to_html(path)`, and event dispatch functions.

## CSS Selector Queries

The `query` module (`lui-tree/src/query.rs`) provides full CSS selector support for runtime queries:

### `query_selector(sel: &str) -> Option<&Node>`

```rust
// Single match (first in document order)
if let Some(target) = tree.query_selector(".sidebar > .active") {
    eprintln!("Active sidebar item: {:?}", target.element.tag_name());
}
```

### `query_selector_all(sel: &str) -> Vec<&Node>`

```rust
// All matches
let disabled_inputs = tree.query_selector_all("input:disabled");
let odd_rows = tree.query_selector_all("tr:nth-child(odd)");
let has_img = tree.query_selector_all("li:has(img)");
```

### `matches(sel: &str) -> bool`

```rust
if let Some(node) = tree.query_selector("#main") {
    if node.matches(".visible") {
        // ...
    }
}
```

### `closest(sel: &str) -> Option<&Node>`

Walks up the ancestor chain:

```rust
if let Some(node) = tree.query_selector("#nested") {
    // Find the nearest ancestor <section>
    if let Some(section) = node.closest("section") {
        eprintln!("Inside section: {:?}", section.element.id());
    }
}
```

The query engine supports a superset of the stylesheet parser's selectors:
- **Combinators:** descendant (` `), child (`>`), next-sibling (`+`), subsequent-sibling (`~`)
- **Attribute selectors:** `[attr]`, `[attr=val]`, `[attr~=val]`, `[attr|=val]`, `[attr^=val]`, `[attr$=val]`, `[attr*=val]`, with optional `i` case flag
- **Pseudo-classes:** `:hover`, `:active`, `:focus`, `:focus-within`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:placeholder-shown`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:nth-child()`, `:nth-last-child()`, `:nth-of-type()`, `:not()`, `:is()`, `:where()`, `:has()`, `:root`, `:scope`, `:lang()`, `:dir()`
- **Selector lists:** comma-separated (`a, b, c`)
- **Pseudo-elements** (`::before`, `::after`) parsed but never match

## Node API

### Tree Traversal

```rust
// Walk from root
let html_node = &tree.root.unwrap();
let body = &html_node.children[0];
let first_child = &body.children[0];

// Path-based access
if let Some(node) = root.at_path(&[0, 2, 1]) {
    // node = root.children[0].children[2].children[1]
    eprintln!("Found: {}", node.element.tag_name());
}

// Walk path to root (useful for event bubbling)
let chain: Vec<&mut Node> = root.ancestry_at_path_mut(&[0, 2, 1]);
// chain[0] = deepest, chain.last() = root
```

### Accessing Element Data

```rust
let node: &Node = /* ... */;

// Tag name
println!("Tag: {}", node.element.tag_name());

// ID, class
if let Some(id) = node.element.id() {
    println!("ID: {}", id);
}
if let Some(class) = node.element.class() {
    println!("Classes: {}", class);
}

// Any attribute
if let Some(href) = node.element.attr("href") {
    println!("Link: {}", href);
}

// Data attribute
if let Some(user_id) = node.element.attr("data-user-id") {
    println!("User: {}", user_id);
}

// Boolean attributes
if node.element.attr("disabled").is_some() {
    println!("Element is disabled");
}
```

### Reading Layout Rectangles

After the layout pass completes, every node's content-box position is cached in `Node::rect`:

```rust
#[derive(Debug, Clone, Copy)]
pub struct NodeRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// After paint/layout:
if let Some(rect) = node.rect {
    println!("Node at ({}, {}), size {}×{}",
        rect.x, rect.y, rect.width, rect.height);
}
```

Store this rect **after** calling `paint_tree` / `compute_layout` — it is populated during the layout pass and unavailable before.

### Children

```rust
for child in &node.children {
    eprintln!("Child tag: {}", child.element.tag_name());
}

let count = node.children.len();
let first = node.children.first();
let last = node.children.last();
```

### Custom Properties

Programmatic CSS custom properties can be set on any node:

```rust
node.set_custom_property("--card-bg", "#f5f5f5");
if let Some(color) = node.custom_property("--card-bg") {
    eprintln!("Card background override: {}", color);
}
node.remove_custom_property("--card-bg");
```

Custom properties inherit to descendants and participate in `var(--card-bg)` resolution during cascade.

## Tree API

### Root Node

```rust
if let Some(root) = &tree.root {
    // Walk root
}
```

### Document Structure

```rust
// Access the <html> element
if let Some(html) = tree.root.as_ref() {
    assert_eq!(html.element.tag_name(), "html");
    let head = &html.children[0];  // <head> typically
    let body = &html.children[1];  // <body> typically
}
```

### Modifying the Tree Programmatically

```rust
// Insert a new node
let new_div = Node::new(Element::Div(m::Div::default()));
tree.insert_node(&[0, 1], 0, new_div);  // parent_path, index, node

// Append a child
let new_p = Node::new(Element::P(m::P::default()));
tree.append_node(&[0, 1], new_p);

// Remove a node
if let Some(removed) = tree.remove_node(&[0, 1, 0]) {
    eprintln!("Removed: {}", removed.element.tag_name());
}

// Template content
if let Some(range) = tree.append_template_content_to_id("row-tmpl", "table-body") {
    eprintln!("Inserted {} children", range.len());
}
```

### Focus and Active Element

```rust
// Focus management
lui_tree::focus(&mut tree, &[0, 1, 3]);  // Focus a specific path
lui_tree::focus(&mut tree, &[]);          // Focus root
lui_tree::blur(&mut tree);                // Clear focus
lui_tree::focus_next(&mut tree, false);    // Tab forward
lui_tree::focus_next(&mut tree, true);     // Shift+Tab backward

// Read active element
if let Some(active) = tree.active_element() {
    eprintln!("Focused: {}", active.element.tag_name());
}
if let Some(value) = tree.active_element().and_then(|n| n.element.attr("value")) {
    eprintln!("Focused value: {}", value);
}
```

### Interaction State

```rust
// Hover status
if let Some(hovered) = tree.hovered_element() {
    eprintln!("Hovered: {}", hovered.element.tag_name());
}
if tree.is_hovered(&[0, 1, 2]) {
    eprintln!("Element at [0,1,2] is in hover chain");
}

// Cursor position
if let Some((x, y)) = tree.cursor_position() {
    eprintln!("Cursor at ({}, {})", x, y);
}
```

## DOM Walking Example

```rust
use lui_parser::parse;
use lui_tree::{Element, Node};

let mut tree = parse(r#"
<!DOCTYPE html>
<html>
  <body>
    <div id="sidebar" class="panel">
      <ul>
        <li class="nav-item active"><a href="/home">Home</a></li>
        <li class="nav-item"><a href="/about">About</a></li>
      </ul>
    </div>
    <main id="content">
      <h1 class="title">Welcome</h1>
      <p>Content here</p>
    </main>
  </body>
</html>
"#);

// Find by ID
let sidebar = tree.get_element_by_id("sidebar").unwrap();
assert_eq!(sidebar.element.tag_name(), "div");

// Query selector
let active_link = tree.query_selector("#sidebar .active a").unwrap();
assert_eq!(active_link.element.attr("href").as_deref(), Some("/home"));

// Walk children
for nav_item in tree.get_elements_by_class_name("nav-item") {
    if let Some(link) = &nav_item.children.first() {
        eprintln!("Nav link: {:?}", link.element.attr("href"));
    }
}

// Path-based access
let body_path = &[0, 1]; // html.children[0] = head, html.children[1] = body
let main_path = &[0, 1, 1]; // body.children[1] = main
if let Some(main) = tree.root.as_ref().unwrap().at_path(main_path) {
    assert_eq!(main.element.id(), Some("content"));
}
```

## See Also

- [HTML Overview](./overview) — The big picture of how HTML integrates with the pipeline
- [Elements](./elements) — Detailed per-element attribute documentation
- [Interactivity](../interactivity/overview) — Event dispatch and mouse/keyboard handling
