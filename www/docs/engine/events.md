---
sidebar_position: 8
---

# Events

lui has a typed DOM-style event system in `lui-events`, with dispatch in `lui-tree`.

## Event Hierarchy

```rust
pub struct HtmlEvent {
    pub event_type: HtmlEventType,
    pub phase: EventPhase,
    pub target_path: Vec<usize>,
    pub current_target_path: Vec<usize>,
    pub bubbles: bool,
    pub cancelable: bool,
    pub default_prevented: bool,
    pub time_origin: Instant,
}
```

## Event Types

| Category | Event Types |
|---|---|
| **Mouse** | `Click`, `DblClick`, `AuxClick`, `MouseDown`, `MouseUp`, `MouseMove`, `MouseEnter`, `MouseLeave`, `ContextMenu` |
| **Keyboard** | `KeyDown`, `KeyUp` |
| **Focus** | `Focus`, `Blur`, `FocusIn`, `FocusOut` |
| **Input** | `Input`, `BeforeInput`, `Change` |
| **Scroll** | `Scroll`, `Wheel` |
| **Form** | `Submit`, `Reset` |
| **Clipboard** | `Copy`, `Cut`, `Paste` |
| **Drag** | `DragStart`, `DragEnd`, `Drag`, `DragEnter`, `DragLeave`, `DragOver`, `Drop` |
| **Selection** | `Select`, `SelectionChange` |
| **Touch** | `TouchStart`, `TouchMove`, `TouchEnd`, `TouchCancel` |
| **Media** | `Load`, `Error`, `Abort`, `CanPlay` |

## Event Phases

```rust
pub enum EventPhase {
    None,
    Capturing,  // root → target (not yet dispatched)
    AtTarget,   // on the target element
    Bubbling,   // target → root (dispatched on ancestors)
}
```

## Bubbling

Mouse events (`mousedown`, `mouseup`, `click`, `dblclick`, `contextmenu`, `auxclick`), keyboard events, focus events, and most input events bubble up the tree. `mouseenter` and `mouseleave` do NOT bubble.

Click synthesis: `mousedown` + `mouseup` on the same element produces a `click` event via deepest-common-ancestor resolution. Drag-to-select suppresses click.

## Callbacks

### Per-node callbacks (on `Node`)

```rust
// Typed event callback — catches all event types
node.on_event.push(Arc::new(|e: &HtmlEvent| {
    match &e.event_type {
        HtmlEventType::Click(mouse_event) => { /* ... */ }
        _ => {}
    }
}));

// Legacy per-event-type callbacks
node.on_click.push(Arc::new(|e: &MouseEvent| { /* ... */ }));
node.on_keydown.push(Arc::new(|e: &HtmlEvent| { /* ... */ }));
```

Available per-type callback slots: `on_click`, `on_mouse_down`, `on_mouse_up`, `on_mouse_move`, `on_mouse_enter`, `on_mouse_leave`, `on_dblclick`, `on_contextmenu`, `on_auxclick`, `on_dragstart`, `on_dragend`, `on_drop`, `on_drag`, `on_dragover`, `on_dragenter`, `on_dragleave`, `on_keydown`, `on_keyup`, `on_focus`, `on_blur`, `on_focusin`, `on_focusout`, `on_input`, `on_beforeinput`, `on_change`, `on_wheel`, `on_copy`, `on_cut`, `on_paste`, `on_scroll`, `on_select`.

### Tree hooks (global interception)

```rust
pub trait TreeHook {
    fn on_event(&mut self, tree: &mut Tree, event: &mut HtmlEvent) -> TreeHookResponse;
    fn on_keyboard_event(&mut self, tree: &mut Tree, event: &mut KeyboardEvent) -> TreeHookResponse;
    fn on_focus_event(&mut self, tree: &mut Tree, event: &mut FocusEvent) -> TreeHookResponse;
    // ... and more
}
```

Hooks can return `TreeHookResponse::Continue` or `TreeHookResponse::Stop` to control propagation.

## Layout-Free Dispatch

When you have a hit-test path but not a `LayoutBox`, use the `lui_tree::dispatch` functions:

```rust
use lui_tree::dispatch;

dispatch_pointer_move(&mut tree, target_path, pos, Some(cursor));
dispatch_mouse_down(&mut tree, target_path, pos, button, Some(cursor));
dispatch_mouse_up(&mut tree, target_path, pos, button, Some(cursor));
```
