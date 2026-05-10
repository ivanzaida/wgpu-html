---
title: CSS Cascade
---

# CSS Cascade & Style Resolution

How raw CSS text becomes a resolved `Style` on every DOM element.

## Overview

The cascade lives in `crates/lui-style/src/lib.rs`. Entry points:

| Function | Line | Purpose |
|---|---|---|
| `cascade()` | 447 | Full cascade with default media context |
| `cascade_with_media()` | 451 | Full cascade with viewport dimensions |
| `cascade_incremental_with_media()` | 509 | Re-cascade only dirty nodes (hover/focus changes) |

Output: `CascadedTree` containing a `CascadedNode` per element, each with a fully resolved `Style`.

## Stylesheet Parsing

**File:** `crates/lui-parser/src/stylesheet.rs`

| Function | Line | Purpose |
|---|---|---|
| `parse_stylesheet()` | 89 | Entry: strip comments, parse rules recursively |
| `parse_rules()` | 185 | Recursive rule parser with `@media` nesting |
| `parse_selector_list()` | 336 | Delegates to query engine's selector parser |
| `parse_inline_style_decls()` | css_parser.rs:78 | Parse `style="..."` attributes |

Each parsed `Rule` contains:

```rust
pub struct Rule {
    selectors: SelectorList,
    declarations: Style,                        // normal declarations
    important: Style,                           // !important declarations
    keywords: HashMap<ArcStr, CssWideKeyword>,  // inherit/initial/unset (normal)
    important_keywords: HashMap<...>,           // inherit/initial/unset (!important)
    media: Vec<MediaQueryList>,                 // enclosing @media conditions
}
```

### Inline Style Parsing

`parse_inline_style_decls()` in `css_parser.rs` separates declarations into four buckets:
- `normal: Style` + `keywords_normal` -- regular declarations
- `important: Style` + `keywords_important` -- `!important` declarations

The `!important` flag is detected via `strip_important()` and routes to the appropriate bucket.

## Specificity

**File:** `crates/lui-tree/src/query.rs`

| Function | Line | Purpose |
|---|---|---|
| `CompoundSelector::specificity()` | 1130 | Per-compound: `(id << 16) \| (class_count << 8) \| tag` |
| `ComplexSelector::specificity()` | 1876 | Sum of all compounds in chain |

Formula: 24-bit value with 8 bits each for IDs, classes (+ attrs + pseudo-classes except `:where`), and tags.

## Cascade Order

The cascade follows CSS-Cascade-3 section 6.4 (computed in `lib.rs` lines 1224-1246):

```
1. Author normal rules      (ascending specificity, stable on source order)
2. Inline normal declarations
3. Author !important rules   (ascending specificity)
4. Inline !important declarations
```

Each layer is applied via `apply_layer()` (line 1424): keywords clear matching values first, then values merge.

## Rule Indexing

**File:** `crates/lui-style/src/lib.rs` lines 202-283

To avoid testing every rule against every element, rules are indexed:

```rust
struct RuleIndex {
    by_id: HashMap<String, Vec<SelectorRuleRef>>,
    by_class: HashMap<String, Vec<SelectorRuleRef>>,
    by_tag: HashMap<String, Vec<SelectorRuleRef>>,
    universal: Vec<SelectorRuleRef>,
}
```

During cascade, `matching_rules_for_element()` (line 1268) looks up candidate rules by the element's ID, classes, and tag. Only candidates are tested for full selector match.

## Cascade Node Walk

`cascade_node()` (line 879) processes one element recursively:

1. Compute `MatchContext` from element path and `InteractionState`
2. Collect matching rules from all prepared stylesheets
3. Apply cascade bands (normal -> inline -> important -> inline-important)
4. Resolve CSS-wide keywords (`inherit`/`initial`/`unset`)
5. Inherit from parent for inherited properties
6. Resolve `var()` references
7. Recurse into children

## Inheritance

**Function:** `inherit_into()` at `lib.rs` line 990

Inherited properties are copied from parent to child if the child has no explicit value. The inherited set includes:

- **Text:** `color`, `font-family`, `font-size`, `font-weight`, `font-style`, `line-height`, `letter-spacing`, `text-align`, `text-transform`, `white-space`, `text-decoration`, `visibility`, `cursor`
- **SVG:** all `fill-*` and `stroke-*` properties
- **Custom properties:** all `--*` variables always inherit

Non-inherited properties remain at their initial value (typically `None`) unless explicitly set.

## CSS-Wide Keywords

**File:** `crates/lui-parser/src/style_props.rs` lines 217-283

```rust
enum CssWideKeyword {
    Inherit,  // use parent's computed value
    Initial,  // reset to spec initial (None)
    Unset,    // inherit if inherited property, initial otherwise
}
```

`apply_keyword()` matches the property name and applies the keyword. The `all` shorthand applies the keyword to every property.

Keywords are stored in side-car `HashMap<ArcStr, CssWideKeyword>` maps and resolved during cascade before inheritance runs.

## var() Resolution

**File:** `crates/lui-parser/src/css_parser.rs` lines 575-680

Two-phase resolution in `resolve_var_references()`:

**Phase 1: Custom property chains** (line 582)
Resolve `var()` inside custom-property values so chains like `--a: var(--b); --b: 10px` collapse. Tracks a `resolving` set to prevent infinite recursion.

**Phase 2: Regular property substitution** (line 597)
For each entry in `var_properties` (properties with unresolved `var()`), substitute variables and re-parse via `apply_css_property()`.

Detection during parsing: if a value contains `var(`, it's stored in `var_properties` instead of typed fields (line 332).

## @media Evaluation

**File:** `crates/lui-style/src/lib.rs` lines 1339-1371

Supported media features:

| Feature | Type |
|---|---|
| `width` / `min-width` / `max-width` | px comparison |
| `height` / `min-height` / `max-height` | px comparison |
| `orientation` | portrait / landscape |

`MediaContext` carries `viewport_width`, `viewport_height`, `scale`, and `media_type`.

Each rule's `media` conditions are ANDed. Within a `MediaQueryList`, queries are ORed (comma-separated).

## Incremental Re-Cascade

**Function:** `cascade_incremental_with_media()` at `lib.rs` line 509

When hover/focus/active state changes:

1. Compute new `InteractionSnapshot` from current state
2. Diff against old snapshot to find which pseudo-classes changed
3. Check if any stylesheet uses those pseudo-classes (via `PseudoClassUsage`)
4. Collect dirty paths: nodes along old and new pseudo-class paths
5. If ancestor-compound pseudo-rules exist, mark entire subtrees dirty
6. Walk cascaded tree, re-cascade only dirty nodes in-place
7. Return `true` if any node changed (layout must re-run)

### Paint-Only Optimization

`pseudo_rules_are_paint_only()` (line 491) checks if all pseudo-class rules only affect paint properties (color, opacity, background-color) and not layout properties (width, padding, display). If true, `PipelineCache` can skip re-layout and patch colors in-place.

## Output Structure

```rust
pub struct CascadedTree {
    root: Option<CascadedNode>,
}

pub struct CascadedNode {
    element: Element,
    style: Style,                          // fully resolved
    children: Vec<CascadedNode>,
    before: Option<PseudoElementStyle>,    // ::before
    after: Option<PseudoElementStyle>,     // ::after
}
```

`CascadedNode::at_path(path: &[usize])` (line 134) navigates to a descendant by child-index path.

## Style Struct

**File:** `crates/lui-models/src/css/style.rs` line 12

~80 typed fields covering layout, colors, borders, text, flexbox, grid, overflow, visual effects, SVG, plus:

- `deferred_longhands: HashMap<ArcStr, ArcStr>` -- recognized but unmodeled properties
- `custom_properties: HashMap<ArcStr, ArcStr>` -- `--name: value` pairs (always inherited)
- `var_properties: HashMap<ArcStr, ArcStr>` -- properties with unresolved `var()` refs
- `reset_properties: HashSet<ArcStr>` -- longhands reset by shorthand declarations
- `keyword_reset_properties: HashSet<ArcStr>` -- longhands reset by CSS-wide keyword on shorthand
