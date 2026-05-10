---
sidebar_position: 3
---

# Forms

Demonstrates form controls: text inputs, checkboxes, radio buttons, range sliders, color pickers, date inputs, file inputs, and textareas.

## Example Content

```html
<form style="max-width: 400px; margin: 32px auto; font-family: sans-serif;">
    <label>Name: <input type="text" placeholder="Enter your name"></label>
    <label>Password: <input type="password"></label>
    <label><input type="checkbox" checked> Remember me</label>
    <fieldset>
        <legend>Color</legend>
        <label><input type="radio" name="color" value="red"> Red</label>
        <label><input type="radio" name="color" value="blue" checked> Blue</label>
    </fieldset>
    <label>Volume: <input type="range" min="0" max="100" value="50"></label>
    <label>Pick color: <input type="color" value="#e94560"></label>
    <label>Date: <input type="date"></label>
    <label>File: <input type="file"></label>
    <textarea placeholder="Message..." rows="4"></textarea>
    <input type="submit" value="Send">
</form>
```

## Running

```bash
cargo run -p lui-demo -- demo/lui-demo/html/styled-inputs.html
```

## What It Shows

- All 22 input types (some with native paint, some as text-like)
- Text editing with caret, selection, and keyboard navigation
- Checkbox/radio click toggle with native paint
- Range slider with track + thumb
- Color picker overlay
- Date picker calendar
- File dialog with filename display
- Multiline textarea editing
