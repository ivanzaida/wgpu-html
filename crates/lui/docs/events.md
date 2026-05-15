# Event implementation status

Tracks which DOM events are wired from platform input through to dispatch,
and which CSS interactive pseudo-classes are fed into the cascade.

## Legend

- **done** — dispatched to DOM listeners and/or fed into cascade
- **partial** — mechanics work but DOM event not fired to listeners
- **todo** — not implemented
- **n/a** — needs platform APIs this engine doesn't have

---

## Mouse events

| Event | Status | Notes |
|-------|--------|-------|
| `click` | done | fires on button release via `handle_mouse_release` |
| `mousedown` | done | fires on button press via `handle_mouse_down` |
| `mouseup` | done | fires on button release via `handle_mouse_release` |
| `mousemove` | done | fires once per `render_frame` when cursor moved; bubbles |
| `mouseenter` | done | fires on each ancestor being entered (shallowest first); no bubble |
| `mouseleave` | done | fires on each ancestor being left (deepest first); no bubble |
| `mouseover` | done | fires on new target; bubbles |
| `mouseout` | done | fires on old target; bubbles |
| `dblclick` | done | fires after 2nd click within 500ms / 5px; triple-click produces exactly one |
| `contextmenu` | done | fires on right-click (button=2) release; no `click` for button!=0 |

## Pointer events

| Event | Status | Notes |
|-------|--------|-------|
| `pointerdown` | done | fires before `mousedown`; pointer_type="mouse", pointer_id=1, is_primary=true |
| `pointerup` | done | fires before `mouseup` |
| `pointermove` | done | fires before `mousemove` in hover transitions |
| `pointerenter` | done | fires before `mouseenter`; no bubble |
| `pointerleave` | done | fires before `mouseleave`; no bubble |
| `pointerover` | done | fires before `mouseover`; bubbles |
| `pointerout` | done | fires before `mouseout`; bubbles |
| `pointercancel` | todo | needs touch/pen cancel handling |
| `gotpointercapture` | todo | needs pointer capture API |
| `lostpointercapture` | todo | needs pointer capture API |

## Wheel / scroll

| Event | Status | Notes |
|-------|--------|-------|
| `wheel` | done | dispatched before scroll mechanics; `preventDefault()` cancels scrolling |
| `scroll` | todo | should fire on elements whose scroll position changed |
| `scrollend` | todo | |

## Keyboard events

| Event | Status | Notes |
|-------|--------|-------|
| `keydown` | done | dispatched to focused element; bubbles; wired from winit `KeyboardInput` |
| `keyup` | done | dispatched to focused element; bubbles; wired from winit |
| `keypress` | todo | deprecated but still widely used |

## Focus events

| Event | Status | Notes |
|-------|--------|-------|
| `focus` | done | fires on click; `set_focus` walks up to nearest focusable element |
| `blur` | done | fires when focus moves away |
| `focusin` | done | like focus but bubbles |
| `focusout` | done | like blur but bubbles |

## Input / form events

| Event | Status | Notes |
|-------|--------|-------|
| `input` | todo | no form element support |
| `change` | todo | |
| `submit` | todo | |
| `reset` | todo | |
| `invalid` | todo | |
| `select` | todo | text selection |

## Drag events

| Event | Status | Notes |
|-------|--------|-------|
| `drag` | todo | |
| `dragstart` | todo | |
| `dragend` | todo | |
| `dragenter` | todo | |
| `dragleave` | todo | |
| `dragover` | todo | |
| `drop` | todo | |

## Touch events

| Event | Status | Notes |
|-------|--------|-------|
| `touchstart` | todo | winit provides touch events — not handled |
| `touchmove` | todo | |
| `touchend` | todo | |
| `touchcancel` | todo | |

## CSS pseudo-classes

| Pseudo | Status | Notes |
|--------|--------|-------|
| `:hover` | done | `hover_path` set via hit-test after each layout pass; one-frame lag |
| `:active` | done | `active_path` set on mousedown, cleared on mouseup; exact-match only (spec says ancestors should match too) |
| `:focus` | done | `focus_path` fed into `InteractionState`; set on click via `set_focus` |
| `:focus-within` | done | cascade matches when `focus_path` is inside element's subtree |
| `:focus-visible` | todo | cascade matching works, needs input-modality heuristic |
| `:target` | todo | `InteractionState.target_path` exists, needs URL fragment tracking |

## Clipboard events

| Event | Status | Notes |
|-------|--------|-------|
| `copy` | todo | |
| `cut` | todo | |
| `paste` | todo | |

## Animation / transition events

| Event | Status | Notes |
|-------|--------|-------|
| `animationstart` | n/a | no CSS animation runtime |
| `animationend` | n/a | |
| `animationiteration` | n/a | |
| `transitionend` | n/a | |
| `transitionstart` | n/a | |
| `transitionrun` | n/a | |
| `transitioncancel` | n/a | |

## Remaining event types (n/a)

The following are defined as types in `lui-core/src/events/` but require
platform APIs or browser features this engine does not implement:

- Web Audio / Speech (`AudioProcessing`, `SpeechRecognition`, etc.)
- WebRTC (`RTCDataChannel`, `RTCTrack`, etc.)
- WebXR (`XRInputSource`, `XRSession`, etc.)
- Gamepad (`GamepadEvent`)
- MIDI (`MIDIConnection`, `MIDIMessage`)
- Service Worker (`Fetch`, `Install`, `Sync`, `Push`, etc.)
- Payment / Presentation / Bluetooth / HID / USB / NFC
- MediaStream / MediaEncrypted / PictureInPicture
- Navigation / PageTransition / BeforeUnload
- WebGL / GPU context events
- Sensors / Device orientation

---

## Dispatch infrastructure

| Feature | Status |
|---------|--------|
| Listener attach/remove (`add_event_listener`) | done |
| Listener options (capture, once, passive) | done |
| Phase-aware dispatch (capture/target/bubble) | done |
| `stopPropagation()` | done |
| `stopImmediatePropagation()` | done |
| `preventDefault()` | done |
| Passive listener `preventDefault` guard | done |
| DOM path resolution (`find_node_path`) | done |
| Hit-testing (layout tree) | done |
| `pointer-events: none` in hit-testing | done |
| `cursor` CSS property → winit window cursor | done |
| Hover dirty flag (skip when cursor static) | done |
| Batched mouse release (mouseup+click, 1 layout pass) | done |

## Scroll infrastructure

| Feature | Status |
|---------|--------|
| Element scroll (`overflow: scroll/auto`) | done |
| Scroll chaining (nested containers) | done |
| Viewport scroll fallback | done |
| Scroll position persistence across layouts | done |
| Scrollbar painting (element + viewport) | done |
| `scrollbar-width` / `scrollbar-color` | done |
| `scrollbar-gutter: stable` / `both-edges` | done |
| Scrollbar click/drag interaction | todo |
