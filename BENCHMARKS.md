# Cascade Benchmarks

## Setup

| Field | Value |
|-------|-------|
| CPU | `(fill in)` |
| OS | `(fill in)` |
| Rust | `(fill in)` |
| Command | `cargo bench --manifest-path crates/lui-cascade/Cargo.toml` |

Tree sizes:
- **Small**: 3 levels × 3 children = ~40 nodes
- **Medium**: 4 levels × 4 children = ~340 nodes
- **Large**: 5 levels × 4 children = ~1,365 nodes

Stylesheets: UA (WHATWG ~605 rules) + author (50 rules: `.c0-.c9` with color/padding/margin + div/span/.d1-.d3 overrides)

### Incremental dirty scenarios (large tree, 1,365 nodes)

| Scenario | Dirty path | Nodes affected | Strategy |
|----------|------------|----------------|----------|
| 0 dirty | `[]` | 0 | Full `clone_subtree` from previous arena |
| 1 dirty leaf | `[0,0,0,0,0]` | ~6 (leaf + 5 ancestors) | Re-cascade one deep path |
| 1 dirty subtree | `[0,0]` | ~853 | Re-cascade 1 of 4 body branches (25% of tree), clone the other 3

---

## Run 1 — Baseline

Date: 2026-05-12

### Full Cascade

| Tree | Nodes | Time | ns/node |
|------|-------|------|---------|
| Small | 40 | 0.97 ms | 24,148 |
| Medium | 340 | 7.83 ms | 23,039 |
| Large | 1,365 | 21.29 ms | 15,595 |

### Incremental Cascade (large tree)

| Scenario | Time |
|----------|------|
| Two full cascades | 42.99 ms |
| Full + incremental, 0 dirty | 22.98 ms |
| Full + incremental, 1 dirty leaf | 22.00 ms |
| Full + incremental, 1 dirty subtree | 24.57 ms |

### Set Stylesheets

| Rules | Time |
|-------|------|
| 10 | 7.68 µs |
| 500 | 247.16 µs |

### Selector Matching

| Query | Time |
|-------|------|
| `.item` | 984 ns |
| `.container > .list .item.active` | 1.26 µs |

---

## Run 2 — FxHash

Date: 2026-05-12
Changes: `std::collections::HashMap` → `rustc_hash::FxHashMap`, `DefaultHasher` → `FxHasher` in: `DeclCache`, `RuleIndex`, `element_cache_key`, `bloom_hash`, `ComputedStyle.extra/custom_properties`, `candidate_rules::seen`, `var_resolution`

### Full Cascade

| Tree | Nodes | Time | ns/node | vs baseline |
|------|-------|------|---------|-------------|
| Small | 40 | 0.91 ms | 22,771 | −5.7% |
| Medium | 340 | 8.58 ms | 25,223 | +9.5% |
| Large | 1,365 | 26.10 ms | 19,124 | +22.6% |

### Incremental Cascade (large tree)

| Scenario | Time | vs baseline |
|----------|------|-------------|
| Two full cascades | 55.21 ms | +28.4% |
| Full + incremental, 0 dirty | 29.07 ms | +26.5% |
| Full + incremental, 1 dirty leaf | 30.09 ms | +36.8% |
| Full + incremental, 1 dirty subtree | 32.24 ms | +31.2% |

### Set Stylesheets

| Rules | Time | vs baseline |
|-------|------|-------------|
| 10 | 9.77 µs | +16.7% |
| 500 | 334.81 µs | +33.3% |

### Selector Matching

| Query | Time | vs baseline |
|-------|------|-------------|
| `.item` | 1.24 µs | +26.4% |
| `.container > .list .item.active` | 1.82 µs | +43.8% |

> ⚠ **Note:** `query_selector` benchmarks have zero code changes yet show +26–44% regression, confirming high run-to-run variance on this machine. The small tree improved by 5.7%. The overall regression magnitude is inflated by variance; actual FxHash impact needs investigation with controlled runs.

---

## Run 3 — SmallVec

Date: `(pending)`

### Full Cascade

| Tree | Nodes | Time | ns/node | vs baseline |
|------|-------|------|---------|-------------|
| Small | 40 | — | — | — |
| Medium | 340 | — | — | — |
| Large | 1,365 | — | — | — |

### Incremental Cascade (large tree)

| Scenario | Time | vs baseline |
|----------|------|-------------|
| Two full cascades | — | — |
| Full + incremental, 0 dirty | — | — |
| Full + incremental, 1 dirty leaf | — | — |
| Full + incremental, 1 dirty subtree | — | — |

### Set Stylesheets

| Rules | Time | vs baseline |
|-------|------|-------------|
| 10 | — | — |
| 500 | — | — |

### Selector Matching

| Query | Time | vs baseline |
|-------|------|-------------|
| `.item` | — | — |
| `.container > .list .item.active` | — | — |

---

## Run 4 — Rule-level bloom prefilter

Date: `(pending)`

### Full Cascade

| Tree | Nodes | Time | ns/node | vs baseline |
|------|-------|------|---------|-------------|
| Small | 40 | — | — | — |
| Medium | 340 | — | — | — |
| Large | 1,365 | — | — | — |

### Incremental Cascade (large tree)

| Scenario | Time | vs baseline |
|----------|------|-------------|
| Two full cascades | — | — |
| Full + incremental, 0 dirty | — | — |
| Full + incremental, 1 dirty leaf | — | — |
| Full + incremental, 1 dirty subtree | — | — |

### Set Stylesheets

| Rules | Time | vs baseline |
|-------|------|-------------|
| 10 | — | — |
| 500 | — | — |

### Selector Matching

| Query | Time | vs baseline |
|-------|------|-------------|
| `.item` | — | — |
| `.container > .list .item.active` | — | — |

---

## Run 5 — Rayon parallel siblings

Date: `(pending)`

### Full Cascade

| Tree | Nodes | Time | ns/node | vs baseline |
|------|-------|------|---------|-------------|
| Small | 40 | — | — | — |
| Medium | 340 | — | — | — |
| Large | 1,365 | — | — | — |

### Incremental Cascade (large tree)

| Scenario | Time | vs baseline |
|----------|------|-------------|
| Two full cascades | — | — |
| Full + incremental, 0 dirty | — | — |
| Full + incremental, 1 dirty leaf | — | — |
| Full + incremental, 1 dirty subtree | — | — |

### Set Stylesheets

| Rules | Time | vs baseline |
|-------|------|-------------|
| 10 | — | — |
| 500 | — | — |

### Selector Matching

| Query | Time | vs baseline |
|-------|------|-------------|
| `.item` | — | — |
| `.container > .list .item.active` | — | — |
