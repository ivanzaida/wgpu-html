---
title: CSS Overview
---

# CSS in lui

lui implements a full CSS property parsing and cascade pipeline designed to mirror real browser behaviour while targeting embedded Rust UI. The CSS engine lives primarily in the `lui-parser` (tokenization, selector parsing, declaration parsing) and `lui-style` (cascade resolution, inheritance, selector matching) crates.

## Parsing Pipeline

1. **Tokenization** — inline `<style>` blocks and linked stylesheets are collected into a single CSS string.
2. **Stylesheet parsing** — the CSS string is split into `selectors { declarations }` rule blocks. `/* comments */` are stripped during this phase. `@media` queries wrapping rule blocks are parsed. `@charset` declarations are recognized and skipped (Rust strings are always UTF-8). `@import` directives are resolved by inlining the referenced CSS from `linked_stylesheets` (with media query wrapping and cycle detection). Other at-rules (`@keyframes`, `@font-face`) are not supported.
3. **Selector parsing** — each comma-separated selector is decomposed into tag, `#id`, `.class` compounds with descendant combinators (`A B`).
4. **Declaration parsing** — each `property: value;` declaration is parsed into a typed Rust enum or struct field. `!important` is recognised and flagged.
5. **Cascade resolution** — the `lui-style` crate walks the DOM tree and for each element computes the final `Style` struct by:
   - Collecting matching UA default rules
   - Collecting matching author rules (sorted by specificity)
   - Overlaying inline `style="..."` attributes
   - Resolving `!important` in the correct band order
   - Applying inheritance for inheritable properties
   - Resolving CSS-wide keywords (`inherit`, `initial`, `unset`)
   - Resolving `var()` references for custom properties

The output is a `CascadedTree<CascadedNode>` where every element has a fully resolved `Style` struct (~80 fields). Layout consumes this tree and never re-parses CSS.

## Supported Value Types

| Category | Types |
|---|---|
| **Lengths** | `px`, `%`, `em`, `rem`, `vw`, `vh`, `vmin`, `vmax`, `auto`, `0`, `calc()`, `min()`, `max()`, `clamp()` |
| **Colors** | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()`, `hsl()`, `hsla()`, ~20 named colors, `transparent`, `currentColor`, CSS Color Level 4 system colors |
| **Keywords** | Property-specific enumerated keywords (e.g., `flex`, `block`, `hidden`, `solid`) |
| **Numbers** | Bare integers for `order`, `z-index`, `flex-grow` |
| **Strings** | Font family names, URL strings, cursor names |
| **Functions** | `calc()`, `min()`, `max()`, `clamp()`, `var()`, `url()`, trigonometric/math (18 AST node types) |

## CSS-Wide Keywords

Every CSS property accepts three CSS-wide keywords:

- **`inherit`** — forces the property to use the parent's computed value, even for non-inherited properties.
- **`initial`** — resets the property to its CSS specification initial value (`None` for optional fields, their default for enums).
- **`unset`** — behaves as `inherit` for inherited properties, `initial` for non-inherited ones.

These are tracked in side-car `HashMap<String, CssWideKeyword>` maps during cascade and resolved against the parent's already-resolved `Style` before inheritance runs.

## `!important` Support

`!important` is fully implemented with correct CSS-Cascade-3 band ordering. Important declarations are parsed into a separate `Style` payload and applied *after* all normal declarations, in a distinct cascade pass:

1. Author normal rules (ascending specificity)
2. Inline normal declarations
3. Author `!important` rules (ascending specificity)
4. Inline `!important` declarations

## Stylesheet Loading

Stylesheets are collected from two sources:

1. **Inline `<style>` blocks** — the cascade engine walks the DOM tree, extracts text content from all `<style>` elements, and concatenates them. If a `<style>` element has a `media` attribute, its content is wrapped in an `@media { }` block.

2. **Linked stylesheets** — the `Tree` struct holds a `linked_stylesheets: HashMap<String, String>` field. Keys are `href` values from `<link rel="stylesheet" href="...">` elements; values are the CSS source. There is no built-in HTTP fetch to populate this map — the host application is responsible for loading linked CSS and inserting it.

Both sources feed into a single parsed `Stylesheet` that is cached per CSS source string via a global `OnceLock<Mutex<HashMap>>` cache.

## UA Default Stylesheet

The user-agent stylesheet is a static `&'static str` compiled into `lui-style` that provides browser-consistent defaults:

- `<head>`, `<style>`, `<script>`, `<noscript>`, `<template>`, `<title>`, `<base>`, `<link>`, `<meta>` → `display: none`
- `<body>` → `display: block; margin: 8px`
- `<h1>`–`<h6>` → block display with appropriate font sizes (2em–0.67em), bold weight, and vertical margins
- Block-level elements (`<p>`, `<ul>`, `<ol>`, `<dl>`, `<blockquote>`, `<figure>`, etc.) → `display: block` with appropriate margins
- Inline emphasis (`<b>`, `<strong>`, `<em>`, `<i>`, `<u>`, `<s>`, `<code>`, `<a>`, `<mark>`, `<small>`, `<sub>`, `<sup>`) → font-weight, font-style, text-decoration, color as appropriate
- `<input>`, `<button>`, `<textarea>`, `<select>` → inline-block display with UA form styling
- Form control styles use CSS Color Module Level 4 system colors (`buttonface`, `field`, `highlight`, etc.)

UA rules use tag selectors only, so they sit at the bottom of the author-normal cascade band. An author tag rule with the same name wins on source order (the UA rules are emitted first).

## Cascade Interaction

The CSS cascade interacts with the DOM through `MatchContext` — a per-element struct computed from the document's `InteractionState`. This enables dynamic pseudo-classes:

- `:hover` — matches when the element's path is a prefix of `state.hover_path`
- `:active` — matches when the element's path is a prefix of `state.active_path`  
- `:focus` — matches when the element's path exactly equals `state.focus_path`

When interaction state changes, an incremental re-cascade runs: only the affected paths are re-evaluated, and if all pseudo-class rules affect only paint properties (not layout), the layout pass is skipped entirely.

## Differences from Full CSS Browser Support

### Fully working

- **Full CSS Level 4 selectors** in cascade — all combinators (`>`, `+`, `~`), `:is()`, `:where()`, `:not()`, `:has()`, `:nth-child(An+B of S)`, 30+ pseudo-classes, attribute selectors with case-insensitive flag
- **Pseudo-elements** — `::before`, `::after`, `::first-line`, `::first-letter`, `::placeholder`, `::selection`, `::marker`
- **`@media`** queries — `width`/`height`/`orientation` with `min`/`max` and `not`
- **CSS gradients** — `linear-gradient()`, `radial-gradient()`, `conic-gradient()` + repeating variants
- **`currentColor`** — resolves to the element's computed `color` value
- **Canvas background** — root/body background propagates to fill the viewport per CSS 2.2 section 14.2

### Not yet implemented

- **At-rules** — `@charset` is recognized and skipped (UTF-8 only); `@import` is fully supported; no `@keyframes`, `@font-face`, `@supports`, `@layer`, `@container`, `@scope`
- **Animations & transitions** — `animation-*` and `transition-*` properties parsed but no animation engine
- **`box-shadow`**, `transform`, `transition`, `animation` — stored as raw strings, not consumed by layout or paint
- **`float`** property — not parsed
- **Table layout** — `display: table` variants parsed but fall through to block layout
- **Multi-column** — `column-count`/`column-width` parsed as deferred longhands, never consumed
- **`z-index`** — parsed and used for sibling sort order, but no independent cross-branch stacking contexts
