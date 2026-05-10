---
sidebar_position: 1
---

# What is lui?

lui is a **GPU-accelerated HTML/CSS rendering engine for Rust**. It takes real HTML5 markup and CSS stylesheets, runs them through a full browser-style pipeline (parser → cascade → layout → paint → render), and draws the result on screen using wgpu — all without a web browser.

## What It Is

- A **layout engine** that understands block flow, Flexbox, CSS Grid, and inline formatting contexts
- A **style engine** that runs the CSS cascade — UA defaults, author rules, inline styles, selector matching, inheritance, and `!important`
- A **text renderer** shaped by HarfBuzz (via cosmic-text), rasterised into a GPU glyph atlas
- A **GPU renderer** with custom WGSL shaders for SDF-rounded quads, alpha-tested glyphs, and textured images

## What It Is NOT

- **Not a web browser** — no tab management, no address bar, no history, no web platform APIs
- **Not an Electron alternative** — you embed lui inside your Rust application, controlling window, input, and the event loop via winit
- **No JavaScript** — this is permanent. There is no `<script>` execution, no JS engine, no `eval`, no `addEventListener`. All logic lives in Rust.
- **Not a full CSS engine** — many CSS3/4 features are intentionally out of scope. See the [Supported CSS](../features/supported-css) page.

## Core Design Philosophy

1. **Keep the HTML/CSS pipeline intact** — the parser, cascade, layout, and paint stages mirror a real browser's internal pipeline. CSS properties behave the way web developers expect: margins collapse, flex items grow and shrink, grid tracks distribute `fr` space.

2. **No JavaScript — ever** — script execution is permanently excluded. Instead, you wire interactivity through Rust closures. `on_click`, `on_mouse_enter`, `on_event` — all typed Rust callbacks.

3. **GPU-first** — the renderer owns a wgpu device, surface, and three GPU pipelines. Rendering is not a bolt-on; every draw command goes through display lists → quad/glyph/image pipelines → GPU frame.

## Workspace Crates

### Core Pipeline

| Crate | Role |
|---|---|
| `lui-parser` | HTML tokenizer, tree builder, CSS declaration parser, stylesheet parser |
| `lui-models` | `Style` struct (~100 fields), CSS enums, ~100 HTML element structs |
| `lui-tree` | `Tree` / `Node` / `Element`, font registration, event callbacks, interaction state, focus, DOM-style queries, tree hooks |
| `lui-style` | Cascade engine: UA stylesheet, selector matching, CSS-wide keywords, inheritance, `var()`, `@media` |
| `lui-layout` | Block flow, Flexbox, Grid, IFC, table, hit testing, image loading, scroll geometry, gradient rasterization |
| `lui-text` | Font database, cosmic-text shaping, glyph atlas (rasterisation + GPU upload) |
| `lui-renderer` | wgpu device/surface, quad pipeline (SDF), glyph pipeline, image pipeline, scissor clipping, screenshots |
| `lui` | Façade: parse → cascade → layout → paint, `PipelineCache`, interactivity, text selection, scroll utilities |
| `lui-events` | Typed DOM-style event structs: `HtmlEvent`, `MouseEvent`, `KeyboardEvent`, etc. |
| `lui-assets` | Asset management: image caching, font loading, preload queue |

### Optional Crates

| Crate | Role |
|---|---|
| `lui-ui` | Component framework: `Component` trait, `El` builder DSL, reactive `Store<T>`, render caching |
| `lui-devtools` | Visual devtools panel (component tree browser, styles inspector, breadcrumb bar, pick mode) |

### Driver Crates (in `drivers/`)

| Crate | Role |
|---|---|
| `lui-driver` | `Driver` trait + `Runtime<D>` abstraction for connecting any windowing system |
| `lui-driver-winit` | winit window driver: `WinitDriver::bind(window, tree)` |
| `lui-driver-egui` | egui/eframe driver: `EguiRunner::show(ui, tree, size)` |
| `lui-driver-bevy` | Bevy plugin: `LuiPlugin`, `HtmlOverlay` resource |

## Use Cases

- **Game UI** — inventory screens, HUDs, settings menus. GPU-native rendering integrates with existing wgpu pipelines.
- **Desktop applications** — tools, editors, and dashboards that benefit from HTML/CSS layout with native Rust backends.
- **Developer tools** — property inspectors, log viewers, profiler panels.
- **Embedded displays** — fixed-layout information screens driven by Rust logic and styled with CSS.

## Next Steps

- [Installation](./installation) — add lui to your `Cargo.toml`
- [Quick Start](./quick-start) — get "Hello World" on screen
- [Engine Architecture](../engine/architecture) — understand the pipeline
