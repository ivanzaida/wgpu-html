# Engine Gaps

Status of CSS/HTML features relative to browser engines.
Updated 2026-05-11.

Legend: **Parsed** = CSS parser recognizes the property.
**Typed** = stored as a Rust enum/struct (not a raw string).
**Layout** = affects box positioning/sizing. **Paint** = produces draw commands.

## Critical — pages look broken without these

### ~~Box-shadow~~ (implemented)
- Parsed: yes, into typed `BoxShadow` struct (offset, blur, spread, color, inset).
- Multiple shadows supported (comma-separated).
- Stored on `LayoutBox` as `Vec<BoxShadow>`.
- Paint: expanded SDF quads with `shadow_sigma` for Gaussian-like blur falloff.
- Shader uses smoothstep on SDF distance for soft edges.
- Border-radius is respected (shadow radii expand with blur).
- Inset shadows: parsed but not rendered yet.
- 8 paint tests + 6 parser unit tests.

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

### Transform-aware hit-testing
- ✅ Implemented. Inverse transform applied to pointer coordinates for
  correct picking of rotated/scaled elements. Children of transformed
  parents also use inverse-transformed coordinates.
- `Transform2D::apply_inverse()` added for hit-test use.

## Implemented

### Transforms (`transform`, `transform-origin`)
- Parsed: yes, into typed `Transform2D` affine matrix.
- All 2D functions: `translate(X/Y)`, `scale(X/Y)`, `rotate`, `skew(X/Y)`, `matrix`.
- Percentage `translate(-50%, -50%)` resolves against border-box.
- `transform-origin`: keywords (`left`, `center`, `right`), `%`, `px`.
- GPU vertex-shader transforms on all three pipelines (quad, glyph, image).
- `fwidth()`-based SDF anti-aliasing adapts to rotation/scale.
- Children inherit parent transform matrix.
- 18 paint tests covering quads, glyphs, images, child inheritance.

### Margin collapsing (adjacent siblings)
- Adjacent sibling margins collapse in block flow (CSS 2.2 §8.3.1).
- Positive+positive: `max(a, b)`. Negative+negative: `min(a, b)`. Mixed: `a + b`.
- Out-of-flow children correctly skipped. 5 layout tests.
- Not yet: parent-child collapsing, empty-block collapsing.

### Form validation
- `:valid` / `:invalid` pseudo-classes in query engine AND CSS cascade.
- Validation: `required`, `minlength`, `maxlength`, `min`/`max`/`step` (number/range).
- Missing: `pattern` regex, `type=email`/`type=url` specific validation.

### List markers (`list-style-type`, `list-style-position`)
- `::marker` pseudo-element generated during cascade, ordinals computed.
- Supports: `disc`, `circle`, `square`, `decimal`, `lower-alpha`, `upper-alpha`, `lower-roman`, `upper-roman`, `none`.

### CSS selectors
- Full CSS Level 4: `:has()`, `:is()`, `:where()`, `:not()`, `:nth-child()`, attribute selectors, all combinators.

### CSS variables
- Full `var()` support with fallbacks, recursive substitution, and cycle detection.

### Render backend abstraction
- `RenderBackend` trait in `lui-render-api`. `lui-renderer-wgpu` is the reference impl.
- `DisplayList` IR in `lui-display-list` — zero GPU types.
- `Runtime<D: Driver, B: RenderBackend>` — driver and backend are independent.
- `lui-text` has no wgpu dependency.

## Nice to have

| Feature | Status |
|---------|--------|
| `:focus-visible` | Not tracked (no keyboard vs pointer distinction) |
| `clip-path` | Not parsed |
| Scroll snap | Not parsed |
| `writing-mode` / vertical text | Not parsed |
| Hyphenation (`hyphens`) | Not parsed |
| `word-break: break-all` | Implemented — inserts U+200B between characters |
| `word-break: keep-all` | Parsed |
| `overflow-wrap: break-word` | Consumed, enables wrapping |
| Counters (`counter-increment`, `content: counter()`) | Not parsed |
| `border-image` | Not parsed |
| Text-shadow | Parsed (raw string), not painted |
| `@media` queries | Parsed and evaluated (viewport-based) |
| `@keyframes` | Not parsed |
| View transitions | Not applicable |
| `:focus-within` | Selector works, cascade support partial |
| `content` on `::before`/`::after` | Parsed (typed enum: `none`/`normal`/`string`) |
