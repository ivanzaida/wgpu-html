---
sidebar_position: 5
---

# Forms and Inputs

lui supports `<input>` (22 types), `<textarea>`, `<button>`, and `<select>`, with full text editing, caret navigation, and selection.

## Text Inputs

`<input type="text">`, `<input type="password">`, `<input type="email">`, `<input type="number">`, `<input type="search">`, `<input type="tel">`, `<input type="url">`:

- Single-line editing with keyboard navigation (arrow keys, Home, End)
- Shift-selection for range selection
- Click to place caret, drag to select
- Placeholder text when empty
- Clipboard: Ctrl+C copy, Ctrl+X cut, Ctrl+V paste
- `<input type="password">`: masked with U+2022 bullets
- Readonly support: `readonly` attribute prevents editing
- Disabled support: `disabled` attribute + `:disabled` pseudo-class

## Checkboxes and Radios

`<input type="checkbox">` and `<input type="radio">`:

- Native GPU-rendered checkmark (checkbox) and dot (radio)
- Checked/unchecked visual state, `border`, and `accent-color` support
- Click to toggle, fires `change` and `input` events
- Radio mutual exclusion within same `name` group
- `checked` attribute for initial state

## Range Slider

`<input type="range">`:

- Native GPU-rendered track with filled portion and thumb
- `min`, `max`, `value`, and `step` attributes
- Drag thumb to adjust value
- Customizable via CSS: `--lui-range-track-color`, `--lui-range-progress-color`, `--lui-range-thumb-color`, `--lui-range-thumb-shadow`
- Default `height: 16px`

## Color Picker

`<input type="color">`:

- Native color swatch with inset border
- Full color picker overlay rendered through the layout/paint pipeline
- Gradient bars for hue and alpha selection
- Editable hex and RGBA text fields
- Configurable via `--lui-color-*` CSS custom properties
- `value` attribute as hex color string

## Date Inputs

`<input type="date">` and `<input type="datetime-local">`:

- Locale-formatted date display (DMY, MDY, YMD patterns)
- Per-segment editing in overwrite mode
- Tab navigation between segments
- Calendar picker overlay with header, grid, and hover/focus states
- `min`, `max`, `step`, `value` (ISO 8601 format)
- Configurable via `--lui-calendar-*` CSS properties (~28 properties)
- `datetime-local` adds HH:MM time segments with validation on blur
- Pluggable `Locale` trait for date patterns and month/weekday names

## File Input

`<input type="file">`:

- Native file dialog via rfd backend
- `accept` attribute for file type filtering
- `multiple` attribute for multi-file selection
- File info stored: name, size (bytes), MIME type, path, last modified
- Label displays selected filenames
- `::file-selector-button` pseudo-element for button styling

## Textarea

`<textarea>`:

- Multiline text editing with full keyboard support
- `overflow: auto` with scrollbars when content exceeds box
- Placeholder text when empty
- `resize` property (`both`, `none`, `horizontal`, `vertical`)
- `white-space: pre-wrap`, monospace font
- `rows` and `cols` attributes for initial size

## Buttons

`<input type="submit">`, `<input type="reset">`, `<input type="button">`, `<button>`:

- Styled with button-like appearance (inset/outset border, buttonface background)
- `<button>` renders child content; `<input>` buttons show value as text
- Focusable with keyboard navigation
- `Submit` button: fires `submit` event on Enter or click
- `Reset` button: resets form controls

## Select

`<select>` with `<option>` and `<optgroup>`:

- Parsed with full attribute support (`multiple`, `size`, `required`, `disabled`)
- Styled as inline-block with border and padding
- **No popup/list interaction yet** — rendered as static box showing text content
- `selected`, `value`, `label` attributes parsed

## Form Submission

Forms with a submit button or text input support the standard submission flow:

```html
<form id="login" method="post">
  <input type="text" name="username" value="alice">
  <input type="password" name="password">
  <input type="checkbox" name="remember" checked>
  <input type="submit" value="Log in">
</form>
```

**Triggers:**
- Click on `<input type="submit">` or `<button type="submit">`
- Enter key in a text/password/email input inside a form
- Space/Enter on a focused submit button

**Flow:**
1. `SubmitEvent` fires on the form with capture → target → bubble phases
2. If `preventDefault()` is called, form data collection is skipped
3. Otherwise, `collect_form_data` walks the form subtree and collects `name` + `value` pairs from:
   - `<input>` (text, password, email, number, search, tel, url, checkbox checked, radio checked, date, datetime-local, color, range, hidden, file)
   - `<textarea>` (text content)
   - Excludes: unchecked checkboxes/radios, submit/reset/button inputs, image inputs
4. `FormDataEvent` fires on the form with a `FormDataId` handle
5. Host retrieves data via `Tree::take_form_data(FormDataId)` → `Vec<FormField>`

**Rust API:**
```rust
// Programmatic submission
lui_tree::submit_form(&mut tree, form_path, Some(submitter_path));

// Retrieve collected data
let fields: Vec<FormField> = tree.take_form_data(form_data_id).unwrap();
// fields[0].name, fields[0].value — ArcStr
```

**Not yet implemented:** `pattern` validation, `min`/`max`/`step`/`minlength`/`maxlength` constraint checking, `form.action`/`method` URL construction, `multipart/form-data` encoding.
