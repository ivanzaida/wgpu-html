---
sidebar_position: 9
---

# Input Handling

Input handling bridges OS-level events (from winit, egui, etc.) to DOM-style event dispatch in the tree.

## Mouse Input

### Core Functions (in `lui/src/interactivity.rs`)

```rust
// Hit-tests and updates hover path. Returns true if hover changed.
pub fn pointer_move(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32)) -> bool;

// Hit-tests, fires on_mouse_down, handles focus, text selection, and form controls.
pub fn mouse_down(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32),
                  button: MouseButton) -> bool;

// Fires on_mouse_up and synthesises click event.
pub fn mouse_up(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32),
                button: MouseButton) -> bool;

// With click count for double/triple-click word/line selection
pub fn mouse_down_with_click_count(tree: &mut Tree, layout: &LayoutBox,
    pos: (f32, f32), button: MouseButton, click_count: u8) -> bool;
```

### Mouse Button Types

```rust
pub enum MouseButton { Primary, Secondary, Middle, Other(u8) }
```

### Enter / Leave Synthesis

`pointer_move` automatically synthesizes `mouseenter`/`mouseleave` events: deepest-first leave on the old hover path, root-first enter on the new hover path.

### Hit Testing

`LayoutBox::hit_path((x, y))` returns the deepest element path at a given coordinate. Elements with `pointer-events: none` are skipped. Hit-test coordinates are in document space (accounting for scroll offsets).

## Keyboard Input

### Key Dispatch

```rust
// Bubbles keydown/keyup along the focused element's ancestry
pub fn key_down(tree: &mut Tree, key: &str, code: &str, repeat: bool) -> bool;
pub fn key_up(tree: &mut Tree, key: &str, code: &str) -> bool;
```

Key and code strings follow DOM conventions: `key: "a"`, `code: "KeyA"`, `key: "Enter"`, `code: "Enter"`.

### Focus Management

```rust
tree.focus(path);      // Focus a specific element
tree.blur();           // Remove focus
tree.focus_next(reverse); // Tab/Shift+Tab traversal
```

Focusable elements: `<button>`, `<a href>`, `<input>` (not hidden), `<textarea>`, `<select>`, `<summary>`, anything with `tabindex >= 0`. Focus traversal cycles through keyboard-focusable paths in document order, wrapping at ends.

Focus events dispatched: `focus` → `focusin` on the new element, `blur` → `focusout` on the old element, with `related_target` carrying the counterpart path.

### Modifier Tracking

```rust
pub enum Modifier { Shift, Ctrl, Alt, Meta }

tree.set_modifier(Modifier::Shift, down);
```

Modifier state is stored on `InteractionState::modifiers` and accessed during event dispatch.

## Text Input

```rust
pub fn text_input(tree: &mut Tree, text: &str);
```

Inserts text at the focused input's caret position. Supports multibyte UTF-8. Non-text inputs (submit, button, etc.) ignore text input.

## Form Control Interaction

The `mouse_down` handler includes logic for:
- **Checkbox/radio toggle** — click to toggle `checked`, fires `change` and `input` events
- **File input** — opens native file dialog via rfd, stores `FileInfo` (name, size, mime)
- **Color picker** — opens color picker overlay
- **Date picker** — opens calendar overlay
- **Range slider** — drag to adjust value
- **Text editing** — click to place caret, drag to select

## Scroll Input

```rust
pub fn wheel_event(tree: &mut Tree, x: f32, y: f32,
                   dx: f64, dy: f64, mode: WheelDeltaMode) -> bool;
```

Scrolls the viewport or the deepest scrollable element under the cursor. Delta mode conversion: pixels → pixels directly, lines → pixels × 40, pages → viewport height.

## Runtime API (for winit integration)

The `Runtime<D>` abstraction provides unified input methods:

```rust
impl<D: Driver> Runtime<D> {
    pub fn on_pointer_move(&mut self, tree: &mut Tree, x: f32, y: f32) -> bool;
    pub fn on_mouse_button(&mut self, tree: &mut Tree, button: MouseButton, pressed: bool) -> bool;
    pub fn on_wheel(&mut self, tree: &mut Tree, pixel_dy: f32, pixel_dx: f32, scale: f32) -> bool;
    pub fn on_key(&mut self, tree: &mut Tree, key: &str, code: &str,
                  pressed: bool, repeat: bool, text: Option<&str>);
    pub fn set_modifier(&self, tree: &mut Tree, modifier: Modifier, down: bool);
    pub fn on_resize(&mut self, tree: &mut Tree, width: u32, height: u32);
    pub fn on_scale_change(&mut self);
}
```
