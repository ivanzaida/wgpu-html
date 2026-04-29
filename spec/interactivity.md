# wgpu-html — Interactivity Spec (Mouse-First)

The plan for moving from "static layout, F12 + Esc only" to a tree
that responds to pointer input: hover / press / click / wheel, plus
the cascade hooks (`:hover`, `:active`, `:focus`, …) and the
plumbing (`pointer-events`, `cursor`, `user-select`) that have to
exist for any of it to behave like a browser.

**Status (2026-04-29):**

- **M-INTER-1 ✅ shipped.** `InteractionState` on `Tree` (now also
  carries `focus_path` and `modifiers`), all four pointer functions
  wired (`pointer_move`, `mouse_down`, `mouse_up`, `pointer_leave`),
  hover/active tracking, enter/leave callbacks, click synthesis,
  `:hover`/`:active` cascade integration via `MatchContext::for_path`,
  typed DOM events through `wgpu-html-events` (`HtmlEvent`,
  `MouseEvent`, `EventPhase`), `buttons_down` bitmask. Most dispatch
  logic moved to `wgpu_html_tree::dispatch` (path-based, no layout
  dep); `wgpu_html::interactivity` is a thin layout-aware wrapper.
- **Focus + keyboard foundations ✅ shipped** (overlapping with
  M-INTER-2 and M-INTER-5). `focus_path` on `InteractionState`,
  `:focus` cascade matching (exact element only — no propagation;
  `:focus-within` not yet), focus/blur/focusin/focusout dispatch
  with `related_target`, Tab + Shift+Tab navigation built into
  `key_down`, `Modifier` enum + `Tree::set_modifier` /
  `Tree::modifiers()` (dispatchers no longer take a `Modifiers`
  parameter — they read tree state). `Tree::focus`, `Tree::blur`,
  `Tree::focus_next`, `Tree::key_down`, `Tree::key_up`,
  `Tree::pointer_leave` as inherent methods. New `focus` module:
  `is_focusable`, `is_keyboard_focusable`, `focusable_paths`,
  `keyboard_focusable_paths`, `next_in_order`, `prev_in_order`,
  plus `Element::tabindex()`. Demo wires winit `KeyboardInput`
  through `wgpu_html_winit::handle_keyboard`.
- **Form fields ⚠️ partial → text editing ✅ shipped.**
  Placeholder rendering via `compute_placeholder_run` (color × 0.5
  alpha, single-line clip + centre, textarea soft-wrap).
  **Text editing shipped:** `EditCursor` on `InteractionState`,
  `text_edit` module (insert/delete/arrows/home/end/select-all),
  `text_input` + `handle_edit_key` dispatchers, `compute_value_run`
  for value rendering, password masking (U+2022), blinking caret
  quad, edit selection highlight, click-to-position caret,
  clipboard (Ctrl+C/V/X), textarea multi-line (Enter, ArrowUp/Down).
  DOM `key` now derived from `event.logical_key` (layout-aware).
  See `spec/input.md` for full status.
  **Fixed:** textarea's UA `overflow: auto` no longer suppresses
  glyphs in following siblings — `DisplayList::finalize` remaps
  `DisplayCommand::clip_index` on retain (AGENTS.md).
  Still ❌: `InputEvent` dispatch, `maxlength`, word-level ops,
  horizontal scroll in overflowing input, undo/redo, IME,
  checkbox/radio toggle, `<select>` menu, `<form>` submit.
- **M-INTER-3 ⚠️ partial.** `TextCursor`/`TextSelection` on
  `InteractionState`, drag-to-select, `select_all_text` /
  `selected_text`, `Ctrl+A`/`Ctrl+C` + `arboard` (now built into
  the `wgpu-html-winit` harness), selection highlight quads in
  `paint.rs`. Form-control edit selection and blinking caret are
  shipped. Word/line select and `user-select` property enforcement
  are still ❌.
- **M-INTER-4 ⚠️ partial.** `scroll_offsets_y: BTreeMap` on
  `InteractionState`, viewport scroll + per-element scroll-container
  offset, scrollbar paint, drag-to-scroll, `MouseWheel`. Scroll +
  scrollbar utilities live in the new public `wgpu_html::scroll`
  module. `Wheel` events are not forwarded to element `on_event`
  callbacks.
- **DOM-style query helpers ✅ shipped.** New `wgpu_html_tree::query`
  module: `SelectorList`, `ComplexSelector`, `CompoundSelector`,
  `Combinator`, plus `Tree::query_selector` /
  `query_selector_all` / `query_selector_path` /
  `query_selector_all_paths` and `Node::*` mirrors. Phase 1 of the
  selector spec (CSS Level 4 subset) is in: tag/id/class compound
  selectors, all four combinators (` `, `>`, `+`, `~`), selector
  lists (`A, B`), all six attribute operators
  (`[a]`/`=`/`~=`/`|=`/`^=`/`$=`/`*=`) with the `i` / `s` case
  flag. Pseudo-classes (`:hover`, `:nth-child`, `:not`, …) and
  pseudo-elements still parse as errors and degrade to "no match"
  via the lenient `From<&str>` path. Full status, grammar, and
  matcher semantics live in `spec/query.md`.
- **`wgpu-html-winit` harness ✅ shipped.** New crate with
  `WgpuHtmlWindow` (full `winit::ApplicationHandler` impl that owns
  the window/renderer/text-context), `AppHook` trait
  (`on_key`/`on_frame`/`on_pointer_move`), built-in viewport
  scroll, scrollbar drag, clipboard, F12 screenshot. The demo's
  `App` + `ApplicationHandler` are gone; profiling is now an
  `AppHook` impl. See §15 for the API surface.
- **M-INTER-2, M-INTER-6 ❌ not yet done.** `pointer-events: none`
  in hit-test, double-click,
  `:focus-within`, `:focus-visible`, `:disabled`, re-cascade caching.

Companion to `spec/events.md` (typed event structs and dispatch
matrix), `roadmap.md` (§M12 interactivity milestones),
`docs/full-status.md` §7 (interactivity), and `text.md` (text
selection lands here, not in the text spec).

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
- No IME / composition events yet. Plain text input is wired for
  `<input>` / `<textarea>`, but composition lifecycle events are not.
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
    /// Path to the element that currently has keyboard focus.
    /// Read by `MatchContext::for_path` to resolve `:focus` (exact
    /// match, not prefix — `:focus` does not propagate to ancestors).
    pub focus_path:       Option<Vec<usize>>,
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
    /// Currently-held modifier keys. Updated by hosts via
    /// `Tree::set_modifier(Modifier, bool)`; read by mouse and
    /// keyboard dispatchers when they fire DOM events. Hosts no
    /// longer thread `Modifiers` through every dispatch call.
    pub modifiers:        Modifiers,
    /// Caret/selection state inside the currently focused `<input>` or
    /// `<textarea>`.
    pub edit_cursor:      Option<EditCursor>,
    /// Blink epoch for the form-control caret.
    pub caret_blink_epoch: Instant,
}
```

Fields **still missing**: `focus_visible` (the `:focus-visible`
flag set on Tab focus, cleared on press) lands with M-INTER-2.

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

The current hit test (`crates/wgpu-html-layout/src/lib.rs`)
returns the deepest descendant whose `border_rect` contains the
point, walking children last-to-first so the topmost paint wins.
That contract stays.

**Shipped:**

- **Overflow clipping.** Ancestor `overflow: hidden | scroll | auto`
  now clips hit-testing to the effective padding-box clip. A child
  painted outside a clipped ancestor no longer receives hits there.
- **Inline / line-box hit testing.** `LayoutBox::hit_text_cursor`
  returns a `TextCursor` for the deepest selectable text run under
  the pointer. Form-control internal value / placeholder text is
  excluded from document-level drag selection via `text_unselectable`.

**Still missing / mocked:**

- **`pointer-events: none`.** A box with `pointer_events == None`
  is invisible to hit-testing — both itself and (per CSS) its
  descendants if they don't override back to `auto`. Implementation:
  `collect_hit_path` skips boxes whose cascaded style sets
  `pointer-events: none`, walks into descendants only when the
  parent is `auto` or the descendant explicitly re-enables.
- **Scroll offset.** Once scrolling exists (§12), descendants of a
  scroll container are hit-tested against `point − scroll_offset`,
  applied at the boundary. The winit harness accounts for viewport
  / element scroll in its own scrollbar paths, but `LayoutBox`
  itself does not carry scroll offsets.
- **Richer text hit payload.** `hit_text_cursor` returns path +
  glyph index. The older planned `HitTest { text: TextHit { line,
  byte_offset, trailing } }` shape below is still future API design,
  not the current public return type.

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

Event structs, event-name constants, bubbling behavior, and the
current event support matrix live in `spec/events.md`.

The interactivity layer currently dispatches mouse down/up/click,
mouseenter/mouseleave, focus/blur/focusin/focusout, and keydown/keyup
through `Node::on_event`. Many additional event structs exist in
`wgpu-html-events` for future parity, but are not emitted yet.

## 7. Press semantics & implicit pointer capture

When `MouseDown { Primary }` fires:

1. Resolve the hit path; ignore if it's `None`. `pointer-events:
   none` is not honoured yet, so it does not filter the hit path.
2. Set `interaction.active_path = Some(path)`.
3. Set `interaction.focus_path = Some(focusable_ancestor(path))`
   if any (§13). `focus_visible` does not exist yet.
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
    pub is_focus:  bool,
}

impl MatchContext {
    pub fn for_path(path: &[usize], state: &InteractionState) -> Self {
        Self {
            is_hover:  path_is_prefix(path, state.hover_path.as_deref()),
            is_active: path_is_prefix(path, state.active_path.as_deref()),
            // `:focus` matches the focused element only — not its
            // ancestors. `:focus-within` (which would propagate)
            // is not yet implemented.
            is_focus:  state.focus_path.as_deref() == Some(path),
        }
    }
}
```

Selector pseudo-classes `:hover`, `:active`, and `:focus` are parsed
and matched against `MatchContext` in the selector-matching pass.
All other pseudo-classes (`:focus-visible`, `:focus-within`,
`:disabled`, `:checked`, structural pseudos) return `false` today.

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

**Not implemented yet.** `cursor` is parsed and stored on `Style`,
but no host-facing resolver or OS cursor update is wired.

Planned per-frame behavior after cascade:

```text
cursor = first ancestor of hover_path whose cascaded `cursor` is set
       fall back to Cursor::Default
```

This would likely be exposed as a resolver over `hover_path` + the
cascaded tree. The host would then map it to the OS cursor by setting
winit's `Window::set_cursor` once per change.

`Cursor::Auto` resolves to a contextual default: `Cursor::Text`
inside a text leaf with `user-select != none`, otherwise
`Cursor::Default`. Matches browser behaviour for "auto means
caret on text".

## 10. `pointer-events` and `user-select`

Already modelled (`crates/wgpu-html-models/src/common/css_enums.rs:209`,
`:215`) and parsed into `Style`, but not honoured downstream:

- `pointer-events: none` — described in §5. Should inherit once
  implemented.
- `user-select: none` — the text leaves under it are excluded
  from drag-selection (§11) and clicks on them never start a
  selection. Should inherit once implemented.
- `user-select: text` (the default) — selectable.
- `user-select: all` — a single click anywhere inside the subtree
  selects the whole subtree. Useful for code blocks.
- `user-select: auto` — same as `text` for text leaves, `none`
  for non-text. CSS quirk preserved.

The cascade does not currently inherit either property; `cursor` is
the only interaction-adjacent property in the typed inheriting set.
Add `pointer-events` / `user-select` inheritance only when their
behavior is actually enforced.

## 11. Text selection

Document text selection is shipped as one contiguous range of
`TextCursor { path, glyph_index }` endpoints across document order:

```rust
pub struct TextSelection {
    pub anchor: TextCursor,
    pub focus:  TextCursor,
}

pub struct TextCursor {
    pub path:        Vec<usize>,
    pub glyph_index: usize,
}
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

Painting emits one highlight quad per line segment of the selection
range. Form-control internal value / placeholder text is marked
`text_unselectable`, so document-level drag selection skips it.

**Still missing:** word select (double-click), line select
(triple-click), Shift+click extension, and `user-select` enforcement
(`none`, `text`, `all`, `auto`).

Copy: `wgpu-html` exposes `select_all_text` / `selected_text`; the
winit harness wires document-level `Ctrl+A` / `Ctrl+C` through
`arboard`. Clipboard access still belongs to the host crate, not the
core tree/layout crates.

Form-control editing has a separate caret path:
`InteractionState::edit_cursor: Option<EditCursor>`, with byte
offsets into the focused `<input>` / `<textarea>` value. Paint emits
edit selection highlights and a blinking caret for this internal
control state.

## 12. Scrolling

**Partial / shipped in the harness.**

- `InteractionState::scroll_offsets_y: BTreeMap<Vec<usize>, f32>`
  stores per-element vertical scroll state by path.
- The public `wgpu_html::scroll` module exposes viewport and
  element scrollbar geometry, scrollbar paint, scroll clamping,
  hit-tests, and element scroll helpers.
- The `wgpu-html-winit` harness handles `MouseWheel`, viewport
  scroll, per-element scroll containers, and scrollbar dragging.
- Paint clips scroll containers and translates descendants by the
  stored offset.

**Still missing / mocked:**

- No DOM `wheel` event is forwarded to `Node::on_event`.
- `LayoutBox` does not embed scroll offsets; they stay on
  `InteractionState`.
- Generic hit-testing does not fully subtract ancestor scroll offsets
  inside `LayoutBox::hit_path`; host scroll paths compensate where
  needed.
- Horizontal element scrolling, smooth scrolling, scroll-snap, and
  momentum are not implemented.

`Wheel`'s `ScrollDelta::Lines` is converted to pixels by
multiplying by a constant (default 16 px / line, configurable).
`ScrollDelta::Pixels` is used as-is.

Smooth scrolling, scroll-snap, momentum: all post-§12.

## 13. Keyboard / focus

Mouse-first, but focus is shared state.

**Shipped:**

- Focusable elements: `<button>` (unless `disabled`), `<a href>`,
  `<input>` (unless `disabled` or `type="hidden"`), `<textarea>`
  (unless `disabled`), `<select>` (unless `disabled`), `<summary>`,
  any element with `tabindex >= 0`. `tabindex < 0` makes an
  element scriptable-focus only (excluded from Tab traversal).
  See `wgpu_html_tree::focus::{is_focusable,
  is_keyboard_focusable, focusable_paths,
  keyboard_focusable_paths}`.
- `Tree::focus(Some(&path))` walks up to the nearest focusable
  ancestor (so clicking a `<span>` inside a `<button>` focuses
  the button), then fires `blur` + `focusout` on the previous
  focus and `focus` + `focusin` on the new one. `related_target`
  carries the other end of the transition.
- `Tree::blur()` clears focus and fires the same blur/focusout
  pair.
- `Tree::focus_next(reverse: bool)` cycles forward (or backward
  with Shift) through `keyboard_focusable_paths` in document
  order, wrapping at the ends.
- `Tree::key_down(key, code, repeat)` fires a `keydown` event on
  the focused element's ancestry (or on the root if nothing is
  focused), then advances focus when `key == "Tab"` (Shift held
  → reverse). `Tree::key_up(key, code)` fires the matching
  `keyup`. Modifier state is read from `tree.interaction.modifiers`
  — hosts call `Tree::set_modifier(Modifier, bool)` from their
  key event handler.
- A primary `mouse_down` walks up to the closest focusable
  ancestor of the hit path and focuses it (or blurs if none).
  Mirrors browser order: `mousedown` fires first, then
  focus/blur.

**Not yet:**

- `Enter` / `Space` on a focused button / link → synthesised
  primary click. (Today the host has to listen for `keydown` and
  call into the model.)
- `Esc` clears focus + selection. (Today the harness exits the
  app on Esc by default; configurable via `with_exit_on_escape`.)
- PageUp / PageDown caret movement. Arrow keys, Home, and End are
  wired for focused `<input>` / `<textarea>` controls, but not for
  arbitrary contenteditable text leaves.
- `:focus-visible` (the "is this focus from keyboard?" flag) and
  `:focus-within` (the propagating-to-ancestors variant).

## 14. Frame loop integration

There's no single `dispatch(ev)` front door — the API is a small
set of focused functions that hosts call from their own event
loop. The two layers:

**Layout-aware wrappers** in `wgpu_html::interactivity` (use these
when you have a `LayoutBox` handy and want hit-testing done for
you):

```rust
pub fn pointer_move(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32)) -> bool;
pub fn mouse_down (tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool;
pub fn mouse_up   (tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool;
// re-exports of the path-based dispatchers below:
pub use wgpu_html_tree::{
    blur, dispatch_pointer_leave as pointer_leave,
    focus, focus_next, key_down, key_up,
};
```

These hit-test (`layout.hit_path`, `layout.hit_text_cursor`) and
forward into the path-based dispatchers below.

**Path-based dispatchers** in `wgpu_html_tree::dispatch` (use
these when the host already knows the target path, or to drive
the engine without a `wgpu-html-layout` dependency):

```rust
// Mouse — caller supplies the hit-tested target_path.
pub fn dispatch_pointer_move (tree: &mut Tree, target: Option<&[usize]>, pos, text_cursor) -> bool;
pub fn dispatch_pointer_leave(tree: &mut Tree);
pub fn dispatch_mouse_down   (tree: &mut Tree, target: Option<&[usize]>, pos, button, text_cursor) -> bool;
pub fn dispatch_mouse_up     (tree: &mut Tree, target: Option<&[usize]>, pos, button, text_cursor) -> bool;
// Focus / keyboard — no layout needed.
pub fn focus     (tree: &mut Tree, path: Option<&[usize]>) -> bool;
pub fn blur      (tree: &mut Tree) -> bool;
pub fn focus_next(tree: &mut Tree, reverse: bool) -> Option<Vec<usize>>;
pub fn key_down  (tree: &mut Tree, key: &str, code: &str, repeat: bool) -> bool;
pub fn key_up    (tree: &mut Tree, key: &str, code: &str) -> bool;
pub fn text_input(tree: &mut Tree, text: &str) -> bool;
```

All dispatchers above except `text_input` are also inherent methods on
`Tree` (`tree.focus(...)`, `tree.dispatch_mouse_down(...)`, etc.).
`text_input` is re-exported as a free function and mutates the focused
editable control when possible.

Modifier state lives on `tree.interaction.modifiers`; hosts
update it via `tree.set_modifier(Modifier::Shift, true/false)`
from their key-event handler. Dispatchers read from there when
they fire DOM events — no `Modifiers` parameter on the public
API.

**Frame loop** (winit example, with the harness from
`wgpu-html-winit`):

```rust
let mut tree = wgpu_html_parser::parse(html);
wgpu_html_winit::register_system_fonts(&mut tree, "DemoSans");
wgpu_html_winit::create_window(&mut tree)
    .with_title("My App")
    .with_hook(MyHook { /* … */ })
    .run()?;
```

The harness owns the window/renderer/text-context, runs cascade →
layout → paint → render on each redraw, forwards mouse/keyboard
input into `tree.dispatch_*`, and provides built-in scroll,
scrollbar drag, clipboard, and screenshot. The host plugs custom
behaviour into `AppHook::on_key` / `on_frame` / `on_pointer_move`.

Hosts that need finer control (or non-winit windowing) skip the
harness and call `wgpu_html::interactivity::*` directly per
window event, and `wgpu_html::paint_tree_returning_layout_profiled`
per frame.

## 15. Public API surface

```
wgpu-html-tree
  + InteractionState                                                     ✅
    (hover/active/focus/selection/scroll/buttons/time_origin/modifiers)
  + Tree::interaction (field), Tree::clear_selection                      ✅ M-INTER-1
  + TextCursor, TextSelection, SelectionColors                            ✅ M-INTER-3
  + focus_path on InteractionState                                        ✅ (focus slice)
  + Modifier { Ctrl, Shift, Alt, Meta }, Modifiers::set                   ✅ (focus slice)
  + Tree::set_modifier, Tree::modifiers()                                 ✅ (focus slice)
  + dispatch module (path-based, no layout dep):                          ✅ (focus slice)
    - dispatch_pointer_move/_leave/_mouse_down/_mouse_up
    - focus, blur, focus_next, key_down, key_up, text_input
    - Tree::focus / blur / focus_next / key_down / key_up /
      pointer_leave / dispatch_mouse_down/up / dispatch_pointer_move
      as inherent methods
    - text_input is a free function today, not an inherent Tree method
  + EditCursor + InteractionState::edit_cursor/caret_blink_epoch           ✅ (input slice)
  + focus module: is_focusable, is_keyboard_focusable,                    ✅ (focus slice)
    focusable_paths, keyboard_focusable_paths,
    next_in_order, prev_in_order, Element::tabindex()
  + query module: CompoundSelector, Tree::query_selector,                 ✅
    query_selector_all, query_selector_path,
    query_selector_all_paths, Node::query_selector* mirrors
    (selector lists, combinators, attribute operators; pseudos mostly errors/no-match)
  + focus_visible flag                                                    ❌ M-INTER-2

wgpu-html-models
  + (existing) Cursor, PointerEvents, UserSelect                         ✅ done

wgpu-html-events
  + HtmlEvent, Event/UI/Mouse/Pointer/Wheel/Keyboard/Focus/Input/         ✅ M-INTER-1
    Composition/Clipboard/Drag/Touch/Animation/Transition/Submit/
    FormData/Toggle/Progress structs, EventPhase, HtmlEventType
  + Mouse, keyboard, and focus dispatch wired                             ✅
  + Wheel/Input/Clipboard/Drag/Touch/Animation/Transition dispatch         ❌

wgpu-html-parser / wgpu-html-style
  + Pseudo-class selector parsing (`:hover`, `:active`, `:focus`)        ✅
  + MatchContext { is_hover, is_active, is_focus }                       ✅
  + MatchContext::for_path reads focus_path (exact match)                ✅ (focus slice)
  + CSS Color Module Level 4 system colors                                ✅
    (canvas, canvastext, linktext, visitedtext, activetext,
     buttonface, buttontext, buttonborder, field, fieldtext,
     highlight, highlighttext, selecteditem, selecteditemtext,
     mark, marktext, graytext, accentcolor, accentcolortext)
  + FontRegistry::find_first generic-family fallback                      ✅
    (sans-serif, serif, monospace, cursive, fantasy, system-ui,
     ui-*, -apple-system, BlinkMacSystemFont)
  + pointer-events / user-select cascade-inheritance entries             ❌ M-INTER-2

wgpu-html-layout
  + hit_text_cursor((x,y)) → Option<TextCursor>                         ✅ M-INTER-3 (partial)
  + Input/Textarea placeholder text run + ::placeholder color            ✅ (forms slice)
    (compute_placeholder_run; both layout_block and
     layout_atomic_inline_subtree paths)
  + Flex max-content intrinsic for non-text non-replaced items           ✅
    (text_intrinsic_main recurses into descendants)
  + overflow-hidden/scroll/auto clip in hit test                          ✅ M-INTER-2
  + pointer-events skip in hit test                                       ❌ M-INTER-2
  + scroll_offset on LayoutBox                                            ❌ M-INTER-4 (offset stays on Tree)

wgpu-html (facade)
  + interactivity::pointer_move, mouse_down, mouse_up                     ✅ M-INTER-1
    (thin layout-aware wrappers; dispatch logic is in
     wgpu-html-tree::dispatch)
  + interactivity re-exports focus, blur, focus_next,                     ✅ (focus slice)
    key_down, key_up, pointer_leave from tree
  + select_all_text, selected_text                                        ✅ M-INTER-3
  + paint emits selection quads from interaction.selection               ✅ M-INTER-3
  + PipelineTimings                                                       ✅ (profiling)
  + on_event (Node::on_event: EventCallback)                             ✅ M-INTER-1
  + scroll module (public):                                               ✅ M-INTER-4
    ScrollbarGeometry, scrollbar_geometry,
    scroll_y_from_thumb_top, paint_viewport_scrollbar,
    translate_display_list_y, clamp_scroll_y, max_scroll_y,
    document_bottom, element_padding_box,
    scrollable_content_height, max_element_scroll_y,
    element_scrollbar_geometry, deepest_scrollable_path_at,
    deepest_element_scrollbar_at, scroll_element_at,
    scroll_element_thumb_to, viewport_to_document, rect_contains

wgpu-html-winit (new)
  + WgpuHtmlWindow harness + create_window(&mut tree)                     ✅ (harness slice)
  + Builders: with_title / with_size / with_exit_on_escape /              ✅
    with_clipboard_enabled / with_screenshot_key / with_hook
  + AppHook trait: on_key, on_frame, on_pointer_move                      ✅
  + EventResponse { Continue, Stop }, HookContext, FrameTimings           ✅
  + Built-in viewport scroll (mouse wheel) + scrollbar drag               ✅
  + Built-in clipboard (Ctrl+A select-all, Ctrl+C copy via arboard)       ✅
  + Built-in screenshot key (default F12)                                 ✅
  + Translators: mouse_button, keycode_to_modifier,                       ✅
    key_to_dom_key, keycode_to_dom_code
  + Forwarders: update_modifiers, forward_keyboard, handle_keyboard       ✅
  + system_font_variants(), register_system_fonts(tree, family)           ✅

wgpu-html-demo
  + Now ~450 lines (was ~1460); App + ApplicationHandler removed         ✅
  + Uses wgpu_html_winit::create_window(...).with_hook(...).run()        ✅
  + DemoHook impl AppHook: F9 profiling toggle + 1-second stats          ✅
  + --renderer=winit|egui CLI flag (winit default; egui via              ✅
    wgpu-html-egui crate)
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
- Pseudo-class parsing: `:hover`, `:active`, `:focus`. Cascade reads
  `MatchContext`.
- `wgpu-html-events` crate: `HtmlEvent`, `MouseEvent`, typed DOM
  event dispatch via `Node::on_event`.
- Demo: cards change style on hover/active; `on_click` callbacks fire.

### M-INTER-2 — `pointer-events`, `overflow` clip, double-click ⚠️ Partial

**Done** (out of order with the original phase plan; landed
during the focus / keyboard slice):

- Ancestor `overflow != visible` clipping in `LayoutBox::hit_path`.
- `:focus` cascade (exact-match, no propagation) via
  `MatchContext::for_path` reading `state.focus_path`.
- Focus state on `InteractionState` (`focus_path`).
- `is_focusable` / `is_keyboard_focusable` predicates.

**Not yet:**

- Hit test honours `pointer-events: none` (self + descendants).
- Double-click synthesis.
- ContextMenu / AuxClick synthesis.
- `:focus-visible` (set on Tab focus, cleared on press) and
  `:focus-within`.
- `:disabled` for `<button disabled>`, `<input disabled>`, etc.
  (`is_focusable` already excludes disabled controls; the cascade
  still needs to match `:disabled`.)
- `user-select: none` recognised but not yet acted on.

### M-INTER-3 — Text selection + caret + clipboard ⚠️ Partial

**Done:**
- `TextCursor` / `TextSelection` on `InteractionState`.
- `hit_text_cursor` in layout for cursor placement.
- Press on text leaf starts selection; drag extends; release commits.
- `select_all_text` / `selected_text` in `wgpu-html`.
- Paint emits selection highlight rectangles.
- Clipboard: document-level `Ctrl+A` + `Ctrl+C` wired in the winit
  harness via `arboard`.
- Drag-select suppresses click synthesis (tested).
- Form-control `EditCursor`, click-to-position caret, edit selection,
  blinking caret quad, typed-text insertion, Backspace/Delete,
  ArrowLeft/Right/Up/Down, Home/End, textarea Enter, password masking.
- Form-control `Ctrl+A/C/V/X` wired in the winit harness.

**Not yet done:**
- Word select (double-click), line select (triple-click).
- `user-select: none / text / all` property enforcement.
- Shift+click to extend selection.
- DOM `InputEvent`, `beforeinput`, `copy`/`cut`/`paste` event
  dispatch; clipboard operations are host shortcuts, not DOM events.

### M-INTER-4 — Wheel + scroll containers ⚠️ Partial

**Done:**
- `scroll_offsets_y: BTreeMap<Vec<usize>, f32>` on `InteractionState`.
- Viewport scroll position + scrollbar paint + drag-to-scroll.
- `MouseWheel` scrolls viewport; deepest scroll-container found and
  scrolled when applicable.
- Per-element scroll container scrollbar quads in `paint.rs`.
- Paint translates scrolled descendants from `InteractionState`.

**Not yet done:**
- `Wheel` event forwarded to element `on_event` callbacks.
- `LayoutBox::scroll_offset` (offset currently lives only in
  `InteractionState`, not embedded in the box tree).
- Hit-testing subtracting ancestor scroll offsets.

### M-INTER-5 — Keyboard navigation ⚠️ Partial

**Done** (out of order; landed during the focus / keyboard slice):

- `keydown` / `keyup` dispatch via `Tree::key_down`,
  `Tree::key_up` (bubbles target → root, fires `on_event` along
  the focus-path's ancestry).
- Tab / Shift+Tab focus traversal built into `key_down`
  (cycles `keyboard_focusable_paths` in document order).
- `wgpu-html-winit::handle_keyboard` translates winit
  `KeyboardInput` → modifier update + DOM `keydown` / `keyup`.

**Not yet:**

- Enter / Space on a focused button / link → synthesised primary
  click (currently the host has to call into the model).
- Esc → clear focus + selection (the harness exits the app on
  Esc by default; configurable via `with_exit_on_escape`).
- PageUp / PageDown and word-level text navigation.
- Arrow keys / Home / End for arbitrary contenteditable text leaves.
- `InputEvent` dispatch for typed characters in `<input>` /
  `<textarea>`; values mutate, but no DOM input event is emitted.

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
- **Modifier-only events.** The winit glue now forwards bare
  modifier keys as `keydown` / `keyup` and also updates
  `InteractionState::modifiers`. No CSS state currently reads
  modifier-only changes directly.
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
`MouseEvent`, `KeyboardEvent`, `FocusEvent`, `EventPhase`, plus
typed structs for future events). Cascade reads interaction state via
`MatchContext::for_path`, matching `:hover`, `:active`, and exact
`:focus`. Text selection, form-control editing, edit caret paint,
clipboard shortcuts, viewport/element scrollbars, and scrollbar drag
are wired in the winit harness.

Remaining work: `pointer-events` skip in hit test, scroll-offset-aware
generic hit testing, double-click/contextmenu/auxclick/wheel DOM event
dispatch, `InputEvent` / clipboard DOM event dispatch,
`:focus-visible` / `:focus-within` / `:disabled`, `user-select`
enforcement, word/line selection, button/link keyboard activation,
and re-cascade caching.

There is still no JavaScript and no animation/transition runtime.
Host-installed Rust callbacks (`on_click`, mouse slots, `on_event`,
and `AppHook`) are the interactivity surface.
