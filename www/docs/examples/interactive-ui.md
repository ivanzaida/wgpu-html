---
sidebar_position: 4
---

# Interactive UI

Demonstrates hover, click, and focus interactions with callbacks wired in Rust.

## Example

```rust
let mut tree = parse(r#"
    <html><body style="margin: 32px; font-family: sans-serif;">
        <button id="my-btn">Click Me</button>
        <p id="counter">Clicks: 0</p>
    </body></html>
"#);

// Wire callbacks
if let Some(btn) = tree.get_element_by_id("my-btn") {
    let counter_id = "counter".to_string();
    let click_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

    btn.on_click.push(std::sync::Arc::new(move |_event| {
        let count = click_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        // Update counter text when re-parsing or by modifying the tree
    }));
}
```

## What It Shows

- Hover tracking (`:hover` pseudo-class)
- Click event synthesis
- Focus management with Tab/Shift+Tab
- `:active`, `:focus` pseudo-classes
- Event bubbling up the DOM tree
- `pointer-events: none` hit-test skip
