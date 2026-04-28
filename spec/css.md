# wgpu-html — CSS Spec

The CSS pipeline this engine implements, what's already wired up, and
what's deliberately out-of-scope or left for follow-ups. Companion to
`spec/text.md` (text rendering) and `docs/status.md` (broader engine
snapshot).

This file is the source of truth for "what does our CSS support look
like today and where is it heading".

---

## 1. Goals

- A typed, fully-resolved `Style` per element, computed once per
  cascade pass. Layout and paint never re-parse CSS.
- Standards-faithful enough to handle the subset of CSS the renderer
  actually paints (block + flex layout, backgrounds, borders, text).
- One source of truth for the property table: `wgpu-html-parser`
  knows the kebab-case names; `wgpu-html-models::Style` is the typed
  shape; `wgpu-html-style` consumes the cascade output.
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
   ▼  wgpu-html-parser            tokenize + tree-build + parse <style> bodies
Tree<Node<Element>>                inline `style="…"` attrs stay raw on each element
   │
   ▼  wgpu-html-style::cascade    selector match + 4-band cascade + keyword
                                  resolution + implicit inheritance
CascadedTree                       per-node fully-resolved Style
   │
   ▼  wgpu-html-layout            consume the typed Style values
LayoutBox tree
```

The parser owns:

- Tokenisation (`tokenizer.rs`).
- Tree building (`tree_builder.rs`).
- CSS declaration parsing (`css_parser.rs`).
- Selector + rule parsing (`stylesheet.rs`).
- The shared property dispatch table (`style_props.rs`).

`wgpu-html-style` owns the cascade itself: matching, ordering,
keyword resolution, inheritance.

---

## 4. Selectors

**Done** — `wgpu-html-parser/src/stylesheet.rs::parse_selector`.

| Form        | Example               | Notes                                  |
|-------------|-----------------------|----------------------------------------|
| Tag         | `div`                 |                                        |
| Id          | `#hero`               |                                        |
| Class       | `.card`               | Multi-class via repetition: `.a.b`     |
| Universal   | `*`                   | Combines with other simple constraints |
| Compound    | `div#hero.card.big`   | All conditions on one element          |
| Selector list (comma) | `h1, h2, .big` | Each comma-separated entry is its own simple selector |

Specificity (CSS-Selectors-3) packed into `u32`:
`(id_count << 16) | (class_count << 8) | tag_count`. Comparing as
plain integers gives the right ordering.

**Missing / partial**
- **Descendant combinator is implemented**. Child `>`, adjacent `+`,
  and general sibling `~` are still rejected and drop the rule.
- **Dynamic pseudo-classes implemented**: `:hover`, `:active`.
- **No attribute selectors**: `[href]`, `[type="text"]`, etc.
- Structural / logical pseudo-classes still missing: `:focus`,
  `:nth-child`, `:not()`, `:is()`, `:where()`, `:checked`, etc.
- **No pseudo-elements**: `::before`, `::after`, `::first-line`,
  `::placeholder`, etc.
- **No namespaces** (`@namespace`, `ns|tag`).

Selector matching supports tree-aware descendant checks plus a
stateful `MatchContext` for `:hover` / `:active`.

## 5. At-rules

**None implemented.** `@media`, `@supports`, `@import`,
`@font-face`, `@keyframes`, `@page`, `@layer`, `@scope`, `@property`
are all unhandled. The stylesheet parser walks `selectors { decls }`
blocks only — anything else is skipped silently.

Closest planned: `@font-face` (`spec/text.md` §12 open question — needs
a host-supplied font resolver to honour `src: url(...)`).

## 6. Property parsing

Source: `wgpu-html-parser/src/css_parser.rs::apply_css_property`
plus its per-property value parsers.

### 6.1 Length values

`parse_css_length` recognises `px`, `%`, `em`, `rem`, `vw`, `vh`,
`vmin`, `vmax`, `auto`, bare `0`. Unknown shapes drop into a `Raw`
fallback that layout treats as zero.

Also supported:
- `calc(...)`, `min(...)`, `max(...)`, `clamp(...)` — parsed into a
  typed math tree and resolved later by layout.

Not yet supported:
- `var(--foo)` / custom properties.
- `ch`, `ex`, `lh`, container-query units (`cqw`/`cqh`/…).

### 6.2 Color values

`parse_css_color` recognises:
- Named colors (~20 common ones — see `wgpu-html-style::color`).
- `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`.
- `rgb(r, g, b)`, `rgba(r, g, b, a)` with comma or slash-alpha.
- `hsl(h, s, l)`, `hsla(h, s, l, a)`.
- `transparent`, `currentcolor`.

Also accepted and preserved as function values:
- `hwb()`, `lab()`, `lch()`, `oklab()`, `oklch()`, `color()`,
  `color-mix()`, `light-dark()`.

Not yet fully resolved:
- Color-mix / color-contrast functions.
- Wide-gamut color spaces.
- `currentcolor` resolution (parsed into `CssColor::CurrentColor` but
  layout currently returns `None` for it).

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
future-facing longhands listed in `wgpu-html-parser/src/shorthands.rs`.

For `animation` and `transition`, the parser now performs per-layer
member extraction with defaults (`0s`, `ease`, `running`, etc.) rather
than only storing the raw shorthand string.

Shorthands reset their member longhands via `Style.reset_properties`
even when some members do not yet have typed storage.

**Recognised but ignored downstream** — see §10 below.

**Outright unknown** properties: silently dropped (parser's match
falls through). No diagnostics yet.

## 7. Cascade

**Done** — `wgpu-html-style::cascade` + `computed_decls`.

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

**File**: `wgpu-html-parser/src/css_parser.rs::strip_important`,
`wgpu-html-parser/src/stylesheet.rs::Rule.important`.

- `prop: value !important;` recognised, with arbitrary whitespace
  between `!` and `important` and case-insensitive `IMPORTANT`.
- A bare `important` keyword without `!` is *not* important.
- Per-rule `important` payloads applied in their own pass after
  normal declarations (CSS-Cascade-3 §6.4 priority bands 4 + 8 in
  our 4-band restriction).
- Within a rule, `color: red; color: blue !important;` resolves
  blue as expected; `color: red !important; color: blue;` keeps red.

**Tests** — `wgpu-html-style::tests::important_*`:
- Lower-spec `!important` beats higher-spec normal.
- Among `!important` declarations specificity still orders.
- `!important` author beats inline normal.
- Inline `!important` beats author `!important`.
- `!important` doesn't leak across properties within a rule.
- `! IMPORTANT` whitespace + case variant parses.

### 7.2 CSS-wide keywords (`inherit / initial / unset`) — Done

**File**: `wgpu-html-parser/src/style_props.rs`.

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

**Tests** — `wgpu-html-style::tests`:
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

**File**: `wgpu-html-style::cascade::inherit_into`.

After the keyword-resolution pass, any typed property still `None` AND
not listed in the keyword map gets the parent's value if the property
is inheritable. Deferred inherited longhands are copied by the same
rule using `wgpu_html_parser::is_inherited(prop)`. The typed
inheritable set is:

```
color, font-family, font-size, font-weight, font-style,
line-height, letter-spacing, text-align, text-transform,
white-space, text-decoration, visibility, cursor.
```

This list mirrors `is_inherited()` in
`wgpu-html-parser/src/style_props.rs` — the same kebab-case strings
are consulted on both the cascade side (for implicit inheritance)
and the keyword-resolution side (for `unset`).

**Missing / future**
- No UA default stylesheet, so `<a>` isn't blue/underlined, `<h1>`
  has no default size, etc. Hosts must declare baselines in their
  own stylesheet. Adding a UA default sheet would be a one-shot
  prepended `Stylesheet` injected at cascade time.
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
  (`wgpu-html-style::collect_stylesheet`).

**Missing**
- `<link rel="stylesheet">` — the parser captures the `href` but
  nothing fetches.
- `@import url(...)` — at-rules aren't parsed; would need a host
  resolver.
- User stylesheets / UA stylesheet (would slot in as additional
  `Stylesheet` instances merged before author rules).

## 9. Computed values

The cascade output (`CascadedTree`) carries the same typed `Style`
struct the parser populated, with one transformation: keyword
overrides are resolved (or collapsed). Computed-value details still
to do:

- **Length resolution.** Layout still receives raw `CssLength`
  values; the resolution to physical pixels happens in
  `wgpu-html-layout::length::resolve` against viewport / parent
  size. CSS spec calls for this to happen at "computed value" time
  for `em`/`rem`/`%` of the element's own font size — we
  approximate.
- **`em` / `rem`.** Currently resolved against a hard-coded 16px
  baseline. Doesn't track font-size cascade.
- **Color.** Stays a `CssColor` enum. `currentcolor` doesn't
  resolve against the element's own `color` property yet.

## 10. Layout / paint consumption

What survives the cascade and actually changes pixels on the screen.

### Honoured by layout (`wgpu-html-layout`)

- `display` (block, flex, grid, and atomic inline variants such as
  `inline-block` / `inline-flex` where the current layout path
  supports them).
- `position` ignored — `top/right/bottom/left` parsed but unused.
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
- `letter-spacing`, `text-align`, `text-decoration`,
  `text-transform`, `white-space`.

### Honoured by paint (`wgpu-html`)

- Per-side border colors / styles (solid, dashed, dotted; double /
  groove / ridge / inset / outset render as solid).
- Background fills with corner radii.
- Glyph quads emitted from shaped text runs (text color resolved
  from the cascaded `color`).

### Recognised but ignored everywhere

`z-index`, `transform`, `transform-origin`, `transition`,
`animation`, `box-shadow`, logical `margin/padding/inset/border-*`
longhands, `background-origin`, `background-attachment`,
multi-layer background members, `outline-*`, `overscroll-*`,
`scroll-margin-*`, `scroll-padding-*`, `text-emphasis-*`,
timeline-related properties, and most other deferred longhands.

## 11. Phases

Each phase ends in something a host can demo or test against.

### C1 — Parser-side property table — ✅

- `wgpu-html-parser/src/style_props.rs` is the single source of
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

- Selector parser and matcher need ancestor / sibling context.
- Selector `Vec<SimpleSelector>` chained by combinator type.
- Matcher walks the tree (or a parent stack) for descendant /
  ancestor relationships.
- Specificity rules: combinators don't add to specificity.
- Tests should cover `div p`, `div > p`, `h1 + p`, `h1 ~ p`.

### C5 — Pseudo-classes (state + structural)

- State pseudo-classes (`:hover`, `:focus`, `:active`,
  `:checked`, `:disabled`) — depends on input handling
  (`spec/devtools.md` D3+).
- Structural (`:nth-child(...)`, `:first-child`, `:last-child`,
  `:only-child`, `:empty`) — purely tree-shape; doable today.
- Logical (`:not(...)`, `:is(...)`, `:where(...)`).
- Each adds 0/10/100 to specificity per Selectors-4.

### C6 — Attribute selectors

- `[attr]`, `[attr=value]`, `[attr~=value]`, `[attr|=value]`,
  `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, plus the
  case-insensitive `i` flag.
- Adds 10 to specificity.
- Needs the parser to retain attributes the typed model dropped
  (today many attributes are parsed into struct fields and the
  raw key/value pairs are discarded).

### C7 — `@media` queries

- Tokenise `@media (...)` blocks, hold them as gated `Stylesheet`s
  inside the parent.
- Evaluate against viewport size + scale at cascade time.
- Re-cascade on resize (the demo already re-runs cascade every
  frame, so practically just a viewport→media-context plumbing
  task).

### C8 — `@import url(...)`

- Parse the at-rule.
- Host-supplied resolver (`Tree::set_css_resolver` or similar) to
  fetch / read the referenced sheet.
- Concatenate before the importing stylesheet.

### C9 — UA default stylesheet

- Pre-canned `Stylesheet` covering the obvious HTML defaults
  (`body { margin: 8px; }`, `h1 { font-size: 2em; font-weight:
  bold; }`, `a { color: -webkit-link; text-decoration: underline;
  }`, etc.).
- Slots in below author rules in the cascade (lowest origin).
- Most useful once `@font-face` / generic font families land —
  before then, hosts must do this themselves.

### C10 — `@font-face`

- Already discussed in `spec/text.md` §12. Parses `src: url(...)`
  through a host-supplied resolver into a synthetic
  `Tree::register_font(...)` call.

### C11 — `var(--foo)` / custom properties

- Custom properties (any `--*` token) are inherited like normal
  inheritable properties.
- `var(--foo, fallback)` substitution at computed-value time.
- Independent of the typed `Style` struct: keep custom-property
  resolution as a side-car string map evaluated before the main
  property parsers.

### C12 — `calc()` / `min()` / `max()` / `clamp()`

- Tokenise the function bodies into typed AST.
- Resolve against the same context layout uses for plain
  `CssLength` values.

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
- **Initial values without a UA sheet.** Right now `initial`
  collapses to `None`, which means inherited-by-the-cascade-pass
  could re-fill it later if we're not careful. The
  `keywords.contains_key($name)` guard in `inherit_into` is what
  keeps that working — any future refactor of the cascade has to
  keep the keyword map alive long enough for that check.
- **Whitespace-only text.** Currently dropped at tree-build
  (`docs/status.md` §1). Once the inline formatting context
  arrives, we'll need to keep at least the runs that sit between
  inline children.

---

## Summary

What works end-to-end today: simple selectors with full
specificity-ordered + `!important`-aware + CSS-wide-keyword-aware
cascade, implicit inheritance for the standard inheriting set, and
a shared property-dispatch table that's the single source of truth
for the parser ↔ cascade boundary.

What doesn't: combinators, pseudo-anything, attribute selectors,
all at-rules (`@media` / `@import` / `@font-face` / `@keyframes` /
…), `calc()` / `var()`, UA defaults, `currentcolor` resolution,
font-relative length resolution.

C1–C3 land the cascade machinery and are the foundation for
everything that follows; C4–C7 unlock realistic stylesheets; C8–C13
are reach goals tied to specific subsystems landing first.
