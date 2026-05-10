---
sidebar_position: 8
---

# Contributing

## Getting Started

1. **Fork** the [repository](https://github.com/ivanzaida/wgpu-html)
2. **Clone** your fork: `git clone https://github.com/YOUR_USER/wgpu-html.git`
3. **Create a branch**: `git checkout -b feature/my-feature`
4. **Build**: `cargo build --workspace`
5. **Run tests**: `cargo test --workspace`

## Development Workflow

### Before Writing Code

- Read `AGENTS.md` for architecture and invariants
- Read `spec/` files for detailed technical specifications
- Check existing tests for patterns to follow
- Look at related features in the codebase

### Code Style

- Follow existing naming conventions (functions, structs, modules)
- Use `rustfmt` (`rustfmt.toml` at workspace root)
- Use `clippy` (`clippy.toml` at workspace root)
- No unnecessary comments — let code be self-documenting
- Prefer inline HTML/CSS in Rust unit tests over external fixtures

### Before Submitting

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

## Where to Add What

| Change | Primary Files |
|---|---|
| New HTML element | `models/src/html/`, `parser/src/attr_parser.rs`, `style/src/ua.css` |
| New CSS property | `models/src/css/style.rs`, `parser/src/style_props.rs` |
| New layout feature | `layout/src/lib.rs` (or dedicated module) |
| New paint feature | `wgpu-html/src/paint.rs` |
| New GPU feature | `renderer/src/` (pipelines) |
| New event type | `events/src/lib.rs`, `tree/src/dispatch.rs` |
| New API function | `wgpu-html/src/lib.rs` |

## Testing Guidelines

- **Layout tests** should assert on the three canonical rectangles: `margin_rect`, `border_rect`, `content_rect`
- **Paint tests** should inspect the generated `DisplayList` for expected draw commands
- **CSS cascade tests** should verify computed `Style` values on specific elements
- **Parser tests** should verify the tree structure and attribute parsing
- Neutralize UA defaults with `body { margin: 0; }` unless testing UA stylesheet behavior
- Demo HTML files under `demo/wgpu-html-demo/html/` are supplementary — add one for visual features

## Commit Style

- Conventional commits: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`
- Keep commits focused on one thing
- Reference the feature/file being changed

## Pull Request Process

1. Ensure all tests pass
2. Add or update tests for new behavior
3. Add a demo HTML page if the change is visual
4. Keep the diff focused — avoid mixing unrelated changes
5. The CI pipeline runs `cargo test --workspace` and `cargo clippy`

## Scope Boundaries

### In Scope
- HTML parsing improvements
- CSS property support
- Layout algorithm improvements
- GPU rendering features
- Interactivity and event handling
- New driver backends
- Performance optimizations

### Out of Scope (Permanently)
- **JavaScript execution** — no `<script>` evaluation, no JS engine, no `eval`
- Web platform APIs beyond rendering
- Tab management, history, navigation
- Accessibility tree
- Print layout
- SVG rendering (beyond current rasterized support)
