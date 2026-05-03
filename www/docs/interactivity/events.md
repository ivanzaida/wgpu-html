---
id: events
title: Event System
---

# Event System

The event system models W3C DOM events via composition (`wgpu-html-events` crate) and delivers them through callback slots on `Node` (`wgpu-html-tree`).

## Event Types

All event types are defined in `wgpu-html-events::events`:

| Event | Fields |
|---|---|
| `Event` | `event_type`, `bubbles`, `cancelable`, `composed`, `target`, `current_target`, `event_phase`, `time_stamp` |
| `UIEvent` | Embeds `Event` as `base`, adds `detail` |
| `MouseEvent` | Embeds `UIEvent`, adds `screen_x/y`, `client_x/y`, `offset_x/y`, `page_x/y`, `button`, `buttons`, `modifier_keys` |
| `KeyboardEvent` | Embeds `UIEvent`, adds `key`, `code`, `location`, `repeat`, `is_composing`, `modifier_keys` |
| `FocusEvent` | Embeds `UIEvent`, adds `related_target` |
| `InputEvent` | Embeds `UIEvent`, adds `data`, `input_type`, `is_composing` |
| `WheelEvent` | Embeds `MouseEvent`, adds `delta_x/y/z`, `delta_mode` |
| `PointerEvent` | Embeds `MouseEvent`, adds `pointer_id`, `width`, `height`, `pressure`, `pointer_type` |
| `CompositionEvent` | Embeds `UIEvent`, adds `data` |
| `ClipboardEvent` | Embeds `Event`, adds clipboard data |
| `DragEvent` | Embeds `MouseEvent`, adds data transfer |
| `TouchEvent` | Multi-touch support |
| `AnimationEvent` | CSS animation events |
| `TransitionEvent` | CSS transition events |
| `SubmitEvent`, `FormDataEvent`, `ToggleEvent`, `ProgressEvent` | Form-related and resource events |

## Bubbling Semantics

| Event | Bubbles |
|---|---|
| `mousedown`, `mouseup`, `click` | Target → root |
| `mouseenter`, `mouseleave` | Do NOT bubble |
| `keydown`, `keyup` | Target → root |
| `focusin`, `focusout` | Target → root |
| `focus`, `blur` | Do NOT bubble |
| `wheel` | Target → root |
| `input` | Target → root |

Bubbling fires listeners on the target first, then each ancestor up to the root. `e.stop_propagation()` is not yet implemented — all listeners on the path fire.

## Callback Slots on Node

Each `Node` carries typed callback slots:

```rust
// Mouse-specific callbacks
node.on_click:    Option<Arc<dyn Fn(&MouseEvent) + Send + Sync>>,
node.on_mouse_down:   Option<Arc<dyn Fn(&MouseEvent) + Send + Sync>>,
node.on_mouse_up:     Option<Arc<dyn Fn(&MouseEvent) + Send + Sync>>,
node.on_mouse_enter:  Option<Arc<dyn Fn(&MouseEvent) + Send + Sync>>,
node.on_mouse_leave:  Option<Arc<dyn Fn(&MouseEvent) + Send + Sync>>,

// General event callback
node.on_event:    Option<Arc<dyn Fn(&HtmlEvent) + Send + Sync>>,
```

All callbacks are `Arc<dyn Fn>` — thread-safe, shareable closures.

## Attaching Callbacks

```rust
use wgpu_html_tree::{Tree, Node, Element};

// Get an element from the tree
if let Some(button) = tree.get_element_by_id("submit-btn") {
    button.on_click = Some(std::sync::Arc::new(|ev: &MouseEvent| {
        println!("Clicked at ({}, {})!", ev.pos.0, ev.pos.1);
    }));
}
```

## Hit Testing Integration

Position-based events (pointer move, mouse down/up) use `LayoutBox::hit_path_scrolled()` to resolve the DOM path from a screen coordinate. The path is then used by the dispatch layer to:

1. Walk the path, firing `on_mouse_down` / `on_mouse_up` / `on_click` on each ancestor.
2. Diff the hover path to fire `on_mouse_enter` / `on_mouse_leave` on changed elements.
3. Store the active path for click synthesis and focus.

## Complete Example

```rust
use wgpu_html::interactivity;
use wgpu_html_tree::MouseButton;

// In your event loop, on pointer move:
let (hover_changed, cursor) = interactivity::pointer_move_with_cursor(
    &mut tree, &layout, (x, y)
);

// On mouse down:
interactivity::mouse_down(&mut tree, &layout, (x, y), MouseButton::Primary);

// On mouse up:
interactivity::mouse_up(&mut tree, &layout, (x, y), MouseButton::Primary);

// On scroll:
tree.dispatch_scroll(&layout, pos, delta, &mut tree.interaction.scroll_offsets_y);
```
