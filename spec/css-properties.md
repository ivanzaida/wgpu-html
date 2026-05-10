# lui — CSS Property Matrix

Exhaustive support table for non-vendor, non-deprecated CSS
properties tracked from MDN's standard CSS property index. Companion
to `spec/css.md`, which explains parser/cascade/layout/paint behavior
in prose.

Status meanings:

- **Yes**: parsed and consumed by layout/paint/text behavior.
- **Partial**: parsed and some behavior works, but browser semantics
  are incomplete.
- **Parsed only**: stored in typed `Style`, but not consumed.
- **Deferred**: recognized in shorthand/deferred-longhand storage, but
  not typed/consumed.
- **No**: not recognized by the parser today.

Source baseline: [MDN CSS properties index](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties),
last modified 2026-03-25, excluding the vendor-prefixed non-standard
section and known deprecated legacy aliases.

| Prop | Supported | Note |
|---|---:|---|
| `--*` | Yes | Custom properties are stored/inherited and resolved through `var()` where the target parser accepts the resolved value. |
| `accent-color` | No | Not modeled. |
| `align-content` | Yes | Consumed by flex/grid alignment. |
| `align-items` | Yes | Consumed by flex/grid alignment. |
| `align-self` | Yes | Consumed by flex/grid item alignment. |
| `alignment-baseline` | No | SVG/text baseline semantics not modeled. |
| `all` | No | CSS-wide reset shorthand not implemented. |
| `anchor-name` | No | Anchor positioning not modeled. |
| `anchor-scope` | No | Anchor positioning not modeled. |
| `animation` | Parsed only | Shorthand/member extraction exists; no animation runtime. |
| `animation-composition` | Deferred | Recognized as animation-related metadata, not consumed. |
| `animation-delay` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `animation-direction` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `animation-duration` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `animation-fill-mode` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `animation-iteration-count` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `animation-name` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `animation-play-state` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `animation-range` | Deferred | Recognized as shorthand metadata, not consumed. |
| `animation-range-end` | Deferred | Stored as deferred longhand, not consumed. |
| `animation-range-start` | Deferred | Stored as deferred longhand, not consumed. |
| `animation-timeline` | Deferred | Timeline metadata only; no animation runtime. |
| `animation-timing-function` | Deferred | Stored through animation shorthand/deferred path, not consumed. |
| `appearance` | No | Native appearance styling not modeled. |
| `aspect-ratio` | No | Not modeled in layout. |
| `backdrop-filter` | No | Filter/compositing pipeline not modeled. |
| `backface-visibility` | No | 3D transform semantics not modeled. |
| `background` | Partial | Color/image/repeat/clip members handled; many members and layers ignored/deferred. |
| `background-attachment` | Deferred | Recognized/stored, not consumed. |
| `background-blend-mode` | No | Blend modes not modeled. |
| `background-clip` | Yes | `border-box`, `padding-box`, `content-box` affect paint rect/radii. |
| `background-color` | Yes | Painted. |
| `background-image` | Partial | URL images load/paint; gradients/functions are parsed but not painted. |
| `background-origin` | Deferred | Recognized/stored, not consumed. |
| `background-position` | Partial | Consumed for URL-backed backgrounds. |
| `background-position-x` | Deferred | Recognized/stored, not consumed separately. |
| `background-position-y` | Deferred | Recognized/stored, not consumed separately. |
| `background-repeat` | Partial | Consumed for URL-backed backgrounds. |
| `background-repeat-x` | No | Not modeled separately. |
| `background-repeat-y` | No | Not modeled separately. |
| `background-size` | Partial | Consumed for URL-backed backgrounds. |
| `baseline-shift` | No | SVG/text baseline semantics not modeled. |
| `baseline-source` | No | Baseline table semantics not modeled. |
| `block-size` | No | Logical sizing not mapped to physical layout. |
| `border` | Partial | Physical shorthand parsed; paint supports common styles, others degrade to solid. |
| `border-block` | Deferred | Logical border shorthand stored, not consumed. |
| `border-block-color` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-end` | Deferred | Logical border shorthand stored, not consumed. |
| `border-block-end-color` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-end-style` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-end-width` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-start` | Deferred | Logical border shorthand stored, not consumed. |
| `border-block-start-color` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-start-style` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-start-width` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-style` | Deferred | Logical border longhand stored, not consumed. |
| `border-block-width` | Deferred | Logical border longhand stored, not consumed. |
| `border-bottom` | Partial | Physical side shorthand consumed. |
| `border-bottom-color` | Yes | Consumed by paint. |
| `border-bottom-left-radius` | Yes | Consumed by paint/layout radius data. |
| `border-bottom-right-radius` | Yes | Consumed by paint/layout radius data. |
| `border-bottom-style` | Partial | Common styles paint; several styles degrade to solid. |
| `border-bottom-width` | Yes | Consumed by layout/paint. |
| `border-collapse` | Deferred | Table layout not implemented. |
| `border-color` | Yes | Physical shorthand consumed. |
| `border-end-end-radius` | No | Logical corner radius not mapped. |
| `border-end-start-radius` | No | Logical corner radius not mapped. |
| `border-image` | Deferred | Recognized/stored, not painted. |
| `border-image-outset` | Deferred | Recognized/stored, not painted. |
| `border-image-repeat` | Deferred | Recognized/stored, not painted. |
| `border-image-slice` | Deferred | Recognized/stored, not painted. |
| `border-image-source` | Deferred | Recognized/stored, not painted. |
| `border-image-width` | Deferred | Recognized/stored, not painted. |
| `border-inline` | Deferred | Logical border shorthand stored, not consumed. |
| `border-inline-color` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-end` | Deferred | Logical border shorthand stored, not consumed. |
| `border-inline-end-color` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-end-style` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-end-width` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-start` | Deferred | Logical border shorthand stored, not consumed. |
| `border-inline-start-color` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-start-style` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-start-width` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-style` | Deferred | Logical border longhand stored, not consumed. |
| `border-inline-width` | Deferred | Logical border longhand stored, not consumed. |
| `border-left` | Partial | Physical side shorthand consumed. |
| `border-left-color` | Yes | Consumed by paint. |
| `border-left-style` | Partial | Common styles paint; several styles degrade to solid. |
| `border-left-width` | Yes | Consumed by layout/paint. |
| `border-radius` | Yes | Includes 1-4 corner expansion and elliptical syntax. |
| `border-right` | Partial | Physical side shorthand consumed. |
| `border-right-color` | Yes | Consumed by paint. |
| `border-right-style` | Partial | Common styles paint; several styles degrade to solid. |
| `border-right-width` | Yes | Consumed by layout/paint. |
| `border-spacing` | Deferred | Table layout not implemented. |
| `border-start-end-radius` | No | Logical corner radius not mapped. |
| `border-start-start-radius` | No | Logical corner radius not mapped. |
| `border-style` | Partial | Common styles paint; several styles degrade to solid. |
| `border-top` | Partial | Physical side shorthand consumed. |
| `border-top-color` | Yes | Consumed by paint. |
| `border-top-left-radius` | Yes | Consumed by paint/layout radius data. |
| `border-top-right-radius` | Yes | Consumed by paint/layout radius data. |
| `border-top-style` | Partial | Common styles paint; several styles degrade to solid. |
| `border-top-width` | Yes | Consumed by layout/paint. |
| `border-width` | Yes | Physical shorthand consumed. |
| `bottom` | Yes | Consumed by positioned layout (absolute/relative/fixed). Sticky is degraded to relative. |
| `box-decoration-break` | No | Fragmented inline decoration semantics not modeled. |
| `box-shadow` | Parsed only | Stored but not painted. |
| `box-sizing` | Yes | `content-box` / `border-box` affect sizing. |
| `break-after` | No | Fragmentation/paged layout not modeled. |
| `break-before` | No | Fragmentation/paged layout not modeled. |
| `break-inside` | No | Fragmentation/paged layout not modeled. |
| `caption-side` | No | Table captions not modeled. |
| `caret` | No | Shorthand not modeled as CSS; edit caret paint is engine state. |
| `caret-animation` | No | CSS caret animation not modeled. |
| `caret-color` | No | CSS caret color not modeled. |
| `caret-shape` | No | CSS caret shape not modeled. |
| `clear` | No | Floats not modeled. |
| `clip-path` | No | Clipping path support not modeled. |
| `clip-rule` | No | SVG clip rule not modeled. |
| `color` | Partial | Text color works; `currentcolor` resolution is incomplete. |
| `color-interpolation` | No | SVG/color interpolation not modeled. |
| `color-interpolation-filters` | No | Filter/color interpolation not modeled. |
| `color-scheme` | No | UA color-scheme switching not modeled. |
| `column-count` | Deferred | Multicol layout not implemented. |
| `column-fill` | Deferred | Multicol layout not implemented. |
| `column-gap` | Yes | Consumed by flex/grid gap behavior. |
| `column-height` | Deferred | Multicol layout not implemented. |
| `column-rule` | Deferred | Multicol layout not implemented. |
| `column-rule-color` | Deferred | Multicol layout not implemented. |
| `column-rule-style` | Deferred | Multicol layout not implemented. |
| `column-rule-width` | Deferred | Multicol layout not implemented. |
| `column-span` | Deferred | Multicol layout not implemented. |
| `column-width` | Deferred | Multicol layout not implemented. |
| `column-wrap` | Deferred | Multicol layout not implemented. |
| `columns` | Deferred | Multicol shorthand stored, not consumed. |
| `contain` | Deferred | Containment semantics not enforced. |
| `contain-intrinsic-block-size` | Deferred | Stored as deferred longhand, not consumed. |
| `contain-intrinsic-height` | Deferred | Stored as deferred longhand, not consumed. |
| `contain-intrinsic-inline-size` | Deferred | Stored as deferred longhand, not consumed. |
| `contain-intrinsic-size` | Deferred | Shorthand stored, not consumed. |
| `contain-intrinsic-width` | Deferred | Stored as deferred longhand, not consumed. |
| `container` | Deferred | Container queries not implemented. |
| `container-name` | Deferred | Container queries not implemented. |
| `container-type` | Deferred | Container queries not implemented. |
| `content` | No | Generated content/pseudo-elements not implemented. |
| `content-visibility` | No | Layout skipping/containment not modeled. |
| `corner-block-end-shape` | No | Corner shape rendering not modeled. |
| `corner-block-start-shape` | No | Corner shape rendering not modeled. |
| `corner-bottom-left-shape` | No | Corner shape rendering not modeled. |
| `corner-bottom-right-shape` | No | Corner shape rendering not modeled. |
| `corner-bottom-shape` | No | Corner shape rendering not modeled. |
| `corner-end-end-shape` | No | Corner shape rendering not modeled. |
| `corner-end-start-shape` | No | Corner shape rendering not modeled. |
| `corner-inline-end-shape` | No | Corner shape rendering not modeled. |
| `corner-inline-start-shape` | No | Corner shape rendering not modeled. |
| `corner-left-shape` | No | Corner shape rendering not modeled. |
| `corner-right-shape` | No | Corner shape rendering not modeled. |
| `corner-shape` | No | Corner shape rendering not modeled. |
| `corner-start-end-shape` | No | Corner shape rendering not modeled. |
| `corner-start-start-shape` | No | Corner shape rendering not modeled. |
| `corner-top-left-shape` | No | Corner shape rendering not modeled. |
| `corner-top-right-shape` | No | Corner shape rendering not modeled. |
| `corner-top-shape` | No | Corner shape rendering not modeled. |
| `counter-increment` | No | CSS counters not modeled. |
| `counter-reset` | No | CSS counters not modeled. |
| `counter-set` | No | CSS counters not modeled. |
| `cursor` | Parsed only | Stored/inherited, but no resolved cursor/OS cursor integration. |
| `cx` | No | SVG geometry property not modeled. |
| `cy` | No | SVG geometry property not modeled. |
| `d` | No | SVG path geometry property not modeled. |
| `direction` | Deferred | Stored as deferred longhand; bidi semantics not implemented. |
| `display` | Partial | Block/flex/grid/inline-block paths exist; many display values degrade. |
| `dominant-baseline` | No | SVG/text baseline semantics not modeled. |
| `dynamic-range-limit` | No | HDR output handling not modeled. |
| `empty-cells` | No | Table layout not implemented. |
| `field-sizing` | No | Form intrinsic sizing behavior not modeled. |
| `fill` | Yes | SVG paint; cascaded + inherited. Serialised into rasterised SVG. |
| `fill-opacity` | Yes | SVG paint; cascaded + inherited. |
| `fill-rule` | Yes | `nonzero` / `evenodd`; cascaded + inherited. |
| `filter` | No | Filter effects not modeled. |
| `flex` | Yes | Shorthand consumed by flex layout. |
| `flex-basis` | Yes | Consumed by flex layout. |
| `flex-direction` | Yes | Consumed by flex layout. |
| `flex-flow` | Deferred | Shorthand recognized; member behavior exists through typed longhands. |
| `flex-grow` | Yes | Consumed by flex layout. |
| `flex-shrink` | Yes | Consumed by flex layout. |
| `flex-wrap` | Yes | Consumed by flex layout. |
| `float` | No | Float layout not implemented. |
| `flood-color` | No | SVG filter property not modeled. |
| `flood-opacity` | No | SVG filter property not modeled. |
| `font` | Partial | Shorthand parses common members; font loading/system matching is limited to registered fonts. |
| `font-family` | Partial | Registered font matching plus generic fallback; no `@font-face` loading. |
| `font-feature-settings` | Deferred | Stored as deferred/font metadata, not consumed by shaper. |
| `font-kerning` | No | Not exposed to shaping. |
| `font-language-override` | No | Not exposed to shaping. |
| `font-optical-sizing` | No | Not exposed to shaping. |
| `font-palette` | No | Color font palette support not modeled. |
| `font-size` | Partial | Feeds shaping; relative computed-value semantics are approximate. |
| `font-size-adjust` | No | Not modeled. |
| `font-style` | Yes | Feeds font matching/shaping. |
| `font-synthesis` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-synthesis-position` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-synthesis-small-caps` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-synthesis-style` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-synthesis-weight` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant-alternates` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant-caps` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant-east-asian` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant-emoji` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant-ligatures` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant-numeric` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variant-position` | Deferred | Stored as deferred/font metadata, not consumed. |
| `font-variation-settings` | No | Variable font axes not wired. |
| `font-weight` | Yes | Feeds font matching/shaping. |
| `font-width` | No | New width axis property not wired. |
| `forced-color-adjust` | No | Forced-colors mode not modeled. |
| `gap` | Yes | Consumed by flex/grid. |
| `grid` | Partial | Grid longhands consumed; full shorthand grammar coverage is limited. |
| `grid-area` | Deferred | Shorthand recognized; not fully expanded into placement behavior. |
| `grid-auto-columns` | Yes | Consumed by grid. |
| `grid-auto-flow` | Yes | Consumed by grid. |
| `grid-auto-rows` | Yes | Consumed by grid. |
| `grid-column` | Yes | Placement shorthand consumed. |
| `grid-column-end` | Yes | Consumed by grid placement. |
| `grid-column-start` | Yes | Consumed by grid placement. |
| `grid-row` | Yes | Placement shorthand consumed. |
| `grid-row-end` | Yes | Consumed by grid placement. |
| `grid-row-start` | Yes | Consumed by grid placement. |
| `grid-template` | Deferred | Shorthand recognized; not fully consumed. |
| `grid-template-areas` | Deferred | Stored, but named-area placement is not implemented. |
| `grid-template-columns` | Yes | Consumed by grid track sizing. |
| `grid-template-rows` | Yes | Consumed by grid track sizing. |
| `hanging-punctuation` | No | Text punctuation layout not modeled. |
| `height` | Yes | Consumed by layout. |
| `hyphenate-character` | No | Hyphenation not modeled. |
| `hyphenate-limit-chars` | No | Hyphenation not modeled. |
| `hyphens` | No | Hyphenation not modeled. |
| `image-orientation` | No | Image orientation handling not modeled. |
| `image-rendering` | No | Sampling mode not exposed. |
| `image-resolution` | No | Image resolution metadata not modeled. |
| `initial-letter` | No | Drop caps/initial-letter layout not modeled. |
| `inline-size` | No | Logical sizing not mapped to physical layout. |
| `inset` | Deferred | Logical/physical shorthand stored; physical `top/right/bottom/left` are consumed separately. |
| `inset-block` | Deferred | Logical inset not mapped. |
| `inset-block-end` | Deferred | Logical inset not mapped. |
| `inset-block-start` | Deferred | Logical inset not mapped. |
| `inset-inline` | Deferred | Logical inset not mapped. |
| `inset-inline-end` | Deferred | Logical inset not mapped. |
| `inset-inline-start` | Deferred | Logical inset not mapped. |
| `interactivity` | No | Not modeled. |
| `interest-delay` | No | Interest invoker behavior not modeled. |
| `interest-delay-end` | No | Interest invoker behavior not modeled. |
| `interest-delay-start` | No | Interest invoker behavior not modeled. |
| `interpolate-size` | No | Animation/interpolation behavior not modeled. |
| `isolation` | No | Compositing/isolation not modeled. |
| `justify-content` | Yes | Consumed by flex/grid alignment. |
| `justify-items` | Yes | Consumed by grid item alignment. |
| `justify-self` | Yes | Consumed by grid item alignment. |
| `left` | Yes | Consumed by positioned layout (absolute/relative/fixed). Sticky is degraded to relative. |
| `letter-spacing` | Yes | Consumed by text shaping. |
| `lighting-color` | No | SVG filter property not modeled. |
| `line-break` | No | Line breaking policy not modeled separately. |
| `line-clamp` | Deferred | Recognized/stored, not consumed. |
| `line-height` | Partial | Feeds text layout; computed-value semantics are approximate. |
| `line-height-step` | No | Not modeled. |
| `list-style` | Deferred | List marker layout not implemented. |
| `list-style-image` | Deferred | List marker layout not implemented. |
| `list-style-position` | Deferred | List marker layout not implemented. |
| `list-style-type` | Deferred | List marker layout not implemented. |
| `margin` | Yes | Physical shorthand consumed. |
| `margin-block` | Deferred | Logical margin not mapped. |
| `margin-block-end` | Deferred | Logical margin not mapped. |
| `margin-block-start` | Deferred | Logical margin not mapped. |
| `margin-bottom` | Yes | Consumed by layout. |
| `margin-inline` | Deferred | Logical margin not mapped. |
| `margin-inline-end` | Deferred | Logical margin not mapped. |
| `margin-inline-start` | Deferred | Logical margin not mapped. |
| `margin-left` | Yes | Consumed by layout. |
| `margin-right` | Yes | Consumed by layout. |
| `margin-top` | Yes | Consumed by layout. |
| `margin-trim` | No | Not modeled. |
| `marker` | Deferred | SVG marker/list marker behavior not modeled. |
| `marker-end` | Deferred | SVG marker behavior not modeled. |
| `marker-mid` | Deferred | SVG marker behavior not modeled. |
| `marker-start` | Deferred | SVG marker behavior not modeled. |
| `mask` | Deferred | Masking not rendered. |
| `mask-border` | Deferred | Masking not rendered. |
| `mask-border-mode` | Deferred | Masking not rendered. |
| `mask-border-outset` | Deferred | Masking not rendered. |
| `mask-border-repeat` | Deferred | Masking not rendered. |
| `mask-border-slice` | Deferred | Masking not rendered. |
| `mask-border-source` | Deferred | Masking not rendered. |
| `mask-border-width` | Deferred | Masking not rendered. |
| `mask-clip` | Deferred | Masking not rendered. |
| `mask-composite` | Deferred | Masking not rendered. |
| `mask-image` | Deferred | Masking not rendered. |
| `mask-mode` | Deferred | Masking not rendered. |
| `mask-origin` | Deferred | Masking not rendered. |
| `mask-position` | Deferred | Masking not rendered. |
| `mask-repeat` | Deferred | Masking not rendered. |
| `mask-size` | Deferred | Masking not rendered. |
| `mask-type` | Deferred | Masking not rendered. |
| `math-depth` | No | Math layout not modeled. |
| `math-shift` | No | Math layout not modeled. |
| `math-style` | No | Math layout not modeled. |
| `max-block-size` | No | Logical sizing not mapped. |
| `max-height` | Yes | Consumed by layout. |
| `max-inline-size` | No | Logical sizing not mapped. |
| `max-width` | Yes | Consumed by layout. |
| `min-block-size` | No | Logical sizing not mapped. |
| `min-height` | Yes | Consumed by layout. |
| `min-inline-size` | No | Logical sizing not mapped. |
| `min-width` | Yes | Consumed by layout. |
| `mix-blend-mode` | No | Blend modes not modeled. |
| `object-fit` | No | Replaced element fitting not modeled. |
| `object-position` | No | Replaced element positioning not modeled. |
| `object-view-box` | No | Replaced element view box not modeled. |
| `offset` | Deferred | Motion path not implemented. |
| `offset-anchor` | Deferred | Motion path not implemented. |
| `offset-distance` | Deferred | Motion path not implemented. |
| `offset-path` | Deferred | Motion path not implemented. |
| `offset-position` | Deferred | Motion path not implemented. |
| `offset-rotate` | Deferred | Motion path not implemented. |
| `opacity` | Partial | Alpha-multiplied into primitives; no isolated compositing layer. |
| `order` | Yes | Consumed by flex layout. |
| `orphans` | No | Fragmentation/paged layout not modeled. |
| `outline` | Deferred | Recognized/stored, not painted. |
| `outline-color` | Deferred | Recognized/stored, not painted. |
| `outline-offset` | Deferred | Recognized/stored, not painted. |
| `outline-style` | Deferred | Recognized/stored, not painted. |
| `outline-width` | Deferred | Recognized/stored, not painted. |
| `overflow` | Partial | Paint clipping, hit-test clipping, and scroll containers exist; DOM scroll events/integrated offsets are incomplete. |
| `overflow-anchor` | No | Scroll anchoring not modeled. |
| `overflow-block` | No | Logical overflow not mapped. |
| `overflow-clip-margin` | No | Clip margin not modeled. |
| `overflow-inline` | No | Logical overflow not mapped. |
| `overflow-wrap` | Deferred | Stored, but line breaking behavior is limited. |
| `overflow-x` | Partial | Affects clipping/scroll axes. |
| `overflow-y` | Partial | Affects clipping/scroll axes. |
| `overscroll-behavior` | Deferred | Stored, not consumed. |
| `overscroll-behavior-block` | Deferred | Stored, not consumed. |
| `overscroll-behavior-inline` | Deferred | Stored, not consumed. |
| `overscroll-behavior-x` | Deferred | Stored, not consumed. |
| `overscroll-behavior-y` | Deferred | Stored, not consumed. |
| `padding` | Yes | Physical shorthand consumed. |
| `padding-block` | Deferred | Logical padding not mapped. |
| `padding-block-end` | Deferred | Logical padding not mapped. |
| `padding-block-start` | Deferred | Logical padding not mapped. |
| `padding-bottom` | Yes | Consumed by layout. |
| `padding-inline` | Deferred | Logical padding not mapped. |
| `padding-inline-end` | Deferred | Logical padding not mapped. |
| `padding-inline-start` | Deferred | Logical padding not mapped. |
| `padding-left` | Yes | Consumed by layout. |
| `padding-right` | Yes | Consumed by layout. |
| `padding-top` | Yes | Consumed by layout. |
| `page` | No | Paged media not modeled. |
| `paint-order` | No | SVG paint ordering not modeled. |
| `perspective` | No | 3D transforms not modeled. |
| `perspective-origin` | No | 3D transforms not modeled. |
| `place-content` | Deferred | Shorthand recognized; underlying align/justify consumed where set directly. |
| `place-items` | Deferred | Shorthand recognized; underlying align/justify consumed where set directly. |
| `place-self` | Deferred | Shorthand recognized; underlying align/justify consumed where set directly. |
| `pointer-events` | Yes | Hit-testing skips `none`; children with `auto` remain hittable. Inherited. |
| `position` | Partial | Static/relative/absolute/fixed mostly work; sticky lacks sticky scroll behavior. |
| `position-anchor` | No | Anchor positioning not modeled. |
| `position-area` | No | Anchor positioning not modeled. |
| `position-try` | No | Anchor positioning fallback not modeled. |
| `position-try-fallbacks` | No | Anchor positioning fallback not modeled. |
| `position-try-order` | No | Anchor positioning fallback not modeled. |
| `position-visibility` | No | Anchor positioning visibility not modeled. |
| `print-color-adjust` | No | Print/forced color adjustment not modeled. |
| `quotes` | No | Generated quotation content not modeled. |
| `r` | No | SVG geometry property not modeled. |
| `reading-flow` | No | Reading order layout/accessibility behavior not modeled. |
| `reading-order` | No | Reading order layout/accessibility behavior not modeled. |
| `resize` | Deferred | Stored, but user resizing controls not implemented. |
| `right` | Yes | Consumed by positioned layout (absolute/relative/fixed). Sticky is degraded to relative. |
| `rotate` | No | Individual transform property not consumed. |
| `row-gap` | Yes | Consumed by flex/grid. |
| `ruby-align` | No | Ruby layout not implemented. |
| `ruby-overhang` | No | Ruby layout not implemented. |
| `ruby-position` | No | Ruby layout not implemented. |
| `rx` | No | SVG geometry property not modeled. |
| `ry` | No | SVG geometry property not modeled. |
| `scale` | No | Individual transform property not consumed. |
| `scroll-behavior` | No | Smooth scrolling not modeled. |
| `scroll-initial-target` | No | Not modeled. |
| `scroll-margin` | Deferred | Stored, not consumed. |
| `scroll-margin-block` | Deferred | Stored, not consumed. |
| `scroll-margin-block-end` | Deferred | Stored, not consumed. |
| `scroll-margin-block-start` | Deferred | Stored, not consumed. |
| `scroll-margin-bottom` | Deferred | Stored, not consumed. |
| `scroll-margin-inline` | Deferred | Stored, not consumed. |
| `scroll-margin-inline-end` | Deferred | Stored, not consumed. |
| `scroll-margin-inline-start` | Deferred | Stored, not consumed. |
| `scroll-margin-left` | Deferred | Stored, not consumed. |
| `scroll-margin-right` | Deferred | Stored, not consumed. |
| `scroll-margin-top` | Deferred | Stored, not consumed. |
| `scroll-marker-group` | No | Scroll marker behavior not modeled. |
| `scroll-padding` | Deferred | Stored, not consumed. |
| `scroll-padding-block` | Deferred | Stored, not consumed. |
| `scroll-padding-block-end` | Deferred | Stored, not consumed. |
| `scroll-padding-block-start` | Deferred | Stored, not consumed. |
| `scroll-padding-bottom` | Deferred | Stored, not consumed. |
| `scroll-padding-inline` | Deferred | Stored, not consumed. |
| `scroll-padding-inline-end` | Deferred | Stored, not consumed. |
| `scroll-padding-inline-start` | Deferred | Stored, not consumed. |
| `scroll-padding-left` | Deferred | Stored, not consumed. |
| `scroll-padding-right` | Deferred | Stored, not consumed. |
| `scroll-padding-top` | Deferred | Stored, not consumed. |
| `scroll-snap-align` | No | Scroll snap not modeled. |
| `scroll-snap-stop` | No | Scroll snap not modeled. |
| `scroll-snap-type` | No | Scroll snap not modeled. |
| `scroll-target-group` | No | Not modeled. |
| `scroll-timeline` | Deferred | Stored, no scroll-driven animation runtime. |
| `scroll-timeline-axis` | Deferred | Stored, no scroll-driven animation runtime. |
| `scroll-timeline-name` | Deferred | Stored, no scroll-driven animation runtime. |
| `scrollbar-color` | No | Native scrollbar styling not modeled. |
| `scrollbar-gutter` | No | Layout reservation for scrollbars not modeled. |
| `scrollbar-width` | No | Native scrollbar styling not modeled. |
| `shape-image-threshold` | No | CSS shapes not modeled. |
| `shape-margin` | No | CSS shapes not modeled. |
| `shape-outside` | No | Float/shape layout not modeled. |
| `shape-rendering` | No | SVG rendering hint not modeled. |
| `speak-as` | No | Speech/CSS counter style output not modeled. |
| `stop-color` | No | SVG gradient property not modeled. |
| `stop-opacity` | No | SVG gradient property not modeled. |
| `stroke` | Yes | SVG paint; cascaded + inherited. |
| `stroke-dasharray` | Yes | SVG stroke; cascaded + inherited. |
| `stroke-dashoffset` | Yes | SVG stroke; cascaded + inherited. |
| `stroke-linecap` | Yes | `butt` / `round` / `square`; cascaded + inherited. |
| `stroke-linejoin` | Yes | `miter` / `round` / `bevel`; cascaded + inherited. |
| `stroke-miterlimit` | No | SVG stroke property not modeled. |
| `stroke-opacity` | Yes | SVG stroke; cascaded + inherited. |
| `stroke-width` | Yes | SVG stroke; cascaded + inherited. |
| `tab-size` | No | Tab stop sizing not modeled. |
| `table-layout` | No | Table layout not implemented. |
| `text-align` | Yes | Consumed by text/line layout. |
| `text-align-last` | No | Not modeled. |
| `text-anchor` | No | SVG text alignment not modeled. |
| `text-autospace` | No | Not modeled. |
| `text-box` | Deferred | Stored, not consumed. |
| `text-box-edge` | Deferred | Stored, not consumed. |
| `text-box-trim` | Deferred | Stored, not consumed. |
| `text-combine-upright` | No | Vertical text layout not modeled. |
| `text-decoration` | Partial | Underline/overline/line-through paint; detailed longhands mostly ignored. |
| `text-decoration-color` | Deferred | Stored, not consumed. |
| `text-decoration-inset` | No | Not modeled. |
| `text-decoration-line` | Deferred | Stored; shorthand path paints common lines. |
| `text-decoration-skip` | Deferred | Stored, not consumed. |
| `text-decoration-skip-ink` | No | Not modeled. |
| `text-decoration-style` | Deferred | Stored, not consumed. |
| `text-decoration-thickness` | Deferred | Stored, not consumed. |
| `text-emphasis` | Deferred | Stored, not consumed. |
| `text-emphasis-color` | Deferred | Stored, not consumed. |
| `text-emphasis-position` | Deferred | Stored, not consumed. |
| `text-emphasis-style` | Deferred | Stored, not consumed. |
| `text-indent` | Deferred | Stored, not consumed. |
| `text-justify` | No | Justification algorithm not modeled. |
| `text-orientation` | No | Vertical writing not modeled. |
| `text-overflow` | No | Ellipsis/clipping text overflow not modeled. |
| `text-rendering` | No | SVG/text rendering hint not modeled. |
| `text-shadow` | Deferred | Stored, not painted. |
| `text-size-adjust` | No | Mobile text autosizing not modeled. |
| `text-spacing-trim` | No | Not modeled. |
| `text-transform` | Yes | Consumed by shaping. |
| `text-underline-offset` | No | Decoration positioning not modeled. |
| `text-underline-position` | No | Decoration positioning not modeled. |
| `text-wrap` | Deferred | Stored, not consumed. |
| `text-wrap-mode` | Deferred | Stored, not consumed. |
| `text-wrap-style` | No | Not modeled. |
| `timeline-scope` | Deferred | Scroll/view timeline metadata only. |
| `top` | Yes | Consumed by positioned layout (absolute/relative/fixed). Sticky is degraded to relative. |
| `touch-action` | No | Touch input not modeled. |
| `transform` | Parsed only | Stored raw; layout/paint/hit-testing ignore it. |
| `transform-box` | No | Transform rendering not consumed. |
| `transform-origin` | Parsed only | Stored raw; transform rendering not consumed. |
| `transform-style` | No | 3D transform semantics not modeled. |
| `transition` | Parsed only | Shorthand/member extraction exists; no transition runtime. |
| `transition-behavior` | Deferred | Stored as transition metadata, not consumed. |
| `transition-delay` | Deferred | Stored as transition metadata, not consumed. |
| `transition-duration` | Deferred | Stored as transition metadata, not consumed. |
| `transition-property` | Deferred | Stored as transition metadata, not consumed. |
| `transition-timing-function` | Deferred | Stored as transition metadata, not consumed. |
| `translate` | No | Individual transform property not consumed. |
| `unicode-bidi` | Deferred | Stored, but bidi layout semantics not implemented. |
| `user-select` | Partial | `none` enforced in text cursor hit-testing; `text`/`all`/`auto` not yet. Inherited. |
| `vector-effect` | No | SVG vector effects not modeled. |
| `vertical-align` | No | Inline/table vertical alignment not modeled. |
| `view-timeline` | Deferred | Stored, no view-timeline runtime. |
| `view-timeline-axis` | Deferred | Stored, no view-timeline runtime. |
| `view-timeline-inset` | Deferred | Stored, no view-timeline runtime. |
| `view-timeline-name` | Deferred | Stored, no view-timeline runtime. |
| `view-transition-class` | No | View transitions not modeled. |
| `view-transition-name` | No | View transitions not modeled. |
| `visibility` | Yes | Inherited and consumed by rendering behavior. |
| `white-space` | Yes | Consumed by text shaping/layout. |
| `white-space-collapse` | Deferred | Stored, not consumed separately. |
| `widows` | No | Fragmentation/paged layout not modeled. |
| `width` | Yes | Consumed by layout. |
| `will-change` | No | Optimization hint ignored. |
| `word-break` | No | Line breaking policy not modeled separately. |
| `word-spacing` | Deferred | Stored, not consumed. |
| `writing-mode` | No | Vertical/logical writing modes not modeled. |
| `x` | No | SVG geometry property not modeled. |
| `y` | No | SVG geometry property not modeled. |
| `z-index` | Partial | Stored on LayoutBox; paint order is tree DFS (no stacking-context reordering). |
| `zoom` | No | Viewport/layout zoom property not modeled. |
