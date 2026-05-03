---
id: css/cascade
title: Cascade & Inheritance
---

# CSS Cascade and Inheritance

The cascade is the algorithm that determines which CSS declarations apply to each element when multiple rules target the same property. wgpu-html implements CSS-Cascade-3 §6 with a 4-band cascade, selector specificity ordering, inline style overrides, CSS-wide keyword resolution, and property inheritance.

## Cascade Order

The cascade applies declarations in the following order, with later bands taking precedence:

### 1. UA Normal Rules (lowest priority)

The user-agent stylesheet provides browser-consistent defaults. Every UA rule uses tag selectors only, so an author rule with equal specificity wins on source order (UA rules are emitted first).

```
html { display: block; }
body { display: block; margin: 8px; }
h1 { display: block; font-size: 2em; font-weight: bold; }
p { display: block; margin-top: 1em; margin-bottom: 1em; }
```

UA rules sorted by specificity ascending (all specificity 1 since they're tag-only), then source order.

### 2. Author Normal Rules

Rules from `<style>` blocks and linked stylesheets, sorted by specificity ascending. On specificity ties, source order prevails (stable sort).

```css
/* specificity 1 */
p { color: black; }

/* specificity 256 — wins over p */
.intro { color: gray; }

/* specificity 257 — wins over .intro */
p.lead { color: #333; }
```

### 3. Author !important Rules

After all normal declarations, important rules from the same sources are applied in ascending specificity order.

```css
p { color: black !important; }    /* wins over any .class without !important */
.intro { color: gray; }          /* loses to p !important */
```

### 4. Inline !important Declarations (highest priority)

```html
<p style="color: red !important;">This text is red</p>
```

## 4-Band Cascade Algorithm

The cascade engine in `computed_decls_in_prepared_stylesheets_with_context()` applies:

1. **Author normal** — for each matched rule, apply `rule.declarations` and `rule.keywords`
2. **Inline normal** — apply `inline_style.normal` and `inline_style.keywords_normal`
3. **Author `!important`** — for each matched rule with important declarations, apply `rule.important` and `rule.important_keywords`
4. **Inline `!important`** — apply `inline_style.important` and `inline_style.keywords_important`

Each layer uses a "Some-wins" field-level merge across all ~80+ `Style` fields. CSS-wide keywords in each layer displace matching values from earlier layers, and a later layer's value displaces an earlier layer's keyword for the same property.

## Inheritance

After the cascade resolves all explicit declarations, implicit inheritance fills in unset inherited properties from the parent's computed style.

### Inherited Properties

The following properties inherit from parent to child:

| Category | Properties |
|---|---|
| **Color** | `color` |
| **Font** | `font-family`, `font-size`, `font-weight`, `font-style` |
| **Text** | `line-height`, `letter-spacing`, `text-align`, `text-transform`, `text-decoration`, `white-space` |
| **Visibility** | `visibility` |
| **Interaction** | `cursor`, `pointer-events`, `user-select` |
| **SVG** | `fill`, `fill-opacity`, `fill-rule`, `stroke`, `stroke-width`, `stroke-opacity`, `stroke-linecap`, `stroke-linejoin`, `stroke-dasharray`, `stroke-dashoffset` |
| **Custom Properties** | All `--*` custom properties always inherit |

### Non-Inherited Properties

All box-model, layout, positioning, flex/grid, overflow, opacity, background, and border properties do **not** inherit.

### Inheritance Mechanism

The `inherit_into()` function in `wgpu-html-style` fills in inherited properties:

```rust
fn inherit_into(child: &mut Style, parent: &Style, keywords: &HashMap<String, CssWideKeyword>) {
  macro_rules! inherit {
    ($(($field:ident, $name:literal)),*) => {
      $(
        if child.$field.is_none()
          && !keywords.contains_key($name)
          && !child.reset_properties.contains($name)
        {
          child.$field = parent.$field.clone();
        }
      )*
    };
  }
  inherit!(
    (color, "color"),
    (font_family, "font-family"),
    (font_size, "font-size"),
    // ... all inherited properties
    (user_select, "user-select"),
  );
  // Custom properties always inherit
  // ...
}
```

Properties are only inherited if:
1. The child has no explicit value for the property (`None`)
2. No CSS-wide keyword was resolved for the property
3. The property was not explicitly reset by a shorthand

## CSS-Wide Keyword Resolution

Three CSS-wide keywords can appear on any property and are resolved against the parent's already-computed style:

| Keyword | Behaviour |
|---|---|
| `inherit` | Uses the parent's computed value (even for non-inherited properties) |
| `initial` | Resets to the CSS specification initial value |
| `unset` | `inherit` for inherited properties, `initial` for non-inherited |

```css
.widget {
  color: inherit;         /* force color from parent */
  display: initial;       /* reset display to inline */
  margin: unset;          /* reset margin to 0 (non-inherited = initial) */
}
```

Keywords are stored in side-car `HashMap<String, CssWideKeyword>` maps during the cascade and resolved by `apply_keyword()` after the cascade bands complete but before inheritance.

## Dynamic Pseudo-Class Integration

The cascade integrates with the document's interaction state through `MatchContext`:

```rust
pub struct MatchContext {
  pub is_hover: bool,
  pub is_active: bool,
  pub is_focus: bool,
  pub is_root: bool,
  pub is_first_child: bool,
  pub is_last_child: bool,
}
```

`MatchContext::for_path(path, interaction_state)` computes the context for any element:
- `:hover` — path is a prefix of `state.hover_path` (element is hovered or is an ancestor of the hovered element)
- `:active` — path is a prefix of `state.active_path`
- `:focus` — path exactly equals `state.focus_path`

### Incremental Re-Cascade

When interaction state changes (hover/active/focus), an incremental cascade runs:
1. Diff the old and new interaction snapshots
2. Collect dirty paths from the divergence point to both old and new leaves
3. If ancestor-compound pseudo-class rules exist (`div:hover .child`), also mark subtrees dirty
4. Re-cascade only the affected nodes in-place
5. If all pseudo-class rules are "paint-only" (no layout-affecting properties), skip re-layout

```rust
pub fn pseudo_rules_are_paint_only(tree: &Tree) -> bool {
  // Returns true when every pseudo-class rule only declares
  // paint properties (color, background, opacity, etc.)
}
```

## @media Query Evaluation

`@media` queries wrapping rule blocks are evaluated per rule during cascade:

```css
@media (min-width: 768px) {
  .sidebar { display: block; }
}

@media (max-width: 767px) {
  .sidebar { display: none; }
}
```

Supported media features:
- `width` / `min-width` / `max-width` (viewport width in CSS pixels)
- `height` / `min-height` / `max-height` (viewport height in CSS pixels)
- `orientation: portrait` / `orientation: landscape`

Media queries are evaluated against a `MediaContext`:

```rust
pub struct MediaContext {
  pub viewport_width: f32,
  pub viewport_height: f32,
  pub scale: f32,
  pub media_type: MediaType,
}
```

The cascade API accepts a `MediaContext`:

```rust
// Full cascade with media context
let result = cascade_with_media(tree, &MediaContext::screen(800.0, 600.0, 1.0));

// Incremental cascade with media
let changed = cascade_incremental_with_media(tree, &mut cached, &old_snapshot, &media);
```

Multiple `@media` conditions on a rule (via multiple `<style media="...">` sources) must ALL match for the rule's declarations to apply (`rule.media.iter().all(...)`).

## Linked Stylesheets and Style Blocks

The cascade collects CSS from two sources:

1. **`<style>` elements** — text content extracted from the DOM tree; `<style media="...">` wraps content in `@media { }`
2. **`linked_stylesheets` map** — `<link rel="stylesheet" href="...">` elements resolved against `Tree::linked_stylesheets: HashMap<String, String>`

Both are concatenated into a single CSS string, parsed into a `Stylesheet`, and cached. The same string always produces the same parsed stylesheet.

## Cascade Resolution Diagram

```
┌──────────────────────────────────────────────┐
│  1. Collect CSS source                        │
│     ├─ Walk DOM for <style> text content      │
│     ├─ Resolve <link> against linked_sheets   │
│     └─ Concatenate + parse → Stylesheet       │
├──────────────────────────────────────────────┤
│  2. Prepare stylesheets                       │
│     ├─ UA stylesheet (static, tag-only)       │
│     ├─ Author stylesheet (parsed + cached)    │
│     └─ Build rule index (by_id, by_class,     │
│        by_tag, universal)                     │
├──────────────────────────────────────────────┤
│  3. For each element (DFS):                   │
│     ├─ Build MatchContext from InteractionState│
│     ├─ Collect matching rules from index      │
│     ├─ Filter by @media                       │
│     ├─ Verify selectors with ancestor chain   │
│     ├─ Sort by (specificity, sheet, index)    │
│     └─ Apply 4-band cascade:                  │
│        ├─ Author normal declarations          │
│        ├─ Inline normal declarations          │
│        ├─ Author !important declarations      │
│        └─ Inline !important declarations      │
├──────────────────────────────────────────────┤
│  4. Post-cascade resolution                   │
│     ├─ Resolve CSS-wide keywords (inherit/     │
│     │  initial/unset) against parent           │
│     ├─ Inherit unset inherited properties      │
│     ├─ Inject programmatic custom properties   │
│     │  from Node::custom_properties            │
│     └─ Resolve var() references                │
├──────────────────────────────────────────────┤
│  5. Output: CascadedTree<CascadedNode>         │
│     Each node carries fully resolved Style    │
└──────────────────────────────────────────────┘
```
