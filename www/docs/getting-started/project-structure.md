---
sidebar_position: 5
---

# Project Structure

## Workspace Layout

```
lui/
├── crates/                       # Engine crates (backend-agnostic)
│   ├── lui/                # Façade: orchestration, paint, interactivity
│   ├── lui-parser/         # HTML tokenizer, CSS parser, stylesheet parser
│   ├── lui-style/          # CSS cascade, selector matching, inheritance
│   ├── lui-layout-old/         # Block, flex, grid, IFC, positioned layout
│   ├── lui-text/           # Font DB, cosmic-text shaping, glyph atlas
│   ├── lui-display-list/   # Backend-agnostic display list IR
│   ├── lui-tree/           # DOM tree, events, focus, query selectors
│   ├── lui-events/         # Typed event structs
│   ├── lui-models/         # Shared types (Style, enums, element structs)
│   ├── lui-ui/             # Component framework
│   └── lui-devtools/       # Devtools inspector
├── renderers/                    # GPU backend implementations
│   ├── lui-render-api/     # RenderBackend trait + RenderError
│   └── lui-renderer-wgpu/  # wgpu backend (quad, glyph, image pipelines)
├── drivers/                      # Platform integrations
│   ├── lui-driver/         # Backend-agnostic Driver + Runtime
│   ├── lui-driver-winit/   # winit window harness
│   ├── lui-driver-egui/    # egui integration
│   └── lui-driver-bevy/    # Bevy plugin
├── demo/                         # Demo applications
│   ├── lui-demo/
│   └── lui-demo-bevy/
├── spec/                         # Technical specifications
├── www/                          # Docusaurus documentation site
│   ├── docs/
│   └── docusaurus.config.js
└── AGENTS.md
```

## Key Source Files

### lui (facade)

| File | Purpose |
|---|---|
| `src/lib.rs` | Top-level orchestration: `paint_tree`, `classify_frame`, `PipelineCache` |
| `src/paint.rs` | DisplayList generation from LayoutBox tree |
| `src/interactivity.rs` | Mouse/keyboard dispatch wrappers, text selection |
| `src/date_picker_overlay.rs` | Date input overlay UI |

### lui-layout-old

| File | Purpose |
|---|---|
| `src/lib.rs` | Core layout: `layout_block`, IFC, positioned, overflow, incremental |
| `src/flex.rs` | CSS Flexbox Level 1 algorithm |
| `src/grid.rs` | CSS Grid Layout Level 1 algorithm |
| `src/length.rs` | CSS length resolution (px, %, em, vw, calc, etc.) |
| `src/table.rs` | Table layout (in progress) |

### lui-renderer-wgpu

| File | Purpose |
|---|---|
| `src/lib.rs` | `Renderer` (implements `RenderBackend`), wgpu surface setup |
| `src/quad_pipeline.rs` | SDF rounded quad pipeline |
| `src/glyph_pipeline.rs` | Glyph atlas text rendering |
| `src/image_pipeline.rs` | Textured image rendering |

## Building and Testing

```bash
cargo build --workspace
cargo test --workspace
cargo run -p lui-demo
```

Targeted test loops:

```bash
cargo test -p lui-layout-old
cargo test -p lui-parser
cargo test -p lui
```
