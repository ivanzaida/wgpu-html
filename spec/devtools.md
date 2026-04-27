# wgpu-html — DevTools Spec

A design for browser-style inspector tooling over the wgpu-html
pipeline. Mirrors the major panels of Chrome / Firefox DevTools
(Elements, Styles, Computed, Layout / Box-model), scoped down to the
features this engine actually exposes.

This spec is intentionally staged: each phase ends in something a host
app can demo, even before text rendering / a panel UI exist. The early
phases produce **data** and **overlays**, the later phases produce
**panels**.

Companion to `roadmap.md` (engine milestones) and `status.md`
(implementation snapshot).

---

## 1. Goals

- **Inspect**: from a host app or test, identify an element, dump its
  computed style, attributes, layout rectangles, render path.
- **Pick**: hover with the mouse, see the element under the cursor
  outlined; click to lock selection.
- **Mutate**: change an element's inline `style`, attributes, text,
  or visibility from devtools, then watch the next frame reflect it.
- **Diagnose**: surface frame timings, display-list size, cascade
  miss / hit counts, layout invalidations.

The engine is single-process, single-window. There is no remote
debugger protocol, no scripting, and the page document is supplied
by the host (no URL bar). Devtools is an **in-process tool** with
a public API plus an opt-in overlay / panel.

## 2. Non-goals

- No JavaScript debugging — the engine has no JS engine.
- No network panel — no fetcher.
- No source-map / breakpoint support.
- No accessibility tree panel until an accessibility tree exists.
- No remote inspector / Chrome-DevTools-Protocol surface (could be a
  follow-up; out of scope here).
- No multi-document / multi-tab story; one `Tree` + `LayoutBox` per
  inspector session.

## 3. Architecture

```
                          ┌──────────────────────────┐
            host app ───▶ │ wgpu_html_devtools::Inspector │
                          │   • snapshot()            │
                          │   • pick_at(point)        │
                          │   • mutate_inline_style() │
                          │   • mutate_text()         │
                          │   • frame_stats()         │
                          └────────────┬─────────────┘
                                       │
                                       │ borrows Tree + LayoutBox
                                       │ (no copies; no extra storage)
                                       ▼
                         existing pipeline (parser → cascade
                                  → layout → paint → render)
                                       │
                                       │ overlay quads pushed into
                                       │ DisplayList just before submit
                                       ▼
                         wgpu_html_renderer::Renderer
```

A new crate `wgpu-html-devtools` owns:

- `Inspector` — the host-facing handle. Stateless except for the
  current `selection: Option<Vec<usize>>` (path into the tree) and a
  set of mutation overrides. Holds no copies of the tree.
- `Overlay` — a `DisplayList` builder that emits quads for the
  selected / hovered element's box-model layers (margin / border /
  padding / content). Plain quad rendering — works today on the
  existing pipeline.
- `Panel` — the side-panel UI itself, **built as a wgpu-html
  document**. The inspector keeps a second `Tree` + `LayoutBox` (the
  "devtools document") authored by the inspector code as HTML+CSS
  strings, laid out and painted by the same engine that paints the
  page. No egui, no other UI toolkit; the engine eats its own dog
  food.

### Why a separate crate

- Keeps the engine crates (parser / style / layout / renderer) free
  of inspector-specific data (selection state, mutation overrides,
  serde-json, etc.).
- Lets host apps opt out completely (zero runtime cost).
- Lets tests link `wgpu-html-devtools` for headless inspection
  scenarios without dragging in a window.

### Self-hosted UI

The panel is a wgpu-html document. The inspector ships a static
HTML+CSS template per panel (Elements, Computed, Styles, Box-model,
Render); each frame the inspector:

1. Patches dynamic regions in that document (tree rows, computed-
   style table cells, edit fields) by rewriting `Element` content
   and inline `style` strings — the same path mutation uses (§6).
2. Runs cascade + layout on the devtools document.
3. Paints it into the host's `DisplayList` after the page's
   display list and after the highlight overlay, so devtools is
   always on top.

Two pipeline runs per frame (page + devtools) is fine: the devtools
document is small (hundreds of nodes max) and shares the renderer's
quad/text pipelines.

Implications:

- Devtools panels block on engine M5 (text rendering). Until then,
  D5+ phases are unavailable, and the inspector exposes the same
  data through the API in §6 so a host (or a test harness, or
  stdout) can render it.
- Every devtools rendering bug is a page rendering bug, and vice
  versa — strong forcing function.
- Anything devtools needs (scroll containers, focus rings,
  hit-testing in nested layout) becomes a feature the engine has
  to support. The devtools spec doubles as a use-case driver for
  the engine roadmap.

## 4. Capabilities

Everything devtools surfaces must be derivable from existing
data structures:

| Capability                       | Source                                                 | Already exists?     |
|----------------------------------|--------------------------------------------------------|---------------------|
| Element tree                     | `Tree`, `Node`, `Element`                              | yes                 |
| Element attributes               | per-element model structs (`Div`, `A`, …)              | yes                 |
| Inline style attribute           | `<element>.style: Option<String>`                      | yes                 |
| Computed style                   | `CascadedNode::style`                                  | yes                 |
| Matched rules + specificity      | `wgpu-html-style` cascade — must expose intermediate   | **needs API**       |
| Layout rectangles (m / b / p / c) | `LayoutBox::{margin_rect, border_rect, content_rect}`  | yes                 |
| Border / corner radii            | `LayoutBox::{border, border_radius, …}`                | yes                 |
| Background fill rect             | `LayoutBox::background_rect`                           | yes                 |
| Hit testing                      | `LayoutBox::hit_path / find_element_from_point`        | yes                 |
| Path → node                      | `Node::at_path_mut`                                    | yes                 |
| Path → ancestry                  | `Node::ancestry_at_path_mut`                           | yes                 |
| Mutate inline style              | `Element::Div(d).style = Some("...")` + re-cascade     | works today         |
| Mutate text                      | `Element::Text(s) = "..."`                             | works today         |
| Display-list stats               | `DisplayList::quads.len()`                             | yes                 |
| Frame timings                    | renderer must expose                                   | **needs API**       |
| Cascade re-run trigger           | host re-runs cascade + layout each frame already       | yes                 |

Two engine-side gaps to fix as part of D1 / D2:

1. `wgpu-html-style::cascade` should optionally return the matched-
   rule list per element (selector + specificity + which declarations
   it contributed). Cheap to record; needed for the Styles panel.
2. `wgpu-html-renderer::Renderer` should record `last_frame_ms`,
   `quad_count`, `submit_ms`. Already has the data, just needs to
   surface it.

## 5. UI surface

### 5.1 Highlight overlay (no text needed)

For a given `LayoutBox`, paint four concentric translucent quads:

```
┌────────────────────────────────────┐  margin  (yellow,  α≈0.25)
│ ┌────────────────────────────────┐ │
│ │ ┌────────────────────────────┐ │ │  border  (orange,  α≈0.30)
│ │ │ ┌────────────────────────┐ │ │ │  padding (green,   α≈0.20)
│ │ │ │                        │ │ │ │  content (blue,    α≈0.15)
│ │ │ └────────────────────────┘ │ │ │
│ │ └────────────────────────────┘ │ │
│ └────────────────────────────────┘ │
└────────────────────────────────────┘
```

All four are existing `push_quad` calls — solid alpha-blended fills,
in order from outside in. Implements identically to Chrome's element
overlay minus the dimension labels (those need text → M5+).

Hover overlay: same, slightly more transparent. Selection overlay:
brighter, pinned across hover changes.

### 5.2 Element picker

- Host opts into "pick" mode by calling `Inspector::start_pick()`.
- On every `CursorMoved`, the host calls
  `inspector.pick_at(pos)`. Internally:
  `layout.find_element_from_point(&mut tree, pos)` →
  `Some(Vec<usize> path)`. Inspector stores the path as `hover`.
- On `MouseInput::Left::Pressed`, copy `hover` into `selection`,
  exit pick mode.
- Esc: cancel pick, clear hover.

Selection path is stored as `Vec<usize>` (not `&mut Node`) so it
survives across frames where the tree gets re-cascaded /
re-laid-out.

### 5.3 Panels (gated on text rendering)

Once M5 (text) lands, the side-panel — itself a wgpu-html document
authored by the inspector and painted by the same engine — populates:

- **Elements**: tree view of `Tree`, rendered as a nested `<ul>`
  with collapse arrows. Selection highlights the corresponding
  overlay. Right-click → "Edit inline style", "Remove element",
  "Toggle visibility".
- **Computed**: flat `<table>` of `CascadedNode::style` non-`None`
  fields for the selected element.
- **Styles**: rule-by-rule view (matched selectors with
  specificity), with a final "inline style" block that's editable.
- **Layout / box-model**: numeric box around the standard four-
  rect diagram (5.1), built as nested `<div>`s so the same engine
  draws it. Click any cell to edit the corresponding longhand in
  the inline style.
- **Render**: display-list quad count, last frame ms, paint ms,
  scroll-of-pipeline (which stage took how long).

Each panel is a small HTML+CSS template stored as a `&'static str`
inside `wgpu-html-devtools`. The inspector mutates the templates'
content nodes per frame using the same `set_text` / `set_inline_style`
APIs the host uses on the page document. No new rendering backend is
introduced — devtools is the engine's first non-trivial in-tree
test fixture.

Until M5: all panel features are still callable as **API methods**
that return text / structured data; the host can render them with
its own UI library or to stdout. The dedicated panel is the
default once text exists.

### 5.4 Layout for the devtools document

The host chooses where the panel lives:

- **Docked**: a `splitter` mode where the host divides the surface
  into `[page | panel]` (or top/bottom). Each side gets its own
  `LayoutBox` rooted at its sub-rect, painted into the same
  `DisplayList`.
- **Overlay**: panel painted over the top-right of the page in a
  semi-transparent box. Fast to add (no resize plumbing), poor for
  real use; acceptable for the bring-up phase.
- **Separate window**: optional, host-driven. The inspector doesn't
  care — it just produces a `DisplayList` for whatever surface the
  host hands it.

The devtools document is laid out at the size of its assigned
sub-rect; mutation happens on the host thread between
`page.layout()` and `devtools.layout()`.

## 6. Public API (sketch)

```rust
// crates/wgpu-html-devtools/src/lib.rs

use wgpu_html_layout::LayoutBox;
use wgpu_html_tree::Tree;

pub struct Inspector {
    selection: Option<Vec<usize>>, // path into Tree
    hover:     Option<Vec<usize>>,
    mode:      Mode,
}

pub enum Mode { Idle, Pick }

impl Inspector {
    pub fn new() -> Self;

    // Selection / picking ----------------------------------------------------
    pub fn selection(&self) -> Option<&[usize]>;
    pub fn start_pick(&mut self);
    pub fn cancel_pick(&mut self);
    pub fn pick_at(&mut self, layout: &LayoutBox, tree: &mut Tree, pt: (f32, f32));
    pub fn click(&mut self); // promote hover → selection if in pick mode

    // Read-only inspection ---------------------------------------------------
    pub fn snapshot<'a>(&self, tree: &'a Tree, layout: &'a LayoutBox)
        -> Option<NodeSnapshot<'a>>;

    pub fn ancestry_snapshot<'a>(&self, tree: &'a Tree, layout: &'a LayoutBox)
        -> Vec<NodeSnapshot<'a>>;

    // Mutation ---------------------------------------------------------------
    pub fn set_inline_style(&self, tree: &mut Tree, css: &str) -> bool;
    pub fn append_inline_style(&self, tree: &mut Tree, css: &str) -> bool;
    pub fn set_text(&self, tree: &mut Tree, text: &str) -> bool;
    pub fn toggle_visibility(&self, tree: &mut Tree) -> bool;

    // Overlay ----------------------------------------------------------------
    pub fn paint_overlay(&self, layout: &LayoutBox, list: &mut DisplayList);
}

pub struct NodeSnapshot<'a> {
    pub path:           &'a [usize],
    pub tag:            &'static str,           // "div", "p", …
    pub id:             Option<&'a str>,
    pub class:          Option<&'a str>,
    pub inline_style:   Option<&'a str>,
    pub computed:       &'a wgpu_html_models::Style,
    pub layout:         &'a LayoutBox,          // pinned at the same path
    pub matched_rules:  Vec<MatchedRule<'a>>,   // post-D2 (cascade exposes)
    pub data_attrs:     &'a HashMap<String, String>,
    pub aria_attrs:     &'a HashMap<String, String>,
}

pub struct MatchedRule<'a> {
    pub selector:        &'a str,        // "#hero .card"
    pub specificity:     u32,
    pub declarations:    &'a [(&'a str, &'a str)],
    pub overridden_by:   Vec<usize>,     // indices of later rules
}

// frame_stats() returns engine-side counters; lives on the Renderer.
```

All mutation methods return `bool` (was-applied), not `Result`,
because the only failure mode is "selection does not resolve to a
node anymore" — which we treat as a no-op. `set_inline_style` is
the primary editor; the separate `set_text` exists because text
nodes don't have a `style` attribute.

## 7. Input handling

Devtools introduces the first real input handling in the demo. The
event arms the `wgpu-html-demo` adds (and that future hosts mirror):

| winit event                        | Devtools effect                           |
|-----------------------------------|-------------------------------------------|
| `CursorMoved(pos)`                | `inspector.pick_at(pos)` if `Mode::Pick`  |
| `MouseInput::Left::Pressed`       | `inspector.click()` if `Mode::Pick`       |
| `KeyboardInput(F12)`              | toggle inspector overlay visibility       |
| `KeyboardInput(Ctrl+Shift+C)`     | enter pick mode                           |
| `KeyboardInput(Esc)`              | cancel pick / clear selection             |

These are all suggestions; the spec only mandates the API. The demo
binds them.

Note: the demo's existing F12 already means "screenshot". A devtools
build should rebind: F12 → toggle overlay, F11 → screenshot (or
similar).

## 8. Integration with the pipeline

Per-frame flow once devtools is wired in:

```
1. parse / cascade / layout / paint   (unchanged)
2. inspector.paint_overlay(&layout, &mut display_list)
   - reads selection / hover paths
   - looks up the corresponding LayoutBox by walking
     `LayoutBox::children[path[0]]…[path[n]]`
   - emits 4 alpha-blended quads (m / b / p / c)
3. renderer.render(&display_list)     (unchanged)
```

`paint_overlay` is purely additive: the unmodified page is
painted first, then highlights on top. No clip stacks needed.

For mutation: the host calls `inspector.set_inline_style(&mut tree,
"...")` from an event handler, then on the next frame the standard
parse(?)/cascade/layout pipeline picks up the change. (The demo
re-cascades + re-lays-out every frame anyway; once we add dirty
tracking it'll still work because mutation flips the dirty bit.)

## 9. Performance / threading

- Inspector state is small (one `Vec<usize>` selection, one for
  hover, plus an enum). Cheap to keep on the host stack.
- Overlay is at most 8 extra quads (4 for hover, 4 for selection)
  per frame — negligible vs. the page's display list.
- No background thread. All inspector work happens on the same
  thread that owns the `Renderer`.
- Recording matched rules during cascade is O(rules × elements);
  enable it only when the inspector is open (a `cascade_with_trace`
  variant, gated by a flag the host flips with `Inspector`).

## 10. Phases

Each phase ends in something the demo can show.

### D1 — Overlay primitive ✅-shape

- New crate `wgpu-html-devtools` with `Inspector::new()`, a
  `selection: Option<Vec<usize>>`, and `paint_overlay`.
- Manual selection: host sets path with `inspector.set_selection(path)`.
- Overlay paints four concentric quads (margin / border / padding /
  content) in distinct colours.
- Demo binding: `Ctrl+1` cycles through the body's children, painting
  the overlay over each.
- No new engine APIs, no input deps beyond what the demo already
  uses.

### D2 — Snapshot API + matched-rule trace

- `cascade_with_trace(tree)` in `wgpu-html-style` — same output as
  `cascade` plus a `Vec<MatchedRule>` per node.
- `Inspector::snapshot` returns `NodeSnapshot` (`tag`, `id`,
  `class`, inline + computed style, layout box, matched rules,
  ancestry).
- `Renderer` exposes `last_frame_ms()`, `quad_count()`.
- A test prints a snapshot to stdout for the inspector-selected
  element. No UI yet.

### D3 — Pointer picker

- Demo handles `CursorMoved` and `MouseInput`.
- `Mode::Pick` toggled by Ctrl+Shift+C; Esc cancels.
- `pick_at` updates `hover`; click promotes to `selection`.
- Overlay shows hover (translucent) + selection (opaque-ish) at
  the same time.

### D4 — Mutation API

- `Inspector::set_inline_style / set_text / toggle_visibility /
  append_inline_style`. Each rewrites the corresponding field on
  the `Element` referenced by `selection`.
- Demo binding: `Ctrl+E` in pick mode prompts on stdout for an
  inline-style override (until M5 brings on-screen input). Effect
  visible next frame.

### D5 — Self-hosted side panel (depends on engine M5 — text)

The panel is a wgpu-html document. The inspector ships a static
HTML+CSS template per panel and patches it each frame.

- Splitter layout: host carves the surface into `[page | panel]`
  rects; each gets its own cascade + layout pass into the shared
  display list.
- Panels delivered in this phase:
  - Tree panel (Elements) — nested `<ul>` driven by the live
    `Tree`.
  - Computed-style table.
  - Box-model diagram with numeric insets, built as nested
    `<div>`s.
- Hard requirements on the engine the panel forces:
  - Inline `style` attribute on every node (already there).
  - Text rendering with adequate metrics (M5).
  - Vertical overflow scrolling on a containing block (sub-spec
    of M6 layout work, listed as an engine prerequisite).

### D6 — Styles panel + live edit

- Render the matched-rule trace from D2.
- Per-rule "edit" button rewrites the rule's source in the
  `<style>` block (whole-document re-parse strategy is fine
  initially).
- Inline-style field is text-editable.

### D7 — Render / perf panel

- Display list quad count.
- Per-stage frame timings (`parse`, `cascade`, `layout`,
  `paint`, `submit`).
- Optional rolling histogram of last N frames.

### D8 — Pixel-coverage / overdraw view

- Single-toggle replacement render: each fragment paints `+α` so
  hot zones go bright. Useful once we ever need to optimise the
  display list.

### D9 — Headless / CI snapshot mode

- `Inspector::snapshot` already returns a serializable view; add
  `serde` derives behind a feature flag.
- `cargo test --features wgpu-html-devtools/snapshot` can produce
  golden JSON of the cascaded + laid-out tree, diffed against
  known-good output. Useful for catching unintended layout drift.

## 11. Open questions

- **Edit-and-persist**: D6 rewrites the live `<style>` block. Do we
  also write back to the source HTML / CSS string the host loaded
  from? Punt to the host (it gets a callback with the new source).
- **Selectors that target shadow DOM**: shadow DOM doesn't exist
  yet, irrelevant.
- **Pseudo-classes (`:hover`) in the live style**: blocked on
  cascade gaining element-state input. Until then, the inspector
  picker can simulate `:hover` only for itself (the highlight),
  not for CSS rules that target it.
- **Z-index / paint order**: the inspector's hover/selection
  overlay always paints last. If we ever add transforms or layered
  paint, the overlay needs its own root layer.
- **Multi-select**: not in scope. Selection is a single path.

---

## Summary

D1–D4 are buildable today on the existing engine — they only need
the new crate, plus tiny `cascade_with_trace` and `Renderer` stat
surfaces. D5+ depends on text rendering (engine M5). The picker
and overlay are the practical day-one win: they turn the
`find_element_from_point` API into something a developer can
actually see and click.
