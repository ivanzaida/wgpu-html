# Engine Gaps

Status of CSS/HTML features relative to browser engines.
Updated 2026-05-11.

Legend: **Parsed** = CSS parser recognizes the property.
**Typed** = stored as a Rust enum/struct (not a raw string).
**Layout** = affects box positioning/sizing. **Paint** = produces draw commands.

## Critical — pages look broken without these

### Box-shadow
- Parsed: yes (raw `ArcStr`)
- Layout: no
- Paint: no
- Impact: cards, modals, dropdowns all lose depth/elevation.

### ~~Transforms~~ (`transform`, `transform-origin`) (implemented)
- Parsed: yes, into typed `Transform2D` affine matrix (translate, scale, rotate, skew, matrix).
- Percentage translate resolves against border-box dimensions.
- Transform-origin parsed (keywords, %, px) and stored on LayoutBox.
- Paint: full affine AABB transform applied to background/border rects and propagated to children.
  Translate, scale, rotate, and skew all produce correct axis-aligned bounding boxes.
- Not yet: transformed hit-testing (pointer events ignore transforms).

### ~~Margin collapsing~~ (implemented)
- Adjacent sibling margins collapse in block flow (CSS 2.2 section 8.3.1).
- Handles positive+positive (max), negative+negative (most negative), and mixed (sum).
- Out-of-flow children are correctly skipped.
- Not yet implemented: parent-child collapsing, empty-block collapsing.

### `@font-face`
- Parsed: no (`@`-rules not handled by the CSS parser)
- Impact: web fonts don't load. Only host-registered `.ttf`/`.otf` files work.

### Bidi / RTL text
- HTML `dir` attribute parsed; CSS `direction` property deferred.
- No UAX#9 bidi algorithm in text shaping.
- Impact: Arabic, Hebrew, and mixed-direction text unreadable.

### Stacking contexts (partial)
- z-index parsed, stored, and used for sibling sort in paint.
- Missing: opacity/transform/filter creating new stacking contexts; cross-branch z-index reordering.
- Impact: deeply nested positioned elements may paint in wrong order.

### Floats
- Not parsed, not stored, no layout logic.
- Impact: any layout relying on `float: left/right` fails. Less critical for app UIs (flex/grid replace floats) but breaks content pages.

## Important — many pages need these

### `position: sticky`
- Parsed: yes (typed enum). Layout: treated identically to `relative`.
- Missing: scroll-based repositioning.

### Transitions / animations
- Parsed: yes (raw `ArcStr` via deferred shorthand).
- No animation engine, no keyframe processing, no time-based interpolation.
- Impact: hover effects, loading spinners, fade-ins, slide-outs all missing.

### `text-overflow: ellipsis`
- Parsed: yes (typed enum `TextOverflow::Clip` / `Ellipsis`). Stored on `Style` and `LayoutBox`.
- Layout: stored on LayoutBox, not yet consumed for truncation.
- Paint: no truncation or ellipsis glyph rendered.

### Filters (`filter`, `backdrop-filter`)
- Not parsed (listed in `DEFERRED_LONGHANDS`).
- Impact: blur, brightness, drop-shadow effects missing.

### Outline
- Not parsed (listed in `DEFERRED_LONGHANDS`).
- Impact: focus rings on keyboard navigation invisible (accessibility concern).

### `<select>` dropdown
- Element recognized; form control slot allocated.
- No dropdown menu rendering, no option list, no keyboard navigation.

### Multi-column layout
- Not parsed (column-count/column-width in `DEFERRED_LONGHANDS`).
- No column balancing or spanning logic.

### `aspect-ratio`
- Not parsed, not stored.

### Border styles: `double`, `groove`, `ridge`, `inset`, `outset`
- Parsed and stored (typed enum).
- Paint: rendered as solid. Only `solid`, `dashed`, `dotted` have distinct rendering.

### Form validation
- `:valid` / `:invalid` pseudo-classes implemented in query engine AND CSS cascade.
- PseudoClassUsage tracks `:valid`/`:invalid` for incremental cascade optimization.
- Validation checks: `required`, `minlength`, `maxlength`, `min`/`max`/`step` (number/range).
- Missing: `pattern` (regex) validation, `type=email`/`type=url` specific validation, `minlength`/`maxlength` on textarea.
- Note: cascade already re-evaluates on value changes (FullPipeline), so CSS like `input:invalid { border: red; }` works.

## Implemented (previously reported as gaps)

### List markers (`list-style-type`, `list-style-position`)
- Fully implemented. Typed enums, `::marker` pseudo-element generated during cascade, ordinals computed, text painted via shaped runs.
- Supports: `disc`, `circle`, `square`, `decimal`, `lower-alpha`, `upper-alpha`, `lower-roman`, `upper-roman`, `none`.

### CSS selectors
- Full CSS Level 4: `:has()`, `:is()`, `:where()`, `:not()`, `:nth-child()`, attribute selectors, all combinators.

### CSS variables
- Full `var()` support with fallbacks, recursive substitution, and cycle detection.

## Nice to have

| Feature | Status |
|---------|--------|
| `:focus-visible` | Not tracked (no keyboard vs pointer distinction) |
| `clip-path` | Not parsed |
| Scroll snap | Not parsed |
| `writing-mode` / vertical text | Not parsed |
| Hyphenation (`hyphens`) | Not parsed |
| `word-break: break-all` | ✅ Implemented — inserts U+200B between characters for any-char breaks |
| `word-break: keep-all` | ✅ Parsed |
| `overflow-wrap: break-word` | ✅ Consumed from deferred longhands — enables wrapping |
| Counters (`counter-increment`, `content: counter()`) | Not parsed |
| `border-image` | Not parsed |
| Text-shadow | Parsed (raw string), not painted |
| `@media` queries | Parsed and evaluated (viewport-based) |
| `@keyframes` | Not parsed |
| View transitions | Not applicable |
| `:focus-within` | Selector works, cascade support partial |
| `content` on `::before`/`::after` | Parsed (typed enum: `none`/`normal`/`string`) |
