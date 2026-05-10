---
title: Input Types
---

# Input Types

lui parses all 22 HTML input types into the `InputType` enum (`lui-models::common::html_enums`). Each type is recognized, focusable, and participates in the event system — but not all types have full visual/interactive fidelity yet.

## Status Table

| Type | Implemented | Text Editable | Notes |
|---|---|---|---|
| `text` | Yes | Yes | Standard single-line text input with caret, selection, placeholder |
| `password` | Yes | Yes | Bullet-masked (U+2022) display; cleartext stored internally; caret byte-boundary remapping |
| `email` | Partial | Yes | Accepts text input; no email validation enforced |
| `search` | Partial | Yes | Behaves as `text`; no search-specific UI (clear button, etc.) |
| `tel` | Partial | Yes | Behaves as `text`; no telephone-specific UI |
| `url` | Partial | Yes | Accepts text input; no URL validation enforced |
| `number` | Yes | Yes | Text editing + ArrowUp/ArrowDown stepping with `min`/`max`/`step`; no spinner UI |
| `date` | Partial | Yes | Accepts text input; no date picker UI |
| `datetime-local` | Partial | Yes | Accepts text input; no datetime picker UI |
| `month` | Partial | Yes | Accepts text input; no month picker UI |
| `week` | Partial | Yes | Accepts text input; no week picker UI |
| `time` | Partial | Yes | Accepts text input; no time picker UI |
| `checkbox` | Yes | No | 13x13 box with checkmark; toggles on click/Space/Enter; fires `input`/`change` events |
| `radio` | Yes | No | 13x13 circle with dot; mutual exclusion within `name` group; fires `input`/`change` events |
| `range` | Yes | No | Slider track + circular thumb; mouse drag updates value; ArrowUp/ArrowDown stepping; respects `min`/`max`/`step` |
| `color` | Yes | No | Displays parsed hex color as filled swatch (default `#000000`); 44x23 intrinsic size |
| `file` | Partial | No | Styled as transparent background; no file chooser dialog |
| `hidden` | Yes | No | `display: none` via UA stylesheet; excluded from rendering |
| `image` | No | No | Parsed; no image rendering or submit-coordinates behavior |
| `button` | Yes | No | Rendered as button; clickable; `border: 2px outset`, `background: buttonface` |
| `submit` | Yes | No | Rendered as button; triggers `on_click` (no form submission) |
| `reset` | Yes | No | Rendered as button; triggers `on_click` (no form reset) |

### Legend

- **Yes** — fully functional for the scope of this engine
- **Partial** — parsed and interactive but missing specialized UI or validation
- **No** — parsed and focusable only; no type-specific behavior

## Attributes

All input types support these attributes (parsed on the `Input` struct):

| Attribute | Type | Description |
|---|---|---|
| `type` | `Option<InputType>` | Determines input behavior; defaults to `text` |
| `value` | `Option<ArcStr>` | Current value |
| `placeholder` | `Option<ArcStr>` | Placeholder text shown when empty |
| `name` | `Option<ArcStr>` | Form control name; used for radio group exclusion |
| `checked` | `Option<bool>` | Checked state for checkbox/radio |
| `disabled` | `Option<bool>` | Disables the control |
| `readonly` | `Option<bool>` | Prevents editing |
| `required` | `Option<bool>` | Parsed; not enforced |
| `autofocus` | `Option<bool>` | Auto-focus on mount |
| `min` | `Option<ArcStr>` | Enforced for `number`/`range` (ArrowUp/ArrowDown clamping) |
| `max` | `Option<ArcStr>` | Enforced for `number`/`range` (ArrowUp/ArrowDown clamping) |
| `step` | `Option<ArcStr>` | Enforced for `number`/`range` (default `1`; snaps on step/drag) |
| `minlength` | `Option<i32>` | Parsed; not enforced |
| `maxlength` | `Option<i32>` | Parsed; not enforced |
| `pattern` | `Option<ArcStr>` | Parsed; not enforced |
| `multiple` | `Option<bool>` | Parsed; not enforced |
| `autocomplete` | `Option<ArcStr>` | Parsed; not consumed |

## Default Styling

**Text-like inputs** (text, password, email, search, tel, url, number, date/time variants):
```css
input {
  display: inline-block;
  border: 2px inset;
  background-color: field;
  color: fieldtext;
  padding: 1px 2px;
}
```

**Button-like inputs** (button, submit, reset):
```css
input[type="button"],
input[type="submit"],
input[type="reset"] {
  border: 2px outset;
  background-color: buttonface;
  color: buttontext;
  padding: 2px 6px;
  text-align: center;
}
```

**Checkbox and radio**:
```css
input[type="checkbox"],
input[type="radio"] {
  width: 13px;
  height: 13px;
  box-sizing: border-box;
  padding: 0;
  margin: 3px 3px 3px 4px;
  border: 1px solid;
}

input[type="radio"] {
  border-radius: 50%;
}
```

**Range slider**:
```css
input[type="range"] {
  width: 129px;
  height: 16px;
  box-sizing: border-box;
  padding: 0;
  margin: 2px;
  border: none;
  background-color: transparent;
}
```

**Color swatch**:
```css
input[type="color"] {
  width: 44px;
  height: 23px;
  box-sizing: border-box;
  padding: 1px 2px;
  border: 1px solid;
}
```

## Keyboard Shortcuts

When a text-editable input has focus:

| Shortcut | Action |
|---|---|
| Ctrl+A | Select all text in field |
| Ctrl+C | Copy selection |
| Ctrl+X | Cut selection |
| Ctrl+V | Paste from clipboard |
| Ctrl+Left/Right | Move cursor by word |
| Ctrl+Shift+Left/Right | Select by word |
| Ctrl+Backspace | Delete word backward |
| Ctrl+Delete | Delete word forward |
| Shift+Left/Right | Extend selection by character |
| Home/End | Move to line start/end |
| ArrowUp/ArrowDown | Move by line (textarea), step value (number/range) |
