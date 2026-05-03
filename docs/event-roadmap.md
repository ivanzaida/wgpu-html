# Event Roadmap

> 27 dedicated Node callback slots, 40 dispatch tests, 8 phases complete.

---

## Current State: Fully Dispatched + Dedicated Slots

| # | Event | Node Slot | Type | Bubbles | Trigger |
|---|---|---|---|---|---|
| 1 | `mousedown` | `on_mouse_down` | `MouseCallback` | yes | primary-button press |
| 2 | `mouseup` | `on_mouse_up` | `MouseCallback` | yes | primary-button release |
| 3 | `mousemove` | `on_mouse_move` | `MouseCallback` | yes | pointer move |
| 4 | `mouseenter` | `on_mouse_enter` | `MouseCallback` | no | pointer enters subtree |
| 5 | `mouseleave` | `on_mouse_leave` | `MouseCallback` | no | pointer leaves subtree |
| 6 | `click` | `on_click` | `MouseCallback` | yes | press+release on same target |
| 7 | `dblclick` | `on_dblclick` | `MouseCallback` | yes | two clicks ≤300ms same element |
| 8 | `contextmenu` | `on_contextmenu` | `MouseCallback` | yes | secondary-button release |
| 9 | `auxclick` | `on_auxclick` | `MouseCallback` | yes | middle-button release |
| 10 | `dragstart` | `on_dragstart` | `MouseCallback` | no | mousedown + ≥5px on draggable |
| 11 | `dragend` | `on_dragend` | `MouseCallback` | no | mouseup after drag |
| 12 | `drop` | `on_drop` | `MouseCallback` | no | mouseup target after drag |
| 13 | `keydown` | `on_keydown` | `EventCallback` | yes | key press; Tab/Shift+Tab built in |
| 14 | `keyup` | `on_keyup` | `EventCallback` | yes | key release |
| 15 | `focus` | `on_focus` | `EventCallback` | no | element gains focus |
| 16 | `blur` | `on_blur` | `EventCallback` | no | element loses focus |
| 17 | `focusin` | `on_focusin` | `EventCallback` | yes | focus entering (bubbles) |
| 18 | `focusout` | `on_focusout` | `EventCallback` | yes | focus leaving (bubbles) |
| 19 | `input` | `on_input` | `EventCallback` | yes | text insert, backspace, delete, enter |
| 20 | `change` | `on_change` | `EventCallback` | yes | blur-value-differs, checkbox/radio toggle |
| 21 | `wheel` | `on_wheel` | `EventCallback` | yes | scroll wheel; preventDefault works |
| 22 | `submit` | — (via `on_event`) | — | yes | Enter in form input, submit button click |
| 23 | `scroll` | `on_scroll` | `EventCallback` | no | overflow container scroll offset change |
| 24 | `selectionchange` | — (via `on_event`) | — | no | text selection start/extend/finalize |
| 25 | `copy` | `on_copy` | `EventCallback` | yes | Ctrl+C before arboard; preventDefault works |
| 26 | `cut` | `on_cut` | `EventCallback` | yes | Ctrl+X before arboard |
| 27 | `paste` | `on_paste` | `EventCallback` | yes | Ctrl+V before arboard |

All slots use `Vec<_>` for additive multi-handler support.

## Event Control

- [x] **`preventDefault`** — `HtmlEvent::prevent_default()` via `Cell<bool>`
- [x] **`stopPropagation`** — `HtmlEvent::stop_propagation()`
- [x] **`stopImmediatePropagation`** — `HtmlEvent::stop_immediate_propagation()`
- [x] **Wheel cancel** — `wheel_event()` returns `bool`; winit skips scroll if prevented
- [x] **Clipboard cancel** — `clipboard_event()` returns `bool`; harness skips arboard if prevented
- [ ] Capture phase

## Form Controls

- [x] Checkbox click/space toggles `checked`, fires `input` + `change`
- [x] Radio click/space checks self + unchecks same-name siblings
- [x] Enter/Space on `<button>` `<a href>` `<input type=submit|reset|button>` synthesizes click
- [x] Enter in form input fires `submit`; click on submit button fires `submit`
- [ ] `<select>` dropdown rendering and interaction

## Drag

- [x] `Node::draggable: bool` flag
- [x] `dragstart` on mousedown + ≥5px, suppresses click
- [x] `dragend` on source, `drop` on release target
- [ ] `drag`, `dragover`, `dragenter`, `dragleave` intermediate events
- [ ] Drag ghost image

---

## Never Emitted (stubs)

- [ ] **`drag` / `dragover` / `dragenter` / `dragleave`** — intermediate drag
- [ ] **`select`** — input/textarea selection range change
- [ ] **`resize`** — surface resize
- [ ] **`pointer*`** (6 events) — pointer unification layer
- [ ] **`touch*`** (3 events) — touch input
- [ ] **`animation*` / `transition*`** — blocked on CSS animations
- [ ] **`composition*`** (3 events) — blocked on IME
- [ ] **`toggle` / `beforetoggle`** — blocked on `<details>`
- [ ] **`formdata`** — no form data construction
- [ ] **`progress`** — blocked on image loading events
- [ ] **`beforeinput`** — not yet dispatched

---

## Remaining Work

| Phase | Task | Priority |
|---|---|---|
| 8 | `select` + `resize` events | Medium |
| 9 | `<select>` dropdown rendering + interaction | Medium |
| 10 | Pointer + touch events | Low |
| 11 | Intermediate drag events | Low |
| 12 | Capture phase | Low |
| — | `animation*` / `transition*` / `composition*` / `progress` | Blocked |

---

## Cross-Cutting

| Concern | File |
|---|---|
| Event structs | `crates/wgpu-html-events/src/` |
| Node slots (27 fields) | `crates/wgpu-html-tree/src/lib.rs` |
| InteractionState | `crates/wgpu-html-tree/src/events.rs` |
| Dispatch | `crates/wgpu-html-tree/src/dispatch.rs` |
| Dispatch tests (40 tests) | `crates/wgpu-html-tree/src/dispatch_tests.rs` |
| El DSL | `crates/wgpu-html-ui/src/el.rs` |
| Ctx helpers | `crates/wgpu-html-ui/src/core/ctx.rs` |
| Winit harness | `crates/wgpu-html-winit/src/` |
| Demo HTML | `crates/wgpu-html-demo/html/events-test.html` |
