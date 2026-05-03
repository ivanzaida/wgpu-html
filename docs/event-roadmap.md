# Event Roadmap

> Roadmap for wiring up all HTML/DOM event types through the engine.
> Tracking both `dispatch.rs` wiring and the `Node` / `El` API surface.

---

## Current State: What Works

### Fully Dispatched + Dedicated Node Slots

- [x] **`mousedown`** — `Node::on_mouse_down`, bubbles target→root
- [x] **`mouseup`** — `Node::on_mouse_up`, bubbles target→root
- [x] **`mousemove`** — `Node::on_mouse_move`, bubbles target→root
- [x] **`mouseenter`** — `Node::on_mouse_enter`, no bubbling
- [x] **`mouseleave`** — `Node::on_mouse_leave`, no bubbling
- [x] **`click`** — `Node::on_click`, synthesized from press+release, bubbles
- [x] **`dblclick`** — `Node::on_dblclick`, 300ms/5px threshold, bubbles
- [x] **`contextmenu`** — `Node::on_contextmenu`, secondary-button release, bubbles
- [x] **`auxclick`** — `Node::on_auxclick`, middle-button release, bubbles
- [x] **`dragstart`** — `Node::on_dragstart`, mousedown + ≥5px on `Node::draggable`, no bubble
- [x] **`dragend`** — `Node::on_dragend`, mouseup after drag, no bubble
- [x] **`drop`** — `Node::on_drop`, mouseup target after drag, no bubble
- [x] **`keydown`** — `Node::on_keydown`, bubbles; Tab/Shift+Tab built in
- [x] **`keyup`** — `Node::on_keyup`, bubbles
- [x] **`focus`** — `Node::on_focus`, target only
- [x] **`blur`** — `Node::on_blur`, target only
- [x] **`focusin`** — `Node::on_focusin`, bubbles
- [x] **`focusout`** — `Node::on_focusout`, bubbles
- [x] **`input`** — `Node::on_input`, text inserts/backspace/delete/enter in textarea
- [x] **`change`** — `Node::on_change`, blur when value differs from focus snapshot + checkbox/radio toggle
- [x] **`wheel`** — `Node::on_wheel`, bubbles; preventDefault works
- [x] **`submit`** — Enter in form input + click on submit button fire `SubmitEvent`

### Callback Slots on `Node` (22 total)

| Slot | Type | Bubbles |
|---|---|---|
| `on_click` | `Vec<MouseCallback>` | yes |
| `on_mouse_down` | `Vec<MouseCallback>` | yes |
| `on_mouse_up` | `Vec<MouseCallback>` | yes |
| `on_mouse_move` | `Vec<MouseCallback>` | yes |
| `on_mouse_enter` | `Vec<MouseCallback>` | no |
| `on_mouse_leave` | `Vec<MouseCallback>` | no |
| `on_dblclick` | `Vec<MouseCallback>` | yes |
| `on_contextmenu` | `Vec<MouseCallback>` | yes |
| `on_auxclick` | `Vec<MouseCallback>` | yes |
| `on_dragstart` | `Vec<MouseCallback>` | no |
| `on_dragend` | `Vec<MouseCallback>` | no |
| `on_drop` | `Vec<MouseCallback>` | no |
| `on_keydown` | `Vec<EventCallback>` | yes |
| `on_keyup` | `Vec<EventCallback>` | yes |
| `on_focus` | `Vec<EventCallback>` | no |
| `on_blur` | `Vec<EventCallback>` | no |
| `on_focusin` | `Vec<EventCallback>` | yes |
| `on_focusout` | `Vec<EventCallback>` | yes |
| `on_input` | `Vec<EventCallback>` | yes |
| `on_change` | `Vec<EventCallback>` | yes |
| `on_wheel` | `Vec<EventCallback>` | yes |
| `on_event` | `Vec<EventCallback>` | varies |

All slots use `Vec<_>` for multi-handler support (additive, like `addEventListener`).

### Event Control

- [x] **`preventDefault`** — `HtmlEvent::prevent_default()` via interior-mutable `Cell<bool>`
- [x] **`stopPropagation`** — `HtmlEvent::stop_propagation()`
- [x] **`stopImmediatePropagation`** — `HtmlEvent::stop_immediate_propagation()`
- [x] **Wheel cancel wired** — `wheel_event()` returns `bool`; winit harness skips scroll if prevented
- [ ] Capture phase (root→target walk before bubble)

### Form Control Interactions

- [x] Checkbox click/space toggles `checked`, fires `input` + `change`
- [x] Radio click/space checks self + unchecks same-name siblings, fires `input` + `change`
- [x] Enter/Space on `<button>` `<a href>` `<input type=submit|reset|button>` synthesizes click
- [x] Enter in form input fires `submit`; click on submit button fires `submit`
- [ ] `<select>` dropdown rendering and interaction

### Drag

- [x] `Node::draggable: bool` flag
- [x] `dragstart` on mousedown + ≥5 px movement
- [x] Drag suppresses click synthesis
- [x] `dragend` on source, `drop` on release target on mouseup
- [ ] `drag`, `dragover`, `dragenter`, `dragleave` intermediate events
- [ ] Drag ghost image / visual feedback

---

## Type System Ready But Never Emitted

- [ ] **`drag` / `dragover` / `dragenter` / `dragleave`** — intermediate drag events
- [ ] **`scroll`** — type constant exists; no scroll events on overflow containers
- [ ] **`select`** — type constant exists; no selectionchange events
- [ ] **`resize`** — type constant exists; pipeline rerun, no event
- [ ] **`pointerdown` / `pointerup` / `pointermove` / `pointerenter` / `pointerleave` / `pointercancel`** — `PointerEvent` struct exists; never used
- [ ] **`touchstart` / `touchmove` / `touchend`** — `TouchEvent` struct exists; no touch input routed
- [ ] **`cut` / `copy` / `paste`** — `ClipboardEvent` struct exists; Ctrl+C uses `arboard` directly
- [ ] **`animationstart` / `animationend` / `animationiteration`** — CSS animations not implemented
- [ ] **`transitionstart` / `transitionend` / `transitionrun` / `transitioncancel`** — CSS transitions not implemented
- [ ] **`compositionstart` / `compositionupdate` / `compositionend`** — no IME
- [ ] **`toggle` / `beforetoggle`** — `<details>` not modeled
- [ ] **`formdata`** — no form data construction
- [ ] **`progress`** — image loading has no progress events

---

## Remaining Phases

### Phase 7 — Select Dropdown `[Medium]`

- [ ] Render dropdown overlay (paint quads for option list)
- [ ] Click-to-open / click-outside-to-close
- [ ] Click on `<option>` selects it, closes dropdown, fires `change`

### Phase 8 — Pointer + Touch `[Low]`

- [ ] Route touch input from `winit::Touch` events
- [ ] Synthesize `PointerEvent` with `pointer_type: Touch | Pen | Mouse`
- [ ] Emit `touchstart` / `touchmove` / `touchend`

### Phase 9 — Clipboard Events `[Low]`

- [ ] Emit `copy` event on `Ctrl+C` before using `arboard`
- [ ] Emit `paste` event on `Ctrl+V` before inserting from `arboard`

### Phase 10 — Other Event Types `[Low]`

- [ ] `scroll` — emit on overflow containers when scroll position changes
- [ ] `select` / `selectionchange` — text selection range changes
- [ ] `resize` — surface resize
- [ ] `toggle` / `beforetoggle` — `<details>` (blocked)
- [ ] Animation / Transition events (blocked on CSS animations)
- [ ] Composition events (blocked on IME)
- [ ] Progress events (blocked on image loading events)
- [ ] Capture phase (root→target walk)

---

## Cross-Cutting: What Goes Where

| Concern | File | Notes |
|---|---|---|
| Event structs + enums | `crates/wgpu-html-events/src/` | `HtmlEvent`, `MouseEvent`, `KeyboardEvent`, etc. All 70 event type names already defined |
| Node callback slots | `crates/wgpu-html-tree/src/lib.rs` | 22 `Vec<Callback>` fields on `Node` struct |
| InteractionState | `crates/wgpu-html-tree/src/events.rs` | `focus_value_snapshot`, `drag_pending`, `drag_active_source`, `last_click_time/path` |
| Dispatch (tree-only) | `crates/wgpu-html-tree/src/dispatch.rs` | All event emission + bubbling logic |
| Dispatch tests | `crates/wgpu-html-tree/src/dispatch_tests.rs` | 35 dispatch tests |
| TreeHook | `crates/wgpu-html-tree/src/tree_hook.rs` | 22 hook methods already exist for all event types |
| Layout-aware wrappers | `crates/wgpu-html/src/interactivity.rs` | `pointer_move`, `mouse_down`, `mouse_up` (hit-test then dispatch) |
| El DSL | `crates/wgpu-html-ui/src/el.rs` | `.on_click(f)`, `.on_keydown(f)`, `.on_dragstart(f)`, etc. with `_cb` variants |
| Ctx factory | `crates/wgpu-html-ui/src/core/ctx.rs` | `ctx.on_click(msg)`, `ctx.on_input(msg)`, `ctx.on_dragstart(msg)`, etc. |
| Winit harness | `crates/wgpu-html-winit/src/` | Wheel dispatch + preventDefault check; scrollbar drag |
| Events crate | `crates/wgpu-html-events/src/` | `Cell<bool>` interior mutability on `Event` for `preventDefault`/`stopPropagation` |

---

## Implicit Design Rules

1. **Mouse events** — dispatched in two layers: type-specific slot + generic `on_event` with full `HtmlEvent`. TreeHook `on_mouse_event` (lite) + `on_event` (full) emitted before each.
2. **Bubbling events** — walk `target_path` → root. Each ancestor gets a clone with updated `current_path`, `event_phase`, `time_stamp`.
3. **Non-bubbling events** — fire only on the target node.
4. **Hook short-circuit** — if any hook returns `Stop`, skip the rest of dispatch.
5. **DOM inheritance** — all typed events compose via `base` fields: `Event { … }` → `UIEvent { base: Event, … }` → `MouseEvent { base: UIEvent, … }`.
6. **Timestamps** — computed as `(Instant::now() - tree.interaction.time_origin).as_secs_f64() * 1000.0`.
7. **Tests** — use `RecordingHook` (from `dispatch_tests.rs`) to assert event type order; use `Arc<Mutex<Vec<_>>>` to collect callbacks on `on_event`.
8. **Dedicated slots** — add to `Node`, wire into the appropriate dispatch function, and expose in `El` DSL + `Ctx`.
