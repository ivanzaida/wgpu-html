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
| GIF    | `.gif`       | Animated; multi-frame, with delays |
| BMP    | `.bmp`       | Uncompressed                       |
| WebP   | `.webp`      | Animated; multi-frame, with delays |

Formats not in the feature list (TIFF, AVIF, HDR, SVG, …) are not
decoded; they fail to load and show as a missing image
(`load_image` returns `None`). Add the matching `image` crate
feature (and a per-format frame iterator next to
`decode_animated_gif` if it's animated) to enable more.

## 3. Schemes

`fetch_image_bytes` (in `crates/wgpu-html-layout/src/lib.rs`)
dispatches on the `src` prefix:

- **`http://` / `https://`** — fetched with `ureq` (rustls TLS)
  via `fetch_remote`. Manual redirect following supports
  cross-scheme hops (http↔https) up to `MAX_REDIRECTS` (5).
  Response bodies are capped at 32 MiB to bound memory. Per-URL
  fetches retry up to `MAX_FETCH_ATTEMPTS` (3) with exponential
  backoff (200 ms → 400 ms → 800 ms) before declaring `Failed`.
- **`data:` URIs** — base64 and percent-encoded payloads decoded
  inline (`fetch_data_uri`). MIME type is ignored; the decoder
  sniffs format from the bytes.
- **Anything else** — treated as a local filesystem path and read
  with `std::fs::read`.

In every case the actual fetch+decode runs on the small bounded
worker pool — never on the layout thread. The first call for a URL
inserts a `Pending` entry and submits a job to the pool; subsequent
calls return `None` while `Pending` and the cached result once
`Ready`. Failures are cached as `Failed` for the TTL window before
being retried.

HTTP `Cache-Control: max-age` (and the `Date` / `Expires` pair as
fallback) is parsed off remote responses and stored on the cache
entry. `sweep_image_cache` honours the per-entry max-age over the
global TTL, so long-cached CDN assets stay warm for hours and
`Cache-Control: no-store` / `no-cache` resources evict on the next
sweep.

`file://` URLs, full HTTP conditional GETs (`If-None-Match` /
`If-Modified-Since` revalidation against still-cached bytes), and
`Cache-Control: must-revalidate` enforcement are still TODO — only
the freshness window is honoured today, not the conditional-GET
optimisation on stale entries.

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
  `Pending` (fetch in flight), `Ready(DecodedAsset)`, or `Failed`.
  `DecodedAsset` is itself either `Still { rgba, w, h }` (a single
  `Arc<Vec<u8>>` of RGBA8 pixels) or `Animated { frames, w, h }`
  (an `Arc<Vec<DecodedFrameRaw>>` of per-frame RGBA buffers and
  delays, used for GIFs).
- `sized_cache` is keyed by `(src, declared_w, declared_h)`. It
  holds the post-resize `ImageData` ready for upload — including,
  for animated assets, the full `Arc<Vec<ImageFrame>>`. `None`
  here is a memoized failure.
- The `Arc<Vec<u8>>`s inside `DecodedAsset` are shared across every
  sized variant of that URL, so a 1 MB image rendered at three
  different declared sizes keeps one 1 MB buffer plus three
  resized variants — never four full-resolution copies. Animated
  assets do the same per frame: one decoded copy in `raw_cache`
  feeds every sized variant in `sized_cache`.

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

A small bounded pool of persistent worker threads (default size
`DEFAULT_POOL_WORKERS = 4`, tunable via `set_image_pool_size(n)`)
is initialised lazily on first job submission. The pool size is
**grow-only**: requesting a larger value spawns the additional
workers immediately; shrinking is a no-op (workers are cheap idle
and the simpler lifecycle is worth keeping). The `pool_sender()`
is a single `Sender<String>`; each worker shares a single
`Arc<Mutex<Receiver<String>>>` and pulls URLs off it in a loop.
For each URL the worker calls `fetch_with_retry` (which wraps
`fetch_image_bytes` with bounded exponential backoff) + then
`decode_asset` (routing GIFs through `decode_animated_gif`,
WebPs through `decode_animated_webp`, everything else through
the single-frame path), writes the outcome — `Ready(asset)` with
captured `max_age` metadata, or `Failed` — into `raw_cache`, and
bumps an `AtomicU64` exposed as `image_load_revision()`. After
every insert it calls `enforce_image_cache_budget()` so a flood of
large remote fetches can't push memory past the configured
[byte budget](#7a-byte-budget-eviction) between periodic sweeps.

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

## 6a. Animated images (GIF + WebP)

GIF and WebP are both decoded as sequences of fully-composited
RGBA frames plus per-frame delays via the `image` crate's
`AnimationDecoder` trait — the decoder applies disposal /
transparency / blending internally so we just consume the yielded
frames as opaque RGBA buffers. Single-frame containers collapse
back to the still path automatically. Magic-byte sniffing in
`decode_asset` routes `GIF87a`/`GIF89a` to `decode_animated_gif`
and `RIFF…WEBP` to `decode_animated_webp`.

- The raw cache stores a `DecodedAsset::Animated { frames, w, h }`
  variant alongside the existing `Still`. Sized cache entries for
  animated assets carry the full `Vec<ImageFrame>` — every frame
  has its own `image_id` (hash of URL + declared size + frame
  index), so the renderer's GPU texture cache treats each frame as
  an independent texture.
- `current_frame(&ImageData)` picks the active frame at call time
  using a process-wide clock anchored on the first call to
  `animation_clock_origin()`. Every animated `<img>` and
  `background-image` of the same GIF stays in lockstep regardless
  of where it appears in the DOM.
- `load_image_url` substitutes the active frame into the
  `image_id`/`data` fields of the returned `ImageData` on every
  call. The cache itself keeps the full frame list so the
  substitution is just a couple of `Arc` clones.
- Layout/paint don't need any animation-specific code: every
  frame is a regular textured quad. Continuous redraws (e.g.
  winit's `ControlFlow::Poll`) advance the animation; hosts that
  redraw on demand can poll `image_load_revision()` once at
  startup and request redraws when it changes — but to drive
  animation forward they need to redraw on a timer too.

Memory cost: a 30-frame, 256×256 GIF holds ~7.8 MiB of decoded
RGBA in `raw_cache` plus the same again per declared-size variant
in `sized_cache`, plus one GPU texture per frame in the renderer's
cache. Pathological multi-thousand-frame GIFs will saturate
texture memory; TTL eviction reclaims them once they idle.

## 7. TTL eviction

Idle entries are reclaimed by `sweep_image_cache`:

- Default TTL: **5 minutes**, configurable via
  `set_image_cache_ttl(Duration)` or — preferred — via the
  `Tree`'s `asset_cache_ttl: Option<Duration>` field, which is
  applied at the start of every `compute_layout` /
  `paint_tree_with_text` call.
- Each cache entry can carry a per-URL `max_age` override sourced
  from HTTP `Cache-Control: max-age` / `Expires`. When set, the
  sweep uses it instead of the global TTL — `entry.effective_ttl`
  picks `max_age.unwrap_or(global_ttl)`.
- The sweep walks both maps with `HashMap::retain` and drops any
  entry whose `last_access` is older than its effective TTL.
- `RawState::Pending` is **never** evicted: dropping it would
  orphan the worker thread's eventual write.
- The sweep is rate-limited to once per `SWEEP_INTERVAL`
  (10 seconds) by a `Mutex<Instant>` last-sweep timestamp, and
  is run opportunistically at the top of `load_image`. No
  background timer thread is needed. After each sweep
  `enforce_image_cache_budget()` runs to honour the byte budget.
- `purge_image_cache()` clears everything regardless of age
  (still preserves `Pending`).

## 7a. Byte-budget eviction

Independent from the TTL pass, `enforce_image_cache_budget`
caps total decoded RGBA in both caches:

- Default budget: 256 MiB, configurable via
  `set_image_cache_budget(bytes)`. `0` disables the cap.
- When over budget, evicts oldest non-`Pending` entries by
  ascending `last_access` until back under. The sized cache (post-
  resize duplicates) is trimmed first; raw cache is the
  source-of-truth and only trimmed if more reclaim is needed.
- Runs every periodic sweep AND immediately after every worker
  insert, so a flood of large remote fetches can't grow memory
  unbounded between sweeps.

## 8. Public API

Re-exported from `wgpu_html::layout`:

```rust
pub fn image_cache_ttl()              -> Duration;
pub fn set_image_cache_ttl(ttl: Duration);
pub fn image_cache_budget()           -> u64;
pub fn set_image_cache_budget(bytes: u64);
pub fn image_pool_size()              -> usize;
pub fn set_image_pool_size(n: usize);
pub fn image_load_revision()          -> u64;
pub fn sweep_image_cache();
pub fn enforce_image_cache_budget();
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

- **No HTTP conditional GETs (yet).** `Cache-Control: max-age` is
  honoured for freshness, but ETag / `Last-Modified` revalidation
  with `If-None-Match` / `If-Modified-Since` isn't wired up — once
  an entry expires we always do a full GET. The infrastructure to
  add this (per-entry meta sidecar + `RemoteMeta` headers in the
  worker) is in place; the missing bit is preserving the previous
  asset across the revalidation window.
- **Process-wide cache.** Multiple documents driven by the same
  process share one cache. This is *by design* for the common case
  (multiple windows of the same app reuse images for free) but
  hosts that need isolation must currently `purge_image_cache()`
  between document switches or namespace their URLs (e.g.
  `doc1://…` prefixes). Lifting the cache state into a
  `ImageCache` struct owned by `Tree` is the natural next step;
  it requires plumbing a cache reference through every layout
  call site and is therefore deliberately deferred.

## 10. Tests

Image loading itself is exercised end-to-end by the demo
(`crates/wgpu-html-demo/html/img-test.html`), which now
references remote URLs, and indirectly by the painter tests in
`crates/wgpu-html/src/paint.rs`. There are no isolated unit
tests for the cache or TTL behaviour yet — adding deterministic
ones requires either a fake clock injection or moving `Instant`
behind a trait, neither of which has been done.
