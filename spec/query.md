# wgpu-html — `querySelector` / `querySelectorAll` Spec

DOM-style CSS selector lookup on the engine's element tree. Lives
inside `wgpu-html-tree::query` and is exposed through the inherent
methods on [`Tree`] and [`Node`]. Companion to `spec/css.md` (which
covers the cascade-side selector engine driving paint) and
`spec/interactivity.md` (which covers element state used by future
state-aware pseudo-classes).

This file is the source of truth for "what `querySelector*` accepts
today and where it's heading".

---

## 0. Status (2026-05-03)

| Phase | Feature | State |
|---|---|---|
| 1 | Universal `*` | ✅ |
| 1 | Type `E` (ASCII case-insensitive) | ✅ |
| 1 | ID `#id` | ✅ |
| 1 | Class `.class` | ✅ |
| 1 | `[attr]` presence | ✅ |
| 1 | `[attr=v]`, `[attr="v"]`, `[attr='v']` | ✅ |
| 1 | `[attr~=v]` (whitespace token) | ✅ |
| 1 | `[attr\|=v]` (dash-match) | ✅ |
| 1 | `[attr^=v]` (prefix) | ✅ |
| 1 | `[attr$=v]` (suffix) | ✅ |
| 1 | `[attr*=v]` (substring) | ✅ |
| 1 | `[attr=v i]` / `[attr=v s]` (case flags) | ✅ |
| 1 | Compound (`a.btn[disabled]`) | ✅ |
| 1 | Selector list (`A, B, C`, dedup) | ✅ |
| 1 | Descendant ` ` | ✅ |
| 1 | Child `>` | ✅ |
| 1 | Next sibling `+` | ✅ |
| 1 | Subsequent sibling `~` | ✅ |
| 2 | `:root`, `:scope`, `:empty` | ✅ |
| 2 | `:first-child`, `:last-child`, `:only-child` | ✅ |
| 2 | `:nth-child()`, `:nth-last-child()` | ✅ |
| 2 | `:first-of-type`, `:last-of-type`, `:nth-of-type()` | ✅ |
| 3 | `:is()`, `:where()`, `:not()` | ✅ |
| 3 | `:has()` | ✅ |
| 4 | `:hover`, `:focus`, `:active`, `:focus-within` | ✅ |
| 4 | `:disabled`, `:enabled`, `:checked`, `:required`, `:optional` | ✅ |
| 4 | `:read-only`, `:read-write`, `:placeholder-shown` | ✅ |
| 4 | `:lang(...)`, `:dir(...)` | ✅ |
| ext | Pseudo-elements (`::before`, `::after`, `::first-line`) | ⚠️ Parse accepted; always match nothing |
| ext | Namespace prefixes (`svg\|circle`) | ❌ |
| ext | CSS escape sequences in identifiers | ❌ |
| ext | `:has()` / scope-relative selectors | ❌ |

**Form-state pseudo-classes via attribute filters.** The cascade
engine already exposes per-element booleans through
`Element::attr` (`disabled`, `checked`, `required`, `readonly`,
`selected`, `multiple`, `autofocus`), so `[disabled]`, `[checked]`,
etc. work today even though the dedicated pseudo-class form
(`:disabled`, `:checked`) doesn't parse yet.

Anything in the ❌ rows parses as an error inside
`SelectorList::parse`, which the lenient `From<&str>` impl collapses
into the "matches nothing" sentinel — so `tree.query_selector(":hover")`
returns `None`, never panics.

---

## 1. Goals

- Selector syntax that's a strict subset of CSS Level 4 — anything
  we accept must mean the same thing as in a browser.
- One traversal per call. `query_selector_all` walks the tree in
  document-order DFS exactly once and returns matches deduplicated
  by element identity (so `A, B` doesn't double-count an element
  matching both arms).
- Paths in the selector layer are interchangeable with paths in the
  rest of `wgpu-html-tree` (focus chain, scroll-offset map keys,
  `at_path_mut`). Anything you obtain from `query_selector_path`
  can be fed straight into `Tree::at_path_mut`,
  `wgpu_html::screenshot_node_to`, etc.
- No allocation in the hot path of single-element matching
  (`CompoundSelector::matches(&Element)`).

## 2. Non-goals

- No CSSOM. There's no JavaScript-facing wrapper, no
  `Element.matches()` reflection, no live `NodeList`.
- No specificity calculation. `query_selector*` cares about whether
  an element matches, not which selector "wins" — that's the
  cascade's job (`spec/css.md`).
- No reactive subscriptions. Hosts that need to track changes
  re-query after each tree mutation; cheap because the tree is
  in-memory and walks are O(N) per call.
- No selector-list result ordering by selector index. Matches are
  always in document order (the WHATWG `querySelectorAll` semantics).

---

## 3. Public API

All methods are inherent on [`Tree`] and [`Node`]. The
`*_path` variants return owned, `Send`-friendly child-index paths
instead of borrows; the borrowing variants need an exclusive `&mut`
on the receiver.

```rust
// On Tree
fn query_selector(&mut self, sel: impl Into<SelectorList>) -> Option<&mut Node>;
fn query_selector_all(&mut self, sel: impl Into<SelectorList>) -> Vec<&mut Node>;
fn query_selector_path(&self, sel: impl Into<SelectorList>) -> Option<Vec<usize>>;
fn query_selector_all_paths(&self, sel: impl Into<SelectorList>) -> Vec<Vec<usize>>;

// On Node — same signatures, scoped to descendants of `self`.
```

Match scope:
- `Tree::query_selector*` searches the whole tree starting at `Tree::root`.
- `Node::query_selector*` searches `self` and its descendants.
- An empty path means the receiver itself matched.

Lookup-by-id remains available as a focused helper:

```rust
Tree::get_element_by_id(&mut self, id: &str) -> Option<&mut Node>;
```

`get_element_by_id` is identical to `query_selector(format!("#{id}"))`
modulo the parse step.

### 3.1 Selector types

```rust
pub struct SelectorList { /* OR-combined complex selectors */ }
pub struct ComplexSelector { compounds, combinators }
pub struct CompoundSelector { tag, id, classes, attrs, … }
pub enum   Combinator { Descendant, Child, NextSibling, SubsequentSibling }
```

All four types live in `wgpu_html_tree` and re-export from the crate
root. `Into<SelectorList>` is implemented for `&str`, `String`,
`&String`, owned/borrowed `CompoundSelector`, owned/borrowed
`ComplexSelector`, and owned/borrowed `SelectorList`. So:

```rust
// One-shot &str — parses each call.
tree.query_selector("a.btn#cta")?;

// Pre-parse once for hot loops or repeated dispatch.
let sel = SelectorList::parse("a.btn[disabled], button[disabled]")?;
for tree in trees.iter_mut() {
    for n in tree.query_selector_all(&sel) {
        n.on_click = None;
    }
}

// Direct single-element test (no allocations).
let sel = CompoundSelector::parse("input[type=password]")?;
if sel.matches(&node.element) { /* … */ }
```

### 3.2 Strict vs. lenient parsing

| Constructor | Returns | Error → |
|---|---|---|
| `SelectorList::parse(&str)` | `Result<SelectorList, String>` | err string with offending char |
| `ComplexSelector::parse(&str)` | `Result<ComplexSelector, String>` | err string |
| `CompoundSelector::parse(&str)` | `Result<CompoundSelector, String>` | err string |
| `SelectorList::from(&str)` | `SelectorList` | empty list (matches nothing) |
| `CompoundSelector::from(&str)` | `CompoundSelector` | `never_matches` sentinel |

`query_selector*` always go through the lenient path — invalid
syntax silently yields no match, same shape as a valid-but-unmatched
selector. Use `SelectorList::parse` up front when you need to
distinguish "wrong syntax" from "no element matches".

---

## 4. Grammar

```
SelectorList    := ComplexSelector ("," ComplexSelector)*
ComplexSelector := CompoundSelector (Combinator CompoundSelector)*
Combinator      := WS+                       # descendant
                 | WS* ">" WS*                # child
                 | WS* "+" WS*                # next sibling
                 | WS* "~" WS*                # subsequent sibling
CompoundSelector := (Type | Universal)? Suffix*
Type             := IdentChar+
Universal        := "*"
Suffix           := "#" Ident
                  | "." Ident
                  | "[" AttrFilter "]"
AttrFilter       := WS* Ident WS*
                    ( ("=" | "~=" | "|=" | "^=" | "$=" | "*=") WS* Value WS* CaseFlag? )?
Value            := Ident | "\"" .*? "\"" | "'" .*? "'"
CaseFlag         := "i" | "I" | "s" | "S"
Ident            := IdentStart IdentChar*
IdentStart       := [A-Za-z_-]
IdentChar        := [A-Za-z0-9_-]
```

Compound terminators (whitespace / `>` / `+` / `~` / `,` / EOF) end
the current compound. Whitespace between two compounds is
significant — it's the descendant combinator.

Top-level commas split selector lists; `[]` and `()` nesting (for
future `:is()` etc.) is tracked, and quoted strings inside `[]`
swallow commas correctly.

---

## 5. Element model bridge

Selectors look up element state through three methods on
`wgpu_html_tree::Element`:

| Method | Returns | Used by |
|---|---|---|
| `tag_name() -> &'static str` | lower-case HTML tag, or `"#text"` | `Type` matching |
| `id() -> Option<&str>` | `id` attribute | `#id` matching |
| `class() -> Option<&str>` | raw `class` attribute (whitespace-separated) | `.class`, `[class~=…]` |
| `attr(name) -> Option<String>` | per-attribute look-up (case-insensitive name) | every `[attr…]` filter |

`Element::attr` covers:

- Global HTML attrs: `id`, `class`, `title`, `lang`, `tabindex`,
  `hidden`, `style` (via the `all_element_variants!` macro).
- `data-*` and `aria-*` map look-ups.
- Per-variant string attributes: `type`, `name`, `value`,
  `placeholder`, `href`, `src`, `alt`, `for`, `content`,
  enum-stringified for `Input.type` / `Button.type`.
- HTML boolean attributes (`disabled`, `readonly`, `required`,
  `checked`, `selected`, `multiple`, `autofocus`) reflect as
  `Some(String::new())` when set, `None` otherwise — so `[disabled]`
  matches a true boolean and `[disabled=""]` matches it too,
  mirroring browser behaviour.

Anything else returns `None` and therefore doesn't match value-form
attribute selectors.

`Text` nodes have no attributes and never match a selector (not
even `*`) — text-vs-element selection happens at the layout / paint
layer, not via `querySelector`.

---

## 6. Matching semantics

`SelectorList::matches(root, path)` is the test the walk uses for
each candidate node. It's defined as:

```
SelectorList    OR over its ComplexSelectors
ComplexSelector right-to-left walk over compounds + combinators
CompoundSelector AND over tag, id, classes, attrs
AttrFilter       op-specific comparison with optional `i` flag
```

### 6.1 Right-to-left combinator walk

Given `A B > C` and a candidate node:

1. The candidate must satisfy `C` (the *subject*).
2. Walk back: pop one path component → check if the parent
   satisfies `B` (Child).
3. Walk back: keep popping until any ancestor satisfies `A`
   (Descendant).

Pseudo-code:

```
fn matches(complex, root, path):
    if not compounds.last().matches(node_at(root, path)): return false
    cur = path
    for (compound, combinator) in zip(reverse(prev compounds), reverse(combinators)):
        match combinator:
          Descendant       -> walk ancestors of cur; require any to match
          Child            -> require parent of cur to match
          NextSibling      -> require previous element sibling of cur to match
          SubsequentSibling-> require any earlier element sibling to match
        cur = the matched ancestor / sibling
    true
```

Sibling combinators (`+`, `~`) skip raw `Element::Text` children
when locating the previous element sibling, matching the CSS spec's
"element sibling" notion. (Without this, `<p>foo<span>bar</span></p>`
would parse `Text + span` as a valid relationship; per spec `+`
means "previous *element* sibling".)

### 6.2 Attribute operator semantics

| Op | Matches when |
|---|---|
| `[a]` | attr present |
| `[a=v]` | attr equals `v` (case-sensitive by default) |
| `[a~=v]` | attr's whitespace-separated tokens include `v` |
| `[a\|=v]` | attr equals `v` or starts with `v-` |
| `[a^=v]` | attr starts with `v` |
| `[a$=v]` | attr ends with `v` |
| `[a*=v]` | attr contains `v` as a substring |

The empty needle (e.g. `[a^=""]`) is true when the attribute
is present, never false. `[attr]` itself is the canonical presence
test.

### 6.3 Case sensitivity

| Element | Default | `i` flag |
|---|---|---|
| Attribute name | ASCII case-insensitive | n/a (always insensitive) |
| Attribute value | case-sensitive | ASCII case-insensitive |
| Type selector | ASCII case-insensitive | n/a |
| `#id` | case-sensitive | n/a |
| `.class` | case-sensitive | n/a |

`s` flag is also accepted as a no-op (it's the default) so
selectors copy-pasted from CSS sources containing explicit
`[type=text s]` parse cleanly.

### 6.4 Selector-list deduplication

`tree.query_selector_all("A, B")` walks the tree once and emits each
matched element a single time. If both `A` and `B` match the same
element, it appears once in the result — same as
`document.querySelectorAll` in browsers.

### 6.5 Document order

All four `*_all*` methods produce matches in document order
(depth-first, root → child[0] → child[0][0] → … → child[1] → …).
`*_path` variants return paths in the same order. The path is
relative to the receiver: empty path means "self".

---

## 7. Examples

```rust
// Find the first password field.
tree.query_selector(r#"input[type="password"]"#);

// Submit buttons in any form.
tree.query_selector_all("form button[type=submit]");

// Visited-style links — both `https://` and `http://`, case-insensitive scheme.
tree.query_selector_all(r#"a[href^="http" i]"#);

// Form controls in an error row, matching either selector.
tree.query_selector_all(".row.error input, .row.error select, .row.error textarea");

// Direct children of the toolbar.
tree.query_selector_all("#toolbar > button");

// Label immediately followed by a required field.
tree.query_selector_all("label + input[required]");

// Subsequent-sibling: any error row after the submit button.
tree.query_selector_all("button[type=submit] ~ .error");

// Substring + dash-match: PDFs in language-tagged English regions.
tree.query_selector_all(r#"[lang|=en] a[href$=".pdf"]"#);

// Reusable parsed selector for hot loops.
let cta = wgpu_html_tree::SelectorList::parse(
    "a.cta:not(.disabled), button.cta:not(.disabled)"
).unwrap_or_default(); // pseudo-classes still err → empty list, OK
```

The last example illustrates a graceful-degradation pattern: if a
selector contains syntax we don't yet support, the lenient
`From<&str>` path returns an empty list, and the call site doesn't
crash — it just doesn't match anything. Use `SelectorList::parse`
when you need to know.

---

## 8. Integration points

### 8.1 Demo stdin command

`crates/wgpu-html-demo/src/winit.rs` runs a stdin reader thread that
parses `make_screenshot [selector]` lines, queues a command, and
wakes the event loop with `window.request_redraw()`. Inside the
per-frame hook the queue drains, calls
`tree.query_selector_path(sel)` to map the selector to a DOM path,
and forwards to `wgpu_html::screenshot_node_to` for an off-screen
node-sized capture. So everything in §0's ✅ list works out of
the box from the demo:

```
> make_screenshot input[type="password"]
> make_screenshot a[href^="https://"][rel~="noopener"]
> make_screenshot label + input[type="password"]
> make_screenshot form > .row > input
```

### 8.2 Layout / paint paths

DOM child indices and layout-tree child indices line up 1:1 in the
current engine (no anonymous boxes are inserted between cascade and
layout). So a path obtained from `query_selector_path` plugs
directly into `wgpu_html::layout_at_path` and
`wgpu_html::screenshot_node_to`. If layout ever starts inserting
anonymous boxes (line boxes, table-row groups, etc.), this
correspondence is what would need to be reviewed.

### 8.3 Cascade engine

The cascade in `wgpu-html-style` has its own selector matcher
because it has different requirements: per-rule specificity,
streaming over an entire stylesheet, hot-path performance during
paint. The two engines share `Element::attr` / `Element::class` /
`Element::tag_name`, so element-state coverage stays in sync, but
they don't share a parser. See `spec/css.md` §4 for the cascade-side
selector subset.

---

## 9. Limitations and known gaps

- **Pseudo-elements like `::before`/`::after`** are parsed but always return no match. Per CSS spec, `querySelector` should ignore pseudo-elements.
- **No namespace prefixes.** `svg|circle` doesn't parse. Our model
  doesn't carry namespaces anyway.
- **No CSS escape sequences.** Selectors like `#has\.dot` or
  `.\31 23` aren't parsed correctly — identifier characters are
  the literal `[A-Za-z0-9_-]` set.
- **`get_element_by_id` is not faster than `query_selector("#id")`**
  because the tree doesn't maintain an id index. If id lookup
  becomes a bottleneck in larger trees, we'll add a lazy
  `HashMap<String, Vec<usize>>` keyed by id; see roadmap.

## 10. Extension plan

Remaining items, in expected priority order:

1. **Pseudo-elements.** `::before` / `::after` currently parse but always return no match. CSS spec says `querySelector` should ignore them — current behavior is conformant but could be more explicit.
2. **CSS escape sequences.** Backslash-escaped identifiers (e.g. `#has\.dot`).
3. **Namespace prefixes.** `svg|circle` style selectors for future SVG namespace support.
4. **Indexed id lookup.** Behind a feature flag that maintains
   `HashMap<String, Vec<usize>>` on tree mutation. Needed only if
   profiling shows id lookup dominating.

---

## 11. Tests

Selector tests live in `crates/wgpu-html-tree/src/query.rs`'s
`#[cfg(test)] mod tests`. Coverage matrix (truncated, current count
~25):

- Compound parsing (existing grammar, attribute operators, case flags).
- Each attribute operator with realistic values (class tokens, lang
  dash-match, href prefix/suffix/substring).
- All four combinators against a small but representative tree.
- Selector-list union, including ancestor/descendant overlap dedup.
- Conversion impls (`&str`, `String`, owned/borrowed
  `CompoundSelector` and `SelectorList`).
- Empty tree safety.
- Lenient path: pseudo-classes don't crash, just match nothing.
- Mutable borrow round-trip (mutate a node found by selector,
  verify the change sticks).
- The original user case (`input[type="password"]`).

When adding a new feature, drop a test next to the existing ones
with the section comment that names the feature.
