# lui — Events Spec

The typed event model, which event structs exist, which events are
actually dispatched today, and which browser-style event names are
currently only placeholders. Companion to `spec/interactivity.md`
(state, hit-testing, focus, selection, scroll) and
`docs/full-status.md`.

JavaScript is permanently out of scope. Events are delivered to
host-installed Rust callbacks on `Node`: the mouse-specific slots
(`on_click`, `on_mouse_down`, `on_mouse_up`, `on_mouse_enter`,
`on_mouse_leave`) and the general `on_event: Fn(&HtmlEvent)` slot.

---

## 1. Architecture

The `lui-events` crate models DOM event inheritance via
composition. Each event struct embeds its parent in a `base` field:

```rust
MouseEvent {
    base: UIEvent {
        base: Event { /* common fields */ },
        detail,
    },
    client_x,
    client_y,
    button,
    buttons,
    // ...
}
```

The dispatch channel is `HtmlEvent`, an enum over concrete event
structs:

```rust
pub enum HtmlEvent {
    Mouse(MouseEvent),
    Pointer(PointerEvent),
    Wheel(WheelEvent),
    Keyboard(KeyboardEvent),
    Focus(FocusEvent),
    Input(InputEvent),
    Composition(CompositionEvent),
    Clipboard(ClipboardEvent),
    Drag(DragEvent),
    Touch(TouchEvent),
    Animation(AnimationEvent),
    Transition(TransitionEvent),
    Submit(SubmitEvent),
    FormData(FormDataEvent),
    Toggle(ToggleEvent),
    BeforeToggle(BeforeToggleEvent),
    Progress(ProgressEvent),
    Generic(Event),
}
```

`NodeId` is a child-index path (`Vec<usize>`) from the tree root. It
matches the path convention used by layout hit-testing and tree query
helpers.

## 2. Base Fields

Every concrete event eventually contains `Event`:

| Field | Supported | Note |
|---|---:|---|
| `event_type` | Yes | `HtmlEventType` stores the string name. |
| `bubbles` | Yes | Set per dispatched event; only bubbling phase is implemented for bubbling events. |
| `cancelable` | Structural only | Stored, but there is no `prevent_default()` API. |
| `composed` | Structural only | Stored; no shadow DOM exists. |
| `target` | Yes | Path to the original target. |
| `current_target` | Yes | Path for the listener currently being invoked. |
| `event_phase` | Partial | `AtTarget` and `BubblingPhase` are used; capture phase is not dispatched. |
| `default_prevented` | Structural only | Stored as data, not mutated by callbacks. |
| `is_trusted` | Yes | Engine-dispatched events set this true. |
| `time_stamp` | Yes | Milliseconds since `InteractionState::time_origin`. |

## 3. Dispatch Surface

Events are dispatched by `lui-tree::dispatch`, with layout-aware
wrappers in `lui::interactivity`.

Current path-based dispatchers:

```rust
dispatch_pointer_move(tree, target_path, pos, text_cursor)
dispatch_pointer_leave(tree)
dispatch_mouse_down(tree, target_path, pos, button, text_cursor)
dispatch_mouse_up(tree, target_path, pos, button, text_cursor)
focus(tree, path)
blur(tree)
focus_next(tree, reverse)
key_down(tree, key, code, repeat)
key_up(tree, key, code)
text_input(tree, text)
```

All of these except `text_input` also exist as inherent methods on
`Tree`. `text_input` is a free function re-exported by
`lui-tree`.

`lui-winit` translates winit mouse/keyboard events into these
dispatchers. The harness also implements scrollbars, viewport/element
scroll, screenshots, and clipboard shortcuts; those host actions do
not automatically imply DOM events unless listed below.

## 4. Event Support Table

| Event | Supported | Note |
|---|---:|---|
| `mousedown` | Yes | Dispatched as `HtmlEvent::Mouse`; bubbles target -> root; also calls `on_mouse_down`. |
| `mouseup` | Yes | Dispatched as `HtmlEvent::Mouse`; bubbles target -> root; also calls `on_mouse_up`. |
| `click` | Yes | Synthesized on primary release using deepest common ancestor; suppressed after non-collapsed drag selection; also calls `on_click`. |
| `mouseenter` | Yes | Synthesized when hover chain gains nodes; non-bubbling; root-first enter order. |
| `mouseleave` | Yes | Synthesized when hover chain loses nodes or pointer leaves surface; non-bubbling; deepest-first leave order. |
| `mousemove` | No | Hover state updates, but no DOM `mousemove` event is emitted. |
| `mouseover`, `mouseout` | No | Not synthesized. |
| `dblclick` | No | No click-count/threshold tracking yet. |
| `contextmenu` | No | Secondary-button host behavior is not converted into a DOM event. |
| `auxclick` | No | Middle/other button click event is not synthesized. |
| `pointerdown`, `pointerup` | Struct only | `PointerEvent` exists, but pointer events are not dispatched. |
| `pointermove`, `pointerenter`, `pointerleave`, `pointerover`, `pointerout` | Struct only | Mouse hover dispatch exists; DOM `PointerEvent` variants are not emitted. |
| `gotpointercapture`, `lostpointercapture`, `pointercancel` | Struct/name only | Explicit pointer capture API does not exist. |
| `keydown` | Yes | Dispatched to focused path or root; bubbles; winit glue forwards modifiers too. |
| `keyup` | Yes | Dispatched to focused path or root; bubbles. |
| `focus` | Yes | Dispatched on new focus target; non-bubbling. |
| `blur` | Yes | Dispatched on previous focus target; non-bubbling. |
| `focusin` | Yes | Dispatched on new focus target; bubbles. |
| `focusout` | Yes | Dispatched on previous focus target; bubbles. |
| `input` | Struct only | `InputEvent` exists; `text_input` mutates values but does not emit DOM `input`. |
| `beforeinput` | Name only | No event struct or dispatch path. |
| `change` | Name only | No form-control change event dispatch. |
| `compositionstart`, `compositionupdate`, `compositionend` | Struct/name only | `CompositionEvent` exists; IME/composition dispatch is missing. |
| `copy` | Struct/name only | Winit harness copies selected text to OS clipboard, but no DOM `ClipboardEvent` is emitted. |
| `cut` | Struct/name only | Form-control shortcut mutates values, but no DOM event is emitted. |
| `paste` | Struct/name only | Harness reads OS clipboard into focused control, but no DOM event is emitted. |
| `wheel` | Struct only | Winit harness scrolls viewport/elements; no DOM `WheelEvent` is forwarded to `on_event`. |
| `scroll`, `scrollend` | Name only | Scroll state changes do not emit DOM events. |
| `select`, `selectstart`, `selectionchange` | Name only | Selection state changes do not emit DOM events. |
| `dragstart`, `drag`, `dragenter`, `dragleave`, `dragover`, `drop`, `dragend` | Struct/name only | `DragEvent` exists; drag-and-drop API is not implemented. |
| `touchstart`, `touchmove`, `touchend`, `touchcancel` | Struct/name only | `TouchEvent` exists; touch input is out of scope for now. |
| `animationstart`, `animationiteration`, `animationend`, `animationcancel` | Struct/name only | `AnimationEvent` exists; CSS animation runtime is missing. |
| `transitionrun`, `transitionstart`, `transitionend`, `transitioncancel` | Struct/name only | `TransitionEvent` exists; CSS transition runtime is missing. |
| `submit` | Struct/name only | `SubmitEvent` exists; form submission behavior is missing. |
| `formdata` | Struct/name only | `FormDataEvent` exists; form data construction is missing. |
| `toggle`, `beforetoggle` | Struct/name only | Toggle structs exist; `<details>` behavior is not implemented. |
| `load`, `loadstart`, `loadend`, `progress`, `abort`, `error` | Struct/name only | `ProgressEvent` exists; resource lifecycle events are not emitted. |
| `resize` | Name only | Window resize drives relayout in hosts; no DOM event is emitted. |
| `invalid`, `reset`, `cancel`, `close` | Name only | Constants exist; no dispatch behavior. |
| Custom / generic events | Structural only | `HtmlEvent::Generic` exists, but there is no public generic dispatch helper. |

## 5. Bubbling And Phases

Implemented bubbling rules:

| Event family | Bubbling | Note |
|---|---:|---|
| Mouse down/up/click | Yes | Target -> root. |
| Mouse enter/leave | No | Fired only on changed hover-chain nodes. |
| Keyboard down/up | Yes | Focused path -> root, or root if nothing is focused. |
| Focus/blur | No | Target only. |
| Focusin/focusout | Yes | Target -> root. |

Capture phase is not implemented. `EventPhase::CapturingPhase` exists
as an enum variant for structural parity, but dispatch never walks
root -> target.

## 6. Coordinates And Buttons

Mouse events currently set screen/client/page/offset coordinates to
the same physical-pixel position supplied by the host. There is no
CSS-pixel conversion inside the event struct layer.

`MouseButton` maps to DOM button values:

| Engine button | DOM `button` | DOM `buttons` bit |
|---|---:|---:|
| `Primary` | `0` | `1` |
| `Middle` | `1` | `4` |
| `Secondary` | `2` | `2` |
| `Other(n)` | `n` | `1 << min(n, 15)` |

`InteractionState::buttons_down` tracks the active `buttons` bitmask.
`InteractionState::modifiers` is read when events are built; hosts
update it through `Tree::set_modifier`.

## 7. Keyboard Events

`lui-winit` maps winit `KeyEvent` into DOM-like `key` and
`code` strings:

- `key` comes from `event.logical_key`, so it follows the user's
  keyboard layout.
- `code` comes from the physical key (`KeyA`, `ArrowLeft`, etc.).
- modifier keys update `InteractionState::modifiers` and are still
  forwarded as `keydown` / `keyup`.
- printable `event.text` is forwarded to `text_input` when Ctrl/Meta
  are not held.

`keydown` also handles built-in Tab traversal. Editing keys
Backspace/Delete/ArrowLeft/ArrowRight/Home/End and textarea
ArrowUp/ArrowDown/Enter mutate the focused text control through the
text-edit path.

## 8. Clipboard And Text Input

Clipboard is host-owned. The winit harness uses `arboard` for:

- document-level `Ctrl+A` / `Ctrl+C`;
- focused form-control `Ctrl+A` / `Ctrl+C` / `Ctrl+X` / `Ctrl+V`.

Those shortcuts mutate selection/value state, but they do not emit
DOM `copy`, `cut`, `paste`, `beforeinput`, or `input` events yet.

## 9. Open Gaps

- Add actual dispatch for `InputEvent` and clipboard events.
- Add `wheel` event dispatch without breaking host scroll defaults.
- Add `mousemove` or pointer-event dispatch if hosts need continuous
  move callbacks beyond `AppHook::on_pointer_move`.
- Add double-click/contextmenu/auxclick synthesis.
- Decide whether event cancellation should exist; today callbacks
  cannot call `prevent_default()` or stop propagation.
- Decide whether capture phase is worth implementing without a DOM API.
