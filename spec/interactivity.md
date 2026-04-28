# wgpu-html — Interactivity Spec (Mouse-First)

The plan for moving from "static layout, F12 + Esc only" to a tree
that responds to pointer input: hover / press / click / wheel, plus
the cascade hooks (`:hover`, `:active`, `:focus`, …) and the
plumbing (`pointer-events`, `cursor`, `user-select`) that have to
exist for any of it to behave like a browser.

**Status (2026-04-29):**

- **M-INTER-1 ✅ shipped.** `InteractionState` on `Tree`, all four
  pointer functions wired (`pointer_move`, `mouse_down`, `mouse_up`,
  `pointer_leave`), hover/active tracking, enter/leave callbacks,
  click synthesis, `:hover`/`:active` cascade integration via
  `MatchContext::for_path`, typed DOM events through `wgpu-html-events`
  (`HtmlEvent`, `MouseEvent`, `EventPhase`), `buttons_down` bitmask.
- **M-INTER-3 ⚠️ partial.** `TextCursor`/`TextSelection` on
  `InteractionState`, drag-to-select, `select_all_text` /
  `selected_text`, `Ctrl+A`/`Ctrl+C` + `arboard` in the demo,
  selection highlight quads in `paint.rs`. Caret overlay, word/line
  select, and `user-select` property enforcement are not yet done.
- **M-INTER-4 ⚠️ partial.** `scroll_offsets_y: BTreeMap` on
  `InteractionState`, viewport scroll + per-element scroll-container
  offset, scrollbar paint, drag-to-scroll, `MouseWheel`. `Wheel`
  events are not forwarded to element `on_event` callbacks.
- **M-INTER-2, M-INTER-5, M-INTER-6 ❌ not yet done.**
  `pointer-events: none`, `overflow`-clip in hit-test, double-click,
  `is_focus` in `MatchContext`, keyboard navigation, re-cascade
  caching.

Companion to `roadmap.md` (§M12 interactivity milestones), `status.md`
§7 (interactivity), and `text.md` (text selection lands here, not in
the text spec).

---

## 1. Goals

- A small, blocking-API event surface a host (winit, egui, browser
  embed) can pump per-frame.
- Mouse: move, enter, leave, down, up, click, double-click, context
  menu, wheel.
- Element-state-aware cascade: `:hover`, `:active`, `:focus`,
  `:focus-visible`, `:focus-within`, `:disabled`, `:checked` (only
  for elements where the underlying model has a notion of "checked",
  e.g. `<input type=checkbox>`).
- `cursor`, `pointer-events`, `user-select` honoured.
- Caret / range text selection on inline text, including drag-to-
  select across multiple text leaves and copy-to-clipboard via the
  host.
- Hover / press chain stays correct under scrolling, layout
  changes, and re-cascade.
- Host opt-in: a tree that never receives events behaves exactly
  like today's tree.

## 2. Non-goals (first pass)

- No drag-and-drop API (`dragstart / dragover / drop`). Reserved.
- No touch / pen / multi-pointer. The model is single-pointer mouse
  with explicit primary / secondary / middle button distinction.
- No IME / composition events; out of scope until `<input>` gets a
  real text path.
- No `pointercapture` semantics beyond the implicit "press target
  keeps receiving moves until release" rule (§7).
- No CSS `transform` interaction — hit-testing still walks
  axis-aligned `border_rect`s. Revisit if/when `transform` lands.
- No accessibility / screen-reader tree.
- No animations / transitions on state change. `:hover` snaps.

## 3. Coordinate system

All input arrives in **physical pixels, top-left origin, +Y down**,
matching the layout's coordinate system. The host is responsible
for converting OS-native coordinates (winit's logical pixels, the
embedder's local-space) into physical pixels via the same
`scale_factor` used by `layout_with_text`.

A pointer position is an `(f32, f32)`. Sub-pixel precision is
preserved end-to-end; hit-tests do not snap.

## 4. Where state lives

> **Element interaction state lives on the `Tree`, not in the
> renderer or a process global.** Mirrors the rule that fonts live
> on the tree: state is per-document, dropped with the document,
> trivial to reset.

**Current shape** (`wgpu-html-tree/src/events.rs`):

```rust
#[derive(Debug, Clone)]
pub struct InteractionState {
    /// Path to the deepest element currently under the pointer.
    pub hover_path:       Option<Vec<usize>>,
    /// Path to the element that received the most recent primary press.
    pub active_path:      Option<Vec<usize>>,
    /// Last known pointer position in physical pixels.
    pub pointer_pos:      Option<(f32, f32)>,
    /// Current text selection, if any.
    pub selection:        Option<TextSelection>,
    /// Whether a primary-button drag currently owns text selection.
    pub selecting_text:   bool,
    /// Colors used to paint selected text/background.
    pub selection_colors: SelectionColors,
    /// Vertical scroll offsets keyed by child-index path.
    pub scroll_offsets_y: BTreeMap<Vec<usize>, f32>,
    /// Instant at creation — used for `Event::time_stamp`.
    pub time_origin:      Instant,
    /// DOM-style bitmask of currently-held mouse buttons (W3C spec).
    pub buttons_down:     u16,
}
```

Fields described in the spec but **not yet on `InteractionState`**:
`focus_path`, `focus_visible` (`:focus` is always `false` in
`MatchContext`). These land with M-INTER-2.

Rationale:

- **One source of truth.** The cascade reads it (`MatchContext::for_path`);
  the painter reads it (for selection rectangles); the host reads it
  (for cursor shape).
- **No global mutable state.** Two trees in the same process do not
  fight over a single hover.
- **Cheap to reset.** `tree.interaction = Default::default()` on
  reload.
- **Survives re-layout.** Paths are stored, not pointers; they
  remain valid as long as the element tree's child structure is
  stable.

## 5. Hit testing — contract & extensions

The current hit test (`crates/wgpu-html-layout/src/lib.rs:225`)
returns the deepest descendant whose `border_rect` contains the
point, walking children last-to-first so the topmost paint wins.
That contract stays. Additions:

- **`pointer-events: none`.** A box with `pointer_events == None`
  is invisible to hit-testing — both itself and (per CSS) its
  descendants if they don't override back to `auto`. Implementation:
  `collect_hit_path` skips boxes whose cascaded style sets
  `pointer-events: none`, walks into descendants only when the
  parent is `auto` or the descendant explicitly re-enables.
- **`overflow: hidden | scroll | auto` clipping.** A point outside
  the parent's content (post-padding) rect inside an `overflow !=
  visible` box must miss every descendant of that box, even if a
  descendant's `border_rect` extends past it. Without this, a
  scrolled-away child still receives clicks. Requires layout to
  carry an `effective_clip_rect` per box, computed by intersecting
  with each ancestor's clip.
- **Scroll offset.** Once scrolling exists (§12), descendants of a
  scroll container are hit-tested against `point − scroll_offset`,
  applied at the boundary. Until §12 lands the offset is always
  zero.
- **Inline / line-box hit testing.** A click inside a paragraph
  resolves to the inline-element subtree, then the specific text
  leaf (line box → glyph cluster → byte index in the source
  string). Required for caret placement (§11). Extension to
  `LayoutBox::hit_path`: the path is element indices; an additional
  `text_hit: Option<TextHit>` (line index, glyph index, byte index,
  is-after-last-glyph) is returned for text leaves.

```rust
pub struct HitTest {
    pub path: Vec<usize>,                 // element indices
    pub text: Option<TextHit>,            // present iff a text leaf was hit
}

pub struct TextHit {
    pub line:           usize,
    pub glyph_index:    usize,
    pub byte_offset:    usize,
    pub trailing:       bool,             // pointer was past glyph centre
}
```

`hit_path` keeps its current signature for callers that don't care
about text; a sibling `hit_test_full((x, y)) -> Option<HitTest>`
returns the richer payload.

## 6. Event types

The host hands events to the engine via a thin API. Engine-internal
types, not winit / web types:

```rust
pub enum InputEvent {
    PointerMove   { pos: (f32, f32) },
    PointerLeave,                                        // OS-level "left window"
    MouseDown     { pos: (f32, f32), button: MouseButton, modifiers: Modifiers },
    MouseUp       { pos: (f32, f32), button: MouseButton, modifiers: Modifiers },
    Wheel         { pos: (f32, f32), delta: ScrollDelta, modifiers: Modifiers },
    // Keyboard lives in §13; included here so the API has one front door.
    KeyDown       { key: Key, modifiers: Modifiers, repeat: bool },
    KeyUp         { key: Key, modifiers: Modifiers },
    Text          { text: String },                      // pre-IME-resolved utf-8
    Focus,                                               // window gained focus
    Blur,                                                // window lost focus
}

pub enum MouseButton { Primary, Secondary, Middle, Other(u8) }

pub enum ScrollDelta {
    Lines  { x: f32, y: f32 },                           // typical mouse wheel
    Pixels { x: f32, y: f32 },                           // trackpad, hi-dpi wheel
}

pub struct Modifiers { pub shift: bool, pub ctrl: bool, pub alt: bool, pub meta: bool }
```

Synthesised, derived events (the engine emits these as state
transitions — the host doesn't send them):

- **PointerEnter / PointerLeave per-element.** Whenever
  `hover_path` changes, every element on the symmetric difference
  of old vs new chain gets a leave (old-only) or enter (new-only).
  Leave runs deepest-first, enter runs root-first, mirroring the
  DOM semantics that an outer enter precedes any inner enter.
- **Click.** A `MouseUp` whose `Primary` press landed on an element
  that is still on the current `hover_path` (i.e., the press
  target is an ancestor of, equal to, or descendant of the release
  target — practically: same path-prefix). The dispatched click
  target is the deepest common ancestor; matches browser behaviour.
- **DoubleClick.** Two clicks within `DOUBLE_CLICK_INTERVAL`
  (default 500 ms) and `DOUBLE_CLICK_RADIUS` (default 5 px), on
  the same target, with the same button.
- **ContextMenu.** Synthesised on `MouseDown { button: Secondary }`
  release at the same target. Hosts that want OS-native context
  menus suppress this and read `tree.interaction` directly.
- **AuxClick.** Middle-button click; same rule as click.

Click thresholds (intervals, radii) are constants today; tunable
later if a host needs different defaults.

## 7. Press semantics & implicit pointer capture

When `MouseDown { Primary }` fires:

1. Resolve the hit path; ignore if it's `None` or
   `pointer-events: none` is in effect.
2. Set `interaction.active_path = Some(path)`.
3. Set `interaction.focus_path = Some(focusable_ancestor(path))`
   if any (§13). Set `focus_visible = false`.
4. Cascade re-runs (§8); `:active` and `:focus` now match.

Until the matching `MouseUp { Primary }`:

- `PointerMove` events update `hover_path` (so descendants see
  proper enter / leave) but `active_path` is preserved verbatim,
  even if the pointer drags outside the press target's box. This
  is implicit pointer capture; the press target keeps receiving
  drag updates.
- A `MouseUp { Primary }` clears `active_path` and synthesises a
  click iff the release path shares the press path's deepest
  common ancestor (§6). Outside that, no click — same as a
  browser's "drag-out cancel".

Other buttons do *not* gate `:active`. Only primary press sets it.

## 8. Cascade integration

**Current state (shipped):** `wgpu-html-style::cascade(&Tree) ->
CascadedTree` already reads `tree.interaction` internally.
`MatchContext::for_path` computes the context for each element:

```rust
pub struct MatchContext {
    pub is_hover:  bool,
    pub is_active: bool,
    pub is_focus:  bool,   // always false until M-INTER-2
}

impl MatchContext {
    pub fn for_path(path: &[usize], state: &InteractionState) -> Self {
        Self {
            is_hover:  path_is_prefix(path, state.hover_path.as_deref()),
            is_active: path_is_prefix(path, state.active_path.as_deref()),
            is_focus:  false,
        }
    }
}
```

Selector pseudo-classes (`:hover`, `:active`) are parsed and matched
against `MatchContext` in the selector-matching pass. `:focus` and all
other pseudo-classes return `false` today.

The proposed `ElementContext` / `cascade_with_state(&Tree,
&InteractionState)` separation from the spec is **not how it works**:
the cascade reads `tree.interaction` directly (the tree carries the
interaction state, so the cascade always has it available). A
`cascade_with_state` variant could still be added for testing.

**Cost.** The demo re-cascades on every hover-triggered redraw. A
hover move that changes the hover path triggers a deferred redraw
(throttled to 16 ms); no change → redraw skipped. Re-cascade caching
(M-INTER-6) is still future work.

## 9. Cursor resolution

Per frame, after cascade:

```text
cursor = first ancestor of hover_path whose cascaded `cursor` is set
       fall back to Cursor::Default
```

Exposed as `tree.interaction.resolved_cursor()` (computed
on demand from `hover_path` + the cascaded tree). The host maps
this to the OS's cursor by setting winit's `Window::set_cursor`
once per change.

`Cursor::Auto` resolves to a contextual default: `Cursor::Text`
inside a text leaf with `user-select != none`, otherwise
`Cursor::Default`. Matches browser behaviour for "auto means
caret on text".

## 10. `pointer-events` and `user-select`

Already modelled (`crates/wgpu-html-models/src/common/css_enums.rs:209`,
`:215`); plumbing to add:

- `pointer-events: none` — described in §5. Inherits.
- `user-select: none` — the text leaves under it are excluded
  from drag-selection (§11) and clicks on them never start a
  selection. Inherits.
- `user-select: text` (the default) — selectable.
- `user-select: all` — a single click anywhere inside the subtree
  selects the whole subtree. Useful for code blocks.
- `user-select: auto` — same as `text` for text leaves, `none`
  for non-text. CSS quirk preserved.

Both properties join the cascade-inheriting set in `text.md` §9
(`color, font_*, …, cursor`). `cursor` is already in the list;
add `pointer_events` and `user_select` once they are honoured.

## 11. Text selection

Selection is one contiguous range of (path, byte-offset) pairs
across the document order:

```rust
pub struct Selection {
    pub anchor: Caret,                 // where the press started
    pub focus:  Caret,                 // where the pointer is now / shift-extended to
}

pub struct Caret {
    pub path:        Vec<usize>,       // path to the text leaf
    pub byte_offset: usize,            // into the leaf's source string (post-text-transform)
    pub affinity:    Affinity,         // upstream / downstream — matters at line wraps
}

pub enum Affinity { Upstream, Downstream }
```

Selection lifecycle:

- **MouseDown { Primary }** on a text leaf with `user-select !=
  none`: `selection = Some({ anchor: caret_at(pos), focus:
  same })`. Visually empty — just a caret.
- **PointerMove** while `active_path` is on a text leaf: update
  `selection.focus = caret_at(pos)`. The selection range is the
  document-order interval `[min(anchor, focus), max(...))`.
- **MouseUp**: selection persists until the next bare-area press
  or programmatic clear (`tree.clear_selection()`).
- **Double-click** on a word: anchor / focus snap to the word
  boundaries (Unicode word break, via cosmic-text's segmenter).
- **Triple-click**: snap to the line box.
- **Shift+MouseDown**: extend instead of replace (move `focus`,
  keep `anchor`).

Painting the selection (separate from this spec's behaviour but
worth noting): emit one quad per line-segment of the selection
range, positioned in the inline pass; emit one thin caret quad
when `selection.is_caret()` and `focus_path == path`. Caret blink
is host-driven (a flag on `interaction`, toggled on a timer).

`user-select: all` short-circuits the press: anchor = first byte
of the subtree, focus = last byte, on a single click.

Copy: the host reads `tree.interaction.selected_text(&tree)` →
`Option<String>`, walks the document tree between anchor and
focus, concatenates source strings (post-`text-transform`),
inserts `\n` at block boundaries, and writes to the OS clipboard.
The engine never touches the clipboard itself.

## 12. Scrolling

Out of scope for the first phase; sketched here so the API doesn't
paint itself into a corner.

When `overflow: scroll | auto` lands:

- Each scroll container carries a `scroll_offset: (f32, f32)` in
  `LayoutBox` (or in a side table keyed by path — TBD).
- `Wheel` events resolve to the deepest hover-chain ancestor with a
  scrollable overflow on the matching axis; that ancestor's offset
  changes; layout itself does not need to re-run.
- Hit-testing inside the container subtracts the offset before
  recursing (§5).
- Painting clips to the container's content box and translates
  descendants by `−offset`.

`Wheel`'s `ScrollDelta::Lines` is converted to pixels by
multiplying by a constant (default 16 px / line, configurable).
`ScrollDelta::Pixels` is used as-is.

Smooth scrolling, scroll-snap, momentum: all post-§12.

## 13. Keyboard / focus (sketch)

Mouse-first, but focus is shared state:

- Tabbable elements: `<button>`, `<a href>`, `<input>`,
  `<textarea>`, `<select>`, anything with `tabindex` ≥ 0.
- `Tab` / `Shift+Tab` cycles forward / backward through the
  document-order list of tabbable elements; sets `focus_path` and
  `focus_visible = true`.
- `Enter` / `Space` on a focused button or link is treated as a
  primary click on its `border_rect` centre.
- `Esc` clears focus and selection.
- Arrow keys / Home / End / PageUp / PageDown move the caret when
  a text leaf has focus and `contenteditable` is `true`. (`<input>` and
  `<textarea>` need `contenteditable`-equivalent plumbing — out of
  scope until the input model gains it.)

## 14. Frame loop integration

Host-side loop with the new API:

```rust
// Host setup, once.
let mut tree    = wgpu_html_parser::parse(html);
tree.register_font(...);
let mut text_ctx = TextContext::new(GLYPH_ATLAS_SIZE);

// Per OS event:
let dirty = wgpu_html::interactivity::dispatch(&mut tree, ev, &last_layout);
if dirty.needs_redraw { window.request_redraw(); }
if let Some(c) = dirty.cursor { window.set_cursor(c); }

// Per frame (in RedrawRequested):
let cascaded = wgpu_html_style::cascade_with_state(&tree, &tree.interaction);
let layout   = wgpu_html_layout::layout_with_text(&cascaded, &mut text_ctx, vw, vh, scale);
let display  = wgpu_html::paint::paint(&layout, &tree.interaction); // selection / caret quads
last_layout  = layout;
text_ctx.atlas.upload(&queue, renderer.glyph_atlas_texture());
renderer.render(&display);
```

`dispatch` is the front door:

```rust
pub fn dispatch(
    tree:   &mut Tree,
    ev:     InputEvent,
    layout: &LayoutBox,
) -> InteractionUpdate;

pub struct InteractionUpdate {
    pub needs_redraw: bool,
    pub cursor:       Option<Cursor>,    // Some when it changed since last call
    pub clipboard:    Option<ClipboardOp>, // Some when a copy/paste was triggered
}

pub enum ClipboardOp { Copy(String), Paste }
```

Hosts that want fine-grained control instead of `dispatch` can call
the internal helpers (`update_hover`, `begin_press`, `end_press`,
`apply_wheel`, `update_caret`) directly. The high-level `dispatch`
is just a switch over `InputEvent`.

`needs_redraw` is `true` whenever the interaction state changed in
a way that affects cascade (so any of `hover/active/focus/
focus_visible/selection`) or scroll offsets. Pure pointer-position
updates with no path change return `false` — the demo's continuous-
redraw loop renders 60 fps anyway, but headless / on-demand
embedders benefit.

## 15. Public API surface

```
wgpu-html-tree
  + InteractionState (hover/active/selection/scroll/buttons/time_origin) ✅ M-INTER-1
  + Tree::interaction (field), Tree::clear_selection                      ✅ M-INTER-1
  + TextCursor, TextSelection, SelectionColors                            ✅ M-INTER-3
  + focus_path, focus_visible                                             ❌ M-INTER-2

wgpu-html-models
  + (existing) Cursor, PointerEvents, UserSelect                         ✅ done

wgpu-html-events
  + HtmlEvent, MouseEvent, UIEvent, Event, EventPhase, HtmlEventType     ✅ M-INTER-1

wgpu-html-parser / wgpu-html-style
  + Pseudo-class selector parsing (`:hover`, `:active`)                  ✅ M-INTER-1
  + MatchContext { is_hover, is_active, is_focus }                       ✅ M-INTER-1
  + MatchContext::for_path(path, &InteractionState)                      ✅ M-INTER-1
  + pointer-events / user-select cascade-inheritance entries             ❌ M-INTER-2

wgpu-html-layout
  + hit_text_cursor((x,y)) → Option<TextCursor>                         ✅ M-INTER-3 (partial)
  + pointer-events skip in hit test                                       ❌ M-INTER-2
  + overflow-hidden clip in hit test                                      ❌ M-INTER-2
  + scroll_offset on LayoutBox                                            ❌ M-INTER-4 (offset lives on Tree today)

wgpu-html (facade)
  + interactivity::pointer_move, mouse_down, mouse_up, pointer_leave     ✅ M-INTER-1
  + select_all_text, selected_text                                        ✅ M-INTER-3
  + paint emits selection quads from interaction.selection               ✅ M-INTER-3
  + PipelineTimings                                                       ✅ (profiling)
  + on_event (Node::on_event: EventCallback)                             ✅ M-INTER-1

wgpu-html-demo
  + winit CursorMoved / MouseInput / MouseScrollDelta → interactivity   ✅ M-INTER-1
  + scrollbar paint + drag-to-scroll                                     ✅ M-INTER-4 (partial)
  + Ctrl+A / Ctrl+C / arboard clipboard                                  ✅ M-INTER-3
  + winit KeyboardInput → InputEvent                                     ❌ M-INTER-5
```

## 16. Phases

Each phase ends in a runnable demo. Phase numbering parallels the
text spec's T1..T7.

### M-INTER-1 — Hover, press, click, focus chain ✅ Done

- `InteractionState` on `Tree`; default-initialised.
- `pointer_move`, `mouse_down`, `mouse_up`, `pointer_leave` in
  `wgpu-html::interactivity`.
- Hover-path tracking with synthesised enter / leave.
- Implicit pointer capture during press (§7).
- Click synthesis via deepest common ancestor.
- Pseudo-class parsing: `:hover`, `:active`. Cascade reads `MatchContext`.
- `wgpu-html-events` crate: `HtmlEvent`, `MouseEvent`, typed DOM
  event dispatch via `Node::on_event`.
- Demo: cards change style on hover/active; `on_click` callbacks fire.

### M-INTER-2 — `pointer-events`, `overflow` clip, double-click ❌

- Hit test honours `pointer-events: none` (self + descendants).
- Hit test honours ancestor `overflow != visible` clip (no scroll
  yet — clip only).
- Double-click synthesis.
- ContextMenu / AuxClick synthesis.
- `:focus-visible` (set on Tab focus, cleared on press) and `:focus-within`.
- `:disabled` for `<button disabled>`, `<input disabled>`, etc.
- `user-select: none` recognised but not yet acted on (§11 lands
  in M-INTER-3).

### M-INTER-3 — Text selection + caret + clipboard ⚠️ Partial

**Done:**
- `TextCursor` / `TextSelection` on `InteractionState`.
- `hit_text_cursor` in layout for cursor placement.
- Press on text leaf starts selection; drag extends; release commits.
- `select_all_text` / `selected_text` in `wgpu-html`.
- Paint emits selection highlight rectangles.
- Clipboard: `Ctrl+A` + `Ctrl+C` wired in demo via `arboard`.
- Drag-select suppresses click synthesis (tested).

**Not yet done:**
- Word select (double-click), line select (triple-click).
- `user-select: none / text / all` property enforcement.
- Caret overlay quad (thin blinking cursor).
- Shift+click to extend selection.

### M-INTER-4 — Wheel + scroll containers ⚠️ Partial

**Done:**
- `scroll_offsets_y: BTreeMap<Vec<usize>, f32>` on `InteractionState`.
- Viewport scroll position + scrollbar paint + drag-to-scroll.
- `MouseWheel` scrolls viewport; deepest scroll-container found and
  scrolled when applicable.
- Per-element scroll container scrollbar quads in `paint.rs`.

**Not yet done:**
- `Wheel` event forwarded to element `on_event` callbacks.
- `LayoutBox::scroll_offset` (offset currently lives only in
  `InteractionState`, not embedded in the box tree).
- Hit-testing subtracting ancestor scroll offsets.

### M-INTER-5 — Keyboard navigation ❌

- `KeyDown / KeyUp / Text` in `InputEvent`.
- Tab / Shift+Tab focus traversal.
- Enter / Space on focused button / link → synthesised click.
- Esc → clear focus + selection.
- Arrow keys / Home / End move the caret in a focused
  contenteditable text leaf.

### M-INTER-6 — Re-cascade caching, hover-path stability ❌

- Skip cascade entirely when no relevant state changed.
- Subtree-scoped re-cascade for hover-only changes.
- Hover-path "stickiness" across layout changes.
- No new user-visible features; perf only.

## 17. Open questions

- **`:hover` on touch.** We aren't supporting touch in this pass,
  but should `:hover` "stick until next tap elsewhere" semantics
  be considered when we eventually do?
- **`:link` / `:visited`.** Browsers gate these on history; we
  have no history. Probably resolve `:link` to "every `<a href>`"
  and ignore `:visited` (CSS spec already lets `:visited` be
  partially restricted; "never matches" is conformant).
- **Element identity vs. paths.** Paths are fragile under tree
  edits. Long-term we may want stable element IDs assigned at
  build time (a `u32` per `Node`); state would then key on those
  instead of `Vec<usize>`. First pass uses paths; the migration
  is local to `InteractionState`.
- **Re-entrant hosts.** A host that runs `dispatch` recursively
  (e.g. from an MCP-style remote driver) would deadlock against
  `&mut Tree`. Not a real concern today; flag for whoever embeds
  the engine in egui first.
- **Modifier-only events.** `Ctrl` press alone never fires
  anything in this model; modifier state lives on the next mouse
  event. Browsers also fire `keydown` events for bare modifiers,
  but no `:hover`-style cascade reads them. Skipped.
- **Pointer capture API.** Web has explicit `setPointerCapture`;
  we have implicit-during-press only. If a host wants to keep
  receiving moves after release (e.g. a knob widget), it has to
  poll `pointer_pos` itself.
- **Animation on state change.** `transition: background-color
  0.2s` is the canonical case. Without a frame-clock and an
  animation system, every pseudo-class snaps. Reserved for
  post-M10.
- **Re-layout on hover.** A `:hover` rule that changes `width` or
  `display` forces re-layout, not just re-cascade. The first
  phase always re-runs `layout_with_text` when cascade changes;
  optimisation (only re-layout the affected subtree) is a later
  pass.

---

## Summary

State lives on the `Tree` (`InteractionState`), same constraint as
fonts. The engine exposes four pointer functions (`pointer_move`,
`mouse_down`, `mouse_up`, `pointer_leave`) in `wgpu-html::interactivity`.
Typed DOM events flow through `wgpu-html-events` (`HtmlEvent`,
`MouseEvent`, `EventPhase`). Cascade reads interaction state via
`MatchContext::for_path`, matching `:hover` and `:active`. Text
selection and Ctrl+C clipboard are wired. Viewport/element scrollbars
paint and respond to drag.

Remaining work: `pointer-events` skip in hit test, `overflow`-clip in
hit test, double-click, `:focus` / `:focus-visible` / `:focus-within`
state, keyboard navigation (Tab, arrow keys), re-cascade caching,
`user-select` enforcement, caret overlay.

No event handlers, no animations, no transforms in hit-testing —
same posture the rest of the engine has. JS is a hard non-goal of
the project (see `roadmap.md`); pseudo-class state on the cascade
is the only "interactivity" surface that ever exists.
