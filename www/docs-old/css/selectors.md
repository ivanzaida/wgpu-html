---
title: CSS Selectors
---

# CSS Selector Reference

> **Note:** The selectors documented below work in stylesheet cascade matching. The `query_selector` and `query_selector_all` APIs in `lui-tree` additionally support child (`>`), sibling (`+` / `~`) combinators, attribute selectors (`[attr]`, `[attr=val]`, etc.), and many pseudo-classes including `:first-child`, `:last-child`, `:nth-child()`, `:not()`, `:is()`, `:where()`, `:has()`, `:root`, `:scope`, `:lang()`, `:dir()`, `:focus-within`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:placeholder-shown`, `:first-of-type`, `:last-of-type`, `:nth-of-type()`, and `:nth-last-child()`.

## Selector Grammar

lui's stylesheet parser supports a flat CSS selector syntax without at-rules or complex pseudo-classes. Each selector is a compound of optional tag, optional `#id`, optional `.class`(es), and optional static pseudo-classes, separated by a descendant combinator (whitespace) or comma.

```
selector_list = selector ("," selector)*
selector      = compound (whitespace compound)*
compound      = tag? id? class* pseudo_class* | "*" class* pseudo_class*
```

## Tag Selectors

Match elements by their HTML tag name:

```css
div { display: block; }
p { margin: 1em 0; }
h1 { font-size: 2em; }
input { padding: 4px 8px; }
```

## ID Selectors

Match a single element by its `id` attribute. Prefixed with `#`:

```css
#main { width: 800px; }
#sidebar { background-color: #f0f0f0; }
```

IDs use the `by_id` index for O(1) lookup during cascade.

## Class Selectors

Match elements by their `class` attribute. Prefixed with `.`:

```css
.highlight { background-color: yellow; }
.button { padding: 8px 16px; }
```

**Multi-class selectors** require all listed classes to match:

```css
.button.primary { background-color: blue; }
/* Matches: <div class="button primary"> */
/* Does NOT match: <div class="button"> */
```

## Universal Selector

The `*` selector matches any element:

```css
* { box-sizing: border-box; }
```

When combined with classes, `*.button` is equivalent to `.button`.

## Descendant Combinator

Whitespace between compounds represents the descendant combinator — it matches an element that is a descendant (at any depth) of the first compound:

```css
nav a { color: blue; }
/* Matches <a> inside <nav>, at any nesting level */

.card .title { font-weight: bold; }
/* Matches .title that is a descendant of .card */

div:hover .tooltip { display: block; }
/* Matches .tooltip inside a hovered div */
```

The cascade engine builds an ancestor chain for each element during the recursive walk. `div p` matches an element `<p>` if any ancestor in the chain is `<div>`.

## Comma-Separated Selector Lists

Commas group multiple selectors sharing the same declarations:

```css
h1, h2, h3 {
  font-family: sans-serif;
  color: #333;
}

#sidebar .link, #footer .link {
  color: #666;
}
```

The comma separates the full selector — the descendant combinator does not distribute over commas.

## Pseudo-Classes in Cascade Matching

The stylesheet parser and cascade support three dynamic pseudo-classes in selector matching:

- **`:hover`** — matches when the element's path is a prefix of the document's hover path
- **`:active`** — matches when the element's path is a prefix of the document's active path
- **`:focus`** — matches when the element's path exactly equals the document's focus path

```css
a:hover {
  color: red;
  text-decoration: underline;
}

button:active {
  background-color: #0044cc;
}

input:focus {
  border-color: blue;
}
```

Pseudo-classes can appear on ancestor compounds in descendant selectors:

```css
.row:hover .actions {
  visibility: visible;
}
```

Other pseudo-classes like `:first-child` and `:nth-child()` are supported in the `query_selector` API but not in stylesheet cascade matching.

## Specificity Calculation

Selector specificity is computed as a packed integer:

```
specificity = (id_count << 16) | (class_count << 8) | tag_count
```

| Selector | ID | Class | Tag | Specificity |
|---|---|---|---|---|
| `p` | 0 | 0 | 1 | 1 |
| `.button` | 0 | 1 | 0 | 256 |
| `div.button` | 0 | 1 | 1 | 257 |
| `#main` | 1 | 0 | 0 | 65536 |
| `#main .button` | 1 | 1 | 0 | 65792 |
| `*` | 0 | 0 | 0 | 0 |
| `nav a` | 0 | 0 | 2 | 2 |

Higher specificity wins. When specificity is equal, source order determines the winner (later rules override earlier rules).

The cascade sorts matched rules by `(specificity, sheet_index, rule_index)`. The UA stylesheet uses tag-only selectors, so any author rule with equal or higher specificity wins.

## Code Examples

### Simple Selector Example

```css
/* Tag selector — specificity 1 */
p { color: #333; }

/* Class selector — specificity 256 */
.note { color: blue; }

/* ID selector — specificity 65536 */
#banner { color: red; }

/* Descendant — specificity 2 */
article p { line-height: 1.6; }

/* Combined — specificity 257 */
p.warning { color: orange; font-weight: bold; }
```

### Specificity Override Example

```css
/* specificity: 0,0,1 */
p { color: black; }

/* specificity: 0,1,0 — wins over p */
.error { color: red; }

/* specificity: 0,1,1 — wins over both */
p.error { color: darkred; }

/* specificity: 1,0,1 — wins over everything above */
#container p { color: navy; }
```

### Hover with Descendant

```css
.card {
  border: 1px solid #ddd;
  padding: 16px;
}

.card:hover {
  border-color: blue;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.card:hover .title {
  color: blue;
}

.card:hover .actions {
  display: flex;
}
```

### Rust: Selector Matcher API

```rust
use lui_style::{matches_selector, matches_selector_in_tree, MatchContext};

// Match without ancestor context
let selector = /* parsed Selector */;
let matches = matches_selector(&selector, &element);

// Match with ancestor chain (for descendant combinators)
let ancestors: Vec<&Element> = vec![&parent_element];
let matches = matches_selector_in_tree(&selector, &element, &ancestors);

// Match with dynamic pseudo-class state
let ctx = MatchContext {
    is_hover: true,
    is_active: false,
    is_focus: false,
    is_root: false,
    is_first_child: true,
    is_last_child: false,
};
let matches = matches_selector_with_context(&selector, &element, &ctx);
```
