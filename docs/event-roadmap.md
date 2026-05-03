# Event Roadmap

> Roadmap for wiring up all HTML/DOM event types through the engine.
> Tracking both `dispatch.rs` wiring and the `Node` / `El` API surface.

---

## Current State: What Works

### Fully Dispatched + Dedicated Node Slots

- [x] **`mousedown`** — `Node::on_mouse_down`, bubbles target→root
- [x] **`mouseup`** — `Node::on_mouse_up`, bubbles target→root
- [x] **`click`** — `Node::on_click`, synthesized from press+release, bubbles
- [x] **`mousemove`** — `Node::on_mouse_move`, bubbles target→root
- [x] **`mouseenter`** — `Node::on_mouse_enter`, no bubbling
- [x] **`mouseleave`** — `Node::on_mouse_leave`, no bubbling

### Dispatched but Only Via Generic `on_event` Slot

- [x] **`keydown`** — bubbles; Tab/Shift+Tab navigation built in
- [x] **`keyup`** — bubbles
- [x] **`focus`** — no bubbling; target only
- [x] **`blur`** — no bubbling; target only
- [x] **`focusin`** — bubbles target→root
- [x] **`focusout`** — bubbles target→root

### Core Events Now Dispatched (Phase 1 - DONE)

- [x] **`input`** — `text_input()` and `handle_edit_key()` fire `InputEvent`
- [x] **`change`** — fires on blur when text-editable value differs from focus snapshot
- [x] **`wheel`** — `Tree::wheel_event()` fires before imperative scroll
- [x] **`submit`** — Enter in form input + click on submit button fire `SubmitEvent`

### Type System Ready But Never Emitted

- [ ] **`dblclick`** — type constant exists; click-count infra in harness, not synthesized
- [ ] **`contextmenu`** — type constant exists; right-click not special-cased
- [ ] **`auxclick`** — type constant exists; middle-button not special-cased
- [ ] **`scroll`** — type constant exists; no scroll events on overflow containers
- [ ] **`select`** — type constant exists; no selectionchange events
- [ ] **`resize`** — type constant exists; pipeline rerun, no event
- [ ] **`dragstart` / `drag` / `dragend` / `dragenter` / `dragover` / `dragleave` / `drop`** — 7 `DragEvent` variants exist; `draggable` attr parsed, no drag subsystem
- [ ] **`pointerdown` / `pointerup` / `pointermove` / `pointerenter` / `pointerleave` / `pointercancel`** — `PointerEvent` struct exists; never used
- [ ] **`touchstart` / `touchmove` / `touchend`** — `TouchEvent` struct exists; no touch input routed
- [ ] **`cut` / `copy` / `paste`** — `ClipboardEvent` struct exists; Ctrl+C uses `arboard` directly, bypassing events
- [ ] **`animationstart` / `animationend` / `animationiteration`** — `AnimationEvent` struct exists; CSS animations not implemented
- [ ] **`transitionstart` / `transitionend` / `transitionrun` / `transitioncancel`** — `TransitionEvent` struct exists; CSS transitions not implemented
- [ ] **`compositionstart` / `compositionupdate` / `compositionend`** — `CompositionEvent` struct exists; no IME
- [ ] **`toggle` / `beforetoggle`** — structs exist; `<details>` not modeled
- [ ] **`formdata`** — `FormDataEvent` struct exists; no form data construction
- [ ] **`progress`** — `ProgressEvent` struct exists; image loading has no progress events

---

## Phase 1 — Core Wiring Gaps `[DONE]`

Wire events that have structs but are never emitted. These are the
smallest diffs with the biggest practical impact.

### 1.1 `input` Event ✅

- [x] Emit `InputEvent` in `dispatch.rs:text_input()` after value mutation
- [x] Event carries `input_type: InputType` (default: `InsertText`)
- [x] Bubbles target→root
- [x] Fires on: typed character insert, backspace, delete, line-break in textarea
- [x] Does **not** fire on: arrow-key navigation, Home/End (those are `selectionchange`)

**Tests:** `text_input_fires_input_event`, `text_input_does_not_fire_input_on_non_editable`, `backspace_fires_input_with_delete_content_backward`

### 1.2 `change` Event ✅

- [x] Detect value commit on blur for `<input>` / `<textarea>`
- [x] `focus_value_snapshot` in `InteractionState` tracks value at focus-gain
- [x] Compared on blur; fires `change` (bubbling) if value differs
- [ ] Detect checkbox/radio toggle (once 5.1 lands)
- [ ] Detect `<select>` option change (once 5.2 lands)

**Tests:** `change_event_fires_when_value_mutated_then_blurred`, `change_event_does_not_fire_when_value_unchanged_on_blur`

### 1.3 `wheel` Event ✅

- [x] `Tree::wheel_event(pos, delta_x, delta_y, delta_mode)` dispatches to hovered element
- [x] `cancelable: true` — ready for `preventDefault()` (once 4.1 lands)
- [x] Bubbles from hovered element to root
- [x] Carry `delta_x`, `delta_y`, `delta_mode` (pixel/line)
- [x] Winit harness calls `tree.wheel_event()` before scroll in both old and new handlers

**Tests:** `wheel_event_dispatches_to_hovered_element`, `wheel_event_with_no_hover_dispatches_to_root`

### 1.4 `submit` Event ✅

- [x] Detect Enter in focused `<input>` inside a `<form>`
- [x] Detect click on `<button type="submit">` (or `<button>` without type) inside a `<form>`
- [x] Emit `SubmitEvent` on the `<form>` element (with `submitter` path)
- [x] `cancelable: true` — user can prevent implicit submission
- [x] `find_ancestor_form(path)` walks up the tree to find `<form>` parent

**Tests:** `enter_in_form_input_fires_submit`, `enter_in_non_form_input_does_not_fire_submit`
- [ ] `submit_event_does_not_fire_on_type_button`

---

## Phase 2 — Dedicated Node Callback Slots `[Medium Priority]`

Add `Node` fields so users don't have to destructure `on_event`.
Mirror what already exists for mouse events.

### Node Fields to Add

- [ ] `pub on_keydown: Option<KeyboardCallback>` — `Arc<dyn Fn(&KeyboardEvent) + Send + Sync + 'static>`
- [ ] `pub on_keyup: Option<KeyboardCallback>`
- [ ] `pub on_focus: Option<FocusCallback>` — `Arc<dyn Fn(&FocusEvent) + Send + Sync + 'static>`
- [ ] `pub on_blur: Option<FocusCallback>`
- [ ] `pub on_focusin: Option<FocusCallback>`
- [ ] `pub on_focusout: Option<FocusCallback>`
- [ ] `pub on_input: Option<InputCallback>` — `Arc<dyn Fn(&InputEvent) + Send + Sync + 'static>`
- [ ] `pub on_change: Option<EventCallback>`
- [ ] `pub on_wheel: Option<WheelCallback>` — `Arc<dyn Fn(&WheelEvent) + Send + Sync + 'static>`

### Type Aliases (in `wgpu-html-tree/src/events.rs`)

- [ ] `pub type KeyboardCallback = Arc<dyn Fn(&KeyboardEvent) + Send + Sync + 'static>`
- [ ] `pub type FocusCallback = Arc<dyn Fn(&FocusEvent) + Send + Sync + 'static>`
- [ ] `pub type InputCallback = Arc<dyn Fn(&InputEvent) + Send + Sync + 'static>`
- [ ] `pub type WheelCallback = Arc<dyn Fn(&WheelEvent) + Send + Sync + 'static>`

### El DSL (in `wgpu-html-ui/src/el.rs`)

Each new slot needs:
- [ ] `El::on_keydown(f)` + `El::on_keydown_cb(cb)`
- [ ] `El::on_keyup(f)` + `El::on_keyup_cb(cb)`
- [ ] `El::on_focus(f)` + `El::on_focus_cb(cb)`
- [ ] `El::on_blur(f)` + `El::on_blur_cb(cb)`
- [ ] `El::on_focusin(f)` + `El::on_focusin_cb(cb)`
- [ ] `El::on_focusout(f)` + `El::on_focusout_cb(cb)`
- [ ] `El::on_input(f)` + `El::on_input_cb(cb)`
- [ ] `El::on_change(f)` + `El::on_change_cb(cb)`
- [ ] `El::on_wheel(f)` + `El::on_wheel_cb(cb)`

### Ctx Helpers (in `wgpu-html-ui/src/core/ctx.rs`)

- [ ] `Ctx::on_keydown(msg)` / `Ctx::keydown_callback(|ev| Msg)`
- [ ] `Ctx::on_input(msg)` / `Ctx::input_callback(|ev| Msg)`
- [ ] `Ctx::on_change(msg)` / `Ctx::change_callback(|ev| Msg)`
- [ ] `Ctx::on_focus(msg)` / `Ctx::focus_callback(|ev| Msg)`
- [ ] `Ctx::on_blur(msg)` / `Ctx::blur_callback(|ev| Msg)`


## Phase 3 — Click Variants `[Medium Priority]`

### 3.1 `dblclick`

- [ ] Track primary-button click count with a configurable threshold (300 ms default)
- [ ] Synthesize `dblclick` after the second click if within threshold
- [ ] Emit sequence: `mousedown → mouseup → click → dblclick` (DOM order)
- [ ] Bubbles target→root

**Tests needed:**
- [ ] `dblclick_fires_on_two_clicks_within_300ms`
- [ ] `dblclick_does_not_fire_when_clicks_too_far_apart`
- [ ] `dblclick_does_not_fire_when_target_changes_between_clicks`

### 3.2 `contextmenu`

- [ ] Detect secondary-button press (right-click)
- [ ] Emit `contextmenu` event; `cancelable: true`
- [ ] Bubbles target→root

### 3.3 `auxclick`

- [ ] Detect middle-button (`button == 1`) press+release on same element
- [ ] Emit `auxclick` after `mouseup`

---

## Phase 4 — Event Control Semantics `[Medium Priority]`

### 4.1 `preventDefault` / `defaultPrevented`

- [ ] Add `default_prevented: bool` to `HtmlEvent`
- [ ] Dispatch checks `default_prevented` after bubbling to decide default behavior
- [ ] Wire for `wheel` (skip scroll), `submit` (skip form submit), `contextmenu` (suppress OS menu)
- [ ] `cancelable: false` events silently ignore `preventDefault()`

### 4.2 `stopPropagation` / `stopImmediatePropagation`

- [ ] Add `propagation_stopped: bool` / `immediate_propagation_stopped: bool` to `HtmlEvent`
- [ ] Check flags in bubble/emit loops
- [ ] `stopImmediatePropagation` skips remaining handlers on same node

### 4.3 Capture Phase

- [ ] Walk root→target before target→root bubble for appropriate event types
- [ ] `EventPhase::CapturingPhase` already defined, never used
- [ ] Dedicated capture callback slots not needed — users route via `on_event` during capture

---

## Phase 5 — Form Control Interactions `[Medium Priority]`

### 5.1 Checkbox / Radio Toggle

- [ ] `mousedown` or `click` on `<input type="checkbox">` flips `checked`
- [ ] `mousedown` or `click` on `<input type="radio">` unchecks siblings in same `name` group, checks target
- [ ] Fires `change` event after toggle (wired in phase 1.2)
- [ ] Fires `input` event after toggle

**Tests needed:**
- [ ] `checkbox_toggles_on_click`
- [ ] `radio_exclusive_selection_by_name`
- [ ] `checkbox_fires_change_event`

### 5.2 `<select>` Dropdown

- [ ] Render dropdown overlay (paint quads for option list)
- [ ] Click-to-open / click-outside-to-close
- [ ] Click on `<option>` selects it, closes dropdown, fires `change`
- [ ] Keyboard: arrow keys to navigate options, Enter to select, Escape to close

### 5.3 Enter/Space on Focused Controls

- [ ] `Space` on focused `<button>` → synthesizes `click`
- [ ] `Enter` on focused `<button>` → synthesizes `click`
- [ ] `Enter` on focused `<a>` → navigates `href`
- [ ] `Enter` on focused `<input type="submit">` → form submit
- [ ] `Enter` on focused `<textarea>` → inserts newline (already done)

---

## Phase 6 — Drag and Drop `[Low Priority]`

- [ ] Track `draggable` attribute on source element
- [ ] `mousedown` + drag distance threshold → emit `dragstart`
- [ ] `mousemove` while dragging → emit `drag` on source, `dragover` / `dragenter` / `dragleave` on potential drop targets
- [ ] `mouseup` on valid drop target → emit `drop` on target, `dragend` on source
- [ ] Drag image / feedback (ghost element during drag)

---

## Phase 7 — Pointer + Touch Unification `[Low Priority]`

- [ ] Route touch input from `winit::Touch` events (currently only `CursorMoved` + `MouseInput`)
- [ ] Synthesize `PointerEvent` with `pointer_type: Touch | Pen | Mouse`
- [ ] Emit `touchstart` / `touchmove` / `touchend`
- [ ] Touch scroll via `touchmove` with inertia
- [ ] Multi-touch pinch-zoom (optional, stretch goal)

---

## Phase 8 — Clipboard Events `[Low Priority]`

- [ ] Emit `copy` event on `Ctrl+C` **before** using `arboard`
- [ ] Emit `cut` event on `Ctrl+X` before removing text and using `arboard`
- [ ] Emit `paste` event on `Ctrl+V` before inserting from `arboard`
- [ ] `cancelable: true` — user can supply custom clipboard data

---

## Phase 9 — Other Event Types `[Low Priority]`

### 9.1 `scroll` Event

- [ ] Emit on overflow containers when scroll position changes
- [ ] Non-bubbling (per DOM spec)
- [ ] Fire after `wheel` → `scroll_offset` update

### 9.2 `select` / `selectionchange`

- [ ] Emit `select` on `<input>` / `<textarea>` when selection range changes
- [ ] Emit `selectionchange` on the document when text selection changes (drag-select)

### 9.3 CSS Animation / Transition Events

- [ ] Blocked on CSS animations/transitions implementation
- [ ] Once anims land: emit `animationstart` / `animationend` / `animationiteration`
- [ ] Once transitions land: emit `transitionstart` / `transitionrun` / `transitionend` / `transitioncancel`

### 9.4 `resize` Event

- [ ] Emit on the document/window when the surface resizes

### 9.5 `toggle` / `beforetoggle`

- [ ] Blocked on `<details>` open/close state model
- [ ] `beforetoggle` before state change, `toggle` after

---

## Cross-Cutting: What Goes Where

| Concern | File | Notes |
|---|---|---|
| Event structs + enums | `crates/wgpu-html-events/src/` | `HtmlEvent`, `MouseEvent`, `KeyboardEvent`, etc. All 70 event type names already defined |
| Node callback slots | `crates/wgpu-html-tree/src/lib.rs` | New fields on `Node` struct |
| Callback type aliases | `crates/wgpu-html-tree/src/events.rs` | `MouseCallback`, `EventCallback`, new `KeyboardCallback` etc. |
| Dispatch (tree-only) | `crates/wgpu-html-tree/src/dispatch.rs` | `key_down`, `mouse_down`, `focus`, `text_input`, etc. |
| Dispatch tests | `crates/wgpu-html-tree/src/dispatch_tests.rs` | Test all new event emissions here |
| TreeHook | `crates/wgpu-html-tree/src/tree_hook.rs` | 22 hook methods already exist for all event types |
| Layout-aware wrappers | `crates/wgpu-html/src/interactivity.rs` | `pointer_move`, `mouse_down`, `mouse_up` (hit-test then dispatch) |
| El DSL | `crates/wgpu-html-ui/src/el.rs` | `on_keydown()`, `on_input()`, etc. |
| Ctx factory | `crates/wgpu-html-ui/src/core/ctx.rs` | `on_keydown(msg)`, etc. |
| Winit harness | `crates/wgpu-html-winit/src/` | Forward new winit events (touch, additional keys) |

---

## Implicit Design Rules

When adding events, follow these patterns from existing code:

1. **Mouse events** — dispatched in two layers: type-specific slot + generic `on_event` with full `HtmlEvent`. TreeHook `on_mouse_event` (lite) + `on_event` (full) emitted before each.
2. **Bubbling events** — walk `target_path` → root. Each ancestor gets a clone with updated `current_path`, `event_phase`, `time_stamp`.
3. **Non-bubbling events** — fire only on the target node.
4. **Hook short-circuit** — if any hook returns `Stop`, skip the rest of dispatch.
5. **DOM inheritance** — all typed events compose via `base` fields: `Event { … }` → `UIEvent { base: Event, … }` → `MouseEvent { base: UIEvent, … }`.
6. **Timestamps** — computed as `(Instant::now() - tree.interaction.time_origin).as_secs_f64() * 1000.0`.
7. **Tests** — use `RecordingHook` (from `dispatch_tests.rs`) to assert event type order; use `Arc<Mutex<Vec<_>>>` to collect callbacks on `on_event`.
8. **Dedicated slots** — add to `Node`, wire into the appropriate dispatch function, and expose in `El` DSL + `Ctx`.
