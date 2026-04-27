# wgpu-html — Implementation Status

A snapshot of what is actually wired up in the codebase versus what is
parsed-but-ignored or not modeled at all. Companion to `roadmap.md`,
which describes the intended milestones; this file describes the
current reality.

Last surveyed against:
`crates/wgpu-html-{parser, models, tree, style, layout, renderer, demo}`
and `crates/wgpu-html/src/paint.rs`.

---

## Pipeline at a glance

```
HTML/CSS string
   │
   ▼  wgpu-html-parser           Tokenizer + tree builder + CSS parser
Tree<Node<Element>>
   │
   ▼  wgpu-html-style            Selector match + cascade + inline override
CascadedTree<CascadedNode>       Per-node Style, no inheritance
   │
   ▼  wgpu-html-layout           Block flow + flex; no inline / text yet
LayoutBox tree                   margin/border/content rects, radii, colors
   │
   ▼  wgpu-html (paint.rs)       LayoutBox → DisplayList<Quad>
DisplayList                      Solid / rounded / stroked / patterned quads
   │
   ▼  wgpu-html-renderer         One instanced-quad pipeline (SDF in WGSL)
Frame on surface
```

---

## 1. HTML parsing — `crates/wgpu-html-parser`

**Done**
- Tokenizer (`tokenizer.rs`): open / close / self-closing tags, attributes
  (quoted, unquoted, boolean), comments, DOCTYPE, raw-text elements
  (`script`, `style`, `textarea`, `title`), entity decoding
  (`&amp; &lt; &gt; &quot; &apos; &nbsp; &#NN; &#xNN;`).
- Tree builder (`tree_builder.rs`): 14-entry void list, self-closing
  recognition, auto-close rules for `p / li / dt / dd / thead / tbody /
  tfoot / tr / th / td / option / optgroup / rt / rp`, EOF auto-close,
  synthetic `<body>` wrap when there are multiple top-level nodes,
  unknown-tag tolerant skip.
- ~100 element variants (`wgpu-html-tree/src/lib.rs`) with per-element
  attribute parsing in `attr_parser.rs` (e.g. `<a>`, `<img>`, `<input>`,
  `<form>` are all structured).
- Global attributes: `id, class, style, title, lang, dir, hidden,
  tabindex, accesskey, contenteditable, draggable, spellcheck,
  translate, role`.
- `aria-*` and `data-*` captured into per-element
  `aria_attrs: HashMap<String, String>` and
  `data_attrs: HashMap<String, String>` (key = the suffix after the
  prefix). All attributes preserved.

**Missing / partial**
- Comments + DOCTYPE are tokenized then **dropped** at tree-build time.
- Unknown tags drop their **entire subtree** silently — no diagnostics.
- No HTML5 insertion-mode state machine (no `<table>` foster-parenting,
  no `</br>` → `<br>` quirk, etc.).
- No `<![CDATA[ ]]>`, no foreign content (SVG / MathML inner nodes).
- Whitespace-only text between tags is dropped — fine today (no inline
  layout) but will need revisiting for M5 / M6.

## 2. CSS parsing — `crates/wgpu-html-parser`

**Done — `apply_css_property` (`css_parser.rs`)**
- Box: `display, position, top/right/bottom/left, width, height,
  min-/max-width/height`.
- Spacing: `margin`, `padding`, both with 1/2/3/4-value shorthand
  expansion; per-side longhands.
- Backgrounds: `background-color, background-clip, background-repeat`;
  `background, background-image, background-size, background-position`
  stored as **raw strings** only.
- Borders: `border` shorthand; per-side `border-{top,right,bottom,left}`
  and `-{width,style,color}` longhands; `border-radius` with `/`-
  separated elliptical syntax and 1–4-corner expansion; per-corner
  `<h> <v>` longhands.
- Typography: `color, font-family, font-size, font-weight, font-style,
  line-height, letter-spacing, text-align, text-transform, white-space,
  text-decoration` (most as raw strings — see below).
- Misc: `overflow / -x / -y, opacity, visibility, z-index`.
- Flex: `flex-direction, flex-wrap, justify-content, align-items,
  align-content, gap, row-gap, column-gap, flex, flex-grow,
  flex-shrink, flex-basis`.
- Length units: `px, %, em, rem, vw, vh, vmin, vmax, auto, 0`, plus
  raw fallback.
- Colors: named (~20), `#rgb / #rgba / #rrggbb / #rrggbbaa`, `rgb(),
  rgba(), hsl(), hsla(), transparent, currentcolor`.

**Stylesheet parser (`stylesheet.rs`)**
- Flat `selectors { decls } …` with `/* */` comment stripping.
- Selectors: tag, `#id`, `.class`, universal `*`, comma-list.
- Specificity packed `(id<<16) | (class<<8) | tag`.

**Missing / partial**
- **Zero at-rules.** `@media, @supports, @import, @keyframes,
  @font-face, @page` are not handled. The parser scans for `{ … }`
  blocks only.
- **No combinators.** Descendant ` `, child `>`, adjacent `+`, sibling
  `~` are explicitly rejected.
- **No attribute selectors** (`[href]`, `[type=text]`).
- **No pseudo-classes / pseudo-elements** (`:hover, :focus, :nth-child,
  ::before, ::after`, …).
- `transform, transition, animation, box-shadow, background-image,
  background-size, background-position, text-decoration, font-family,
  grid-*` are all stored as **raw `Option<String>`** — never structured.
- No `!important`. No `inherit / initial / unset`.
- No `calc()`, no `var(…)`, no custom properties (`--foo`).

## 3. Style model — `crates/wgpu-html-models`

**Done**
- `Style` struct exposes ~80 `Option<…>` fields covering everything in
  §2 above (`css/style.rs`).
- Structured enums in `common/css_enums.rs`:
  - `CssLength`, `CssColor`, `Display`, `Position`, `BackgroundRepeat`,
    `BackgroundClip`, `BorderStyle` (incl. `Double / Groove / Ridge /
    Inset / Outset`), `FontWeight, FontStyle, TextAlign, TextTransform,
    WhiteSpace, Overflow, Visibility, FlexDirection, FlexWrap,
    JustifyContent, AlignItems, AlignContent, Cursor, PointerEvents,
    UserSelect, BoxSizing`.

**Missing / partial**
- No structured types for shadows, gradients, transforms, filters,
  masks, clip-paths.
- HTML element structs carry many attributes (`html/*.rs`); most are
  parsed but never consumed downstream.

## 4. Cascade — `crates/wgpu-html-style`

**Done**
- `cascade(&Tree) → CascadedTree`. Walks tree, gathers `<style>` block
  text, parses once, and produces a final `Style` per node.
- Match order: rules in ascending specificity (stable on ties), then
  the element's inline `style="…"` on top.
- `matches_selector`: tag, `#id`, multi-`.class` (whitespace-split),
  universal `*`.
- Field-by-field "Some-wins" merge across all 80+ Style fields
  (`merge.rs`).

**Missing / partial**
- **No inheritance.** `color, font-family, font-size, line-height,
  text-align`, etc. do **not** propagate from parent to child.
- **No UA default stylesheet.** `<h1>` has no default size, `<body>`
  no default margin, `<a>` is not blue/underlined, etc.
- No pseudo-classes / pseudo-elements (the matcher takes no element
  state).
- No combinators (selectors are simple-only).
- No `!important`.
- `<link rel="stylesheet">` is not loaded — only inline `<style>`
  block contents are gathered.
- `currentColor` resolves to `None` (no foreground-color tracking).

## 5. Layout — `crates/wgpu-html-layout`

**Block flow — done**
- Vertical stacking inside parent's content box.
- Margin / padding per side or via shorthand fallback.
- `box-sizing: content-box | border-box`.
- `width`: explicit (px / %, etc.) or fills container.
- `height`: explicit, else sum of children's margin-rect heights.
- Length resolution (`length.rs`): `Px`, `Percent` (vs parent),
  `Em / Rem` (constant 16px), `Vw / Vh / Vmin / Vmax` (vs viewport),
  `Zero`. `Auto` and `Raw` → `None`.
- Border widths per side; border colors and styles carried into
  `LayoutBox`.
- `border-radius`: per-corner H+V resolution against width/height,
  V defaulting to H, plus CSS-3-spec corner-overflow clamping
  (`clamp_corner_radii`).
- `background-clip: border-box | padding-box | content-box`, with
  inner-radius reduction so curvature stays concentric.

**Flex — done (`flex.rs`)**
- `flex-direction`: row / column / row-reverse / column-reverse.
- `justify-content`: start / end / center / flex-start / flex-end /
  left / right / space-between / space-around / space-evenly.
- `align-items`: start / end / center / flex-start / flex-end /
  stretch (baseline falls through to start).
- `gap` (single value, on the main axis).
- Two-pass: temporary lay-out at (0, 0) to measure, then re-lay at
  final positions, including synthesizing cross-axis sizes when
  `align-items: stretch` and the parent has a known cross dimension.

**Hit testing — done**
- `LayoutBox::hit_path((x, y)) → Option<Vec<usize>>`.
- `LayoutBox::find_element_from_point(&mut Tree, point) →
  Option<&mut Node>` and `find_elements_from_point` for the full
  ancestor chain (deepest → root). Topmost (last child) wins on
  overlap.
- `Node::at_path_mut`, `Node::ancestry_at_path_mut` in `wgpu-html-tree`.

**Missing / partial**
- **No text layout.** `Element::Text` produces a zero-size box.
- **No inline formatting context.** No line boxes, no `<span>`-style
  flow, no `display: inline / inline-block`.
- **No floats.**
- **No positioned layout.** `position` and `top/right/bottom/left` are
  parsed and stored but never read by layout.
- No `z-index` honoured — paint order is tree DFS only.
- No `overflow: hidden | scroll | auto` clipping.
- **No grid.**
- `flex-grow / flex-shrink / flex-basis` parsed but **not used**.
- No `flex-wrap` (single-line only).
- No baseline alignment.
- `min-/max-width/height` parsed but not enforced.
- `auto` margin centring not implemented (resolves to 0).
- `em / rem` use a hard-coded 16px constant — no font cascade.
- `text-align, white-space, letter-spacing, text-transform,
  text-decoration` parsed but unused (no text path).
- Transforms not applied.

## 6. Renderer / paint — `crates/wgpu-html-renderer`, `crates/wgpu-html/src/paint.rs`

**GPU pipeline (one shader: `shaders/quad.wgsl`)**
- Single instanced-quad pipeline. Per-quad data: rect, color,
  per-corner H+V radii, per-side stroke widths, 4-float pattern
  descriptor `(kind, dash, gap, _)`.
- Shader modes: filled solid; rounded fill (SDF); stroked rounded
  ring (SDF); dashed/dotted patterned ring (one-sided).
- Pushers: `push_quad`, `push_quad_rounded`,
  `push_quad_rounded_ellipse`, `push_quad_stroke`,
  `push_quad_stroke_ellipse`, `push_quad_stroke_patterned`.

**Paint translation (`wgpu-html/src/paint.rs`)**
- Backgrounds: solid color into the rect chosen by `background-clip`,
  with elliptical corner radii. Color resolution covers named, hex,
  rgb/rgba, hsl/hsla, transparent; sRGB → linear in `layout/color.rs`.
- Borders, sharp box: per-side edge quads with per-side color and
  style. `solid` → one quad; `dashed`/`dotted` → segment loop;
  `none/hidden` → skipped; `double / groove / ridge / inset / outset`
  → fall through to `solid`.
- Borders, rounded box, **uniform color + solid** → single SDF ring
  quad.
- Borders, rounded box, **mixed** colors / styles → per-side
  one-sided ring quads. Dashed/dotted with uniform-circular corners
  use the shader's patterned ring; otherwise dashed/dotted fall back
  to straight segments along the side's straight portion (corner
  curves are bare — acknowledged limitation).

**Renderer (`renderer/src/lib.rs`)**
- wgpu instance / adapter / device / queue, sRGB surface, vsync,
  surface kept alive via `Arc`.
- Single render pass: clear + quads.
- F12 screenshot via `capture_next_frame_to`; PNG export in
  `screenshot.rs`.

**Missing / partial**
- **No text rendering.** No glyph atlas, no font loader, no shaping,
  no glyph quads. The shader has no text path.
- **No images.** `<img src>` is parsed; never fetched or drawn. No
  image fills, no `background-image`.
- **No gradients.** `linear-gradient(…)` stays a raw string.
- **No box-shadow.**
- **No clipping.** `overflow: hidden / scroll` does not clip child
  paint — there's no scissor / stencil pass.
- No transforms, no opacity layers, no filters, no blend modes
  (per-quad alpha only).
- Border styles `double / groove / ridge / inset / outset` render as
  plain solid.
- Dashed/dotted on rounded boxes only follow the curve when **all
  four** corners are uniform-circular; otherwise corners stay bare.
- `currentColor` returns `None`.
- A border without an explicit color is skipped (no fallback to
  `color` / `currentColor`).

## 7. Demo + interactivity — `crates/wgpu-html-demo`

**Done**
- winit window, wgpu renderer, loads `html/flex-test.html` via
  `include_str!`.
- Re-parses, re-cascades, re-lays-out and re-paints **every frame**;
  `request_redraw` in `about_to_wait` keeps it in a continuous
  redraw loop.
- Window events: `CloseRequested`, `Resized`, F12 screenshot, Esc to
  exit.

**Missing / partial**
- **No mouse input.** No `CursorMoved / MouseInput / MouseWheel`
  handling. The hit-test API in §5 has no caller.
- No hover, click, focus, or any UI state. Pseudo-classes wouldn't
  have anywhere to read state from anyway.
- No keyboard input besides F12 / Esc — `<input>` and `<textarea>`
  are inert.
- No scrolling.
- No event dispatch / bubbling. `Node::ancestry_at_path_mut` exists
  but is unused outside tests.
- **No JavaScript / `<script>` execution.** Script content is
  captured into the tree but never run.
- The document is a compile-time constant — no URL loading, no live
  editing.

---

## Bottom line

**What actually paints to screen today**

- Solid-color rectangles (sharp, rounded, or fully elliptical) honouring
  `background-clip`.
- Borders with per-side color and style: solid edges always; dashed /
  dotted as straight segments on sharp boxes; SDF rings (single, or
  per-side, or patterned) on rounded boxes when corners are uniform-
  circular.
- A static layout: block flow with margin / border / padding /
  box-sizing, plus a non-wrapping single-line flexbox.
- Cascade: simple selectors (tag / id / class / universal) plus inline
  `style`, no inheritance, no UA defaults.

**What is parsed but currently inert**

`flex-grow / -shrink / -basis`, `position`, `top/right/bottom/left`,
`min/max-width/height`, `overflow`, `z-index`, `transform`,
`transition`, `animation`, `box-shadow`, all `grid-*`,
`background-image / -size / -position`, `font-family`,
`text-decoration`, `letter-spacing`, `white-space`, `text-align`,
`text-transform`, `cursor`, `pointer-events`, `user-select`.

**What is not modeled at all**

Text layout, images, gradients, shadows, transforms, clipping,
scrolling, inheritance, pseudo-classes, combinators, attribute
selectors, `@media` / `@import` / other at-rules, `calc()`, custom
properties, JavaScript, mouse / keyboard input in the demo.
