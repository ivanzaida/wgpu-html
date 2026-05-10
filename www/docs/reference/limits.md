---
sidebar_position: 5
---

# Limits

## Image Dimensions

| Limit | Value |
|---|---|
| Maximum decoded image size | 8192 × 8192 px |
| Exceeded images | Scaled down during decode |
| Texture atlas size | 2048 × 2048 px |
| Maximum gradient stops | No explicit limit |

## Memory

| Limit | Value |
|---|---|
| Image raw cache | TTL-based eviction, configured via `tree.asset_cache_ttl` |
| Image sized cache | Same TTL, byte-budget eviction |
| Instance buffers | Power-of-two growth, 1,048,576 max |
| Profiler ring buffer | 240 frames (~4s at 60fps) |

## Tree Limits

| Limit | Value |
|---|---|
| Maximum DOM depth | None (limited by stack space for recursion) |
| Maximum children per node | None (limited by Vec capacity) |
| Maximum inline stylesheets | None (limited by memory) |
| Maximum registered fonts | None (limited by memory) |

## Rendering Limits

| Limit | Value |
|---|---|
| Maximum quads per frame | Instance buffer capacity (~500K at max growth) |
| Maximum glyphs per frame | Instance buffer capacity |
| Maximum images per frame | Instance buffer capacity |
| Maximum nested clip depth | None (stack-based) |
| Viewport size | Platform-dependent (surface limits) |

## WebGPU/WASM (future)

When targeting wasm via WebGPU:
- Minimum buffer size: 256 bytes (alignment requirement)
- Buffer readback uses asynchronous mapping with polling
- Surface format may differ from native
- No filesystem access for font loading (must use embedded data or fetch)

## Performance Characteristics

| Operation | Complexity |
|---|---|
| Cascade (cold) | O(n × m) where n = nodes, m = applicable rules |
| Cascade (cached) | O(dirty_nodes) |
| Layout (block) | O(n) per subtree |
| Layout (flex) | O(items × iterations) |
| Layout (grid) | O(items × tracks) |
| Paint | O(n) |
| Hit testing | O(depth) |
| Display list | O(draw_commands) |
