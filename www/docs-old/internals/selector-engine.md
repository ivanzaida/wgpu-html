---
title: Selector Engine
---

# Selector Matching & Query Engine

How CSS selectors are parsed, represented, and matched against elements.

## Data Structures

**File:** `crates/lui-tree/src/query.rs`

### Selector Types (lines 60-153)

```
SelectorList          -- comma-separated union (OR)
  └─ ComplexSelector  -- chain of compounds with combinators
       ├─ compounds: Vec<CompoundSelector>
       └─ combinators: Vec<Combinator>   // ' ', '>', '+', '~'
```

**CompoundSelector** (line 116):
- `tag`, `id`, `classes`, `attrs`, `pseudo_classes`, `pseudo_element`, `never_matches`

**Combinator** (line 128): Descendant (space), Child (`>`), NextSibling (`+`), SubsequentSibling (`~`)

### Pseudo-Classes (line 61)

| Category | Pseudo-classes |
|---|---|
| Logical | `:not()`, `:is()`, `:where()`, `:has()` |
| Structural | `:first-child`, `:last-child`, `:only-child`, `:empty`, `:root`, `:nth-child()`, `:nth-last-child()`, `:first-of-type`, `:last-of-type`, `:nth-of-type()` |
| State | `:disabled`, `:enabled`, `:checked`, `:required`, `:optional`, `:read-only`, `:read-write`, `:placeholder-shown` |
| Interaction | `:hover`, `:focus`, `:active`, `:focus-within` |
| Other | `:lang()`, `:dir()` |

### Attribute Filters (line 22)

```rust
pub struct AttrFilter {
    name: String,
    op: AttrOp,              // Exists, =, ~=, |=, ^=, $=, *=
    value: Option<String>,
    case_insensitive: bool,  // [attr=value i]
}
```

## Parsing

| Function | Line | Purpose |
|---|---|---|
| `SelectorList::parse()` | 270 | Split on commas, parse each complex selector |
| `ComplexSelector::parse()` | 287 | Parse compound chain with combinators |
| `CompoundSelector::parse()` | 1053 | Parse tag, #id, .class, [attr], :pseudo |
| `parse_nth_formula()` | 768 | Parse An+B, "odd", "even", or integer |
| `parse_attr_filter()` | 863 | Parse `[name op="value" i]` |

`:has()` parses relative selectors with leading combinator via `parse_relative_selector()` (line 688).

`:nth-child()` supports `of S` syntax via `parse_nth_with_of()` (line 724).

## Matching Algorithm

### ComplexSelector Matching (line 1880)

`matches_in_tree()` evaluates right-to-left (subject compound first):

1. Test rightmost compound against target element
2. Walk left through combinators:
   - **Descendant** (space): walk up ancestors until match or root
   - **Child** (`>`): pop one ancestor level, must match
   - **NextSibling** (`+`): find previous element sibling, must match
   - **SubsequentSibling** (`~`): find any prior element sibling that matches

### SelectorList Matching (line 1982)

Returns `true` if ANY selector in the list matches (OR semantics).

### CompoundSelector Matching

A compound matches if ALL of its parts match (AND semantics):
- Tag: element tag name (case-insensitive)
- ID: element id attribute
- Classes: all listed classes present
- Attributes: all `AttrFilter`s satisfied (line 1172)
- Pseudo-classes: all satisfied per `match_pseudo_class()` (line 1244)

### Pseudo-Class Evaluation (line 1244)

**Structural pseudo-classes** use helper functions:
- `element_position_1based()` (line 1406) -- 1-based child index
- `first_of_type_index()` (line 1468) -- first sibling with matching tag
- `:nth-child(An+B of S)` uses `element_position_1based_of()` (line 1423) to count only matching children

**Interaction pseudo-classes** use `MatchContext`:
- `:hover` -- path is prefix of `hover_path` (matches element and ancestors)
- `:active` -- path equals `active_path`
- `:focus` -- path equals `focus_path`
- `:focus-within` -- `focus_path` starts with target path

**State pseudo-classes** test element properties:
- `:checked` -- `Input.checked` or `OptionElement.selected`
- `:disabled` -- element's `disabled` field
- `:read-only` -- `Input.readonly` or `Textarea.readonly`
- `:placeholder-shown` -- placeholder exists and value is empty

### :has() Matching (line 1621)

Handles relative selector syntax:
- `:has(> .child)` -- child combinator from subject
- `:has(+ .next)` -- next-sibling from subject
- `:has(~ .sibling)` -- subsequent-sibling from subject
- `:has(.descendant)` -- descendant (default)

## Specificity

**CompoundSelector::specificity()** (line 1130):

```
(id_count << 16) | (class_count << 8) | tag_count
```

- ID: 1 if present
- Classes: count of classes + attributes + pseudo-classes (`:where` contributes 0)
- Tag: 1 if present

**ComplexSelector::specificity()** (line 1876): sum of all compound specificities.

## Query API

**File:** `crates/lui-tree/src/query.rs` lines 2040-2151

| Method | Line | Purpose |
|---|---|---|
| `Node::query_selector()` | 2051 | First matching `&mut Node` |
| `Node::query_selector_all()` | 2058 | All matching nodes |
| `Tree::query_selector()` | 2105 | With `InteractionState` context |
| `Tree::query_selector_all()` | 2114 | Batch query with interaction state |
| `collect_matching_paths()` | 1989 | DFS to find all matching paths |
| `first_match_path()` | 2011 | Short-circuit on first match |

## Cascade Integration

**File:** `crates/lui-style/src/lib.rs`

The cascade uses a separate `MatchContext` (line 31) with boolean fields (`is_hover`, `is_active`, `is_focus`) rather than full paths. `MatchContext::for_path()` (line 74) computes these from `InteractionState`.

Rule indexing (line 202) pre-sorts rules into `by_id`, `by_class`, `by_tag`, `universal` buckets. During cascade, `matching_rules_for_element()` (line 1268) looks up candidates by the element's ID/classes/tag, then runs full selector matching only on candidates.
