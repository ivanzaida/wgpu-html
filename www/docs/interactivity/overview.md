---
title: Interactivity Overview
---

# Interactivity Overview

Interactivity in wgpu-html is driven by `Tree::interaction` (`InteractionState`), which holds the live interaction state. All dispatch logic lives in `wgpu-html-tree` (re-exported through `wgpu-html::interactivity`).

## InteractionState

```rust
pub struct InteractionState {
    pub pointer_pos: Option<(f32, f32)>,
    pub hover_path: Option<Vec<usize>>,
    pub active_path: Option<Vec<usize>>,
    pub focus_path: Option<Vec<usize>>,
    pub modifiers: Modifiers,
    pub selection: Option<TextSelection>,
    pub selecting_text: bool,
    pub scroll_offsets_y: BTreeMap<Vec<usize>, f32>,
    pub edit_cursor: Option<EditCursor>,
    pub caret_blink_epoch: Instant,
    pub selection_colors: SelectionColors,
    pub double_click_origin_ms: u64,
    pub double_click_count: u8,
    pub double_click_pos: (f32, f32),
}
```

## Interaction Flow

```
Host event (winit / egui)
    │
    ▼  LayoutBox::hit_path_scrolled(pos)
Vec<usize> path
    │
    ▼  Tree::dispatch_pointer_move / mouse_down / key_down / ...
InteractionState updated, callbacks fired
    │
    ▼  Cascade re-evaluates :hover / :active / :focus pseudo-classes
    │
    ▼  Layout + Paint produce new DisplayList
    │
    ▼  Renderer draws frame
```

## Pointer/Mouse Pipeline

```rust
// Mouse movement
interactivity::pointer_move(tree, layout, pos);
interactivity::pointer_move_with_cursor(tree, layout, pos); // also returns Cursor

// Button press/release
interactivity::mouse_down(tree, layout, pos, MouseButton::Primary);
interactivity::mouse_up(tree, layout, pos, MouseButton::Primary);

// Pointer leave
interactivity::pointer_leave(tree);
```

Button presses synthesize clicks on release (if over the same element). Click counting is built in for double-click word selection and triple-click line selection.

## Keyboard Pipeline

```rust
interactivity::key_down(tree, &key_event, modifiers);
interactivity::key_up(tree, &key_event, modifiers);
```

Built-in keyboard handling includes Tab/Shift+Tab focus navigation, text input forwarding to focused inputs, Ctrl+A select-all, and Escape blur.

## Focus Management

```rust
interactivity::focus(tree, path);      // Focus a specific element
interactivity::blur(tree);             // Clear focus
interactivity::focus_next(tree, false); // Tab to next focusable
interactivity::focus_next(tree, true);  // Shift+Tab to previous
```

Focus moves to the deepest focusable ancestor of a mousedown hit, matching browser behavior.

## Text Selection

```rust
pub struct TextSelection {
    pub anchor: TextCursor,
    pub focus: TextCursor,
}

pub struct TextCursor {
    pub path: Vec<usize>,
    pub glyph_index: usize,
}
```

Selection is driven by mouse drag with the primary button. The engine supports select-all, word select (double-click), line select (triple-click), and Ctrl+C copy via `arboard`.

## Form Field Editing

Focused `<input>` and `<textarea>` elements receive keyboard text input. An `EditCursor` tracks byte-offset caret position with optional selection anchor. Text editing operations (insert, delete, backspace, arrow keys, home/end) are implemented in `text_edit.rs`.

## Scrolling

`InteractionState::scroll_offsets_y` maps element paths to their current vertical scroll offset. Mouse wheel events scroll the viewport and nested `overflow: scroll` / `overflow: auto` containers. Scrollbar paint (10px track, drag-to-scroll) is in `wgpu-html::scroll`.

## Sub-Pages

- [Event System](./events) — event types, bubbling, callbacks
- [Focus + Keyboard](./focus-keyboard) — focus management, Tab navigation, modifiers
- [Form Controls](./forms) — text editing, placeholder, caret, input types
- [Text Selection](./text-selection) — drag-select, word/line select, clipboard
- [Scrolling](./scrolling) — viewport scroll, nested scroll, scrollbar
