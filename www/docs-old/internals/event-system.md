---
title: Event System
---

# Event System & Interactivity

How pointer events, keyboard input, focus, and scroll are handled.

## InteractionState

**File:** `crates/lui-tree/src/events.rs` line 166

Central state machine tracking all interaction:

| Field | Type | Purpose |
|---|---|---|
| `hover_path` | `Option<Vec<usize>>` | Path to deepest hovered element |
| `active_path` | `Option<Vec<usize>>` | Path to element with pressed mouse button |
| `focus_path` | `Option<Vec<usize>>` | Path to keyboard-focused element |
| `edit_cursor` | `Option<EditCursor>` | Caret position + selection in text input |
| `selection` | `Option<TextSelection>` | Global text selection across boxes |
| `selecting_text` | `bool` | True during mouse-drag text selection |
| `scroll_offsets` | `BTreeMap<Vec<usize>, ScrollOffset>` | Per-element scroll amounts |
| `pointer_pos` | `Option<(f32, f32)>` | Last known pointer position |
| `buttons_down` | `u16` | Bitmask of held buttons (DOM format) |
| `modifiers` | `Modifiers` | Current shift/ctrl/alt/meta state |
| `drag_pending` | `Option<(Vec<usize>, (f32, f32))>` | Potential drag source |
| `drag_active_source` | `Option<Vec<usize>>` | Active drag source |
| `drag_target_path` | `Option<Vec<usize>>` | Current drag target |
| `last_click_time` | `Instant` | For dblclick detection (300ms window) |
| `focus_value_snapshot` | `Option<String>` | Value at focus time, compared on blur for `change` event |
| `caret_blink_epoch` | `Instant` | Last cursor movement, for 500ms blink |

## Hit Testing

**File:** `crates/lui-layout-old/src/lib.rs`

| Method | Line | Purpose |
|---|---|---|
| `hit_path()` | 607 | Deepest `LayoutBox` containing point |
| `hit_path_scrolled()` | 619 | Same but adjusts for per-element scroll offsets |
| `hit_text_cursor()` | 702 | Text cursor (path + glyph index) under pointer |
| `hit_text_cursor_scrolled()` | 644 | Scroll-aware text cursor |
| `hit_glyph_boundary()` | 742 | Glyph index by X position within text run |

Hit testing is depth-first, last-child-first (topmost on overlap). Layout paths map 1:1 to tree element paths.

## Higher-Level Wrappers

**File:** `crates/lui/src/interactivity.rs`

Layout-aware wrappers combining hit-test + dispatch:

| Function | Line | Purpose |
|---|---|---|
| `pointer_move()` | 37 | Hit-test + `dispatch_pointer_move()` |
| `pointer_move_with_cursor()` | 46 | Same, also returns CSS cursor for OS icon |
| `mouse_down()` | 62 | Hit-test + `dispatch_mouse_down()` + edit cursor |
| `mouse_up()` | 213 | Hit-test + `dispatch_mouse_up()` |

Text selection helpers:
- `word_byte_range()` (line 159) -- word boundaries for double-click
- `line_byte_range()` (line 186) -- line boundaries for triple-click
- `edit_cursor_for_click_count()` (line 141) -- set selection anchor by click count

## Event Dispatch

**File:** `crates/lui-tree/src/dispatch.rs`

### Pointer Move (line 508)

`dispatch_pointer_move()`:
1. Update `pointer_pos`
2. If `selecting_text`: extend selection via text cursor
3. **Hover path diff** (line 529): compare old vs new hover path
   - Fire `mouseleave` on nodes leaving hover (deepest-first)
   - Fire `mouseenter` on nodes entering hover (root-first)
   - Matches W3C mouseenter/mouseleave semantics (no-bubble, ancestor ordering)
4. Fire `mousemove` on current target
5. **Drag-start** (line 539): if drag_pending and moved >= 5px, fire `dragstart`
6. **Drag/dragover** (line 552): while dragging, fire `drag`/`dragover`, track target changes for `dragenter`/`dragleave`

Hover chain management via `update_hover()` (line 309): common-prefix algorithm finds divergence point, walks out of old path then into new path.

### Mouse Down (line 699)

`dispatch_mouse_down()`:
1. Set `active_path` = target
2. Update `buttons_down` bitmask
3. **Focus management** (line 764): find deepest focusable ancestor via `focusable_ancestor()` (line 869), call `set_focus()`
4. Fire `mousedown` (capture -> target -> bubble)
5. Set `drag_pending` if element has `draggable=true`
6. Position edit cursor in focused text input (glyph index -> byte offset)
7. Fire `dblclick` if within 300ms of last click on same element

### Mouse Up (line 800)

`dispatch_mouse_up()`:
1. Clear `active_path`
2. If dragging: clear drag state, fire `dragend`
3. Fire `mouseup` (capture -> target -> bubble)
4. **Click synthesis** (line 829): if primary button and target shares root with press target, fire `click`
5. **Form submission** (line 852): check if target is submit button
6. **Checkbox/radio toggle** (line 857): toggle `checked` state

### Event Bubbling (line 244)

`bubble()` implements W3C event propagation:
- **Capture phase**: root -> target parent
- **Target phase**: exact target
- **Bubble phase**: target parent -> root (only for bubbling events)

`fire_mouse_slot()` (line 171) executes callbacks at a single node. Reads `node.on_mouse_down`, `node.on_click`, `node.on_event` (generic handler). Propagation stops if callback calls `stopPropagation()`.

## Focus Management

**File:** `crates/lui-tree/src/dispatch.rs`

### set_focus() (line 895)

Central focus update:

1. **Blur old** (if changed):
   - Fire `blur` (non-bubbling) on old element
   - Fire `focusout` (bubbling) on old element and ancestors
   - If value changed since focus, fire `change`
2. **Init edit cursor** for new focus:
   - `<input>` text types: cursor at end of value
   - `<textarea>`: cursor at end of content
   - Other types (checkbox, button): `None`
3. **Snapshot value** for change detection on blur
4. **Focus new** (if present):
   - Fire `focus` (non-bubbling) on new element
   - Fire `focusin` (bubbling) on new element and ancestors

### Focus Navigation

| Function | Line | Purpose |
|---|---|---|
| `focus()` | 1151 | Move focus to path (or its focusable ancestor) |
| `blur()` | 1161 | Clear focus |
| `focus_next()` | 1168 | Tab / Shift+Tab navigation |

`focus_next()` collects all keyboard-focusable elements in document order via `keyboard_focusable_paths()`, then advances to next/previous.

`focusable_ancestor()` (line 869) walks from target toward root, returns first focusable element (button, text input, textarea, a[href], etc.).

## Keyboard Events

`key_down()` / `key_up()`: fire keyboard events on focused element with capture -> target -> bubble.

Special key handling:
- **Tab**: calls `focus_next()` to move focus
- **Enter**: fires submit on form or click on button
- **ArrowUp/Down**: for select elements, cycle options

## Scroll

**File:** `crates/lui/src/scroll.rs`

| Function | Line | Purpose |
|---|---|---|
| `scrollbar_geometry()` | 94 | Compute track + thumb rects |
| `scroll_y_from_thumb_top()` | 129 | Inverse: thumb position -> scroll offset |
| `document_bottom()` | 72 | Deepest content bottom (stops at overflow-clipped) |
| `max_scroll_y()` | 57 | Maximum scroll before hitting content end |

Scroll offsets stored in `InteractionState.scroll_offsets` per element path. Updated by wheel event dispatch (clamped to valid range). Read by hit-testing to adjust pointer coordinates.

## Re-Cascade Trigger

**File:** `crates/lui/src/lib.rs` lines 350-365

After any dispatch:
1. Capture old interaction snapshot before dispatch
2. After dispatch, check if `hover_path`, `active_path`, or `focus_path` changed
3. If changed, call `cascade_incremental_with_media()` -- re-cascade only affected elements
4. If stylesheet has no pseudo-class rules, cascade is skipped entirely
