---
sidebar_position: 2
---

# Known Issues

## Layout

### z-index Stacking Contexts
**Symptom:** A deeply nested `z-index: 999` element paints behind a shallow `z-index: 1` element in a different subtree.

**Cause:** Sibling sort by z-index works correctly, but there are no independent stacking contexts. Cross-branch ordering follows tree DFS, which can override z-index expectations.

**Workaround:** Keep z-indexed elements as siblings within the same parent.

### Border Style Rendering
**Symptom:** `border-style: double`, `groove`, `ridge`, `inset`, `outset` render as solid lines.

**Cause:** Only `solid`, `dashed`, `dotted`, `none`, and `hidden` are implemented in the quad pipeline shader.

### Dashed/Dotted on Elliptical Corners
**Symptom:** Dashed/dotted borders on elements with non-uniform border-radius (elliptical corners) show straight segments at corners.

**Cause:** The patterned border shader only follows curved corners for uniform-circular radii.

### em/rem Default Size
**Symptom:** `em` and `rem` units use exactly 16px when no explicit font-size is inherited.

**Cause:** UA stylesheet font-sizes are not globally applied; the root font-size is not resolved until layout. A hard-coded 16px fallback is used.

### Position: Sticky
**Symptom:** `position: sticky` behaves the same as `position: relative`.

**Cause:** Sticky positioning logic is not implemented. The element is offset like `relative` but doesn't react to scroll position.

## Parser

### Unknown Tags
**Symptom:** Unknown HTML tags cause their entire subtree to be silently dropped.

**Cause:** The parser only recognizes ~96 tag names. Unrecognized tags (e.g., `<figure>`, `<menu>`, `<q>`) are dropped at tree-build time.

### Whitespace Handling
**Symptom:** Whitespace-only text nodes between tags are dropped.

**Cause:** The tree builder skips text nodes containing only whitespace characters.

## Rendering

### Nested Rounded Clips
**Symptom:** When two nested elements both have `overflow: hidden` and `border-radius`, only the innermost clip's rounded corners are used.

**Cause:** Composing multiple rounded SDF clips requires shader-level intersection of two SDFs, which is not yet implemented. Rectangular scissor intersections work correctly.

### Animated Image Frames
**Symptom:** Animated GIF/WebP images may not advance frames in some engine configurations.

**Cause:** Frame selection uses a process-wide clock anchor that relies on the host calling `render_frame` on each frame tick. If the host throttles rendering, animation frame rates are affected.

## Interactivity

### Scroll Hit-Testing
**Symptom:** Click coordinates inside a scroll container may not be correctly mapped to the scrolled content for hit-testing.

**Cause:** Element scroll offsets are applied at paint time but may not be subtracted from hit-test coordinates for all interactions. Viewport scrolling is handled; element-level scroll hit-testing is partial.

### No `<select>` Dropdown
**Symptom:** The `<select>` element renders as a static block with no interactive dropdown.

**Cause:** The popup menu rendering and interaction system for `<select>` is not yet implemented.

### Wheel Events on Element Callbacks
**Symptom:** Mouse wheel events scroll the element but are not forwarded to element `on_event` callbacks.

**Cause:** Wheel events are consumed for scroll handling before callback dispatch on the focused element.

## Performance

### Flex/Grid Incremental Relayout
**Symptom:** Changing one child inside a flex or grid container causes the entire container to re-layout.

**Cause:** Flex and grid have cross-item dependencies (free space redistribution, track sizing). Incremental layout falls back to full re-layout for these container types.

### First Frame Latency
**Symptom:** The first frame after loading a large document or many images may have higher latency.

**Cause:** Image loading and font rasterization happen on first use. Preload with `tree.preload_asset()` to mitigate.
