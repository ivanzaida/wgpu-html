---
title: Focus + Keyboard
---

# Focus + Keyboard

Focus management and keyboard event dispatch are built into the tree's dispatch layer (`wgpu-html-tree::dispatch`).

## Focus State

Focus is stored as a path in `InteractionState::focus_path`:

```rust
tree.interaction.focus_path = Some(vec![0, 2, 1]);  // element at path [0,2,1] is focused
```

### Setting Focus

```rust
// Programmatic focus
interactivity::focus(&mut tree, Some(vec![0, 1]));

// Clear focus
interactivity::blur(&mut tree);

// Tab to next focusable
interactivity::focus_next(&mut tree, false);    // Tab
interactivity::focus_next(&mut tree, true);     // Shift+Tab (reverse)
```

Focus is also set automatically on mouse down (primary button): the deepest focusable ancestor of the hit path receives focus, or focus is cleared if no ancestor is focusable.

## Focusable Predicate

An element is keyboard-focusable if:

```rust
fn is_keyboard_focusable(element: &Element) -> bool {
    matches!(element,
        Element::Button(_) |
        Element::Input(input) if !matches!(input.type_, "hidden") |
        Element::TextArea(_) |
        Element::Select(_) |
        Element::Details { summary: _, open: _ } |
        _ if element.attr("tabindex")
            .and_then(|t| t.parse::<i32>().ok())
            .is_some_and(|t| t >= 0)
    )
}
```

Also focusable via `tabindex`:
- `tabindex="0"`: focusable in DOM order
- `tabindex="-1"`: focusable programmatically, not by Tab
- `tabindex` > 0: focusable with priority (higher values first)

Additionally, `<a>` elements with an `href` attribute are focusable.

## Tab / Shift+Tab Navigation

Built into `key_down()`:

```rust
// Tab key → focus next
if key == "Tab" && !modifiers.shift {
    focus_next(tree, false);
}
// Shift+Tab → focus previous
if key == "Tab" && modifiers.shift {
    focus_next(tree, true);
}
```

The order is depth-first DOM traversal. `tabindex` values order elements before DOM order.

## Modifier State

Modifiers are tracked as a bitmask:

```rust
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}
```

Hosts update modifiers before dispatching keyboard events:

```rust
tree.set_modifier(Modifier::Shift, true);   // Shift pressed
tree.set_modifier(Modifier::Ctrl, false);   // Ctrl released
```

Modifier state is read during dispatch (e.g., Ctrl+C copy check).

## Keyboard Event Dispatch

```rust
// In your event loop:
interactivity::key_down(&mut tree, &key_event, modifiers);
interactivity::key_up(&mut tree, &key_event, modifiers);
```

`key_down` handles:
1. Esc → blur focus
2. Tab / Shift+Tab → focus navigation
3. Ctrl+A → select all text
4. Ctrl+C → copy selection to clipboard
5. Text input forwarding to focused form controls
6. `on_key_down` / `on_key_up` callback dispatch on the focused element and bubbling

## Callback Dispatch

When a key event fires, the engine:

1. Looks up `focus_path` to find the focused element.
2. Fires `on_key_down` / `on_key_up` on that element.
3. Bubbles up to root, firing on each ancestor that has the callback.

The general `on_event` callback also fires for keyboard events through the event type system.
