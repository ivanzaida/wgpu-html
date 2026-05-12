---
title: What is lui?
---

# What is lui?

lui is a **GPU-accelerated HTML/CSS rendering engine for Rust**. It takes real HTML5 markup and CSS stylesheets, runs them through a full browser-style pipeline (parser → cascade → layout → paint → render), and draws the result on screen using wgpu — all without a web browser.

## What It Is

- A **layout engine** that understands block flow, Flexbox, CSS Grid, and inline formatting contexts
- A **style engine** that runs the CSS cascade — UA defaults, author rules, inline styles, selector matching, inheritance, and `!important`
- A **text renderer** shaped by HarfBuzz (via cosmic-text), rasterised into a GPU glyph atlas
- A **GPU renderer** with custom WGSL shaders for SDF-rounded quads, alpha-tested glyphs, and textured images
- A **component framework** with Elm-architecture state management, scoped CSS, and render caching

## What It Is NOT

- **Not a web browser** — no tab management, no address bar, no history, no web platform APIs
- **Not an Electron alternative** — you embed lui inside your Rust application, and you control the window, input, and event loop via winit
- **No JavaScript** — this is permanent. There is no `<script>` execution, no JS engine, no `eval`, no `addEventListener`. All logic lives in Rust.
- **Not a full CSS engine** — many CSS3/4 features are intentionally out of scope. See the [Status page](../status) for what is and isn't supported.

## Core Design Philosophy

lui is built around three principles:

1. **Keep the HTML/CSS pipeline intact** — the parser, cascade, layout, and paint stages mirror a real browser's internal pipeline. This means CSS properties behave the way web developers expect: margins collapse (mostly), flex items grow and shrink, grid tracks distribute `fr` space.

2. **No JavaScript — ever** — script execution is permanently excluded. Instead, you wire interactivity through Rust closures and the component framework. `on_click`, `on_mouse_enter`, `on_event` — all typed Rust callbacks. The component framework provides reactive state management without a runtime scripting language.

3. **GPU-first** — the renderer owns a wgpu device, surface, and three GPU pipelines. Rendering is not a bolt-on; every draw command goes through display lists → quad/glyph/image pipelines → GPU frame.

## The 14 Crates

lui is organised into 14 crates, each with a focused responsibility:

| Crate | Role |
|---|---|
| `lui-parser` | HTML tokenizer, tree builder, CSS declaration parser, stylesheet parser |
| `lui-models` | `Style` struct (~80 fields), CSS enums, ~100 HTML element structs |
| `lui-tree` | `Tree` / `Node` / `Element`, font registration, event callbacks, interaction state, focus, DOM-style query helpers |
| `lui-style` | Cascade engine: UA stylesheet, selector matching, field merge, CSS-wide keywords, inheritance, color handling |
| `lui-text` | Font database, text shaping (cosmic-text), glyph atlas (rasterisation + GPU upload) |
| `lui-events` | Typed DOM-style event structs: `HtmlEvent`, `MouseEvent`, `KeyboardEvent`, `FocusEvent`, `InputEvent`, event phases, bubbling semantics |
| `lui-layout-old` | Block flow, Flexbox, Grid, inline formatting context, hit testing, image loading/cache, scroll geometry, placeholder shaping |
| `lui-renderer` | wgpu device/surface, quad pipeline (SDF), glyph pipeline, image pipeline, scissor clipping, screenshot |
| `lui` | Façade: `parse → cascade → layout → paint`, interactivity wrappers, `PipelineTimings`, text selection, scroll utilities |
| `lui-winit` | winit ↔ engine glue: type translators, input forwarders, batteries-included `LuiWindow` harness |
| `lui-ui` | Elm-architecture component framework: `Component` trait, `El` builder DSL, reactive `Store<T>`, render caching |
| `lui-devtools` | Visual devtools panel (component tree browser, styles inspector, breadcrumb bar) |
| `lui-egui` | Alternative `egui` / `eframe` integration backend |
| `lui-demo` | Thin shell over `lui-winit` for running demo HTML files |

## Use Cases

lui is designed for **Rust applications** that need rich, styled UI without a web browser dependency:

- **Game UI** — inventory screens, HUDs, settings menus, dialogue boxes. GPU-native rendering integrates with existing wgpu pipelines.
- **Desktop applications** — tools, editors, and dashboards that benefit from HTML/CSS layout with native Rust backends.
- **Developer tools** — property inspectors, log viewers, profiler panels. Embeddable alongside egui.
- **Kiosk / embedded displays** — fixed-layout information screens driven by Rust logic and styled with CSS.

## Comparison to Web Browsers vs RmlUI

lui sits between a full browser engine and RmlUI. Compared to **web browsers** (Servo, WebKit, Blink), lui is far smaller (no JavaScript, no Web APIs, no networking stack beyond image loading) and targets embedded UI rather than general web content. Compared to **RmlUI**, lui matches HTML/CSS semantics more closely (CSS Grid, custom properties, `calc()`/`var()`, the full CSS cascade) but lacks RmlUI's advanced visual features (animations, transforms, gradients, decorators, table/float layout). See the [full comparison](../comparison-lui-vs-rmlui) for details.

## Next Steps

- [Installation](./installation) — add lui to your `Cargo.toml`
- [Quick Start](./quick-start) — get "Hello World" on screen in minutes
- [Implementation Status](../status) — see what's done and what's in progress
