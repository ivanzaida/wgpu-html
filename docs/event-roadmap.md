# Event Roadmap

> 31 dedicated Node callback slots, 40 dispatch tests, 10 phases complete.

---

## Fully Dispatched + Dedicated Slots

| # | Event | Node Slot | Type | Bubbles |
|---|---|---|---|---|
| 1 | `mousedown` | `on_mouse_down` | `MouseCallback` | yes |
| 2 | `mouseup` | `on_mouse_up` | `MouseCallback` | yes |
| 3 | `mousemove` | `on_mouse_move` | `MouseCallback` | yes |
| 4 | `mouseenter` | `on_mouse_enter` | `MouseCallback` | no |
| 5 | `mouseleave` | `on_mouse_leave` | `MouseCallback` | no |
| 6 | `click` | `on_click` | `MouseCallback` | yes |
| 7 | `dblclick` | `on_dblclick` | `MouseCallback` | yes |
| 8 | `contextmenu` | `on_contextmenu` | `MouseCallback` | yes |
| 9 | `auxclick` | `on_auxclick` | `MouseCallback` | yes |
| 10 | `dragstart` | `on_dragstart` | `MouseCallback` | no |
| 11 | `drag` | `on_drag` | `MouseCallback` | no |
| 12 | `dragover` | `on_dragover` | `MouseCallback` | no |
| 13 | `dragend` | `on_dragend` | `MouseCallback` | no |
| 14 | `drop` | `on_drop` | `MouseCallback` | no |
| 15 | `keydown` | `on_keydown` | `EventCallback` | yes |
| 16 | `keyup` | `on_keyup` | `EventCallback` | yes |
| 17 | `focus` | `on_focus` | `EventCallback` | no |
| 18 | `blur` | `on_blur` | `EventCallback` | no |
| 19 | `focusin` | `on_focusin` | `EventCallback` | yes |
| 20 | `focusout` | `on_focusout` | `EventCallback` | yes |
| 21 | `beforeinput` | `on_beforeinput` | `EventCallback` | yes |
| 22 | `input` | `on_input` | `EventCallback` | yes |
| 23 | `change` | `on_change` | `EventCallback` | yes |
| 24 | `wheel` | `on_wheel` | `EventCallback` | yes |
| 25 | `scroll` | `on_scroll` | `EventCallback` | no |
| 26 | `select` | `on_select` | `EventCallback` | no |
| 27 | `copy` | `on_copy` | `EventCallback` | yes |
| 28 | `cut` | `on_cut` | `EventCallback` | yes |
| 29 | `paste` | `on_paste` | `EventCallback` | yes |
| 30 | `submit` | — (`on_event`) | — | yes |
| 31 | `selectionchange` | — (`on_event`) | — | no |
| 32 | `resize` | — (`on_event`) | — | no |

All slots use `Vec<_>` for multi-handler (additive, like `addEventListener`).

## Event Control

- [x] `preventDefault` — `HtmlEvent::prevent_default()` via `Cell<bool>`
- [x] `stopPropagation` / `stopImmediatePropagation`
- [x] Wheel cancel — harness skips scroll if prevented
- [x] Clipboard cancel — harness skips arboard if prevented
- [x] `beforeinput` cancel — skips mutation if prevented
- [ ] Capture phase (deferred)

## Form Controls

- [x] Checkbox/radio toggle, fires `input` + `change`
- [x] Enter/Space on buttons/links synthesizes click
- [x] Enter in form input fires `submit`
- [x] `beforeinput` before every text mutation, cancelable
- [ ] `<select>` dropdown (requires layout/paint)

## Drag

- [x] `dragstart` at 5px threshold, `drag` on move, `dragover` on hover
- [x] `dragend` + `drop` on mouseup, `draggable: bool` on Node
- [ ] `dragenter` / `dragleave` (requires persistent drag-target tracking)

## Never Emitted (stubs)

- [ ] `dragenter` / `dragleave` — drag target enter/leave
- [ ] `pointer*` (6 events) — pointer unification
- [ ] `touch*` (3 events) — touch input
- [ ] `animation*` / `transition*` — blocked on CSS
- [ ] `composition*` — blocked on IME
- [ ] `toggle` / `beforetoggle` — blocked on `<details>`
- [ ] `formdata` — no form data construction
- [ ] `progress` — blocked on image loading events

## Demo

```
cargo run -p wgpu-html-demo -- crates/wgpu-html-demo/html/events-test.html
```

Live test zones for all 32 dispatched events with stderr logging.
