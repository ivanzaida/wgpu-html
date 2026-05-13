# Layout Benchmarks

## Setup

| Field | Value |
|-------|-------|
| CPU | `(fill in)` |
| OS | `(fill in)` |
| Rust | `(fill in)` |
| Command | `cargo bench --manifest-path crates/lui-layout/Cargo.toml` |

Stylesheets: UA (WHATWG ~605 rules) + reset (`* { margin:0; padding:0; border-width:0 }`)

### Optimization Backlog

| # | Optimization | Affects | Status | Run |
|---|-------------|---------|--------|-----|
| 0 | Reuse `TextContext` across frames (`LayoutEngine`) | all | done | 1 |
| 1 | Eliminate `LayoutCache` HashMap clone on incremental path | incremental | done | 1 |
| 2 | Skip `build_box` for clean subtrees in incremental mode | incremental | pending | — |
| 3 | Arena-allocate `LayoutBox` children (replace per-node `Vec`) | all | pending | — |
| 4 | Pre-allocate rects `Vec` from previous frame's count | all | pending | — |
| 5 | Cache text shaping results across layout passes | inline | pending | — |

### Fixtures

#### Block

| Fixture | Description | Nodes |
|---------|-------------|-------|
| 50 stacked divs | Flat list, each with height/width/padding/margin | ~50 |
| 200 stacked divs | Same pattern, 4× | ~200 |
| nested 4×3 | 4 levels deep, 3 children each | ~120 |
| nested 3×8 | 3 levels deep, 8 children each | ~585 |

#### Flex

| Fixture | Description | Nodes |
|---------|-------------|-------|
| row 10 items | Single flex row, `flex:1`, gap | 10 |
| row 50 items | Same, 50 items | 50 |
| wrap 5×4 | `flex-wrap:wrap`, 20 items | 20 |
| wrap 10×8 | Same, 80 items | 80 |
| nested 3 deep | Alternating row/column flex, 3 levels × 3 items | ~40 |
| nested 4 deep | Same, 4 levels | ~120 |

#### Grid

| Fixture | Description | Nodes |
|---------|-------------|-------|
| 4×4 fixed | `1fr` columns, 16 cells | 16 |
| 10×6 fixed | Same, 60 cells | 60 |
| auto 24 items | `repeat(4, 1fr)`, auto-placement | 24 |
| auto 100 items | Same, 100 items | 100 |

#### Table

| Fixture | Description | Nodes |
|---------|-------------|-------|
| 5×4 simple | thead + tbody, border-spacing | ~24 |
| 20×6 simple | Same pattern, 120 cells | ~126 |
| 50×8 simple | 400 cells | ~408 |
| 20×6 colspan | Every 3rd row has `colspan=2` | ~106 |

#### Inline

| Fixture | Description | Nodes |
|---------|-------------|-------|
| 20 spans | `<span>` words in 400px container | 20 |
| 100 spans | Same, line breaking | 100 |
| 500 spans | Heavy text shaping + wrapping | 500 |

#### Positioned

| Fixture | Description | Nodes |
|---------|-------------|-------|
| 20 absolute | `position:absolute` children in relative parent | 20 |
| 100 absolute | Same, 100 elements | 100 |

#### Mixed / End-to-end

| Fixture | Description | Nodes |
|---------|-------------|-------|
| dashboard page | Flex sidebar + grid cards (9) + table (3×3) | ~40 |
| cascade + layout | Dashboard with full cascade | ~40 |
| large mixed tree | Nested blocks (3×5) + nested flex (3×3) + grid (6×4) | ~300 |

---

## Run 0 — Broken baseline (TextContext recreated per call)

Date: 2026-05-13

> ⚠ Every call to `layout_tree()` created a new `TextContext`, which calls `FontSystem::new()` (~9.5 ms scanning system fonts). Numbers below are dominated by this fixed overhead and do not reflect actual layout performance.

### Block Layout

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| 50 stacked divs | 50 | 11.05 ms | 221,000 |
| 200 stacked divs | 200 | 13.39 ms | 66,950 |
| nested 4×3 | 120 | 11.14 ms | 92,833 |
| nested 3×8 | 585 | 16.40 ms | 28,034 |

### Flex Layout

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| row 10 items | 10 | 10.33 ms | 1,033,000 |
| row 50 items | 50 | 10.94 ms | 218,800 |
| wrap 5×4 | 20 | 10.16 ms | 508,000 |
| wrap 10×8 | 80 | 12.24 ms | 153,000 |
| nested 3 deep | 40 | 10.30 ms | 257,500 |
| nested 4 deep | 120 | 12.86 ms | 107,167 |

### Grid Layout

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| 4×4 fixed | 16 | 9.86 ms | 616,250 |
| 10×6 fixed | 60 | 10.64 ms | 177,333 |
| auto 24 items | 24 | 9.76 ms | 406,667 |
| auto 100 items | 100 | 11.48 ms | 114,800 |

### Table Layout

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| 5×4 simple | 24 | 9.83 ms | 409,583 |
| 20×6 simple | 126 | 10.64 ms | 84,444 |
| 50×8 simple | 408 | 14.09 ms | 34,534 |
| 20×6 colspan | 106 | 11.03 ms | 104,057 |

### Inline Layout

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| 20 spans | 20 | 9.98 ms | 499,000 |
| 100 spans | 100 | 11.05 ms | 110,500 |
| 500 spans | 500 | 16.87 ms | 33,740 |

### Positioned Layout

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| 20 absolute | 20 | 9.93 ms | 496,500 |
| 100 absolute | 100 | 11.30 ms | 113,000 |

### Mixed Layout

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| dashboard page | 40 | 10.31 ms | 257,750 |

### End-to-End (cascade + layout)

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| cascade + layout | 40 | 10.85 ms | 271,250 |
| large mixed tree | 300 | 13.64 ms | 45,467 |

### Incremental Layout

| Fixture | Time | vs full |
|---------|------|---------|
| full baseline (dashboard) | 10.03 ms | — |
| incremental 0 dirty | 10.76 ms | +7.3% |
| incremental 1 dirty leaf | 9.78 ms | −2.5% |
| incremental 1 dirty near root | 9.45 ms | −5.8% |
| large full baseline | 11.52 ms | — |
| large incremental 1 leaf | 10.17 ms | −11.7% |

> Incremental gains invisible — font-init overhead (~9.5 ms) dominates every measurement.

---

## Run 1 — Baseline (LayoutEngine + shared TextContext + no-clone cache)

Date: 2026-05-13
Changes: `LayoutEngine` struct owns `TextContext`, reused across calls. `LayoutCache` split into immutable snapshot + `DirtySet` (no HashMap clone on incremental path).

### Block Layout

| Fixture | Nodes | Time | ns/node | vs Run 0 |
|---------|-------|------|---------|----------|
| 50 stacked divs | 50 | 704 µs | 14,080 | −93.6% |
| 200 stacked divs | 200 | 3.08 ms | 15,400 | −77.0% |
| nested 4×3 | 120 | 820 µs | 6,831 | −92.6% |
| nested 3×8 | 585 | 5.91 ms | 10,103 | −64.0% |

### Flex Layout

| Fixture | Nodes | Time | ns/node | vs Run 0 |
|---------|-------|------|---------|----------|
| row 10 items | 10 | 231 µs | 23,100 | −97.8% |
| row 50 items | 50 | 1.23 ms | 24,600 | −88.8% |
| wrap 5×4 | 20 | 610 µs | 30,500 | −94.0% |
| wrap 10×8 | 80 | 2.45 ms | 30,625 | −80.0% |
| nested 3 deep | 40 | 789 µs | 19,725 | −92.3% |
| nested 4 deep | 120 | 3.18 ms | 26,500 | −75.3% |

### Grid Layout

| Fixture | Nodes | Time | ns/node | vs Run 0 |
|---------|-------|------|---------|----------|
| 4×4 fixed | 16 | 221 µs | 13,813 | −97.8% |
| 10×6 fixed | 60 | 881 µs | 14,683 | −91.7% |
| auto 24 items | 24 | 359 µs | 14,958 | −96.3% |
| auto 100 items | 100 | 1.39 ms | 13,900 | −87.9% |

### Table Layout

| Fixture | Nodes | Time | ns/node | vs Run 0 |
|---------|-------|------|---------|----------|
| 5×4 simple | 24 | 246 µs | 10,250 | −97.5% |
| 20×6 simple | 126 | 1.35 ms | 10,714 | −87.3% |
| 50×8 simple | 408 | 4.89 ms | 11,985 | −65.3% |
| 20×6 colspan | 106 | 1.20 ms | 11,321 | −89.1% |

### Inline Layout

| Fixture | Nodes | Time | ns/node | vs Run 0 |
|---------|-------|------|---------|----------|
| 20 spans | 20 | 245 µs | 12,250 | −97.5% |
| 100 spans | 100 | 1.26 ms | 12,600 | −88.6% |
| 500 spans | 500 | 7.09 ms | 14,180 | −58.0% |

### Positioned Layout

| Fixture | Nodes | Time | ns/node | vs Run 0 |
|---------|-------|------|---------|----------|
| 20 absolute | 20 | 257 µs | 12,850 | −97.4% |
| 100 absolute | 100 | 1.32 ms | 13,200 | −88.3% |

### Mixed Layout

| Fixture | Nodes | Time | ns/node | vs Run 0 |
|---------|-------|------|---------|----------|
| dashboard page | 40 | 541 µs | 13,525 | −94.8% |

### End-to-End (cascade + layout)

| Fixture | Nodes | Time | ns/node |
|---------|-------|------|---------|
| cascade + layout | 40 | 962 µs | 24,050 |
| large mixed tree | 300 | 4.27 ms | 14,233 |

### Incremental Layout

| Fixture | Time | vs full | vs Run 0 |
|---------|------|---------|----------|
| full baseline (dashboard) | 542 µs | — | −94.6% |
| incremental 0 dirty | 543 µs | +0.2% | −95.0% |
| incremental 1 dirty leaf | 391 µs | −27.9% | −96.0% |
| incremental 1 dirty near root | 579 µs | +6.8% | −93.9% |
| large full baseline | 13.12 ms | — | n/a |
| large incremental 1 leaf | 523 µs | −96.0% | −94.9% |

---

## Run 2 — `(pending)`

Date: `(pending)`
Changes: `(describe what changed)`

### Block Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 50 stacked divs | 50 | — | — | — |
| 200 stacked divs | 200 | — | — | — |
| nested 4×3 | 120 | — | — | — |
| nested 3×8 | 585 | — | — | — |

### Flex Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| row 10 items | 10 | — | — | — |
| row 50 items | 50 | — | — | — |
| wrap 5×4 | 20 | — | — | — |
| wrap 10×8 | 80 | — | — | — |
| nested 3 deep | 40 | — | — | — |
| nested 4 deep | 120 | — | — | — |

### Grid Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 4×4 fixed | 16 | — | — | — |
| 10×6 fixed | 60 | — | — | — |
| auto 24 items | 24 | — | — | — |
| auto 100 items | 100 | — | — | — |

### Table Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 5×4 simple | 24 | — | — | — |
| 20×6 simple | 126 | — | — | — |
| 50×8 simple | 408 | — | — | — |
| 20×6 colspan | 106 | — | — | — |

### Inline Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 20 spans | 20 | — | — | — |
| 100 spans | 100 | — | — | — |
| 500 spans | 500 | — | — | — |

### Positioned Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 20 absolute | 20 | — | — | — |
| 100 absolute | 100 | — | — | — |

### Mixed Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| dashboard page | 40 | — | — | — |

### End-to-End (cascade + layout)

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| cascade + layout | 40 | — | — | — |
| large mixed tree | 300 | — | — | — |

### Incremental Layout

| Fixture | Time | vs full | vs Run 1 |
|---------|------|---------|----------|
| full baseline (dashboard) | — | — | — |
| incremental 0 dirty | — | — | — |
| incremental 1 dirty leaf | — | — | — |
| incremental 1 dirty near root | — | — | — |
| large full baseline | — | — | — |
| large incremental 1 leaf | — | — | — |
