# lui — CSS Spec

The CSS pipeline this engine implements, what's already wired up, and
what's deliberately out-of-scope or left for follow-ups. Companion to
`spec/text.md` (text rendering) and `docs/full-status.md` (broader engine
snapshot).

This file is the source of truth for "what does our CSS support look
like today and where is it heading".

For the exhaustive per-property matrix, see `spec/css-properties.md`.

---

## 1. Goals

- A typed, fully-resolved `Style` per element, computed once per
  cascade pass. Layout and paint never re-parse CSS.
- Standards-faithful enough to handle the subset of CSS the renderer
  actually paints (block + flex layout, backgrounds, borders, text).
- One source of truth for the property table: `lui-parser`
  knows the kebab-case names; `lui-models::Style` is the typed
  shape; `lui-style` consumes the cascade output.
- No global mutable state. The cascade is a pure function over a
  `Tree` plus its embedded `<style>` blocks.

## 2. Non-goals

- No browser parity. We don't pretend to be a CSS engine; we honour
  the subset documented below and ignore the rest.
- No JavaScript-driven re-style. Any host-side mutation re-runs the
  whole cascade.
- No CSSOM. The `Stylesheet`, `Rule`, `Selector`, `Style` types are
  internal data structures — there's no JS-facing wrapper on top.
- No quirks-mode reinterpretation: the parser is permissive but the
  cascade always uses standards-mode semantics.

## 3. Pipeline

```
HTML string
   │
   ▼  lui-parser            tokenize + tree-build + parse <style> bodies
Tree<Node<Element>>                inline `style="…"` attrs stay raw on each element
   │
   ▼  lui-style::cascade    selector match + 4-band cascade + keyword
                                  resolution + implicit inheritance
CascadedTree                       per-node fully-resolved Style
   │
   ▼  lui-layout-old            consume the typed Style values
LayoutBox tree
```

The parser owns:

- Tokenisation (`tokenizer.rs`).
- Tree building (`tree_builder.rs`).
- CSS declaration parsing (`css_parser.rs`).
- Selector + rule parsing (`stylesheet.rs`).
- The shared property dispatch table (`style_props.rs`).

`lui-style` owns the cascade itself: matching, ordering,
keyword resolution, inheritance.

---

## 4. Selectors

**Done** — `lui-parser/src/stylesheet.rs::parse_selector`.

| Form        | Example               | Notes                                  |
|-------------|-----------------------|----------------------------------------|
| Tag         | `div`                 |                                        |
| Id          | `#hero`               |                                        |
| Class       | `.card`               | Multi-class via repetition: `.a.b`     |
| Universal   | `*`                   | Combines with other simple constraints |
| Compound    | `div#hero.card.big`   | All conditions on one element          |
| Descendant combinator | `main .card` | Tree-aware ancestor matching           |
| Selector list (comma) | `h1, h2, .big` | Each comma-separated entry is its own selector |
| Attribute presence/equality | `[hidden]`, `[type=submit]` | Exact equality only                    |
| Pseudo-classes | `:hover`, `:active`, `:focus`, `:root`, `:first-child`, `:last-child` | `:visited` parses but never matches |

Specificity (CSS-Selectors-3) packed into `u32`:
`(id_count << 16) | (class_count << 8) | tag_count`. Comparing as
plain integers gives the right ordering.

**Missing / partial**
- Child `>`, adjacent `+`, and general sibling `~` are still rejected
  and drop the rule.
- Attribute selectors only support `[attr]` and `[attr=value]`.
  Operators `~=`, `|=`, `^=`, `$=`, `*=` and the case-insensitive
  `i` flag are not implemented.
- Structural / logical pseudo-classes still missing:
  `:nth-child`, `:only-child`, `:empty`, `:not()`, `:is()`,
  `:where()`, `:checked`, `:disabled`, etc.
- **No pseudo-elements**: `::before`, `::after`, `::first-line`,
  `::placeholder`, etc.
- **No namespaces** (`@namespace`, `ns|tag`).


Selector matching supports tree-aware descendant checks plus a
stateful `MatchContext` for `:hover` / `:active` / `:focus`.
See `spec/interactivity.md` §8 for cascade integration details.

## 5. At-rules

**`@media` — ✅ Done.** `@media screen and (max-width: 500px) { … }`
blocks are parsed, held as gated rule-lists on each `Stylesheet`, and
evaluated at cascade time against a `MediaContext` (viewport width,
height, orientation, scale). Supports `min-width`, `max-width`,
`min-height`, `max-height`, `orientation: portrait | landscape`, and
the `not` prefix for negating the entire query list. The cascade entry
point `cascade_with_media(tree, &media_ctx)` gates each block's rules
on query match.

**Other at-rules — ❌ Not implemented.** `@supports`, `@import`,
`@font-face`, `@keyframes`, `@page`, `@layer`, `@scope`, `@property`
are all unhandled. The stylesheet parser walks `selectors { decls }`
blocks and `@media { … }` blocks only — anything else is skipped
silently.

## 6. Property parsing

Source: `lui-parser/src/css_parser.rs::apply_css_property`
plus its per-property value parsers.

### 6.1 Length values

`parse_css_length` recognises `px`, `%`, `em`, `rem`, `vw`, `vh`,
`vmin`, `vmax`, `auto`, bare `0`. Unknown shapes drop into a `Raw`
fallback that layout treats as zero.

Also supported:
- `calc(...)`, `min(...)`, `max(...)`, `clamp(...)` — parsed into a
  typed math tree and resolved later by layout.
- `var(--foo)` and custom properties — captured by the parser,
  inherited through the cascade side-car maps, resolved before the
  final typed property parser runs.

Not yet supported:
- `ch`, `ex`, `lh`, container-query units (`cqw`/`cqh`/…).

### 6.2 Color values

`parse_css_color` recognises:
- Named colors (~20 common ones — see `lui-style::color`).
- `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`.
- `rgb(r, g, b)`, `rgba(r, g, b, a)` with comma or slash-alpha.
- `hsl(h, s, l)`, `hsla(h, s, l, a)`.
- `transparent`, `currentcolor`.

**CSS Color Module Level 4 system colors** — recognised by both
the parser validator (`is_supported_named_color`) and the layout
color resolver (`lui_layout_old::color::named_color`). Used by
the UA stylesheet for form controls (`background-color: buttonface`,
`color: fieldtext`, …) so those rules cascade cleanly:

| Keyword | Light-mode RGB | Notes |
|---|---|---|
| `canvas` / `canvastext` | white / black | document area |
| `buttonface` / `buttontext` / `buttonborder` | #ddd / black / #6f6f6f | UA button defaults |
| `field` / `fieldtext` | white / black | UA `<input>` / `<textarea>` defaults |
| `linktext` / `visitedtext` / `activetext` | #00e / #551a8b / red | anchor states |
| `highlight` / `highlighttext` / `selecteditem` / `selecteditemtext` | #38f / white | text selection |
| `mark` / `marktext` | yellow / black | `<mark>` |
| `graytext` | #808080 | disabled text |
| `accentcolor` / `accentcolortext` | #38f / white | system accent |

We don't track `prefers-color-scheme` yet, so dark-mode UAs would
just pick different RGB. Author CSS routinely overrides these,
so the exact values matter less than not failing the cascade.

Also accepted and preserved as function values:
- `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`, `color()`,
  `color-mix()`, `light-dark()`.

Not yet fully resolved:
- Color-mix / color-contrast functions.
- Wide-gamut color spaces.
- `currentcolor` resolution (parsed into `CssColor::CurrentColor` but
  layout currently returns `None` for it).

### 6.2.1 Generic font-family fallback — Done

`FontRegistry::find_first` (in `lui-tree::fonts`) walks the
CSS family list left-to-right and returns the first family that
has a registered face. **If no listed family matches and any
entry is a CSS generic keyword**, it falls back to the best
`(weight, style)`-scoring face from the entire registry.
Recognised generics: `sans-serif`, `serif`, `monospace`,
`cursive`, `fantasy`, `system-ui`, `ui-sans-serif`, `ui-serif`,
`ui-monospace`, `ui-rounded`, `math`, `emoji`, `fangsong`,
`-apple-system`, `BlinkMacSystemFont`.

This makes plain `font-family: sans-serif` resolve whatever font
the host registered (typically via
`lui_winit::register_system_fonts(tree, "MyFamily")`)
without needing the host to also register the generic alias.

### 6.3 Properties — typed vs deferred vs ignored

**Typed `Style` fields** (parsed into `Option<Enum>` /
`Option<CssLength>` / `Option<CssColor>` / numeric / typed vectors):

`display, position, top/right/bottom/left, width, height,
min-/max-width, min-/max-height, margin (+ per-side), padding (+ per-
side), color, background, background-color, background-image,
background-repeat, background-clip, background-size,
background-position, border (shorthand),
border-{top,right,bottom,left} (shorthand),
border-width, border-style, border-color, per-side border longhands,
border-radius (1–4 corner expansion + `/` elliptical syntax) and
per-corner radius longhands with optional `<h> <v>`, font-size,
font-weight (named + numeric), font-style, line-height,
letter-spacing, text-align, text-transform, white-space, overflow
(+ per-axis), opacity, visibility, z-index, flex-direction,
flex-wrap, justify-content, align-items, align-content, align-self,
order, gap, row-gap, column-gap, flex, flex-grow, flex-shrink,
flex-basis, font-family, text-decoration, cursor, pointer-events,
user-select, box-sizing, grid-template-columns, grid-template-rows,
grid-auto-columns, grid-auto-rows, grid-auto-flow, grid-column,
grid-column-start/end, grid-row, grid-row-start/end,
justify-items, justify-self, transform, transform-origin,
transition, animation, box-shadow.`

**Deferred longhands** (recognized, stored by kebab-case in
`Style.deferred_longhands`, and carried through cascade for future
implementation): members produced by modern or partially-implemented
shorthands such as `animation-*`, `transition-*`, logical
`margin/padding/inset/border-*`, `background-origin`,
`background-attachment`, `background-position-x/y`, `font-variant-*`,
`font-stretch`, `list-style-*`, `outline-*`, `overscroll-*`,
`scroll-margin-*`, `scroll-padding-*`, `text-decoration-*`,
`text-emphasis-*`, `scroll-timeline-*`, `view-timeline-*`, and other
future-facing longhands listed in `lui-parser/src/shorthands.rs`.

For `animation` and `transition`, the parser now performs per-layer
member extraction with defaults (`0s`, `ease`, `running`, etc.) rather
than only storing the raw shorthand string.

Shorthands reset their member longhands via `Style.reset_properties`
even when some members do not yet have typed storage.

**Typed and consumed end-to-end**: box model sizing, margin, padding,
border widths/colors/radii, block/flex/grid layout, basic positioned
layout, background color/clip/URL images, text/font properties,
overflow, visibility, opacity, and selection-related paint data.

**Typed but mocked or not yet consumed**: `transform`,
`transform-origin`, `transition`, `animation`, `box-shadow`,
`z-index`, `cursor`, `pointer-events`, `user-select`, most non-block
`display` variants (`table`, `list-item`, `ruby`, `contents`, ...),
and true sticky positioning. See §10 for downstream behavior.

**Outright unknown** properties: silently dropped (parser's match
falls through). No diagnostics yet.

### 6.4 Property support table

| Prop | Supported | Note |
|---|---:|---|
| `display` | Partial | Block/flex/grid/inline-block paths exist; table/list-item/ruby/contents parse but degrade to existing block/inline semantics. |
| `position` | Partial | `relative`, `absolute`, and `fixed` affect layout; `sticky` parses but behaves relative-like, without sticky scroll thresholds. |
| `top/right/bottom/left` | Partial | Used by relative/out-of-flow positioning; no logical inset support beyond deferred storage. |
| `width`, `height` | Yes | Explicit, auto, percentages, and supported length math resolve in layout. |
| `min-width`, `max-width`, `min-height`, `max-height` | Yes | Used as layout clamps. |
| `margin`, `margin-*` | Yes | Physical shorthand and per-side longhands are consumed; logical margins are deferred only. |
| `padding`, `padding-*` | Yes | Physical shorthand and per-side longhands are consumed; logical padding is deferred only. |
| `box-sizing` | Yes | `content-box` and `border-box`. |
| `border`, `border-*`, `border-width/style/color` | Partial | Width/color consumed; `solid`, `dashed`, `dotted`, `none`, `hidden` paint distinctly; double/groove/ridge/inset/outset degrade to solid. |
| `border-radius`, corner radius longhands | Yes | Includes 1-4 value expansion and elliptical `/` syntax. |
| `color` | Partial | Feeds text color; `currentcolor` is parsed but still weakly resolved. |
| `background-color` | Yes | Painted with clipping/radii. |
| `background` | Partial | Color/image/repeat/clip members handled; multi-layer and many modern members are deferred/ignored. |
| `background-image` | Partial | URL-backed images load/paint; gradients and other functions parse but do not paint. |
| `background-size`, `background-position`, `background-repeat` | Partial | Consumed for URL backgrounds. |
| `background-clip` | Yes | `border-box`, `padding-box`, `content-box`. |
| `background-origin`, `background-attachment`, `background-position-x/y` | Deferred | Recognized/stored but not consumed. |
| `opacity` | Partial | Alpha-multiplied into primitives; no isolated compositing layer. |
| `visibility` | Yes | Cascades/inherits and affects rendering behavior. |
| `z-index` | Parsed only | Stored but no stacking-context or paint-order effect. |
| `overflow`, `overflow-x`, `overflow-y` | Partial | Paint clipping, scroll containers, scrollbars, and hit-test clipping exist; scroll offsets live on `Tree`, not `LayoutBox`. |
| `font-family` | Partial | Registered font matching plus generic fallback; no `@font-face` loading. |
| `font-size`, `font-weight`, `font-style`, `line-height` | Partial | Feed shaping; font-relative computed-value semantics are approximate. |
| `letter-spacing`, `text-align`, `text-transform`, `white-space` | Yes | Consumed by shaping/layout. |
| `text-decoration` | Partial | Underline/overline/line-through paint; color/style/thickness longhands are deferred. |
| `text-indent`, `text-shadow`, `word-spacing`, `overflow-wrap` | Deferred | Recognized where listed in shorthand/deferred tables, not consumed. |
| `flex`, `flex-*`, `justify-content`, `align-*`, `gap`, `order` | Yes | Flexbox path consumes the typed values. |
| `grid-*`, `justify-items`, `justify-self` | Yes | Grid track sizing, placement, auto-flow, and item alignment consume typed values. |
| `cursor` | Parsed only | Stored/inherited, but no resolved cursor API or OS cursor update. |
| `pointer-events` | Parsed only | Stored but hit-testing does not skip `pointer-events: none`; not inherited yet. |
| `user-select` | Parsed only | Stored but document selection does not enforce `none/text/all/auto`; not inherited yet. |
| `transform`, `transform-origin` | Parsed only | Stored as raw values; layout, paint, and hit-testing ignore them. |
| `transition`, `transition-*` | Parsed only | Shorthand member extraction exists; no animation runtime or events. |
| `animation`, `animation-*` | Parsed only | Shorthand member extraction exists; no keyframes/runtime/events. |
| `box-shadow` | Parsed only | Stored but not painted. |
| `outline-*` | Deferred | Recognized/stored but not painted. |
| `scroll-margin-*`, `scroll-padding-*`, `overscroll-*`, timeline props | Deferred | Recognized/stored for future scrolling/timeline work. |
| `list-style-*`, `marker-*`, `columns`, `column-*` | Deferred | Recognized/stored but no list marker or multicol layout behavior. |
| `float`, `clear` | No | Not modeled. |
| `vertical-align` | No | Not modeled in the current `Style`/layout path. |
| `direction`, `unicode-bidi` | Deferred/No | `direction` is deferred; bidi layout semantics are not implemented. |
| `--*`, `var()` | Yes | Custom properties inherit through side-car maps and reparse into typed values where the destination parser accepts them. |
| CSS-wide keywords | Yes | `inherit`, `initial`, `unset` resolve through cascade keyword side-car maps. |
| Unknown properties | No | Silently dropped; no diagnostics. |

## 7. Cascade

**Done** — `lui-style::cascade` + `computed_decls`.

Per CSS-Cascade-3 §6.4, restricted to author + inline origins (no UA
or user origin layers):

1. Author normal rules — matched on `(tag, id, classes, universal)`,
   sorted ascending by specificity (stable sort preserves source
   order on ties).
2. Inline `style="…"` normal declarations.
3. Author `!important` rules — same matching + ordering, but reading
   each rule's parallel `important` payload.
4. Inline `style="…"` `!important` declarations.

Each layer carries two parallel things: a `Style` of values and a
`HashMap<String, CssWideKeyword>` of CSS-wide keyword overrides. The
merge step (`apply_layer`):

- For each property the layer declared as a keyword: clear the
  matching value field, record the keyword in the running map.
- Then merge the layer's values; each `Some(value)` sets the field
  and drops any keyword the running map had for the same property.

After all four bands are applied, `computed_decls` returns the
running `(values, keywords)` pair. `cascade_node` then resolves the
keyword map against the parent's already-resolved style and runs
the implicit inheritance pass.

### 7.1 `!important` — Done

**File**: `lui-parser/src/css_parser.rs::strip_important`,
`lui-parser/src/stylesheet.rs::Rule.important`.

- `prop: value !important;` recognised, with arbitrary whitespace
  between `!` and `important` and case-insensitive `IMPORTANT`.
- A bare `important` keyword without `!` is *not* important.
- Per-rule `important` payloads applied in their own pass after
  normal declarations (CSS-Cascade-3 §6.4 priority bands 4 + 8 in
  our 4-band restriction).
- Within a rule, `color: red; color: blue !important;` resolves
  blue as expected; `color: red !important; color: blue;` keeps red.

**Tests** — `lui-style::tests::important_*`:
- Lower-spec `!important` beats higher-spec normal.
- Among `!important` declarations specificity still orders.
- `!important` author beats inline normal.
- Inline `!important` beats author `!important`.
- `!important` doesn't leak across properties within a rule.
- `! IMPORTANT` whitespace + case variant parses.

### 7.2 CSS-wide keywords (`inherit / initial / unset`) — Done

**File**: `lui-parser/src/style_props.rs`.

Detected case-insensitively per declaration. Each keyword is stored
in the side-car keyword map (`keywords_normal` /
`keywords_important`) instead of the value-`Style`. Resolution
happens at the end of cascade against the parent's resolved style:

| Keyword   | Inherited prop | Non-inherited prop |
|-----------|----------------|--------------------|
| `inherit` | parent value   | parent value       |
| `initial` | `None` (no UA defaults tracked) | `None` |
| `unset`   | parent value (acts like `inherit`) | `None` (acts like `initial`) |

Root element (no parent): `inherit` and `unset` collapse to `None`.

The dispatch macro in `style_props.rs` plus shorthand metadata in
`shorthands.rs` are the source of truth for inherited properties,
keyword fan-out, and shorthand reset behaviour. They are used by:

- `is_inherited(prop)` — drives the `unset` branch.
- `apply_keyword(values, parent, prop, kw)` — per-property
  resolution against the parent.
- `clear_value_for(prop, &mut Style)` — wipe a field, shorthand, or
  deferred longhand when a later layer or the same block declares a
  keyword for that property.
- `merge_values_clearing_keywords(values, keywords, src)` — value
  merge that drops the matching keyword and honours
  `Style.reset_properties` for shorthand member resets.

**Tests** — `lui-style::tests`:
- `inherit` on `background-color` (non-inherited) takes the parent.
- `initial` on `color` blocks the implicit-inheritance pass.
- `unset` is `inherit`-flavoured for `color`, `initial`-flavoured
  for `background-color`.
- Within one block, source order resolves a value vs keyword for
  the same property both ways.
- An `!important inherit` at lower specificity beats an inline
  normal value and forces parent-value resolution.
- A root `color: inherit` collapses to `None`.

### 7.3 Inheritance — Done (with per-property table)

**File**: `lui-style::cascade::inherit_into`.

After the keyword-resolution pass, any typed property still `None` AND
not listed in the keyword map gets the parent's value if the property
is inheritable. Deferred inherited longhands are copied by the same
rule using `lui_parser::is_inherited(prop)`. The typed
inheritable set is:

```
color, font-family, font-size, font-weight, font-style,
line-height, letter-spacing, text-align, text-transform,
white-space, text-decoration, visibility, cursor.
```

`pointer-events` and `user-select` will join this set when
M-INTER-2 wires them into hit-testing / selection enforcement;
they're parsed but not yet inherited.

This list mirrors `is_inherited()` in
`lui-parser/src/style_props.rs` — the same kebab-case strings
are consulted on both the cascade side (for implicit inheritance)
and the keyword-resolution side (for `unset`).

**Missing / future**
- `direction` and `text-orientation` aren't modeled at all.
- `font-size: <percent>` on a child should resolve against the
  parent's *computed* font-size in pixels; today our cascade
  preserves the typed `CssLength::Percent` and layout resolves it
  against the viewport. This is a layout-side gap that lands when
  font-relative length resolution gets implemented (see §10).

## 8. Stylesheet sources

**Done**
- Inline element `style="..."` attribute (per-element).
- `<style>` element bodies anywhere in the document — gathered into a
  single `Stylesheet` at cascade time
  (`lui-style::collect_stylesheet`).
- **UA default stylesheet** (`lui-style/src/ua.rs`) — `display:
  none` for `<head>/<style>/<script>/…`, `body { margin: 8px }`,
  heading sizes/weights (`h1`–`h6`), block-level margins, inline
  emphasis (`b, strong, a, code, …`). Injected as the lowest-priority
  `Stylesheet` in the cascade.

**Missing**
- `<link rel="stylesheet">` — the parser captures the `href` but
  nothing fetches.
- `@import url(...)` — at-rules aren't parsed; would need a host
  resolver.

## 9. Computed values

The cascade output (`CascadedTree`) carries the same typed `Style`
struct the parser populated, with one transformation: keyword
overrides are resolved (or collapsed). Computed-value details still
to do:

- **Length resolution.** Layout still receives raw `CssLength`
  values; the resolution to physical pixels happens in
  `lui-layout-old::length::resolve` against viewport / parent
  size. CSS spec calls for this to happen at "computed value" time
  for `em`/`rem`/`%` of the element's own font size — we
  approximate.
- **`em` / `rem`.** Currently resolved against a hard-coded 16px
  baseline. Doesn't track font-size cascade.
- **Color.** Stays a `CssColor` enum. `currentcolor` doesn't
  resolve against the element's own `color` property yet.

## 10. Layout / paint consumption

What survives the cascade and actually changes pixels on the screen.

### Honoured by layout (`lui-layout-old`)

- `display` (block, flex, grid, and atomic inline variants such as
  `inline-block` / `inline-flex` where the current layout path
  supports them). Several parsed display values (`table`,
  `list-item`, `ruby`, `contents`, etc.) currently degrade into the
  existing block/inline paths rather than implementing their CSS
  semantics.
- `position` basic support: `relative` offsets the normal-flow box;
  `absolute` and `fixed` are laid out out-of-flow with physical
  insets. `sticky` is parsed but behaves like relative positioning;
  there is no scroll-threshold sticky behavior.
- `width, height, min/max-{width,height}` — honoured for explicit
  values; `auto` and percentages computed against parent content
  width.
- `margin`, `padding` (per-side or shorthand fallback).
- `box-sizing` (`content-box` / `border-box`).
- `border` width/style/color per side, plus radius (incl. elliptical).
- `background-color`, `background-clip`, `background-image`,
  `background-position`, `background-size`, `background-repeat`
  (URL-backed images only; function images are parsed but not painted).
- `flex-direction`, `flex-wrap`, `justify-content`, `align-items`,
  `align-content`, `align-self`, `order`, `gap`, `row-gap`,
  `column-gap`, `flex-grow`, `flex-shrink`, `flex-basis`.
- Grid track sizing and placement from the typed `grid-*` longhands.
- `color`, `font-size`, `font-weight`, `font-style`,
  `line-height`, `font-family` — feed into text shaping (`spec/text.md`),
  with the inheritance pass making them flow through the document.
  Generic font families (`sans-serif`, `serif`, `monospace`, …) fall
  back to any registered face when no listed family matches; see §6.2.1.
- `letter-spacing`, `text-align`, `text-decoration`,
  `text-transform`, `white-space`.
- `opacity` is carried to paint and alpha-multiplied into painted
  primitives, but does not create an isolated compositing layer.
- **`<input>` / `<textarea>` `placeholder` attribute** — when the
  field has no `value` (and a textarea has no children), layout
  shapes the placeholder text and attaches it as the box's
  `text_run`, painted at `cascaded color × alpha 0.5` (the default
  browser `::placeholder` styling). Single-line inputs vertically
  centre and horizontally clip the run; textareas soft-wrap inside
  `content_rect.w`. `type="hidden"` and non-empty `value` /
  textarea content suppress the placeholder.

### Honoured by paint (`lui`)

- Per-side border colors / styles (solid, dashed, dotted; double /
  groove / ridge / inset / outset render as solid).
- Background fills with corner radii.
- Glyph quads emitted from shaped text runs (text color resolved
  from the cascaded `color`).
- Selection-highlight rectangles and (future) caret overlay; see
  `spec/interactivity.md` §11.

### Recognised but ignored everywhere

`z-index`, `transform`, `transform-origin`, `transition`,
`animation`, `box-shadow`, `cursor`, `pointer-events`,
`user-select`, logical `margin/padding/inset/border-*` longhands,
`background-origin`, `background-attachment`, multi-layer background
members, `outline-*`, `overscroll-*`, `scroll-margin-*`,
`scroll-padding-*`, `text-emphasis-*`, timeline-related properties,
and most other deferred longhands.

Also missing or mocked:
- `float`, `clear`, and `vertical-align` are not modeled.
- Background image functions such as gradients are parsed as
  functions but only URL-backed images currently paint.
- `currentcolor` is parsed but still weakly resolved; properties that
  rely on it may fall back or disappear instead of using the element
  `color`.
- Border styles `double`, `groove`, `ridge`, `inset`, and `outset`
  are accepted but paint as solid.

## 11. Phases

Each phase ends in something a host can demo or test against.

### C1 — Parser-side property table — ✅

- `lui-parser/src/style_props.rs` is the single source of
  truth: every supported property listed exactly once with
  `(struct_field, "kebab-case", inherited?)`.
- The macro generates `clear_value_for`, `apply_keyword`,
  `is_inherited`, `merge_values_clearing_keywords` from the table.
- Re-used by `parse_inline_style_decls` (within-layer mutual
  exclusion of values vs keywords) and by the cascade.

### C2 — `!important` — ✅

- Per-declaration recognition with whitespace + case tolerance.
- `Rule.important` parallel `Style`, plus
  `Rule.important_keywords`.
- 4-band cascade order in `computed_decls`.

### C3 — CSS-wide keywords — ✅

- `inherit`, `initial`, `unset` per CSS-Values-3 §6.1.
- Resolved against parent in `cascade_node`.
- Suppress implicit inheritance for keyword-touched properties.

### C4 — Combinators (descendant / child / sibling)

- Descendant combinator — ✅ done.
- Child and sibling combinators still need parser + matcher support.
- Selector `Vec<SimpleSelector>` should be extended with explicit
  combinator type for `>`, `+`, and `~`.
- Specificity rules: combinators don't add to specificity.
- Tests should cover `div > p`, `h1 + p`, `h1 ~ p`.

### C5 — Pseudo-classes (state + structural)

- **State pseudo-classes `:hover`, `:active`, `:focus` — ✅ done.**
  Parsed in `stylesheet.rs`; matched via
  `MatchContext { is_hover, is_active, is_focus }` derived in
  `lui-style::cascade` from `InteractionState`'s
  `hover_path` / `active_path` / `focus_path`. `:focus` is
  exact-match (only the focused element, not its ancestors);
  `:focus-within` (which would propagate) is not yet implemented.
  See `spec/interactivity.md` for the interaction-state wiring.
- **Remaining state pseudo-classes** (`:focus-visible`,
  `:focus-within`, `:disabled`, `:checked`) — not yet matched.
  Note that `is_focusable` already excludes `disabled` form
  controls from focus traversal, so a separate `:disabled`
  cascade hook is the only missing piece for that one.
- Structural `:root`, `:first-child`, `:last-child` — ✅ done.
- Remaining structural (`:nth-child(...)`, `:only-child`, `:empty`)
  — purely tree-shape; doable independently.
- `:visited` parses but deliberately never matches because link
  history is not tracked.
- Logical (`:not(...)`, `:is(...)`, `:where(...)`).
- Supported pseudo-classes currently add to the class-specificity
  bucket. Future logical pseudo-classes need their Selectors-4
  specificity exceptions (`:where()` = 0, `:is()` / `:not()` use
  their argument specificity).

### C6 — Attribute selectors

- `[attr]` and `[attr=value]` — ✅ done.
- Remaining operators: `[attr~=value]`, `[attr|=value]`,
  `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, plus the
  case-insensitive `i` flag.
- Adds 10 to specificity.

### C7 — `@media` queries — ✅ Done

Tokenised `@media (…) { … }` blocks are held as gated `Rule` payloads
inside each `Stylesheet`. At cascade time, `cascade_with_media(tree,
&media_ctx)` evaluates each block's query list against viewport width,
height, orientation, and scale. Supported features: `min-width`,
`max-width`, `min-height`, `max-height`, `orientation`, and the `not`
prefix for negation. Demo re-cascades on resize every frame, so
responsive breakpoints work out of the box.

### C8 — `@import url(...)`

- Parse the at-rule.
- Host-supplied resolver (`Tree::set_css_resolver` or similar) to
  fetch / read the referenced sheet.
- Concatenate before the importing stylesheet.

### C9 — UA default stylesheet — ✅ Done

The UA stylesheet lives in `lui-style/src/ua.rs` and is injected
as the lowest-priority `Stylesheet` in every cascade pass. It covers:
`display: none` for non-rendered elements (`<head>`, `<style>`,
`<script>`, …), `body { margin: 8px }`, heading sizes and weights
(`h1`–`h6`), block-level margins (`p`, `ul`, `ol`, `dl`, …), and
inline emphasis (`b`, `strong`, `em`, `i`, `u`, `s`, `code`, `a`,
`mark`, `small`, `sub`, `sup`). See `spec/text.md` §2 for details.

### C10 — `@font-face`

- Already discussed in `spec/text.md` §12. Parses `src: url(...)`
  through a host-supplied resolver into a synthetic
  `Tree::register_font(...)` call.

### C11 — `var(--foo)` / custom properties — ✅ Done

- Custom properties (any `--*` token) are captured as side-car strings
  and inherited through the cascade.
- `var(--foo, fallback)` is resolved before reparsing the destination
  property into typed `Style` storage.
- Limitations: no CSSOM, no diagnostics, and variables only have
  visible effect where the resolved value is accepted by the existing
  typed parser.

### C12 — `calc()` / `min()` / `max()` / `clamp()` — ✅ Done

- Function bodies are tokenised into typed length math.
- Layout resolves the common length math functions against the same
  context used for plain `CssLength` values.
- Advanced numeric functions and contexts outside length resolution
  may still degrade through raw or unsupported paths.

### C13 — Pseudo-elements

- `::before` / `::after` — generated content, requires an
  anonymous box layer the engine doesn't have yet.
- Out of scope until the inline formatting context is in place.

---

## 12. Open questions

- **Property → field map duplication.** `style_props.rs` and the
  parser's `apply_css_property` independently list every property
  name. We're one mismatch away from silent breakage. A shared
  `proc_macro` or `build.rs` could collapse them — worth doing once
  the property count starts growing.
- **Ordering vs source order on selector ties.** We rely on stable
  sort + insertion order. The parser walks a string left-to-right
  so ordering is deterministic, but a documented test case would
  help.
- **Initial values without a UA sheet.** The UA sheet now exists
  (`lui-style/src/ua.rs`), so `initial` collapsing to `None`
  is less dangerous than before. However `initial` still resolves to
  `None` (not the CSS specified-initial value), which means the
  `keywords.contains_key($name)` guard in `inherit_into` is what
  keeps "`initial` blocks inheritance" working — any future refactor
  of the cascade has to keep the keyword map alive long enough for
  that check.
- **Whitespace-only text.** Currently dropped at tree-build
  (`docs/full-status.md` §1). Once the inline formatting context
  arrives, we'll need to keep at least the runs that sit between
  inline children.

---

## Summary

What works end-to-end today: simple and descendant selectors with full
specificity-ordered + `!important`-aware + CSS-wide-keyword-aware
cascade, implicit inheritance for the standard inheriting set, a UA
default stylesheet, attribute presence/equality selectors, dynamic
`:hover` / `:active` / `:focus` and simple structural pseudo-classes,
custom properties with `var()` substitution, length math
(`calc` / `min` / `max` / `clamp`), and a shared property-dispatch
table that's the single source of truth for the parser ↔ cascade
boundary.

What doesn't: remaining combinators (child/sibling) and most pseudo-
classes in the *stylesheet parser + cascade matcher* (notably
`:focus-visible`, `:focus-within`, `:disabled`, `:checked`,
`nth-*`, logical) — though the `query_selector*` engine supports
all of these. Also missing: pseudo-elements, `<link>` stylesheet
loading, reliable `currentcolor` resolution, font-relative length
resolution, and many recognized-but-mocked visual properties such as
transforms, shadows, z-index stacking, and transitions/animations.

C1–C3 land the cascade machinery and are the foundation for
everything that follows; C4–C7 unlock realistic stylesheets; C8–C10
and C13 are reach goals tied to specific subsystems, while C11–C12
are partial/done and now mostly need hardening around edge cases.
