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
| 2 | Skip `build_box` for clean subtrees in incremental mode | incremental | done | 2 |
| 3 | Arena-allocate `LayoutBox` children (replace per-node `Vec`) | all | done | 3 |
| 4 | Pre-allocate rects `Vec` from previous frame's count | all | done | 4 |
| 5 | Cache text shaping results across layout passes | all | done | 5 |

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

## Run 2 — Skip build_box for clean subtrees

Date: 2026-05-13
Changes: `build_box_incremental` checks dirty set; clean nodes get a leaf LayoutBox (no children allocated). Cache clone synthesizes children from snapshots via stored node/style pointers.

### Block Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 50 stacked divs | 50 | 722 µs | 14,440 | +2.6% |
| 200 stacked divs | 200 | 2.90 ms | 14,500 | −5.8% |
| nested 4×3 | 120 | 848 µs | 7,067 | +3.5% |
| nested 3×8 | 585 | 5.99 ms | 10,239 | +1.4% |

### Flex Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| row 10 items | 10 | 230 µs | 23,000 | −0.4% |
| row 50 items | 50 | 1.23 ms | 24,600 | +0.0% |
| wrap 5×4 | 20 | 613 µs | 30,650 | +0.5% |
| wrap 10×8 | 80 | 2.48 ms | 31,000 | +1.2% |
| nested 3 deep | 40 | 800 µs | 20,000 | +1.4% |
| nested 4 deep | 120 | 3.12 ms | 26,000 | −1.9% |

### Grid Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 4×4 fixed | 16 | 220 µs | 13,750 | −0.5% |
| 10×6 fixed | 60 | 863 µs | 14,383 | −2.0% |
| auto 24 items | 24 | 356 µs | 14,833 | −0.8% |
| auto 100 items | 100 | 1.47 ms | 14,700 | +5.8% |

### Table Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 5×4 simple | 24 | 246 µs | 10,250 | +0.0% |
| 20×6 simple | 126 | 1.36 ms | 10,794 | +0.7% |
| 50×8 simple | 408 | 4.86 ms | 11,912 | −0.6% |
| 20×6 colspan | 106 | 1.21 ms | 11,415 | +0.8% |

### Inline Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 20 spans | 20 | 253 µs | 12,650 | +3.3% |
| 100 spans | 100 | 1.27 ms | 12,700 | +0.8% |
| 500 spans | 500 | 7.06 ms | 14,120 | −0.4% |

### Positioned Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| 20 absolute | 20 | 276 µs | 13,800 | +7.4% |
| 100 absolute | 100 | 1.39 ms | 13,900 | +5.3% |

### Mixed Layout

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| dashboard page | 40 | 528 µs | 13,200 | −2.4% |

### End-to-End (cascade + layout)

| Fixture | Nodes | Time | ns/node | vs Run 1 |
|---------|-------|------|---------|----------|
| cascade + layout | 40 | 958 µs | 23,950 | −0.4% |
| large mixed tree | 300 | 4.02 ms | 13,400 | −5.9% |

### Incremental Layout

| Fixture | Time | vs full | vs Run 1 |
|---------|------|---------|----------|
| full baseline (dashboard) | 531 µs | — | −2.0% |
| incremental 0 dirty | 522 µs | −1.7% | −3.9% |
| incremental 1 dirty leaf | 62.5 µs | −88.2% | **−84.0%** |
| incremental 1 dirty near root | 582 µs | +9.6% | +0.5% |
| large full baseline | 13.07 ms | — | −0.4% |
| large incremental 1 leaf | 425 µs | −96.7% | **−18.7%** |

> **Summary:** Full layout within noise (opt doesn't affect full path). Incremental 1-dirty-leaf: **62.5 µs** (was 391 µs, −84%). Large incremental 1-leaf: **425 µs** (was 523 µs, −19%). Skipping box generation for clean subtrees eliminates allocation overhead proportional to tree size.

---

## Run 3 — Arena-allocate LayoutBox children (bumpalo)

Date: 2026-05-13
Changes: `LayoutBox.children` changed from `Vec<LayoutBox>` to `bumpalo::collections::Vec<LayoutBox>`. Arena owned by `LayoutTree` (via `ManuallyDrop` for correct drop order). `&Bump` threaded through all layout functions.

### Block Layout

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| 50 stacked divs | 50 | 687 µs | 13,740 | −4.8% |
| 200 stacked divs | 200 | 2.98 ms | 14,900 | +2.8% |
| nested 4×3 | 120 | 813 µs | 6,775 | −4.1% |
| nested 3×8 | 585 | 5.96 ms | 10,188 | −0.5% |

### Flex Layout

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| row 10 items | 10 | 229 µs | 22,900 | −0.4% |
| row 50 items | 50 | 1.26 ms | 25,200 | +2.4% |
| wrap 5×4 | 20 | 614 µs | 30,700 | +0.2% |
| wrap 10×8 | 80 | 2.52 ms | 31,500 | +1.6% |
| nested 3 deep | 40 | 769 µs | 19,225 | −3.9% |
| nested 4 deep | 120 | 3.10 ms | 25,833 | −0.6% |

### Grid Layout

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| 4×4 fixed | 16 | 231 µs | 14,438 | +5.0% |
| 10×6 fixed | 60 | 861 µs | 14,350 | −0.2% |
| auto 24 items | 24 | 350 µs | 14,583 | −1.7% |
| auto 100 items | 100 | 1.46 ms | 14,600 | −0.7% |

### Table Layout

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| 5×4 simple | 24 | 237 µs | 9,875 | −3.7% |
| 20×6 simple | 126 | 1.36 ms | 10,794 | +0.0% |
| 50×8 simple | 408 | 4.62 ms | 11,324 | −4.9% |
| 20×6 colspan | 106 | 1.20 ms | 11,321 | −0.8% |

### Inline Layout

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| 20 spans | 20 | 250 µs | 12,500 | −1.2% |
| 100 spans | 100 | 1.24 ms | 12,400 | −2.4% |
| 500 spans | 500 | 6.76 ms | 13,520 | −4.3% |

### Positioned Layout

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| 20 absolute | 20 | 272 µs | 13,600 | −1.4% |
| 100 absolute | 100 | 1.36 ms | 13,600 | −2.2% |

### Mixed Layout

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| dashboard page | 40 | 515 µs | 12,875 | −2.5% |

### End-to-End (cascade + layout)

| Fixture | Nodes | Time | ns/node | vs Run 2 |
|---------|-------|------|---------|----------|
| cascade + layout | 40 | 927 µs | 23,175 | −3.2% |
| large mixed tree | 300 | 3.92 ms | 13,067 | −2.5% |

### Incremental Layout

| Fixture | Time | vs full | vs Run 2 |
|---------|------|---------|----------|
| full baseline (dashboard) | 514 µs | — | −3.2% |
| incremental 0 dirty | 504 µs | −1.9% | −3.4% |
| incremental 1 dirty leaf | 59.3 µs | −88.5% | −5.1% |
| incremental 1 dirty near root | 557 µs | +8.4% | −4.3% |
| large full baseline | 12.71 ms | — | −2.8% |
| large incremental 1 leaf | 382 µs | −97.0% | **−10.1%** |

> **Summary:** Modest but consistent improvement across the board. Largest gains on big trees: 50×8 table −4.9%, 500 spans −4.3%, large mixed tree −2.5%, large incremental −10.1%. Arena eliminates per-node malloc/free overhead; deallocation is a single Bump reset instead of hundreds of free() calls.

---

## Run 4 — Pre-allocate rects Vec

Date: 2026-05-13
Changes: `LayoutCache` stores previous frame's rects count. `LayoutEngine::layout()` and incremental paths use `Vec::with_capacity(prev_count)` to avoid rects Vec reallocations.

### Block Layout

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| 50 stacked divs | 50 | 657 µs | 13,140 | −4.4% |
| 200 stacked divs | 200 | 2.78 ms | 13,900 | −6.7% |
| nested 4×3 | 120 | 799 µs | 6,658 | −1.7% |
| nested 3×8 | 585 | 5.91 ms | 10,103 | −0.9% |

### Flex Layout

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| row 10 items | 10 | 233 µs | 23,300 | +1.7% |
| row 50 items | 50 | 1.20 ms | 24,000 | −4.8% |
| wrap 5×4 | 20 | 616 µs | 30,800 | +0.3% |
| wrap 10×8 | 80 | 2.52 ms | 31,500 | +0.0% |
| nested 3 deep | 40 | 754 µs | 18,850 | −2.0% |
| nested 4 deep | 120 | 3.10 ms | 25,833 | +0.0% |

### Grid Layout

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| 4×4 fixed | 16 | 232 µs | 14,500 | +0.4% |
| 10×6 fixed | 60 | 858 µs | 14,300 | −0.3% |
| auto 24 items | 24 | 345 µs | 14,375 | −1.5% |
| auto 100 items | 100 | 1.47 ms | 14,700 | +0.7% |

### Table Layout

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| 5×4 simple | 24 | 224 µs | 9,333 | −5.5% |
| 20×6 simple | 126 | 1.23 ms | 9,762 | −9.4% |
| 50×8 simple | 408 | 4.25 ms | 10,417 | −8.0% |
| 20×6 colspan | 106 | 1.14 ms | 10,755 | −5.0% |

### Inline Layout

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| 20 spans | 20 | 247 µs | 12,350 | −1.2% |
| 100 spans | 100 | 1.22 ms | 12,200 | −1.6% |
| 500 spans | 500 | 6.45 ms | 12,900 | −4.6% |

### Positioned Layout

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| 20 absolute | 20 | 253 µs | 12,650 | −7.0% |
| 100 absolute | 100 | 1.37 ms | 13,700 | +0.7% |

### Mixed Layout

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| dashboard page | 40 | 528 µs | 13,200 | +2.5% |

### End-to-End (cascade + layout)

| Fixture | Nodes | Time | ns/node | vs Run 3 |
|---------|-------|------|---------|----------|
| cascade + layout | 40 | 932 µs | 23,300 | +0.5% |
| large mixed tree | 300 | 3.72 ms | 12,400 | −5.2% |

### Incremental Layout

| Fixture | Time | vs full | vs Run 3 |
|---------|------|---------|----------|
| full baseline (dashboard) | 490 µs | — | −4.7% |
| incremental 0 dirty | 531 µs | +8.4% | +5.4% |
| incremental 1 dirty leaf | 61.5 µs | −87.4% | +3.7% |
| incremental 1 dirty near root | 543 µs | +10.8% | −2.5% |
| large full baseline | 11.92 ms | — | **−6.2%** |
| large incremental 1 leaf | 381 µs | −96.8% | −0.3% |

> **Summary:** Biggest wins on large trees where rects Vec had many reallocations: large mixed −5.2%, large full baseline −6.2%, 50×8 table −8.0%, 20×6 table −9.4%. Small trees within noise (already few rects). Free functions (first call) unaffected since they start with capacity 0.

---

## Run 5 — Cache text shaping across layout passes

Date: 2026-05-13
Changes: Added `TextContext::shape()` and `TextContext::break_into_lines()` with HashMap cache. Layout now calls these instead of `font_ctx.shape()` directly. Repeat layout passes hit cache for all text nodes — cosmic-text `shape_until_scroll` bypassed entirely.

### Block Layout

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| 50 stacked divs | 50 | 40.0 µs | 800 | **−93.9%** |
| 200 stacked divs | 200 | 160 µs | 800 | **−94.2%** |
| nested 4×3 | 120 | 81.7 µs | 681 | **−89.8%** |
| nested 3×8 | 585 | 932 µs | 1,593 | **−84.2%** |

### Flex Layout

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| row 10 items | 10 | 13.1 µs | 1,310 | **−94.4%** |
| row 50 items | 50 | 63.1 µs | 1,262 | **−94.7%** |
| wrap 5×4 | 20 | 28.7 µs | 1,435 | **−95.3%** |
| wrap 10×8 | 80 | 114 µs | 1,425 | **−95.5%** |
| nested 3 deep | 40 | 54.2 µs | 1,355 | **−92.8%** |
| nested 4 deep | 120 | 204 µs | 1,700 | **−93.4%** |

### Grid Layout

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| 4×4 fixed | 16 | 15.1 µs | 944 | **−93.5%** |
| 10×6 fixed | 60 | 59.6 µs | 993 | **−93.1%** |
| auto 24 items | 24 | 21.3 µs | 888 | **−93.8%** |
| auto 100 items | 100 | 89.9 µs | 899 | **−93.9%** |

### Table Layout

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| 5×4 simple | 24 | 20.0 µs | 833 | **−91.1%** |
| 20×6 simple | 126 | 105 µs | 833 | **−91.5%** |
| 50×8 simple | 408 | 354 µs | 868 | **−91.7%** |
| 20×6 colspan | 106 | 95.4 µs | 900 | **−91.6%** |

### Inline Layout

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| 20 spans | 20 | 16.7 µs | 835 | **−93.2%** |
| 100 spans | 100 | 79.2 µs | 792 | **−93.5%** |
| 500 spans | 500 | 546 µs | 1,092 | **−91.5%** |

### Positioned Layout

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| 20 absolute | 20 | 16.7 µs | 835 | **−93.4%** |
| 100 absolute | 100 | 76.9 µs | 769 | **−94.4%** |

### Mixed Layout

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| dashboard page | 40 | 36.9 µs | 923 | **−93.0%** |

### End-to-End (cascade + layout)

| Fixture | Nodes | Time | ns/node | vs Run 4 |
|---------|-------|------|---------|----------|
| cascade + layout | 40 | 437 µs | 10,925 | **−53.1%** |
| large mixed tree | 300 | 1.63 ms | 5,433 | **−56.2%** |

### Incremental Layout

| Fixture | Time | vs full | vs Run 4 |
|---------|------|---------|----------|
| full baseline (dashboard) | 35.4 µs | — | **−92.8%** |
| incremental 0 dirty | 34.1 µs | −3.7% | **−93.6%** |
| incremental 1 dirty leaf | 36.6 µs | +3.4% | **−40.5%** |
| incremental 1 dirty near root | 76.5 µs | +116% | **−85.9%** |
| large full baseline | 10.85 ms | — | **−9.0%** |
| large incremental 1 leaf | 364 µs | −96.6% | −4.5% |

> **Summary:** Text shaping cache delivers **~93% improvement** across all layout benchmarks on the second+ call. `shape()` and `break_into_lines()` now hit a HashMap cache, completely bypassing cosmic-text's Buffer/shaping pipeline. First call still pays full cost; subsequent calls with same text+style are essentially free. End-to-end (cascade+layout) shows −53% since cascade isn't cached here. Large tree full baseline −9% (first call benefits from cross-node cache hits for identical text).
