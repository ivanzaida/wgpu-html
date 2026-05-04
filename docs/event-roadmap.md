# Event Roadmap

> 33 dedicated Node callback slots, 40 dispatch tests, 11 phases complete.

---

## Fully Dispatched + Dedicated Slots

| # | Event | Node Slot | Type | Bubbles | Notes |
|---|---|---|---|---|---|
| 1 | `mousedown` | `on_mouse_down` | `MouseCallback` | yes | primary-button press |
| 2 | `mouseup` | `on_mouse_up` | `MouseCallback` | yes | primary-button release |
| 3 | `mousemove` | `on_mouse_move` | `MouseCallback` | yes | every pointer move |
| 4 | `mouseenter` | `on_mouse_enter` | `MouseCallback` | no | root-first chain |
| 5 | `mouseleave` | `on_mouse_leave` | `MouseCallback` | no | deepest-first chain |
| 6 | `click` | `on_click` | `MouseCallback` | yes | press+release same target |
| 7 | `dblclick` | `on_dblclick` | `MouseCallback` | yes | two clicks ≤300ms same element |
| 8 | `contextmenu` | `on_contextmenu` | `MouseCallback` | yes | secondary-button release |
| 9 | `auxclick` | `on_auxclick` | `MouseCallback` | yes | middle-button release |
| 10 | `dragstart` | `on_dragstart` | `MouseCallback` | no | mousedown + ≥5px on `Node::draggable` |
| 11 | `drag` | `on_drag` | `MouseCallback` | no | source element, each pointer move |
| 12 | `dragover` | `on_dragover` | `MouseCallback` | no | element under pointer while dragging |
| 13 | `dragenter` | `on_dragenter` | `MouseCallback` | no | root-first enter chain during drag |
| 14 | `dragleave` | `on_dragleave` | `MouseCallback` | no | deepest-first leave chain during drag |
| 15 | `dragend` | `on_dragend` | `MouseCallback` | no | source element on mouseup |
| 16 | `drop` | `on_drop` | `MouseCallback` | no | release target on mouseup after drag |
| 17 | `keydown` | `on_keydown` | `EventCallback` | yes | Tab/Shift+Tab built in |
| 18 | `keyup` | `on_keyup` | `EventCallback` | yes | |
| 19 | `focus` | `on_focus` | `EventCallback` | no | target only |
| 20 | `blur` | `on_blur` | `EventCallback` | no | target only |
| 21 | `focusin` | `on_focusin` | `EventCallback` | yes | |
| 22 | `focusout` | `on_focusout` | `EventCallback` | yes | |
| 23 | `beforeinput` | `on_beforeinput` | `EventCallback` | yes | cancelable — skips mutation if prevented |
| 24 | `input` | `on_input` | `EventCallback` | yes | text inserts, backspace, delete, enter |
| 25 | `change` | `on_change` | `EventCallback` | yes | blur-value-differs, checkbox/radio toggle |
| 26 | `wheel` | `on_wheel` | `EventCallback` | yes | cancelable — skips scroll if prevented |
| 27 | `scroll` | `on_scroll` | `EventCallback` | no | overflow container offset change |
| 28 | `select` | `on_select` | `EventCallback` | no | input/textarea cursor selection change |
| 29 | `copy` | `on_copy` | `EventCallback` | yes | cancelable — skips arboard if prevented |
| 30 | `cut` | `on_cut` | `EventCallback` | yes | cancelable — skips arboard if prevented |
| 31 | `paste` | `on_paste` | `EventCallback` | yes | cancelable — skips arboard if prevented |

### Via `on_event` only (no dedicated slot)

| Event | Trigger |
|---|---|
| `submit` | Enter in form input, submit button click |
| `selectionchange` | text selection start/extend/finalize on root |
| `resize` | surface resize on root |

All slots use `Vec<_>` for additive multi-handler (like `addEventListener`).

## Event Control

- [x] `preventDefault` — `ev.prevent_default()` via `Cell<bool>`
- [x] `stopPropagation` / `stopImmediatePropagation`
- [x] Wheel cancel — harness skips scroll if prevented
- [x] Clipboard cancel — harness skips arboard if prevented
- [x] `beforeinput` cancel — skips mutation if prevented
- [ ] Capture phase (deferred)

## Form Controls

- [x] Checkbox/radio click toggle, fires `input` + `change`
- [x] Enter/Space on buttons/links/checkboxes synthesizes click
- [x] Enter in form input fires `submit`
- [x] `beforeinput` before every text mutation, cancelable
- [ ] `<select>` dropdown (requires layout/paint rendering)

## Drag & Drop

- [x] `Node::draggable: bool` flag
- [x] `dragstart` at 5px threshold, suppresses click
- [x] `drag` on source during move
- [x] `dragover` on hovered element during drag
- [x] `dragenter` / `dragleave` on drag target changes
- [x] `dragend` + `drop` on mouseup
- [ ] Drag ghost image / visual feedback

## Known Bug

`position: fixed` elements don't stick to the viewport during scroll.
`translate_display_list_y` shifts all quads/glyphs by `-scroll_y` uniformly.
Fixed-element drawables need to be skipped during this translation.
Fix requires threading `viewport_scroll_y` through `paint_tree_cached` →
`paint_box_in_clip` and skipping viewport scroll for fixed subtrees.

## Never Emitted (blocked)

| Event | Blocked on |
|---|---|
| `pointer*` (6 events) | pointer unification layer |
| `touch*` (3 events) | touch input routing from winit |
| `animation*` / `transition*` | CSS animations/transitions |
| `composition*` (3 events) | IME integration |
| `toggle` / `beforetoggle` | `<details>` DOM model |
| `formdata` | form data construction |
| `progress` | image load progress events |

## Demo

```
cargo run -p wgpu-html-demo -- crates/wgpu-html-demo/html/events-test.html
```
