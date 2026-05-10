---
sidebar_position: 4
---

# CSS Engine

The CSS engine converts raw stylesheets and inline styles into a fully resolved `Style` on every DOM element. It lives in `crates/lui-style`.

## Pipeline

```
CSS text (inline + <style> blocks)
  → Parse stylesheets      → Vec<Rule> with selectors + declarations
  → Index rules            → by-id, by-class, by-tag maps
  → Cascade per element    → apply matching rules in specificity order
  → Resolve keywords       → inherit / initial / unset
  → Inherit from parent    → copy inherited properties
  → Resolve var()          → substitute CSS custom properties
  → CascadedTree           → Style per node
```

## Stylesheet Parsing

Located in `crates/lui-parser/src/stylesheet.rs`. Parses `<style>` block content into `Vec<Rule>`:

```rust
struct Rule {
    selectors: SelectorList,
    declarations: Style,     // normal declarations
    important: Style,        // !important declarations
    keywords: HashMap<...>,  // inherit/initial/unset (normal)
    media: Vec<MediaQueryList>, // @media conditions
}
```

## Cascade Order

Following CSS-Cascade-3 §6.4:

```
1. Author normal rules      (ascending specificity)
2. Inline normal declarations
3. Author !important rules   (ascending specificity)
4. Inline !important declarations
```

## Specificity

24-bit value: 8 bits each for IDs, classes (+ attrs + pseudo-classes), and tags:

```
(id_count << 16) | (class_count << 8) | tag_count
```

`:where()` selectors contribute zero specificity. `:is()` and `:not()` take the specificity of their most specific argument.

## Rule Indexing

To avoid testing every rule against every element, rules are indexed in maps:

```rust
RuleIndex {
    by_id: HashMap<String, Vec<SelectorRuleRef>>,
    by_class: HashMap<String, Vec<SelectorRuleRef>>,
    by_tag: HashMap<String, Vec<SelectorRuleRef>>,
    universal: Vec<SelectorRuleRef>,
}
```

Only candidate rules matching an element's ID, classes, or tag are tested for full selector matching.

## Inheritance

These properties inherit from parent to child:
- `color`, `font-family`, `font-size`, `font-weight`, `font-style`, `line-height`
- `letter-spacing`, `text-align`, `text-transform`, `white-space`, `text-decoration`
- `visibility`, `cursor`, `pointer-events`, `user-select`
- `opacity` (multiplied, not replaced)
- SVG: `fill-*`, `stroke-*`
- Custom properties: all `--*` variables

## CSS-Wide Keywords

| Keyword | Behavior |
|---|---|
| `inherit` | Use parent's computed value |
| `initial` | Reset to spec initial value |
| `unset` | Inherit if inherited property, initial otherwise |

## var() / Custom Properties

CSS custom properties (`--name: value`) are:
- Parsed and stored per-element
- Always inherited
- Resolved recursively with cycle detection
- Substituted into regular properties at cascade time

```css
:root { --accent: #ff6b6b; }
.button { background: var(--accent); }
```

## @media Queries

Supported features:

| Feature | Type |
|---|---|
| `width` / `min-width` / `max-width` | px comparison |
| `height` / `min-height` / `max-height` | px comparison |
| `orientation` | portrait / landscape |

The `not` keyword is supported. Comma-separated queries within a media list are ORed. Nested `@media` blocks are ANDed with their parent conditions.

## Incremental Re-Cascade

When hover/focus/active state changes, only affected nodes are re-cascaded in-place. This avoids reprocessing the entire tree and is the basis for `PipelineCache`'s partial-cascade and repaint-only fast paths.
