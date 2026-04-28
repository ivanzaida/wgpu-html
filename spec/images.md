# wgpu-html — Image Support Spec

How `<img>` elements load, decode, cache, and render. Companion to
`roadmap.md` (M-img) and `status.md`.

Status: shipped. `<img>` decodes through the `image` crate, paints
via the renderer's image pipeline (textured quads with optional
rounded clip), and supports both local-filesystem `src` and
remote `http(s)` URLs. Remote loads run on a background thread;
the layout pass never blocks on network I/O. A two-level
process-wide cache plus TTL-based eviction keeps memory bounded
across long sessions.

---

## 1. Goals

- Render `<img>` as a textured quad sized by intrinsic image
  dimensions, HTML `width` / `height` attributes, or CSS
  `width` / `height` (in that fallback order).
- Accept `src` values that are filesystem paths *or* `http(s)`
  URLs, with the same engine code on both sides.
- Never block layout / paint on network fetches: a slow or
  unreachable host must not stall the UI thread.
- Decode each unique `src` exactly once, even when the same URL
  is rendered at multiple declared sizes.
- Reclaim memory for images that haven't been used recently, with
  a configurable TTL.

## 2. Supported formats

The decoder is the `image` crate, configured (`Cargo.toml`) with
`default-features = false` plus an explicit feature list:

| Format | Extension(s) | Notes                              |
|--------|--------------|------------------------------------|
| PNG    | `.png`       | 8-bit RGBA — preferred            |
| JPEG   | `.jpg/.jpeg` | Baseline + progressive             |
| GIF    | `.gif`       | First frame only (no animation)    |
| BMP    | `.bmp`       | Uncompressed                       |
| WebP   | `.webp`      | Lossy + lossless still images only |

Formats not in the feature list (TIFF, AVIF, HDR, animated GIF,
animated WebP, SVG, …) are not decoded; they fail to load and
show as a missing image (`load_image` returns `None`). Add the
matching `image` crate feature to enable more.

## 3. Schemes

`load_image_url` (in `crates/wgpu-html-layout/src/lib.rs`) dispatches
on the `src` prefix:

- **`http://` / `https://`** — fetched with `ureq` (rustls TLS).
  Response bodies are capped at 32 MiB to bound memory.
- **Anything else** — treated as a local filesystem path and read
  with `std::fs::read`.

In both cases the actual fetch+decode runs on a small bounded
worker pool — never on the layout thread. The first call for a URL
inserts a `Pending` entry and submits a job to the pool; subsequent
calls return `None` while `Pending` and the cached result once
`Ready`. Failures are cached as `Failed` for the TTL window before
being retried.

`data:` URIs, `file://` URLs, redirects across schemes, and HTTP
caching headers (ETag / Cache-Control) are not yet supported.

## 4. Sizing

For an `<img src="…" width="W" height="H" style="width:CW;height:CH">`
the final size is determined as:

1. The image is decoded to its intrinsic `(decoded_w, decoded_h)`.
2. HTML `width` / `height` attributes — if present — override the
   intrinsic dimensions. Each axis is independent; missing
   attribute keeps the intrinsic value on that axis.
3. If the resulting `(w, h)` differs from `(decoded_w, decoded_h)`
   the RGBA buffer is resized once with Lanczos3 and that resized
   variant is cached.
4. CSS `width` / `height` from the cascade override the layout
   box size and stretch the textured quad accordingly — no
   re-decode, no re-resize, the GPU does the scaling at draw
   time.

The renderer's GPU texture cache is keyed by `image_id`, which
hashes `src + declared_w + declared_h`. The same URL at multiple
HTML-attribute sizes therefore lives as separate textures; the
same URL at multiple CSS sizes shares a single texture.

## 5. Two-level cache

`crates/wgpu-html-layout/src/lib.rs` holds two `OnceLock<Mutex<HashMap<…>>>`:

```
raw_cache:   HashMap<String,   CacheEntry<RawState>>
sized_cache: HashMap<SizedKey, CacheEntry<Option<ImageData>>>
```

- `raw_cache` is keyed by `src` only. `RawState` is one of
  `Pending` (fetch in flight), `Ready { rgba, w, h }` (decoded
  RGBA8 in an `Arc<Vec<u8>>`), or `Failed`.
- `sized_cache` is keyed by `(src, declared_w, declared_h)`. It
  holds the post-resize `ImageData` ready for upload. `None` here
  is a memoized failure.
- The `Arc<Vec<u8>>` in `RawState::Ready` is shared across every
  sized variant of that URL, so a 1 MB image rendered at three
  different declared sizes keeps one 1 MB buffer plus three
  resized variants — never four full-resolution copies.

Each `CacheEntry<V>` carries a `last_access: Instant` that is
refreshed on every cache hit (`entry.touch()`).

## 6. Non-blocking loads via the worker pool

```
load_image
    │
    ├─ fast path: sized_cache hit → touch + return
    │
    ├─ raw_cache hit (Pending)  → return None
    │                  (Ready)  → resize → cache → return
    │                  (Failed) → cache None → return None
    │
    └─ raw_cache miss
         │
         └─ insert Pending,
            submit fetch+decode job to pool,
            return None (placeholder)
```

A small bounded pool of `FETCH_POOL_WORKERS` (currently `4`)
persistent worker threads is initialised lazily on first job
submission. The `pool_sender()` is a single `Sender<String>`; each
worker shares a single `Arc<Mutex<Receiver<String>>>` and pulls
URLs off it in a loop. For each URL the worker calls
`fetch_image_bytes` + `decode_rgba`, writes the outcome
(`Ready` / `Failed`) into `raw_cache`, and bumps an `AtomicU64`
exposed as `image_load_revision()`.

Both http(s) URLs and local filesystem paths flow through the same
pool — the dispatch on scheme happens inside `fetch_image_bytes` on
the worker side, so the layout thread itself never blocks on
network *or* disk I/O.

Hosts that don't already redraw every frame can poll
`image_load_revision()` in their event loop and request a
relayout when it changes; the demo doesn't need it because
winit's `ControlFlow::Poll` already redraws continuously.

### Preloading

To start downloading important assets at startup so the first
frame doesn't paint placeholders, push them into
`Tree::preload_queue` via `Tree::preload_asset(src)` once at
construction. Every layout pass walks the queue and calls
`wgpu_html_layout::preload_image(url)` on each entry; that
function is idempotent — already-known URLs are a hashmap-lookup
no-op — so it's cheap to leave in place. `preload_image` is also
exposed as a public free function for ad-hoc calls outside a
`Tree` context.

## 7. TTL eviction

Idle entries are reclaimed by `sweep_image_cache`:

- Default TTL: **5 minutes**, configurable via
  `set_image_cache_ttl(Duration)` or — preferred — via the
  `Tree`'s `asset_cache_ttl: Option<Duration>` field, which is
  applied at the start of every `compute_layout` /
  `paint_tree_with_text` call.
- The sweep walks both maps with `HashMap::retain` and drops any
  entry whose `last_access` is older than the TTL.
- `RawState::Pending` is **never** evicted: dropping it would
  orphan the worker thread's eventual write.
- The sweep is rate-limited to once per `SWEEP_INTERVAL`
  (10 seconds) by a `Mutex<Instant>` last-sweep timestamp, and
  is run opportunistically at the top of `load_image`. No
  background timer thread is needed.
- `purge_image_cache()` clears everything regardless of age
  (still preserves `Pending`).

## 8. Public API

Re-exported from `wgpu_html::layout`:

```rust
pub fn image_cache_ttl()              -> Duration;
pub fn set_image_cache_ttl(ttl: Duration);
pub fn image_load_revision()          -> u64;
pub fn sweep_image_cache();
pub fn purge_image_cache();
pub fn preload_image(src: &str);
```

On `wgpu_html::tree::Tree`:

```rust
pub asset_cache_ttl: Option<Duration>,  // None → keep current default
pub preload_queue:   Vec<String>,       // walked every layout pass
pub fn preload_asset(&mut self, src: impl Into<String>);
```

Setting the field is the recommended path: it travels with the
document and survives across renderer instances.

## 9. Caveats and known gaps

- **Pool size is fixed at 4.** `FETCH_POOL_WORKERS` is a `const`
  for now; tuning will need a setter (and a way to grow the pool
  beyond what was lazily initialised). 4 is enough for typical
  pages without saturating the network on residential links.
- **No HTTP cache semantics.** ETags, `Cache-Control`,
  `If-Modified-Since`, and similar are ignored. The TTL is the
  only freshness control.
- **No redirects across schemes.** `ureq` follows http→http and
  https→https redirects by default but not http↔https.
- **No `data:` URIs.** Trivial to add inside `fetch_image_bytes`.
- **Animated images flatten to first frame.** GIF / animated WebP
  decode their first frame and stop there; there is no animation
  driver in the renderer yet.
- **Process-wide cache.** Multiple documents driven by the same
  process share one cache. Two trees with conflicting
  `asset_cache_ttl`s last-applied-wins; if you need per-document
  isolation, hold the lowest TTL globally.
- **No upper bound on cache size.** TTL is the only eviction
  trigger. Hosts that load thousands of unique images within a
  single TTL window will see growing memory until the next sweep.
  An LRU cap is straightforward to add by sorting the
  `last_access` timestamps and trimming the tail.
- **No retry-on-failure.** A `Failed` entry is treated as cached
  data: the image won't be retried until the TTL expires. For
  transient network errors a small retry budget on the worker
  side would be friendlier.

## 10. Tests

Image loading itself is exercised end-to-end by the demo
(`crates/wgpu-html-demo/html/img-test.html`), which now
references remote URLs, and indirectly by the painter tests in
`crates/wgpu-html/src/paint.rs`. There are no isolated unit
tests for the cache or TTL behaviour yet — adding deterministic
ones requires either a fake clock injection or moving `Instant`
behind a trait, neither of which has been done.
