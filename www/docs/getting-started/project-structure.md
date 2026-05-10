---
sidebar_position: 5
---

# Project Structure

## Workspace Layout

```
wgpu-html/
├── crates/                    # All Rust crates
│   ├── wgpu-html/             # Façade: orchestration, paint, interactivity
│   ├── wgpu-html-parser/      # HTML tokenizer, CSS parser, stylesheet parser
│   ├── wgpu-html-style/       # CSS cascade, selector matching, inheritance
│   ├── wgpu-html-layout/      # Block, flex, grid, IFC, positioned layout
│   ├── wgpu-html-text/        # Font DB, cosmic-text shaping, glyph atlas
│   ├── wgpu-html-renderer/    # wgpu pipelines (quad, glyph, image)
│   ├── wgpu-html-tree/        # DOM tree, events, focus, query selectors
│   ├── wgpu-html-events/      # Typed event structs
│   ├── wgpu-html-models/      # Shared types (Style, enums, element structs)
│   ├── wgpu-html-winit/       # winit window harness
│   ├── wgpu-html-ui/          # Component framework
│   ├── wgpu-html-egui/        # egui integration backend
│   └── wgpu-html-devtools/    # Devtools inspector
├── demo/
│   └── wgpu-html-demo/        # Demo application
│       ├── html/              # Example HTML pages
│       └── src/               # Demo source
├── spec/                      # Technical specifications
├── tests/                     # Integration tests
├── www/                       # Docusaurus documentation site
│   ├── docs/                  # Documentation pages
│   ├── docusaurus.config.js   # Site configuration
│   └── sidebars.js            # Sidebar structure
└── AGENTS.md                  # Contributor guide
```

## Key Source Files

### wgpu-html (facade)

| File | Purpose |
|---|---|
| `src/lib.rs` | Top-level orchestration: `paint_tree`, `classify_frame`, `PipelineCache` |
| `src/paint.rs` | DisplayList generation from LayoutBox tree |
| `src/interactivity.rs` | Mouse/keyboard dispatch wrappers, text selection |
| `src/date_picker_overlay.rs` | Date input overlay UI |

### wgpu-html-layout

| File | Purpose |
|---|---|
| `src/lib.rs` | Core layout: `layout_block`, IFC, positioned, overflow, incremental |
| `src/flex.rs` | CSS Flexbox Level 1 algorithm |
| `src/grid.rs` | CSS Grid Layout Level 1 algorithm |
| `src/length.rs` | CSS length resolution (px, %, em, vw, calc, etc.) |
| `src/table.rs` | Table layout (in progress) |

### wgpu-html-renderer

| File | Purpose |
|---|---|
| `src/lib.rs` | `Renderer`, `render()`, wgpu surface setup |
| `src/quad_pipeline.rs` | SDF rounded quad pipeline |
| `src/glyph_pipeline.rs` | Glyph atlas text rendering |
| `src/image_pipeline.rs` | Textured image rendering |

## Building and Testing

```bash
cargo build --workspace
cargo test --workspace
cargo run -p wgpu-html-demo
```

Targeted test loops:

```bash
cargo test -p wgpu-html-layout
cargo test -p wgpu-html-parser
cargo test -p wgpu-html
```
