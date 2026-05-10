---
sidebar_position: 5
---

# Custom Elements

lui supports a custom element system for extensibility.

## Registering Custom Elements

Custom elements are tags containing a hyphen (`-`) that are recognized by the parser:

```rust
tree.register_custom_element(
    "my-counter",      // tag name (must contain '-')
    |node: &Node| {    // factory: takes parsed attributes, returns a Node
        Node::div()
            .with_child(Node::text("Count: 0"))
            .on_click(|_| { /* handle click */ })
    }
);
```

The parser stores custom elements as `Element::CustomElement` with their attributes preserved. When `resolve_custom_elements()` is called, the factory replaces each custom element with the produced node.

## Resolution

```rust
tree.resolve_custom_elements();
```

This walks the tree, finds all `CustomElement` nodes, calls their registered factories, and replaces them in-place. Resolution happens before cascade.

## Use Cases

- **Component abstractions** — wrap complex UI patterns in reusable custom element tags
- **Third-party widget integration** — define custom tags like `<data-table>`, `<chart-graph>`
- **Progressive enhancement** — start with custom elements for prototyping, replace with native elements later

## Limitations

- No Shadow DOM or `closed` mode support
- No lifecycle callbacks (connectedCallback, disconnectedCallback, etc.)
- No attribute change observation
- Slots (`<slot>`) are parsed but not distributed
- Custom elements participate in CSS cascade and selector matching as regular inline elements unless their factory produces different display values
