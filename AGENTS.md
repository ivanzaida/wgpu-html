# AGENTS.md

## Scope and first reads
- This workspace is a Rust HTML/CSS renderer built on `wgpu`; **JavaScript is permanently out of scope** (`docs/full-status.md`).
- Start with `docs/full-status.md`, then trace the runtime through `crates/lui/src/lib.rs`, `crates/lui/src/paint.rs`, and `crates/lui-demo/src/main.rs`.
- There were no existing repo-local agent instruction files or `README.md` matches in the workspace-wide convention search.

## Big picture architecture
- The core pipeline is: `lui-parser` → `lui-style` → `lui-layout` → `lui::paint` → `lui-renderer`.
- `lui-parser::parse()` builds a `lui_tree::Tree` from HTML; inline CSS and `<style>` blocks stay attached to the tree.
- `lui-style::cascade()` applies UA rules + author rules + inline style + inheritance, producing a `CascadedTree`. Dynamic pseudo-classes come from `Tree.interaction`.
- `lui-layout::layout_with_text()` is the main geometry stage. It resolves sizes, flex/grid, text shaping, overflow, images, and returns a `LayoutBox` tree.
- `lui::paint` converts `LayoutBox` into a backend-agnostic `DisplayList`; `lui-renderer::Renderer::render()` is the only GPU-facing stage.
- Keep those boundaries clean: parsing/style/layout should resolve semantics up front so paint/render stay dumb.

## Important cross-crate invariants
- `LayoutBox` child structure mirrors the source `Tree`; hit-testing/event dispatch depend on path compatibility (`crates/lui-layout/src/lib.rs`, `crates/lui-tree/src/lib.rs`).
- Visual reordering can move boxes without reordering children. Example: flex `order` changes coordinates but source order stays intact for hit-testing (`crates/lui-layout/src/tests.rs`).
- Interactivity flows through `Tree.interaction` and node callback fields (`on_click`, `on_mouse_enter`, etc.); the demo wires these with `tree.get_element_by_id(...)`.
- Image loading is owned by the layout crate, not the renderer: async fetch/decode, cache TTL, preload queue, and animated-frame selection all live in `crates/lui-layout/src/lib.rs`.
- Text selection is split across crates: layout provides hit/cursor geometry, `lui` stores selection helpers, and paint renders highlight/background.

## Developer workflow
- Main validation loop: `cargo test --workspace`.
- Common targeted loops:
  - `cargo test -p lui-layout`
  - `cargo test -p lui-parser`
  - `cargo test -p lui`
  - `cargo run -p lui-demo`
- The demo page is hard-coded by `const DEFAULT_DOC: &str = include_str!("../html/flex-browser-like.html");` in `crates/lui-demo/src/main.rs`. To exercise another demo, change that include.
- Demo controls from current code: `F12` saves a PNG screenshot, `Esc` exits, `Ctrl+A` selects all text, `Ctrl+C` copies selection.

## Project-specific conventions
- Most regression coverage is inline HTML/CSS in Rust unit tests, not external fixtures. Follow patterns in `crates/lui-layout/src/tests.rs` and `crates/lui/src/paint.rs`.
- Tests commonly neutralize UA defaults with `body { margin: 0; }`; do that unless you are explicitly testing UA stylesheet behavior.
- Geometry assertions use the three canonical rectangles: `margin_rect` (flow spacing), `border_rect` (paint box), `content_rect` (child/layout box). Be explicit about which one you mean.
- Demo HTML files under `crates/lui-demo/html/` are living browser-parity cases; if a bug is visual, add or update a focused HTML demo alongside the Rust regression test.
- Prefer current code and `docs/full-status.md` when a `spec/*.md` note lags behind implementation.

## Change guidance
- For parser/style changes, trace through to layout tests because unsupported selectors/properties often fail later as missing geometry, not parse errors.
- For layout changes, inspect both `crates/lui-layout/src/lib.rs` and the dedicated helpers in `flex.rs`, `grid.rs`, and `length.rs` before editing.
- For paint/render changes, check both display-list generation (`crates/lui/src/paint.rs`) and renderer consumption (`crates/lui-renderer/src/lib.rs`, `quad_pipeline.rs`, `glyph_pipeline.rs`, `image_pipeline.rs`).
- If you add a new interactive behavior, update both dispatch (`crates/lui/src/interactivity.rs`) and the cascade/state assumptions in `lui-style`.

## Known-fixed bugs (reference for future debugging)

### "No text after textarea" — stale `DisplayCommand::clip_index` after `finalize()` retain

**Symptom:** Every glyph following a `<textarea>` (or any `overflow ≠ visible`
element with no painted children) was invisible.  Quad-based drawables
(backgrounds, borders, underlines) survived, so boxes were *visible but
empty* — deeply misleading.

**Root cause:** `DisplayList::finalize()` called `clips.retain(…)` to drop
empty clip ranges, which shifted all subsequent slot indices.  Commands
that were stamped with `clip_index = N` at push time now pointed past
their clip's new position.  The `glyph_pipeline` (and `quad_pipeline`)
resolve per-command `clip_index` via a positional `clip_slots` table —
a shifted index either hit a *different* slot (whose `glyph_range` was
empty → silent skip) or hit `None` → draw not issued.

**Why only textarea:** `<textarea>` is the one stock element that ships with
UA-default `overflow: auto` **and** no rendered children (its placeholder
is painted as the box's own `text_run`, before `push_clip`).  So the clip
range it opens is always empty → retained out → indices shift.  Elements
with `overflow: hidden` that have children don't trigger it because their
children paint *inside* the range, keeping it non-empty.

**Fix:** `DisplayList::finalize()` now builds an old → new index remap
*before* `retain` and patches every `DisplayCommand::clip_index`
accordingly.  Commands on a dropped slot fall back to the nearest
surviving predecessor.

**Tests:**
- `lui-renderer`: `finalize_remaps_command_clip_index_when_empty_ranges_dropped` — pushes a glyph, opens+closes an empty clip, pushes more content, finalizes, asserts every command's `clip_index` is in-bounds and lands on the correct post-retain slot.
- `lui`: `glyphs_after_overflow_auto_sibling_are_not_clipped` — synthetic layout with an `overflow:auto` block followed by a text leaf, checks no clip suppresses the text glyph.
- The existing quad-only guards (`real_textarea_in_flex_row_does_not_clip_following_block_quad`, `overflow_auto_in_flex_row_does_not_clip_block_sibling_below`) were not sufficient alone — they only walked `quad_range`, not `glyph_range`.

**Files changed:** `crates/lui-renderer/src/paint.rs` (fix + test),
`crates/lui/src/paint.rs` (test).

## Debugging
- `cargo run -p lui-demo <file.html>` launches a WINIT window with selected file as html content.
- STDIN command `make_screenshot` saves a PNG screenshot of the current window.
- STDIN command `make_screenshot <query>` from `spec/query.md` saves a PNG screenshot of the element.
