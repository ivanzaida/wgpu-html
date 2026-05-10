---
sidebar_position: 3
---

# Supported Selectors

wgpu-html supports **full CSS Level 4 selectors** via `wgpu-html-tree::query`.

## Combinators

| Combinator | Syntax | Support |
|---|---|---|
| Descendant | `A B` | ✅ |
| Child | `A > B` | ✅ |
| Adjacent sibling | `A + B` | ✅ |
| General sibling | `A ~ B` | ✅ |

## Attribute Selectors

All operators with optional case sensitivity flags:

| Operator | Syntax | Support |
|---|---|---|
| Presence | `[attr]` | ✅ |
| Exact match | `[attr="val"]` | ✅ |
| Whitespace-separated | `[attr~="val"]` | ✅ |
| Exact or hyphen-prefix | `[attr\|="val"]` | ✅ |
| Prefix | `[attr^="val"]` | ✅ |
| Suffix | `[attr$="val"]` | ✅ |
| Substring | `[attr*="val"]` | ✅ |
| Case-insensitive | `[attr="val" i]` | ✅ |
| Case-sensitive | `[attr="val" s]` | ✅ |

## Basic Selectors

| Selector | Support |
|---|---|
| Type/tag | ✅ |
| Universal (`*`) | ✅ |
| ID (`#id`) | ✅ |
| Class (`.class`) | ✅ (multiple classes per element) |

## Logical Pseudo-classes

| Selector | Support | Notes |
|---|---|---|
| `:is()` | ✅ | Specificity = most specific argument |
| `:where()` | ✅ | Zero specificity |
| `:not()` | ✅ | Specificity = most specific argument |
| `:has()` | ✅ | Relative selector, matched via subtree walk |

## Structural Pseudo-classes

| Selector | Support | Notes |
|---|---|---|
| `:root` | ✅ | |
| `:empty` | ✅ | |
| `:nth-child(an+b)` | ✅ | Full formula support |
| `:nth-last-child(an+b)` | ✅ | |
| `:first-child` | ✅ | |
| `:last-child` | ✅ | |
| `:only-child` | ✅ | |
| `:nth-of-type(an+b)` | ✅ | |
| `:nth-last-of-type(an+b)` | ✅ | |
| `:first-of-type` | ✅ | |
| `:last-of-type` | ✅ | |
| `:only-of-type` | ✅ | |

## State Pseudo-classes

| Selector | Support | Notes |
|---|---|---|
| `:hover` | ✅ | Via `InteractionState::hover_path` |
| `:active` | ✅ | Via `InteractionState::active_path` |
| `:focus` | ✅ | Via `InteractionState::focus_path` |
| `:focus-within` | ✅ | In query engine (not cascade yet) |
| `:focus-visible` | ❌ | Not tracked |
| `:disabled` | ✅ | In query engine (not cascade yet) |
| `:enabled` | ✅ | In query engine |
| `:checked` | ✅ | In query engine (not cascade yet) |
| `:required` | ✅ | In query engine |
| `:optional` | ✅ | In query engine |
| `:read-only` | ✅ | In query engine |
| `:read-write` | ✅ | In query engine |
| `:placeholder-shown` | ✅ | In query engine |
| `:default` | ✅ | In query engine |

## Other Pseudo-classes

| Selector | Support |
|---|---|
| `:lang()` | ✅ |
| `:dir()` | ✅ |
| `:scope` | ✅ |

## Pseudo-elements

| Selector | Support | Notes |
|---|---|---|
| `::before` | ✅ | Content from `content` property |
| `::after` | ✅ | Content from `content` property |
| `::first-line` | ✅ | Color only |
| `::first-letter` | ✅ | Color only |
| `::placeholder` | ✅ | Input placeholder color |
| `::selection` | ✅ | Text selection colors |
| `::file-selector-button` | ✅ | File input button |
| `::lui-popup`, `::lui-canvas`, `::lui-input` | ✅ | Internal popup/picker pseudo-elements |
| `::lui-calendar-*` | ✅ | Calendar grid styling (~10 pseudo-elements) |

## Query Engine API

```rust
tree.query_selector("div.container > p:first-child");
tree.query_selector_all("input[type=text]:focus");
tree.query_selector_path("a.active");
tree.query_selector_all_paths("li:nth-child(odd)");
```

`Node` mirrors the same API for scoped queries relative to that node.
