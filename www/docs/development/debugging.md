---
sidebar_position: 3
---

# Debugging

## Visual Debugging

The fastest way to debug layout/rendering issues is running the demo with an HTML test case:

```bash
cargo run -p wgpu-html-demo -- my-test.html
```

- **F12** — save a screenshot for visual comparison
- **F9** — dump profiler timing to stderr
- **`make_screenshot`** via STDIN — save screenshot programmatically
- **`make_screenshot <query>`** — screenshot a specific element

## Devtools

Enable the devtools panel for interactive inspection:

```rust
let mut devtools = Devtools::attach(&mut tree, false);
```

The devtools window shows:
- Full DOM tree with expand/collapse
- Computed CSS properties per element
- Breadcrumb path from root
- Pick mode to select elements visually

## Layout Debugging

The `LayoutBox` tree can be inspected at any element:

```rust
let box_at_path = wgpu_html::layout_at_path(&layout, path);
println!("margin_rect: {:?}", box_at_path.map(|b| b.margin_rect));
println!("border_rect: {:?}", box_at_path.map(|b| b.border_rect));
println!("content_rect: {:?}", box_at_path.map(|b| b.content_rect));
```

## Display List Inspection

The `DisplayList` can be examined before submitting to the renderer:

```rust
let display_list = wgpu_html::paint_tree_with_text(...);

println!("Quads: {}", display_list.quads.len());
println!("Glyphs: {}", display_list.glyphs.len());
println!("Images: {}", display_list.images.len());
println!("Clips: {}", display_list.clips.len());
println!("Commands: {}", display_list.commands.len());

for cmd in &display_list.commands {
    println!("  cmd: kind={:?} clip={}", cmd.kind, cmd.clip_index);
}
```

## Profiler

Enable the profiler for timing breakdown:

```rust
let mut devtools = Devtools::attach(&mut tree, true); // true = enable profiler
```

Press F9 to dump a summary. The profiler tracks:
- Cascade time per frame
- Layout time (block, flex, grid breakdown)
- Paint time
- Counts: nodes, quads, glyphs, layout boxes
- Frame classification (full pipeline, partial cascade, repaint-only)

## Common Debugging Points

| Issue | Check |
|---|---|
| Element invisible | `display: none` in cascade? `visibility: hidden`? |
| Wrong position | Box model: margin_rect vs border_rect vs content_rect |
| Text not showing | Font registered? `color` resolved? Clip range correct? |
| Click not working | `pointer-events: none`? Hit-test path correct? |
| Layout wrong | flex/grid properties set? overflow? positioning? |
| Performance issue | Profiler dump → identify slow stage |
