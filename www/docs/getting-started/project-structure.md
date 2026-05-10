---
sidebar_position: 5
---

# Project Structure

## Workspace Layout

```
lui/
├── crates/                    # All Rust crates
│   ├── lui/             # Façade: orchestration, paint, interactivity
│   ├── lui-parser/      # HTML tokenizer, CSS parser, stylesheet parser
│   ├── lui-style/       # CSS cascade, selector matching, inheritance
│   ├── lui-layout/      # Block, flex, grid, IFC, positioned layout
│   ├── lui-text/        # Font DB, cosmic-text shaping, glyph atlas
│   ├── lui-renderer/    # wgpu pipelines (quad, glyph, image)
│   ├── lui-tree/        # DOM tree, events, focus, query selectors
│   ├── lui-events/      # Typed event structs
│   ├── lui-models/      # Shared types (Style, enums, element structs)
│   ├── lui-winit/       # winit window harness
│   ├── lui-ui/          # Component framework
│   ├── lui-egui/        # egui integration backend
│   └── lui-devtools/    # Devtools inspector
├── demo/
│   └── lui-demo/        # Demo application
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

### lui (facade)

| File | Purpose |
|---|---|
| `src/lib.rs` | Top-level orchestration: `paint_tree`, `classify_frame`, `PipelineCache` |
| `src/paint.rs` | DisplayList generation from LayoutBox tree |
| `src/interactivity.rs` | Mouse/keyboard dispatch wrappers, text selection |
| `src/date_picker_overlay.rs` | Date input overlay UI |

### lui-layout

| File | Purpose |
|---|---|
| `src/lib.rs` | Core layout: `layout_block`, IFC, positioned, overflow, incremental |
| `src/flex.rs` | CSS Flexbox Level 1 algorithm |
| `src/grid.rs` | CSS Grid Layout Level 1 algorithm |
| `src/length.rs` | CSS length resolution (px, %, em, vw, calc, etc.) |
| `src/table.rs` | Table layout (in progress) |

### lui-renderer

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
cargo run -p lui-demo
```

Targeted test loops:

```bash
cargo test -p lui-layout
cargo test -p lui-parser
cargo test -p lui
```
