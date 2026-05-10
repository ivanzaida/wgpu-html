# lui — Text Input Spec

Text editing for `<input>` and `<textarea>` form controls.
Companion to `spec/interactivity.md` (focus/keyboard foundations),
`spec/text.md` (text rendering pipeline), and `spec/css.md`
(cascade / `:focus` / `::placeholder`).

This file is the source of truth for "what the form-control text
editing surface looks like today and where it's heading".

---

## 0. Status (2026-04-29)

| Feature | State | Notes |
|---|---|---|
| `<input>` value storage (`Input::value`) | ✅ | `lui-models/src/html/input.rs` |
| `<textarea>` value storage (`Textarea::value`) | ✅ | Falls back to RAWTEXT children when `None` |
| `EditCursor` on `InteractionState` | ✅ | Byte offsets into the value string |
| Character insertion via `KeyEvent.text` | ✅ | Layout-aware; respects user's keyboard layout |
| Backspace / Delete | ✅ | Grapheme-boundary aware (char-level) |
| Arrow left / right | ✅ | With Shift for selection extension |
| Home / End | ✅ | Line-aware for multi-line; Shift-extends |
| Arrow up / down (textarea) | ✅ | `\n`-delimited line navigation |
| Enter (textarea newline) | ✅ | Inserts `\n`; ignored for `<input>` |
| Ctrl+A (select all in field) | ✅ | |
| Ctrl+C / Ctrl+X / Ctrl+V | ✅ | Via `arboard` in the winit harness |
| Password masking (`type="password"`) | ✅ | U+2022 BULLET per char in `compute_value_run` |
| Value rendering (`compute_value_run`) | ✅ | Shaped via `shape_text_run`; suppresses placeholder |
| Placeholder suppression on non-empty value | ✅ | Both `<input>` and `<textarea>` paths |
| Caret rendering (blinking) | ✅ | 1.5 px quad, 500 ms on / 500 ms off |
| Edit selection highlight | ✅ | Reuses `paint_selection_background` |
| Click-to-position caret | ✅ | In `interactivity::mouse_down`; glyph-level |
| Focus init / clear of `edit_cursor` | ✅ | `set_focus` in `dispatch.rs` |
| DOM `key` from `event.logical_key` | ✅ | Replaces US-QWERTY `key_to_dom_key` map |
| Ctrl/Meta guard on text insertion | ✅ | Shortcuts don't mutate the value |
| `InputEvent` dispatch via `on_event` | ❌ | Event struct exists but not yet fired |
| `readonly` enforcement | ✅ | Blocks mutations; navigation still works |
| `disabled` enforcement | ✅ | Not focusable → no editing possible |
| `maxlength` enforcement | ❌ | Field exists on model; not checked |
| Word-level delete (Ctrl+Backspace) | ❌ | |
| Word-level movement (Ctrl+Arrow) | ❌ | |
| Double-click word select | ❌ | |
| Triple-click line select | ❌ | |
| Soft-wrap-aware vertical navigation | ❌ | Currently uses `\n` boundaries only |
| Horizontal scroll in overflowing input | ❌ | Glyphs clip at content edge; no scroll |
| Undo / Redo | ❌ | |
| IME / composition (`WindowEvent::Ime`) | ❌ | Spec §2 non-goal for first pass |
| `<input type="number/date/range/...">` | ❌ | Only text-like types supported |

### Input type support

All 22 HTML input types are parsed into `InputType` variants
(`lui-models/src/common/html_enums.rs`). Rendering and
editing support varies:

| `type` | Parsed | Focusable | Editable | Renders as | Notes |
|---|---|---|---|---|---|
| `text` | ✅ | ✅ | ✅ | text field | Default type |
| `password` | ✅ | ✅ | ✅ | bullet-masked field | U+2022 per char |
| `email` | ✅ | ✅ | ✅ | text field | No validation UI |
| `search` | ✅ | ✅ | ✅ | text field | No clear button |
| `tel` | ✅ | ✅ | ✅ | text field | No format enforcement |
| `url` | ✅ | ✅ | ✅ | text field | No validation UI |
| `number` | ✅ | ✅ | ✅ | text field | No spin buttons; accepts any text |
| `hidden` | ✅ | ❌ | ❌ | nothing | UA `display: none` |
| `button` | ✅ | ✅ | ❌ | button-like box | Value shown as label; not editable |
| `submit` | ✅ | ✅ | ❌ | button-like box | No form submission |
| `reset` | ✅ | ✅ | ❌ | button-like box | No form reset |
| `checkbox` | ✅ | ✅ | ❌ | empty box | No toggle; no check mark |
| `radio` | ✅ | ✅ | ❌ | empty box | No toggle; no dot |
| `file` | ✅ | ✅ | ❌ | empty box | No file picker; no "Choose File" label |
| `image` | ✅ | ✅ | ❌ | empty box | No `src` image rendering |
| `color` | ✅ | ✅ | ❌ | empty box | No color swatch or picker |
| `range` | ✅ | ✅ | ❌ | empty box | No slider track or thumb |
| `date` | ✅ | ✅ | ✅ | text field | No date picker; accepts any text |
| `datetime-local` | ✅ | ✅ | ✅ | text field | No datetime picker |
| `month` | ✅ | ✅ | ✅ | text field | No month picker |
| `week` | ✅ | ✅ | ✅ | text field | No week picker |
| `time` | ✅ | ✅ | ✅ | text field | No time picker |

"Editable" means the text editing pipeline (`text_input` /
`handle_edit_key`) accepts keystrokes for that type. Types
marked ❌ in the Editable column are excluded by
`read_editable_value` (dispatch.rs) which skips `Hidden`,
`Checkbox`, and `Radio`. All other types fall through to the
text-field path — they accept typed text but have no type-
specific validation, formatting, or custom UI.

---

## 1. Goals

- Type into `<input>` (single-line) and `<textarea>` (multi-line)
  with the full keyboard: printable characters, backspace, delete,
  arrow keys, home/end, enter (textarea), and clipboard.
- Caret and selection highlight that match the existing document-
  level text selection visuals.
- Password masking for `type="password"`.
- Layout-correct: typed value replaces placeholder, text shapes
  through the same `shape_text_run` pipeline as everything else.
- No global state. Edit cursor lives on `InteractionState`, same
  rule as focus/selection/scroll.

## 2. Non-goals (first pass)

- No IME / composition events. Out of scope until CJK input or
  dead-key combining is needed.
- No `contenteditable`. Form controls only.
- No `<input type="number">` spin buttons, `type="date"` pickers,
  `type="range"` sliders, or `type="color"` swatches.
- No undo/redo stack. Single-level undo could land as a follow-up.
- No drag-to-select inside a form control (drag-select is document-
  level only). Selection is keyboard-driven (Shift+arrows, Ctrl+A).

## 3. Architecture

```
winit KeyEvent
   │
   ├─ logical_key ──► named_key_to_dom() ──► DOM `key` string
   │                                          ▼
   │                                   tree.key_down(key, code, repeat)
   │                                          │
   │                                   handle_edit_key(tree, key)
   │                                          │ navigation / mutation
   │                                          ▼
   │                                   text_edit::*  (pure fns)
   │
   ├─ event.text ──► (guard: !ctrl && !meta)
   │                   ▼
   │            text_input(tree, text)
   │                   │
   │                   ▼
   │            text_edit::insert_text
   │
   └─ physical_key ──► keycode_to_dom_code() ──► DOM `code` string
```

### 3.1 Data flow

1. **Host → tree**: `forward_keyboard` extracts `logical_key` for
   the DOM `key` string (layout-aware, replaces the old US-QWERTY
   physical-key map) and `event.text` for the printable character.
   `key_down` runs first (fires `KeyboardEvent`, then calls
   `handle_edit_key`); then `text_input` inserts the text if Ctrl
   and Meta are not held.

2. **Tree mutation**: `text_input` and `handle_edit_key` read the
   focused element's value via `read_editable_value`, call the
   appropriate `text_edit::*` pure function, and write the result
   back with `write_value`. `edit_cursor` is updated on every
   mutation or navigation. `caret_blink_epoch` is reset so the
   caret stays visible while typing.

3. **Layout**: `compute_value_run` (adjacent to
   `compute_placeholder_run`) shapes the value string with the
   element's cascaded style. For password inputs, each char is
   replaced with U+2022 before shaping. Single-line inputs clip
   and vertically centre; textareas soft-wrap. Both call sites
   (block path + flex/inline-block path) try value first,
   placeholder second.

4. **Paint**: `paint_box_in_clip` checks `EditCaretInfo` (built
   from `InteractionState` in `paint_tree_returning_layout_profiled`).
   When the current path matches `focus_path`:
   - If `selection_bytes` is set, `paint_selection_background` draws
     the highlight and glyph colours are overridden within the range.
   - If `caret_visible` (blink phase), a 1.5 px quad is drawn at
     the cursor's glyph position.

### 3.2 Crate ownership

| Concern | Crate | File |
|---|---|---|
| `EditCursor` struct | `lui-tree` | `events.rs` |
| Pure edit operations | `lui-tree` | `text_edit.rs` |
| Dispatch + value mutation | `lui-tree` | `dispatch.rs` |
| Value / placeholder layout | `lui-layout` | `lib.rs` |
| Caret + selection paint | `lui` | `paint.rs` |
| `EditCaretInfo` construction | `lui` | `lib.rs` |
| Click-to-position caret | `lui` | `interactivity.rs` |
| `KeyEvent.text` → `text_input` | `lui-winit` | `lib.rs` |
| Clipboard shortcuts | `lui-winit` | `window.rs` |
| `Input` model | `lui-models` | `html/input.rs` |
| `Textarea` model | `lui-models` | `html/textarea.rs` |

## 4. `EditCursor`

```rust
pub struct EditCursor {
    pub cursor: usize,                    // byte offset in value
    pub selection_anchor: Option<usize>,  // None = collapsed caret
}
```

Lives on `InteractionState.edit_cursor: Option<EditCursor>`.
`None` when focus is not on a text-editable control. Initialized
to `collapsed(value.len())` by `set_focus`; cleared on blur.

Byte offsets always sit on a `char` boundary. All `text_edit::*`
functions enforce this invariant.

`caret_blink_epoch: Instant` on `InteractionState` records the
last cursor movement or value mutation. Paint uses it to derive
the blink phase: visible when `elapsed % 1000 < 500`.

## 5. Pure edit operations (`text_edit.rs`)

Signature pattern:
```rust
fn op(value: &str, cursor: &EditCursor, ...) -> (String, EditCursor)
// or for navigation-only:
fn op(value: &str, cursor: &EditCursor, extend_selection: bool) -> EditCursor
```

No tree access, no side effects. Testable in isolation.

| Function | Trigger | Mutates value |
|---|---|---|
| `insert_text` | printable chars | yes |
| `insert_line_break` | Enter (textarea) | yes |
| `delete_backward` | Backspace | yes |
| `delete_forward` | Delete | yes |
| `delete_selection` | internal helper | yes |
| `move_left` | ArrowLeft | no |
| `move_right` | ArrowRight | no |
| `move_home` | Home | no |
| `move_end` | End | no |
| `move_up` | ArrowUp (textarea) | no |
| `move_down` | ArrowDown (textarea) | no |
| `select_all` | Ctrl+A | no |

All navigation functions take `extend_selection: bool` (Shift held).
When true, `selection_anchor` is set to the old cursor position (or
preserved if already set); when false, the selection collapses.

Multi-line navigation (`move_up`/`move_down`) scans for `\n`
characters. This handles hard line breaks correctly but does not
account for soft wraps — a long line that wraps visually is treated
as one line. Soft-wrap-aware navigation requires layout info and
is deferred.

## 6. Value rendering

`compute_value_run` in `lui-layout/src/lib.rs`:

1. Reads `input.value` or `textarea.value` (falls back to RAWTEXT
   children for textarea).
2. For `type="password"`, replaces each char with U+2022.
3. Calls `shape_text_run(text, style, max_width, false, ctx)`.
4. Single-line inputs: horizontal clip to `content_rect.w` +
   vertical centering (same as placeholder path).
5. Textareas: soft-wrap via `max_width = Some(content_rect.w)`.
6. Color: cascaded `color` at full opacity (not the 50% alpha
   used by `compute_placeholder_run`).

Both layout paths (block and inline-block/flex) call
`compute_value_run` first; if it returns `None` (empty value),
they fall through to `compute_placeholder_run`.

## 7. Caret and selection rendering

### 7.1 Caret

`EditCaretInfo` is built from `InteractionState` at paint time:

```rust
pub struct EditCaretInfo<'a> {
    pub focus_path: &'a [usize],
    pub cursor_byte: usize,
    pub selection_bytes: Option<(usize, usize)>,
    pub caret_visible: bool,
}
```

`byte_offset_to_glyph_index` converts the cursor byte offset
to a glyph index using `ShapedRun::byte_boundaries`. The caret
quad is placed at `glyphs[idx-1].x + glyphs[idx-1].w` (or `0.0`
for index 0). Line height comes from the matching
`ShapedLine.height`.

Continuous redraw: `window.rs` calls `window.request_redraw()`
after every frame while `edit_cursor.is_some()` so the blink
cycles without user input.

### 7.2 Edit selection highlight

When `selection_bytes` is `Some((start, end))`, the existing
`paint_selection_background` function is reused with the glyph
range converted from byte offsets. Glyph foreground colour is
overridden within the selection range (same as document-level
selection).

## 8. Click-to-position

`interactivity::mouse_down` (`lui/src/interactivity.rs`):
after `dispatch_mouse_down` sets focus on a form control, the
layout tree is walked to the focused element's `LayoutBox`. The
click x-position is compared against each glyph's midpoint to
find the closest inter-glyph boundary. The glyph index is
converted to a byte offset via `byte_boundaries` and written to
`edit_cursor.cursor`.

## 9. Clipboard

Handled in the winit harness (`window.rs`), gated on
`edit_cursor.is_some()`:

| Shortcut | Action |
|---|---|
| Ctrl+C | Copy `value[start..end]` to clipboard (no-op if collapsed) |
| Ctrl+X | Copy + `text_input(tree, "")` to delete selection |
| Ctrl+V | Read clipboard → `text_input(tree, &pasted)` |
| Ctrl+A | Handled by `handle_edit_key` → `text_edit::select_all` |

When `edit_cursor` is `None`, the existing document-level
shortcuts (`run_select_all`, `run_copy_selection`) take over.

## 10. Keyboard layout support

`forward_keyboard` derives the DOM `key` string from
`event.logical_key` (winit 0.30's `Key` enum) via
`named_key_to_dom` for named keys and `Key::Character(ch)` for
printable characters. This replaces the old `key_to_dom_key`
function which hardcoded a US-QWERTY physical-key map.

Text insertion comes from `event.text` (`Option<SmolStr>`), which
is the OS input method's composed output — correct for any
keyboard layout. Insertion is suppressed when Ctrl or Meta is held
(those key combinations are shortcuts, not text).

The old `key_to_dom_key(KeyCode, bool) -> &str` remains available
for callers that don't have a `KeyEvent`, but it is no longer used
by the main event path.

## 11. Open questions

- **Soft-wrap cursor navigation.** ArrowUp/Down currently scan for
  `\n` boundaries. A textarea with long wrapping lines won't
  navigate visually. Fix: store line-break byte offsets from the
  most recent layout pass in `InteractionState` and use them for
  vertical movement.
- **Horizontal scroll for overflowing `<input>`.** Currently the
  value text is clipped at the content edge. Browsers scroll the
  text so the caret is always visible. This needs a per-element
  horizontal scroll offset, similar to the existing vertical
  `scroll_offsets_y`.
- **`maxlength` enforcement.** The `maxlength` attribute is parsed
  into the model but not checked during insertion.
- **`InputEvent` dispatch.** The event struct
  (`lui_events::InputEvent`) exists with `data`,
  `input_type`, and `is_composing` fields. Firing it after each
  mutation is straightforward but not yet wired.
- **Undo / redo.** A simple stack of `(old_value, old_cursor)` on
  `InteractionState` would give single-step undo. Grouping rapid
  keystrokes into one undo unit (browser behaviour) is harder.
- **Word-level operations.** Ctrl+Backspace (delete word),
  Ctrl+Arrow (jump word) need a Unicode word-boundary scanner.
  `unicode-segmentation` is the standard crate; adding it as a
  dependency is the main cost.
