---
title: Form Controls
---

# Form Controls

Form control editing is implemented in `lui-tree::text_edit` (252 lines) and driven by keyboard input dispatch.

## Text Editing

The `text_edit` module provides pure functions that take current value + cursor and return new value + cursor:

| Function | Description |
|---|---|
| `insert_text(value, cursor, text)` | Insert characters at cursor position |
| `delete_forward(value, cursor)` | Delete character after cursor |
| `delete_backward(value, cursor)` | Delete character before cursor |
| `move_cursor_left(value, cursor, selection_anchor)` | Left arrow (with Shift-select) |
| `move_cursor_right(value, cursor, selection_anchor)` | Right arrow |
| `move_cursor_home(value, cursor, selection_anchor)` | Home key |
| `move_cursor_end(value, cursor, selection_anchor)` | End key |
| `insert_line_break(value, cursor)` | Enter in textareas |
| `select_all(value)` | Ctrl+A equivalent |
| `delete_selection(value, cursor, selection_anchor)` | Delete selected range |
| `delete_word_backward(value, cursor)` | Ctrl+Backspace |
| `delete_word_forward(value, cursor)` | Ctrl+Delete |

All operations are **multibyte/UTF-8 safe** — `prev_char()` and `next_char()` navigate by `char` boundaries using `str::is_char_boundary()`.

### Shift+Select

When Shift is held, arrow keys extend the selection rather than moving a collapsed caret:

```rust
pub fn move_cursor_right(
    value: &str, cursor: usize, selection_anchor: Option<usize>
) -> (usize, Option<usize>)
```

If `selection_anchor` is `None`, it's set to the current cursor position before moving.

## Placeholder Rendering

When an `<input>` or `<textarea>` has a `placeholder` attribute but no value, the placeholder text is shaped and painted:

- Shaped at 50% alpha of the normal text color.
- Centered vertically in `<input>` elements.
- Word-wrapped in `<textarea>` elements.
- Excluded from document-level text selection (`text_unselectable: true`).

## Password Masking

`<input type="password">` values are displayed as U+2022 (bullet) characters:

```rust
let display_value = if is_password {
    "\u{2022}".repeat(value.chars().count())
} else {
    value.to_string()
};
```

The shaped display value is used for render only; the underlying `value` attribute stores the real text.

## Blinking Caret

A 1.5px-wide vertical quad blinks on/off every 500ms:

```rust
let elapsed = Instant::now().duration_since(tree.interaction.caret_blink_epoch);
let visible = (elapsed.as_millis() / 500) % 2 == 0;
```

The caret epoch resets on every user interaction (click, keypress) so the caret is always visible immediately after input.

## Click-to-Position Caret

When clicking on an empty input (showing placeholder), the caret goes to position 0, not inside the placeholder text. For non-empty fields, glyph-level accuracy is used:

```rust
let glyph_idx = run.glyphs.iter()
    .position(|g| g.x + g.w * 0.5 > click_x)
    .unwrap_or(run.glyphs.len());
let byte_offset = run.byte_boundaries[glyph_idx];
```

## Supported Input Types

| Type | Behavior |
|---|---|
| `text` | Standard text input |
| `password` | Bullet-masked text input |
| `email` | Text input (validation not enforced) |
| `search` | Text input |
| `tel` | Text input |
| `url` | Text input |
| `number` | Text input with ArrowUp/ArrowDown stepping (`min`/`max`/`step` enforced) |
| `date`, `datetime-local`, `month`, `week`, `time` | Text input (no picker UI) |
| `checkbox` | 13x13 toggle with checkmark; click/Space/Enter toggles; fires `input`/`change` |
| `radio` | 13x13 circle with dot; mutual exclusion within `name` group |
| `range` | Slider with track + thumb; mouse drag + ArrowUp/ArrowDown; respects `min`/`max`/`step` |
| `color` | Displays parsed hex value as filled swatch (default `#000000`) |
| `button`, `submit`, `reset` | Rendered as buttons, clickable |
| Other types | Fall back to `text` behavior |

## Known Gaps

- **No `<select>` dropdown**: The element renders but has no popup menu.
- **No form submission**: Submit buttons trigger `on_click` but don't send data.
- **No input validation** (pattern, required, min/max for text types): These attributes are parsed but not enforced.
- **No undo/redo** (Ctrl+Z / Ctrl+Y): No undo stack.
