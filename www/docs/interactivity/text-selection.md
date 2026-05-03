---
id: text-selection
title: Text Selection
---

# Text Selection

Text selection allows users to highlight document text by dragging, double-clicking, or triple-clicking. The selection state lives in `InteractionState::selection`.

## TextCursor and TextSelection

```rust
pub struct TextCursor {
    pub path: Vec<usize>,       // Path to the text LayoutBox
    pub glyph_index: usize,     // Index into the glyph array
}

pub struct TextSelection {
    pub anchor: TextCursor,     // Where the drag started
    pub focus: TextCursor,      // Where the drag is currently
}
```

A selection where `anchor == focus` is *collapsed* (no visible highlight). The selection highlight is painted across all text runs between anchor and focus in document order.

## Drag-to-Select

Primary-button drag sets `InteractionState::selecting_text = true` and updates `focus`:

```rust
tree.dispatch_mouse_down(target, pos, MouseButton::Primary, cursor);
// On move while primary is down:
tree.dispatch_mouse_move_with_selection(target, pos, cursor);
```

Selection highlight quads are painted per line within the affected text runs. The highlight color is configurable via `SelectionColors`.

## select_all_text / selected_text

```rust
// Ctrl+A
wgpu_html::select_all_text(&mut tree, &layout);

// Get selected text for clipboard
if let Some(text) = wgpu_html::selected_text(&tree, &layout) {
    // Copy to clipboard...
}
```

`select_all_text()` finds the first and last text cursors in document order and sets a selection spanning them. `selected_text()` extracts the visible text from the selection range.

## Word Select (Double-Click)

Double-clicking selects the word under the pointer:

```rust
wgpu_html::select_word_at_cursor(&mut tree, &layout, &cursor);
```

Word boundaries use `char::is_alphanumeric` and `_` as word characters. Whitespace and punctuation form separate tokens.

## Line Select (Triple-Click)

Triple-clicking selects the entire shaped line:

```rust
wgpu_html::select_line_at_cursor(&mut tree, &layout, &cursor);
```

This selects all glyphs in the `ShapedLine` containing the hit cursor.

## Selection Highlight

The paint stage emits filled quads behind selected text:

```rust
// Background highlight quad per selected glyph range
list.push_quad(Quad {
    rect: highlight_rect,
    color: selection_colors.background,
    radii_h: [0.0; 4], radii_v: [0.0; 4],
    stroke: [0.0; 4], pattern: [0.0; 4],
});
```

The highlight is painted *before* glyphs, so text appears on top. Selection colors default to browser-like blue highlight with white text.

## user-select: none

```css
.unselectable { user-select: none; }
```

`LayoutBox::user_select` suppresses selection for a box and its descendants. The hit test and paint both skip boxes with `user_select == UserSelect::None`.

## Ctrl+A / Ctrl+C

| Shortcut | Action |
|---|---|
| Ctrl+A | `select_all_text()` — selects all text in the document |
| Ctrl+C | Copies `selected_text()` to the OS clipboard via `arboard` |

These are handled in the default `key_down` dispatch. Hosts using `WgpuHtmlWindow` get them automatically; custom integrations add them manually.
