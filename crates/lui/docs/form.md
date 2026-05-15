# Form implementation status

Tracks interactive form element support — value state, user input, rendering, and events.

## Legend

- **done** — fully wired and tested
- **partial** — core logic exists, rendering or edge cases missing
- **todo** — not implemented
- **n/a** — blocked on missing subsystem

---

## Phase 1: Text Input

### Core infrastructure

| Feature | Status | Notes |
|---------|--------|-------|
| `EditCursor` type (byte offset + selection anchor) | done | `lui-core/text_selection.rs` |
| `FormControlState` side-table on `Lui` | done | keyed by DOM path, same pattern as `element_scroll` |
| Pure text editing functions (16 ops) | done | `lui-core/text_edit.rs`, 21 unit tests |
| `handle_form_key()` keyboard → mutation pipeline | done | maps Backspace/Delete/arrows/Home/End/Ctrl+A |
| `handle_text_input()` character insertion | done | called from `handle_form_key` for printable chars |
| `ensure_form_state()` lazy init from value attr | done | triggers on focus |
| `form_value(path)` public getter | done | returns current runtime value |
| `is_text_editable(path)` check | done | text/password/email/search/tel/url/number + textarea |

### Events

| Feature | Status | Notes |
|---------|--------|-------|
| `input` event on text mutation | done | fires after insert/delete with `input_type` and `data` |
| `change` event on blur | done | fires only if value differs from focus snapshot |
| `beforeinput` event (cancellable) | done | fires before each mutation; preventDefault cancels insert/delete |

### Rendering

| Feature | Status | Notes |
|---------|--------|-------|
| Value text shaping and painting | done | `paint/form.rs` — shapes value, emits glyphs with scroll offset |
| Caret rendering (blinking cursor line) | done | 1.5px quad at cursor byte-offset x-pos, 500ms blink via `caret_blink_epoch` |
| In-field text selection highlight | done | blue highlight quad between selection anchor and cursor |
| Placeholder text (when value empty) | done | half-opacity text from `placeholder` attr |
| Password bullet masking | done | replaces chars with `•` during paint |
| Horizontal scroll for overflow text | partial | `FormControlState.scroll_x` threaded to paint, not yet auto-scrolled on cursor movement |
| Click-to-position caret in input | todo | hit_form_cursor — shape value, hit-test byte offset |

### Input types

| Type | Status | Notes |
|------|--------|-------|
| `text` | partial | editing works, rendering todo |
| `password` | partial | editing works, bullet masking todo |
| `email` | partial | editing works, no validation |
| `search` | partial | editing works |
| `tel` | partial | editing works |
| `url` | partial | editing works, no validation |
| `number` | partial | editing works, no step/min/max enforcement |
| `textarea` | partial | editing works incl. multiline (Enter, up/down), rendering todo |

### Winit integration

| Feature | Status | Notes |
|---------|--------|-------|
| `KeyboardInput` → `handle_key_down` | done | wired in winit_driver.rs |
| IME `Ime::Commit` → `handle_text_input` | todo | need to handle winit IME events |

---

## Phase 2: Checkbox, Radio, Button, Form Submission

### Toggle controls

| Feature | Status | Notes |
|---------|--------|-------|
| Checkbox toggle on click | todo | toggle `checked` attr, fire input+change |
| Radio mutual exclusion | todo | uncheck same-name siblings in form |
| Checkbox/radio paint (checkmark, dot) | todo | paint/form.rs |
| `:checked` pseudo-class | done | cascade matches via `attrs.contains_key("checked")` |

### Buttons

| Feature | Status | Notes |
|---------|--------|-------|
| `<button>` click event | done | existing click dispatch |
| `<button type="submit">` triggers form submit | todo | |
| `<button type="reset">` resets form | todo | |
| Space/Enter keyboard activation | todo | synthesize click on focused button |

### Form submission

| Feature | Status | Notes |
|---------|--------|-------|
| `FormData::collect()` — walk form, collect name/value | todo | `lui-core/form_data.rs` |
| `submit` event (cancellable) | todo | fire before collection |
| `formdata` event | todo | fire after collection |
| Enter in text input submits enclosing form | todo | |
| Form reset (restore default values) | todo | |

---

## Phase 3: Select / Option

| Feature | Status | Notes |
|---------|--------|-------|
| `SelectState` (selected_index, open, highlighted) | todo | |
| Click to open/close dropdown | todo | |
| Option click selects and closes | todo | |
| Arrow key navigation in open dropdown | todo | |
| Display selected option text when closed | todo | |
| Dropdown overlay painting | todo | paint/select.rs |
| `:checked` on selected option | todo | |
| Multiple select | todo | |

---

## Phase 4: Specialized Inputs

| Type | Status | Notes |
|------|--------|-------|
| `number` — step up/down, spinner buttons | todo | |
| `range` — track + thumb, drag interaction | todo | reuse scrollbar drag pattern |
| `color` — swatch + HSV picker overlay | todo | port from v1 |
| `date` — segment editing + calendar | todo | port from v1 |
| `datetime-local` — date + time segments | todo | port from v1 |
| `file` — file selector button | todo | needs platform file dialog |
| `hidden` | done | `display: none` via UA stylesheet |

---

## CSS pseudo-classes

| Pseudo | Status | Notes |
|--------|--------|-------|
| `:enabled` / `:disabled` | done | checked via `disabled` attr in selector_match.rs |
| `:checked` | done | checked via `checked` attr |
| `:required` / `:optional` | done | checked via `required` attr |
| `:readonly` / `:readwrite` | done | checked via `readonly` attr |
| `:placeholder-shown` | partial | checks children empty — should check form_state value |
| `:valid` / `:invalid` | todo | needs validation engine |
| `:in-range` / `:out-of-range` | todo | needs min/max checking |
| `:indeterminate` | done | checked via `indeterminate` attr |
| `:default` | todo | |
| `:autofill` | todo | |

## CSS pseudo-elements

| Pseudo | Status | Notes |
|--------|--------|-------|
| `::placeholder` | todo | needs form value painting |
| `::field-text` | todo | editable text region |
| `::checkmark` | todo | checkbox/radio visual |
| `::file-selector-button` | todo | |
