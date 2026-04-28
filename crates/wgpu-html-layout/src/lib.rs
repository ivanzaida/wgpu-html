//! Block layout.
//!
//! Walks a `CascadedTree` (one Style per node, already cascaded) and
//! produces a `LayoutBox` tree positioned in physical pixels. The renderer
//! consumes the result directly — it never re-resolves CSS.
//!
//! Scope (M4):
//! - Block formatting context only: every element stacks vertically inside
//!   its parent's content box.
//! - Margin and padding (per-side or shorthand) are honoured.
//! - Width auto-fills the parent's content width; height auto-fits content.
//! - Borders are not drawn yet (treated as zero); inline / flex / floats
//!   come in later milestones.
//! - Text nodes contribute zero height; M5 brings real text layout.

use wgpu_html_models::Style;
use wgpu_html_models::common::css_enums::{BorderStyle, BoxSizing, CssLength, Display, Overflow};
use wgpu_html_style::{CascadedNode, CascadedTree};
use wgpu_html_text::{ParagraphSpan, PositionedGlyph, ShapedRun, TextContext};
use wgpu_html_tree::{Element, Node, Tree};

mod color;
mod flex;
mod grid;
mod length;

pub use color::{Color, resolve_color};

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// One frame of an animated image as it sits in the raw (pre-resize)
/// cache. `delay_ms` comes straight from the source format and is
/// clamped to a sensible minimum (10 ms) for sources that report a
/// 0-tick delay.
#[derive(Debug, Clone)]
struct DecodedFrameRaw {
    rgba: Arc<Vec<u8>>,
    delay_ms: u32,
}

/// Decode result placed into [`RawState::Ready`]. Single-frame
/// formats (PNG, JPEG, …) become `Still`; GIFs (and any future
/// animated decoders) become `Animated` with the per-frame delays
/// preserved.
#[derive(Debug, Clone)]
enum DecodedAsset {
    Still { rgba: Arc<Vec<u8>>, w: u32, h: u32 },
    Animated { frames: Arc<Vec<DecodedFrameRaw>>, w: u32, h: u32 },
}

/// State of a unique `src` in the raw (pre-resize) decode cache.
#[derive(Clone)]
enum RawState {
    /// A background thread is currently fetching/decoding this URL.
    /// Subsequent `load_image` calls return `None` (placeholder) until
    /// it transitions to `Ready` or `Failed`.
    Pending,
    /// Decoded asset (still or animated). The `Arc`s inside let every
    /// sized variant share the underlying RGBA buffers when no resize
    /// is needed.
    Ready(DecodedAsset),
    /// A previous fetch/decode attempt failed; memoized so we don't
    /// retry forever.
    Failed,
}

/// Cache entry wrapping a value with its last-access timestamp. Used
/// for both the raw-by-URL and the sized-by-(URL,w,h) caches so that
/// stale entries (not touched by any `load_image` call within the
/// configured TTL) can be reclaimed by [`sweep_image_cache`].
struct CacheEntry<V> {
    value: V,
    last_access: Instant,
}

impl<V> CacheEntry<V> {
    fn new(value: V) -> Self {
        Self { value, last_access: Instant::now() }
    }
    fn touch(&mut self) {
        self.last_access = Instant::now();
    }
}

/// Per-URL raw decode cache. Keyed by `src` only — independent of the
/// `<img>` element's declared width/height — so the same URL rendered
/// at multiple sizes is fetched and decoded exactly once.
fn raw_cache() -> &'static Mutex<HashMap<String, CacheEntry<RawState>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, CacheEntry<RawState>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Sized cache: keyed by `(src, declared_w, declared_h)`. Holds the
/// final, possibly-resized `ImageData` ready for the renderer. `None`
/// here means "src failed to load"; that propagates up so the layout
/// pass treats it as a missing image.
type SizedKey = (String, Option<u32>, Option<u32>);
fn sized_cache() -> &'static Mutex<HashMap<SizedKey, CacheEntry<Option<ImageData>>>> {
    static CACHE: OnceLock<Mutex<HashMap<SizedKey, CacheEntry<Option<ImageData>>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Cache TTL in seconds. Entries (in either cache) whose `last_access`
/// is older than this are dropped on the next sweep. Default: 5 min.
/// Configurable via [`set_image_cache_ttl`].
static CACHE_TTL_SECS: AtomicU64 = AtomicU64::new(300);

/// How often the opportunistic sweep is allowed to run, regardless of
/// how many `load_image` calls happen in between. Avoids walking the
/// maps on every layout frame.
const SWEEP_INTERVAL: Duration = Duration::from_secs(10);

/// Timestamp of the most recent sweep. Lazily initialised on first
/// access. Holds a `Mutex<Instant>` rather than an atomic because
/// `Instant` isn't `AtomicU64`-compatible portably.
fn last_sweep() -> &'static Mutex<Instant> {
    static LAST: OnceLock<Mutex<Instant>> = OnceLock::new();
    LAST.get_or_init(|| Mutex::new(Instant::now()))
}

/// Returns the current cache TTL.
pub fn image_cache_ttl() -> Duration {
    Duration::from_secs(CACHE_TTL_SECS.load(Ordering::Acquire))
}

/// Override the cache TTL. Entries last accessed more than this long
/// ago are dropped on the next sweep. A TTL of zero seconds disables
/// caching effectively (every sweep clears the maps), so prefer values
/// of at least a few seconds.
pub fn set_image_cache_ttl(ttl: Duration) {
    CACHE_TTL_SECS.store(ttl.as_secs(), Ordering::Release);
}

/// Drop every cached image regardless of age. Pending fetches are
/// preserved (the in-flight worker still has a reference to the URL
/// and will try to insert its result on completion).
pub fn purge_image_cache() {
    if let Ok(mut cache) = sized_cache().lock() {
        cache.clear();
    }
    if let Ok(mut cache) = raw_cache().lock() {
        cache.retain(|_, entry| matches!(entry.value, RawState::Pending));
    }
}

/// Walk both caches and drop any entry whose `last_access` is older
/// than the configured TTL. Skips `RawState::Pending` so an in-flight
/// fetch isn't lost. Safe to call from any thread; called
/// opportunistically at the top of `load_image`, so manual invocation
/// is rarely needed.
pub fn sweep_image_cache() {
    let ttl = image_cache_ttl();
    let now = Instant::now();
    if let Ok(mut cache) = sized_cache().lock() {
        cache.retain(|_, entry| now.duration_since(entry.last_access) < ttl);
    }
    if let Ok(mut cache) = raw_cache().lock() {
        cache.retain(|_, entry| {
            // Never evict an in-flight fetch (we'd lose the result).
            matches!(entry.value, RawState::Pending)
                || now.duration_since(entry.last_access) < ttl
        });
    }
}

/// Run [`sweep_image_cache`] if `SWEEP_INTERVAL` has elapsed since the
/// last sweep. Cheap to call from hot paths.
fn maybe_sweep() {
    let now = Instant::now();
    let due = match last_sweep().lock() {
        Ok(mut last) => {
            if now.duration_since(*last) >= SWEEP_INTERVAL {
                *last = now;
                true
            } else {
                false
            }
        }
        Err(_) => false,
    };
    if due {
        sweep_image_cache();
    }
}

/// Monotonic counter bumped every time a background fetch completes
/// (success *or* failure). Hosts that don't already redraw every frame
/// can poll [`image_load_revision`] and trigger a relayout when it
/// changes — e.g. in winit's `about_to_wait`.
fn revision() -> &'static AtomicU64 {
    static REV: AtomicU64 = AtomicU64::new(0);
    &REV
}

/// Returns a counter that increments each time a remote image finishes
/// loading. Compare against the previously observed value to decide
/// whether a relayout/redraw is warranted.
pub fn image_load_revision() -> u64 {
    revision().load(Ordering::Acquire)
}

/// Fetch the raw bytes for an `<img>` `src`. Supports `http://` and
/// `https://` URLs (blocking via `ureq`, intended to run on a worker
/// thread); anything else is treated as a local filesystem path.
fn fetch_image_bytes(src: &str) -> Option<Vec<u8>> {
    if src.starts_with("http://") || src.starts_with("https://") {
        let resp = ureq::get(src).call().ok()?;
        // Cap at a sane upper bound (32 MiB) to avoid unbounded reads.
        let mut buf = Vec::new();
        resp.into_reader()
            .take(32 * 1024 * 1024)
            .read_to_end(&mut buf)
            .ok()?;
        Some(buf)
    } else {
        std::fs::read(src).ok()
    }
}

/// Decode arbitrary image bytes to a still or animated [`DecodedAsset`].
/// GIFs are detected by magic bytes and routed through the animated
/// decoder; everything else takes the single-frame path. A GIF that
/// only contains one frame collapses to `Still` after decoding so
/// downstream code never has to special-case it.
fn decode_asset(bytes: &[u8]) -> Option<DecodedAsset> {
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        if let Some(asset) = decode_animated_gif(bytes) {
            return Some(asset);
        }
        // GIF magic but the decoder failed — fall through to the
        // single-frame path which may still extract the first frame
        // via the generic `image::load_from_memory` route.
    }
    let dyn_img = image::load_from_memory(bytes).ok()?;
    let rgba = dyn_img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    Some(DecodedAsset::Still {
        rgba: Arc::new(rgba.into_raw()),
        w,
        h,
    })
}

/// Decode every frame of an animated GIF, preserving per-frame delays.
/// Each yielded `Frame` from `image::AnimationDecoder` is already a
/// fully-composited canvas (the decoder applies disposal/transparency
/// internally), so we just copy its RGBA buffer.
fn decode_animated_gif(bytes: &[u8]) -> Option<DecodedAsset> {
    use image::AnimationDecoder;
    use image::codecs::gif::GifDecoder;

    let decoder = GifDecoder::new(std::io::Cursor::new(bytes)).ok()?;
    let frames = decoder.into_frames().collect_frames().ok()?;
    if frames.is_empty() {
        return None;
    }
    let (w, h) = {
        let first = frames.first()?;
        (first.buffer().width(), first.buffer().height())
    };
    if frames.len() == 1 {
        // Static GIF: collapse to the still path so the animation
        // machinery never kicks in.
        let buffer = frames.into_iter().next()?.into_buffer();
        return Some(DecodedAsset::Still {
            rgba: Arc::new(buffer.into_raw()),
            w,
            h,
        });
    }
    let decoded: Vec<DecodedFrameRaw> = frames
        .into_iter()
        .map(|f| {
            let (numer, denom) = f.delay().numer_denom_ms();
            // Defend against pathological metadata: `denom == 0` is
            // illegal but seen in the wild; `0`-delay frames cause a
            // tight repaint loop, so clamp to ~100 fps.
            let delay = if denom == 0 { 100 } else { numer / denom.max(1) };
            let delay_ms = delay.max(10);
            let buffer = f.into_buffer();
            DecodedFrameRaw {
                rgba: Arc::new(buffer.into_raw()),
                delay_ms,
            }
        })
        .collect();
    Some(DecodedAsset::Animated {
        frames: Arc::new(decoded),
        w,
        h,
    })
}

/// Compute a stable `image_id` for an (URL, declared size, optional
/// frame index) tuple. Animated frames get distinct ids so the
/// renderer's GPU texture cache stores each as its own texture.
fn make_image_id(
    src: &str,
    declared_w: Option<u32>,
    declared_h: Option<u32>,
    frame_index: Option<usize>,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    src.hash(&mut hasher);
    declared_w.hash(&mut hasher);
    declared_h.hash(&mut hasher);
    frame_index.hash(&mut hasher);
    hasher.finish()
}

/// Resize a single RGBA buffer if the declared dimensions differ from
/// the intrinsic ones; otherwise share the input `Arc`. Returns
/// `None` only when `ImageBuffer::from_raw` rejects the input length
/// (shouldn't happen for buffers we produced ourselves, but the
/// signature is conservative).
fn resize_rgba(
    src_rgba: Arc<Vec<u8>>,
    decoded_w: u32,
    decoded_h: u32,
    target_w: u32,
    target_h: u32,
) -> Option<Arc<Vec<u8>>> {
    if target_w == decoded_w && target_h == decoded_h {
        return Some(src_rgba);
    }
    let buf = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
        decoded_w,
        decoded_h,
        (*src_rgba).clone(),
    )?;
    let resized = image::imageops::resize(
        &buf,
        target_w,
        target_h,
        image::imageops::FilterType::Lanczos3,
    );
    Some(Arc::new(resized.into_raw()))
}

/// Build a sized `ImageData` from a raw cache entry. For still images
/// this is just a (possibly-shared) RGBA buffer + dims. For animated
/// images, every frame is resized, each gets its own `image_id`, and
/// the result's top-level `image_id`/`data` point at the first
/// frame; per-frame entries live in `frames`. Callers substitute the
/// active frame via [`current_frame`] before painting.
fn build_sized(
    src: &str,
    asset: &DecodedAsset,
    declared_w: Option<u32>,
    declared_h: Option<u32>,
) -> Option<ImageData> {
    match asset {
        DecodedAsset::Still { rgba, w, h } => {
            let target_w = declared_w.unwrap_or(*w);
            let target_h = declared_h.unwrap_or(*h);
            let data = resize_rgba(rgba.clone(), *w, *h, target_w, target_h)?;
            Some(ImageData {
                image_id: make_image_id(src, declared_w, declared_h, None),
                data,
                width: target_w,
                height: target_h,
                frames: None,
            })
        }
        DecodedAsset::Animated { frames, w, h } => {
            let target_w = declared_w.unwrap_or(*w);
            let target_h = declared_h.unwrap_or(*h);
            let mut sized = Vec::with_capacity(frames.len());
            for (i, f) in frames.iter().enumerate() {
                let data = resize_rgba(f.rgba.clone(), *w, *h, target_w, target_h)?;
                sized.push(ImageFrame {
                    image_id: make_image_id(src, declared_w, declared_h, Some(i)),
                    data,
                    delay_ms: f.delay_ms,
                });
            }
            let first = sized.first()?;
            let head_id = first.image_id;
            let head_data = first.data.clone();
            Some(ImageData {
                image_id: head_id,
                data: head_data,
                width: target_w,
                height: target_h,
                frames: Some(Arc::new(sized)),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Animation: process-relative clock + active-frame selection
// ---------------------------------------------------------------------------

/// Wall-clock anchor used to drive every animated image. All
/// `<img>`/`background-image` instances of the same animated GIF
/// stay in lockstep (and across the whole document) because they
/// all index off this single shared origin.
fn animation_clock_origin() -> Instant {
    static ORIGIN: OnceLock<Instant> = OnceLock::new();
    *ORIGIN.get_or_init(Instant::now)
}

/// Pick the currently-visible frame index given a frame list and the
/// process-relative animation clock. Loops infinitely (modulo the
/// total animation duration). Falls back to frame 0 for empty / all-
/// zero-delay sequences.
fn current_frame_index(frames: &[ImageFrame]) -> usize {
    let total: u64 = frames.iter().map(|f| f.delay_ms as u64).sum();
    if total == 0 {
        return 0;
    }
    let elapsed = animation_clock_origin().elapsed().as_millis() as u64;
    let t = elapsed % total;
    let mut acc = 0u64;
    for (i, f) in frames.iter().enumerate() {
        acc += f.delay_ms as u64;
        if t < acc {
            return i;
        }
    }
    frames.len() - 1
}

/// If `data` is animated, return a copy whose `image_id`/`data` point
/// at the currently-visible frame; otherwise return `data` unchanged.
/// Cheap: at most a couple of `Arc` clones.
pub(crate) fn current_frame(data: &ImageData) -> ImageData {
    match &data.frames {
        Some(frames) if !frames.is_empty() => {
            let i = current_frame_index(frames);
            let f = &frames[i];
            ImageData {
                image_id: f.image_id,
                data: f.data.clone(),
                width: data.width,
                height: data.height,
                frames: data.frames.clone(),
            }
        }
        _ => data.clone(),
    }
}

// ---------------------------------------------------------------------------
// Image fetch worker pool
// ---------------------------------------------------------------------------

/// How many concurrent fetch+decode workers run. Conservative default:
/// most pages have only a handful of images and decoding a megapixel
/// PNG is fast, so a small pool keeps thread count bounded without
/// becoming a bottleneck. Tunable later if needed.
const FETCH_POOL_WORKERS: usize = 4;

/// Sender side of the bounded work queue feeding the fetch+decode
/// worker pool. Lazily initialised on first job submission; once
/// initialised the pool runs for the life of the process.
fn pool_sender() -> &'static Mutex<Sender<String>> {
    static SENDER: OnceLock<Mutex<Sender<String>>> = OnceLock::new();
    SENDER.get_or_init(|| {
        let (tx, rx) = channel::<String>();
        let rx = Arc::new(Mutex::new(rx));
        for _ in 0..FETCH_POOL_WORKERS {
            spawn_fetch_worker(Arc::clone(&rx));
        }
        Mutex::new(tx)
    })
}

/// Spawn one persistent worker thread. The worker takes the next URL
/// off the shared `Receiver`, fetches+decodes it, and writes the
/// outcome (`Ready` or `Failed`) into `raw_cache`. Termination
/// happens implicitly when the `Sender` is dropped (i.e. on process
/// exit) — `recv` returns `Err` and the worker exits.
fn spawn_fetch_worker(rx: Arc<Mutex<Receiver<String>>>) {
    std::thread::spawn(move || {
        loop {
            let url = match rx.lock() {
                Ok(guard) => match guard.recv() {
                    Ok(u) => u,
                    Err(_) => break,
                },
                Err(_) => break,
            };
            let outcome = match fetch_image_bytes(&url).as_deref().and_then(decode_asset) {
                Some(asset) => RawState::Ready(asset),
                None => RawState::Failed,
            };
            if let Ok(mut cache) = raw_cache().lock() {
                cache.insert(url, CacheEntry::new(outcome));
            }
            revision().fetch_add(1, Ordering::AcqRel);
        }
    });
}

/// Submit `url` to the worker pool. Caller must have already inserted
/// a `RawState::Pending` entry into `raw_cache` so concurrent calls
/// for the same URL coalesce on the cache rather than enqueuing
/// duplicate jobs.
fn submit_fetch_job(url: String) {
    if let Ok(tx) = pool_sender().lock() {
        let _ = tx.send(url);
    }
}

/// Pre-warm the cache for `src` without actually returning the
/// `ImageData`. If the URL is already known (whether `Pending`,
/// `Ready`, or `Failed`) this is a no-op — making it safe to call on
/// every layout pass for a static preload list. Otherwise the URL is
/// inserted as `Pending` and a fetch+decode job is queued on the
/// pool.
///
/// Use this to start downloading important above-the-fold assets at
/// startup so the first layout doesn't paint empty placeholders.
pub fn preload_image(src: &str) {
    if src.is_empty() {
        return;
    }
    if let Ok(mut cache) = raw_cache().lock() {
        if cache.contains_key(src) {
            return;
        }
        cache.insert(src.to_owned(), CacheEntry::new(RawState::Pending));
    } else {
        return;
    }
    submit_fetch_job(src.to_owned());
}

/// Load an `<img>` element's `src` and produce decoded RGBA8 ready for
/// the GPU. Returns `None` if `src` is unset, or if the image isn't
/// available *yet*.
///
/// All loads (`http://`, `https://`, local filesystem paths) are
/// dispatched to a small bounded worker pool. The first call kicks
/// off the fetch and returns `None`; subsequent calls return
/// `Some(..)` once a worker has decoded and cached the bytes. While
/// the entry is `Pending` no duplicate jobs are enqueued.
///
/// Both raw decoded bytes and per-size resize results are cached
/// process-wide. The same URL rendered at multiple declared
/// `width`/`height`s reuses the single decoded buffer.
pub(crate) fn load_image(img: &wgpu_html_models::Img) -> Option<ImageData> {
    let src = img.src.as_deref()?;
    load_image_url(src, img.width, img.height)
}

/// Lower-level entry point: load a decoded `ImageData` for a given URL
/// and optional declared (post-resize) dimensions. CSS
/// `background-image: url(...)` calls this directly with no declared
/// size hints; `<img>` goes through [`load_image`] which pulls them
/// off the element. All caching, async fetch, and TTL behaviour is
/// shared between the two entry points.
pub(crate) fn load_image_url(
    src: &str,
    declared_w: Option<u32>,
    declared_h: Option<u32>,
) -> Option<ImageData> {
    let sized_key: SizedKey = (src.to_owned(), declared_w, declared_h);

    // Opportunistically reclaim memory from idle entries.
    maybe_sweep();

    // Fast path: we already produced the sized result for this URL+size.
    // Touch its `last_access` so it stays warm. For animated assets
    // we substitute the *current* frame on every hit so the renderer
    // sees per-frame `image_id`s as time advances; the cached entry
    // itself keeps the full frame list intact.
    if let Ok(mut cache) = sized_cache().lock() {
        if let Some(entry) = cache.get_mut(&sized_key) {
            entry.touch();
            return entry.value.as_ref().map(current_frame);
        }
    }

    // Look up (or initialise) the per-URL raw entry. Touch on hit.
    // Both remote (http/https) and local filesystem paths are
    // dispatched to the bounded worker pool; the first sighting of a
    // URL inserts a `Pending` entry so concurrent layout passes don't
    // enqueue duplicate jobs.
    let raw = {
        let mut cache = raw_cache().lock().ok()?;
        match cache.get_mut(src) {
            Some(entry) => {
                entry.touch();
                entry.value.clone()
            }
            None => {
                cache.insert(src.to_owned(), CacheEntry::new(RawState::Pending));
                drop(cache);
                submit_fetch_job(src.to_owned());
                return None;
            }
        }
    };

    let full = match raw {
        RawState::Pending => return None,
        RawState::Failed => None,
        RawState::Ready(asset) => build_sized(src, &asset, declared_w, declared_h),
    };

    // Cache the (full, all-frames) sized variant. Failures are also
    // memoized so repeated misses stay O(1).
    if let Ok(mut cache) = sized_cache().lock() {
        cache.insert(sized_key, CacheEntry::new(full.clone()));
    }
    full.as_ref().map(current_frame)
}

// ---------------------------------------------------------------------------
// CSS background-image resolution
// ---------------------------------------------------------------------------

/// Extract the URL from a CSS `url(...)` token. Accepts unquoted,
/// `"…"`-quoted, and `'…'`-quoted forms; returns `None` for any
/// other syntax (including gradient functions, `none`, etc.).
fn extract_url(value: &str) -> Option<String> {
    let v = value.trim();
    let inner = v.strip_prefix("url(")?.strip_suffix(')')?.trim();
    let unquoted = if (inner.starts_with('"') && inner.ends_with('"'))
        || (inner.starts_with('\'') && inner.ends_with('\''))
    {
        if inner.len() < 2 {
            return None;
        }
        &inner[1..inner.len() - 1]
    } else {
        inner
    };
    let trimmed = unquoted.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse a single token from `background-size` / `background-position`
/// into a length in physical pixels. Supports `<n>px`, bare `<n>`
/// (interpreted as pixels), `<n>%` (resolved against `container`), and
/// the keyword `auto` (returned as `None`). Returns `None` for any
/// unrecognised input — callers fall back to a sensible default.
fn parse_bg_axis(token: &str, container: f32) -> Option<f32> {
    let t = token.trim().to_ascii_lowercase();
    if t == "auto" || t.is_empty() {
        return None;
    }
    if let Some(stripped) = t.strip_suffix('%') {
        return stripped.trim().parse::<f32>().ok().map(|p| container * p / 100.0);
    }
    let numeric = t.strip_suffix("px").unwrap_or(&t);
    numeric.trim().parse::<f32>().ok()
}

/// Resolve `background-size` to a per-tile (width, height) pair in
/// physical pixels. Supports `auto`, `cover`, `contain`, single
/// `<length-percentage>` (applied to width, height auto), and a
/// `<lp> <lp>` pair. Aspect ratio is preserved when one axis is
/// `auto` (the standard CSS behaviour).
fn resolve_bg_size(value: Option<&str>, img_w: u32, img_h: u32, bg_w: f32, bg_h: f32) -> (f32, f32) {
    let intrinsic_w = img_w as f32;
    let intrinsic_h = img_h as f32;
    if intrinsic_w <= 0.0 || intrinsic_h <= 0.0 || bg_w <= 0.0 || bg_h <= 0.0 {
        return (intrinsic_w.max(0.0), intrinsic_h.max(0.0));
    }
    let raw = value.map(str::trim).unwrap_or("auto");
    let lower = raw.to_ascii_lowercase();
    if lower == "auto" || lower.is_empty() {
        return (intrinsic_w, intrinsic_h);
    }
    if lower == "cover" {
        let scale = (bg_w / intrinsic_w).max(bg_h / intrinsic_h);
        return (intrinsic_w * scale, intrinsic_h * scale);
    }
    if lower == "contain" {
        let scale = (bg_w / intrinsic_w).min(bg_h / intrinsic_h);
        return (intrinsic_w * scale, intrinsic_h * scale);
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    let aspect = intrinsic_h / intrinsic_w;
    match parts.as_slice() {
        [w_s] => {
            let w = parse_bg_axis(w_s, bg_w).unwrap_or(intrinsic_w);
            (w, w * aspect)
        }
        [w_s, h_s] => {
            let w_opt = parse_bg_axis(w_s, bg_w);
            let h_opt = parse_bg_axis(h_s, bg_h);
            match (w_opt, h_opt) {
                (Some(w), Some(h)) => (w, h),
                (Some(w), None) => (w, w * aspect),
                (None, Some(h)) => (h / aspect, h),
                (None, None) => (intrinsic_w, intrinsic_h),
            }
        }
        _ => (intrinsic_w, intrinsic_h),
    }
}

/// Resolve a single token of `background-position` for one axis to a
/// pixel offset within the background area. Accepts the per-axis
/// keywords (`left`/`right` map to 0% / 100% on the x axis,
/// `top`/`bottom` to 0% / 100% on the y axis, `center` to 50%) as
/// well as `<length>` and `<percentage>`. The CSS rule is "anchor
/// point of the image equals anchor point of the box" expressed as
/// `(box - tile) * percent + length_offset`.
fn resolve_bg_position_axis(token: &str, box_size: f32, tile_size: f32, is_x: bool) -> f32 {
    let t = token.trim().to_ascii_lowercase();
    let percent: Option<f32> = match t.as_str() {
        "left" if is_x => Some(0.0),
        "right" if is_x => Some(100.0),
        "top" if !is_x => Some(0.0),
        "bottom" if !is_x => Some(100.0),
        "center" => Some(50.0),
        _ => None,
    };
    if let Some(p) = percent {
        return (box_size - tile_size) * p / 100.0;
    }
    if let Some(stripped) = t.strip_suffix('%') {
        if let Ok(p) = stripped.trim().parse::<f32>() {
            return (box_size - tile_size) * p / 100.0;
        }
    }
    let numeric = t.strip_suffix("px").unwrap_or(&t);
    numeric.trim().parse::<f32>().unwrap_or(0.0)
}

/// Resolve `background-position` to `(off_x, off_y)` in physical
/// pixels relative to the background area's top-left corner. Default
/// is `0% 0%` (top-left).
fn resolve_bg_position(
    value: Option<&str>,
    bg_w: f32,
    bg_h: f32,
    tile_w: f32,
    tile_h: f32,
) -> (f32, f32) {
    let raw = value.map(str::trim).unwrap_or("");
    if raw.is_empty() {
        return (0.0, 0.0);
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    match parts.as_slice() {
        [single] => {
            // CSS: a single value is the x coordinate; y is `center`.
            let x = resolve_bg_position_axis(single, bg_w, tile_w, true);
            let y = resolve_bg_position_axis("center", bg_h, tile_h, false);
            (x, y)
        }
        [a, b] => {
            // Disambiguate axis-only keywords: if either token is a
            // y-axis-only keyword (top/bottom) it must be the y value
            // even when listed first. CSS lets you write
            // `top right` and `right top` interchangeably for the
            // two-keyword form.
            let is_y = |t: &str| matches!(t, "top" | "bottom");
            let is_x = |t: &str| matches!(t, "left" | "right");
            if is_y(&a.to_ascii_lowercase()) || is_x(&b.to_ascii_lowercase()) {
                let y = resolve_bg_position_axis(a, bg_h, tile_h, false);
                let x = resolve_bg_position_axis(b, bg_w, tile_w, true);
                (x, y)
            } else {
                let x = resolve_bg_position_axis(a, bg_w, tile_w, true);
                let y = resolve_bg_position_axis(b, bg_h, tile_h, false);
                (x, y)
            }
        }
        _ => (0.0, 0.0),
    }
}

/// Tile a single image across (a portion of) `bg` according to the
/// repeat mode, given the per-tile size and the initial tile offset
/// (relative to `bg`'s top-left). Returns the absolute on-screen
/// rectangle for every tile that intersects `bg`. For axes that don't
/// repeat, only the seed tile is emitted; for `repeat` / `repeat-x` /
/// `repeat-y` we walk both directions from the seed by `tile_w`/
/// `tile_h` until we leave `bg`.
fn compute_bg_tiles(
    bg: Rect,
    tile_w: f32,
    tile_h: f32,
    off_x: f32,
    off_y: f32,
    repeat: wgpu_html_models::common::css_enums::BackgroundRepeat,
) -> Vec<Rect> {
    use wgpu_html_models::common::css_enums::BackgroundRepeat as BR;
    let mut tiles = Vec::new();
    if tile_w <= 0.0 || tile_h <= 0.0 || bg.w <= 0.0 || bg.h <= 0.0 {
        return tiles;
    }
    let seed_x = bg.x + off_x;
    let seed_y = bg.y + off_y;
    let repeat_x = matches!(repeat, BR::Repeat | BR::RepeatX);
    let repeat_y = matches!(repeat, BR::Repeat | BR::RepeatY);

    let xs: Vec<f32> = if repeat_x {
        let mut start = seed_x;
        while start > bg.x {
            start -= tile_w;
        }
        let mut xs = Vec::new();
        let mut x = start;
        while x < bg.x + bg.w {
            xs.push(x);
            x += tile_w;
        }
        xs
    } else {
        // Skip the single tile entirely if it's outside the bg area.
        if seed_x + tile_w <= bg.x || seed_x >= bg.x + bg.w {
            Vec::new()
        } else {
            vec![seed_x]
        }
    };
    let ys: Vec<f32> = if repeat_y {
        let mut start = seed_y;
        while start > bg.y {
            start -= tile_h;
        }
        let mut ys = Vec::new();
        let mut y = start;
        while y < bg.y + bg.h {
            ys.push(y);
            y += tile_h;
        }
        ys
    } else {
        if seed_y + tile_h <= bg.y || seed_y >= bg.y + bg.h {
            Vec::new()
        } else {
            vec![seed_y]
        }
    };

    for &y in &ys {
        for &x in &xs {
            tiles.push(Rect::new(x, y, tile_w, tile_h));
        }
    }
    tiles
}

/// Top-level: turn `style.background_image` (+ associated longhands)
/// into a [`BackgroundImagePaint`] positioned within `bg`. Returns
/// `None` when there's no `url(...)` reference, the image hasn't
/// finished loading yet, or the resolved tile size collapses to zero.
fn resolve_background_image(style: &Style, bg: Rect) -> Option<BackgroundImagePaint> {
    use wgpu_html_models::common::css_enums::BackgroundRepeat as BR;
    let raw = style.background_image.as_deref()?;
    let url = extract_url(raw)?;
    let img = load_image_url(&url, None, None)?;

    let (tile_w, tile_h) = resolve_bg_size(
        style.background_size.as_deref(),
        img.width,
        img.height,
        bg.w,
        bg.h,
    );
    if tile_w <= 0.0 || tile_h <= 0.0 {
        return None;
    }
    let (off_x, off_y) = resolve_bg_position(
        style.background_position.as_deref(),
        bg.w,
        bg.h,
        tile_w,
        tile_h,
    );
    let repeat = style.background_repeat.clone().unwrap_or(BR::Repeat);
    let tiles = compute_bg_tiles(bg, tile_w, tile_h, off_x, off_y, repeat);
    if tiles.is_empty() {
        return None;
    }
    Some(BackgroundImagePaint {
        image_id: img.image_id,
        data: img.data,
        width: img.width,
        height: img.height,
        tiles,
    })
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Axis-aligned rectangle in physical pixels, top-left origin.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Half-open hit test: a point on the top/left edge is inside,
    /// a point on the bottom/right edge is not.
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// Per-side insets in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Insets {
    pub const fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }
    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

/// One corner's radius. `h` is the horizontal extent, `v` the vertical.
/// Equal components describe a circular corner; otherwise an ellipse.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Radius {
    pub h: f32,
    pub v: f32,
}

impl Radius {
    pub const fn zero() -> Self {
        Self { h: 0.0, v: 0.0 }
    }
    pub const fn circle(r: f32) -> Self {
        Self { h: r, v: r }
    }
    pub fn is_zero(self) -> bool {
        self.h <= 0.0 && self.v <= 0.0
    }
}

/// Per-corner elliptical radii in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadii {
    pub top_left: Radius,
    pub top_right: Radius,
    pub bottom_right: Radius,
    pub bottom_left: Radius,
}

impl CornerRadii {
    pub const fn zero() -> Self {
        Self {
            top_left: Radius::zero(),
            top_right: Radius::zero(),
            bottom_right: Radius::zero(),
            bottom_left: Radius::zero(),
        }
    }
    pub fn any_nonzero(&self) -> bool {
        !self.top_left.is_zero()
            || !self.top_right.is_zero()
            || !self.bottom_right.is_zero()
            || !self.bottom_left.is_zero()
    }
}

/// Per-side border colors, top / right / bottom / left.
#[derive(Debug, Clone, Copy, Default)]
pub struct BorderColors {
    pub top: Option<Color>,
    pub right: Option<Color>,
    pub bottom: Option<Color>,
    pub left: Option<Color>,
}

impl BorderColors {
    pub fn any(&self) -> bool {
        self.top.is_some() || self.right.is_some() || self.bottom.is_some() || self.left.is_some()
    }
}

/// Per-side border styles. `None` means the property was unset; paint
/// treats that as `solid` for sides that have a width and a colour
/// (matching the convention `border: <w> <c>` implies "solid" in
/// CSS source — we already inflate the shorthand to `solid` in the
/// parser for that case, but per-side longhand can leave it unset).
#[derive(Debug, Clone, Default)]
pub struct BorderStyles {
    pub top: Option<BorderStyle>,
    pub right: Option<BorderStyle>,
    pub bottom: Option<BorderStyle>,
    pub left: Option<BorderStyle>,
}

/// One laid-out box. Coordinates are absolute (already translated for
/// every parent on the path); the paint pass just reads them.
#[derive(Debug, Clone)]
pub struct LayoutBox {
    /// Includes margin. Used for sibling stacking.
    pub margin_rect: Rect,
    /// The rect a background / border would paint (margin excluded).
    pub border_rect: Rect,
    /// The rect children are laid out into (margin + border + padding excluded).
    pub content_rect: Rect,
    /// Resolved background color, if any.
    pub background: Option<Color>,
    /// Rectangle the background fills. Driven by `background-clip`:
    /// `border-box` (default) → equal to `border_rect`; `padding-box` →
    /// border-rect inset by border thickness; `content-box` → equal to
    /// `content_rect`.
    pub background_rect: Rect,
    /// Per-corner radii to use when painting the background — already
    /// reduced from the outer radii to match `background_rect`.
    pub background_radii: CornerRadii,
    /// Per-side border thickness, in physical pixels.
    pub border: Insets,
    /// Per-side border color. `None` for a side means that edge is
    /// skipped during paint (we don't track foreground color yet, so
    /// there's no spec-default fallback).
    pub border_colors: BorderColors,
    /// Per-side border style. Currently honoured for `solid`, `dashed`,
    /// `dotted`, `none`, `hidden`. Other values render as solid.
    pub border_styles: BorderStyles,
    /// Per-corner radii. Currently parsed and laid out but **not yet
    /// rendered** (paint emits straight-edge quads).
    pub border_radius: CornerRadii,
    pub kind: BoxKind,
    /// For text leaves (`BoxKind::Text`): the shaped + atlas-packed
    /// glyph run, in run-relative pixel coordinates with `(0, 0)` at
    /// the line-box top-left. `None` when no font is registered or
    /// the text was empty.
    pub text_run: Option<ShapedRun>,
    /// For text leaves: the resolved foreground color used when paint
    /// emits glyph quads. Defaults to opaque black if unset.
    pub text_color: Option<Color>,
    /// CSS `text-decoration-line`s active on this box (parsed from
    /// the `text-decoration` shorthand). Painted as solid quads at
    /// the appropriate vertical offset for text leaves.
    pub text_decorations: Vec<TextDecorationLine>,
    /// Effective `overflow` value (after `overflow-x` / `overflow-y`
    /// resolution). `Visible` is the no-op default; anything else
    /// asks the paint pass to clip descendants to this box's
    /// padding-box rect.
    ///
    /// v1 collapses the two axes: when either axis is non-`Visible`,
    /// both axes clip together. Independent per-axis clipping is a
    /// follow-up.
    pub overflow: Overflow,
    /// Decoded image data for `<img>` elements. `None` for non-image
    /// boxes. The `Arc` allows cheap cloning through the display list.
    pub image: Option<ImageData>,
    /// Pre-computed CSS `background-image` paint info, if the element
    /// has one. Carries the decoded RGBA texture data plus a list of
    /// already-positioned tile rectangles (one for `no-repeat`, many
    /// for `repeat` modes). The painter just iterates the tiles.
    pub background_image: Option<BackgroundImagePaint>,
    pub children: Vec<LayoutBox>,
}

/// Decoded RGBA image ready for GPU upload. For animated sources
/// (GIF), `image_id` and `data` always point at the currently-active
/// frame as picked by [`current_frame`]; the full frame list is in
/// `frames` so callers that need to know the asset is animated can
/// inspect it.
#[derive(Debug, Clone)]
pub struct ImageData {
    /// Unique identifier — hashes (URL, declared size, frame index).
    /// For still images the frame component is `None`; for animated,
    /// it's the current frame's index. Drives the renderer's GPU
    /// texture cache.
    pub image_id: u64,
    /// RGBA8 pixel data, `width × height × 4` bytes — *current frame*
    /// for animated assets.
    pub data: std::sync::Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    /// Per-frame data for animated assets. `None` for still images.
    /// The list is shared via `Arc` so cheap to clone through layout.
    pub frames: Option<std::sync::Arc<Vec<ImageFrame>>>,
}

/// One frame of an animated image, fully resized to the requested
/// declared dimensions and ready for upload as a standalone GPU
/// texture (each frame's `image_id` is unique).
#[derive(Debug, Clone)]
pub struct ImageFrame {
    pub image_id: u64,
    pub data: std::sync::Arc<Vec<u8>>,
    /// Display duration, milliseconds. Clamped to ≥10 ms during
    /// decode to defend against pathological GIFs that report 0.
    pub delay_ms: u32,
}

/// Pre-computed CSS `background-image` paint metadata. The texture
/// is uploaded once per `image_id`; the painter emits one image quad
/// per entry in `tiles`.
#[derive(Debug, Clone)]
pub struct BackgroundImagePaint {
    /// Same `image_id` scheme as [`ImageData`] — keys the renderer's
    /// GPU texture cache.
    pub image_id: u64,
    /// Decoded RGBA8 bytes for the source image (intrinsic size).
    pub data: std::sync::Arc<Vec<u8>>,
    /// Texture upload size (intrinsic dimensions of the decoded
    /// image). Each tile rect is drawn at its own size; UVs map the
    /// full `[0,1]²` to the rect, so the renderer stretches as
    /// needed for `cover` / `contain` / explicit lengths.
    pub width: u32,
    pub height: u32,
    /// On-screen rectangles to paint into. Already positioned in
    /// absolute physical pixels and filtered to those that overlap
    /// the box's `background_rect`.
    pub tiles: Vec<Rect>,
}

/// Single decoration line drawn over / under / through a text run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecorationLine {
    Underline,
    LineThrough,
    Overline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxKind {
    /// A regular block element.
    Block,
    /// A text leaf. The shaped run lives in `LayoutBox::text_run`.
    Text,
}

impl LayoutBox {
    /// Index path from `self` to the deepest descendant whose
    /// `border_rect` contains `point`. An empty path means `self` is the
    /// deepest match. `None` if the point is outside `self`.
    ///
    /// The layout tree mirrors the source element tree 1:1, so this
    /// path can be applied to a [`Tree`] / [`Node`] to navigate to the
    /// corresponding element.
    pub fn hit_path(&self, point: (f32, f32)) -> Option<Vec<usize>> {
        let (x, y) = point;
        if !self.border_rect.contains(x, y) {
            return None;
        }
        let mut path: Vec<usize> = Vec::new();
        collect_hit_path(self, x, y, &mut path);
        Some(path)
    }

    /// Hit-test the layout at `point` and return a mutable reference
    /// to the matching element node in `tree`. Use this to read or
    /// modify the source element (style, text, attributes, etc.).
    /// Returns `None` if the point is outside `self` or the tree has
    /// no root.
    ///
    /// `tree` must be the same tree this layout was produced from; we
    /// rely on the layout's child structure mirroring the element
    /// tree's child structure 1:1.
    ///
    /// On overlap, children are walked last-to-first so the topmost
    /// (last-painted) hit wins.
    pub fn find_element_from_point<'a>(
        &self,
        tree: &'a mut Tree,
        point: (f32, f32),
    ) -> Option<&'a mut Node> {
        let path = self.hit_path(point)?;
        tree.root.as_mut()?.at_path_mut(&path)
    }

    /// Like [`Self::find_element_from_point`] but returns the entire
    /// ancestor chain of element nodes from the deepest hit up to (and
    /// including) the tree root. Empty when the point is outside or
    /// the tree has no root.
    ///
    /// Soundness inherits from [`Node::ancestry_at_path_mut`]: the
    /// returned `&mut` references alias into nested subtrees of the
    /// same borrow, so two of them must never be dereferenced at the
    /// same time.
    pub fn find_elements_from_point<'a>(
        &self,
        tree: &'a mut Tree,
        point: (f32, f32),
    ) -> Vec<&'a mut Node> {
        let Some(path) = self.hit_path(point) else {
            return Vec::new();
        };
        let Some(root) = tree.root.as_mut() else {
            return Vec::new();
        };
        root.ancestry_at_path_mut(&path)
    }
}

fn collect_hit_path(b: &LayoutBox, x: f32, y: f32, path: &mut Vec<usize>) {
    for (i, child) in b.children.iter().enumerate().rev() {
        if child.border_rect.contains(x, y) {
            path.push(i);
            collect_hit_path(child, x, y, path);
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Lay the cascaded tree out into a viewport of `viewport_w × viewport_h`
/// physical pixels, using `text_ctx` to shape any text leaves. The
/// returned root box's `margin_rect` covers the viewport.
pub fn layout_with_text(
    tree: &CascadedTree,
    text_ctx: &mut TextContext,
    viewport_w: f32,
    viewport_h: f32,
    scale: f32,
) -> Option<LayoutBox> {
    let root = tree.root.as_ref()?;
    let mut ctx = Ctx {
        viewport_w,
        viewport_h,
        scale,
        text: TextCtx { ctx: text_ctx },
    };
    Some(layout_block(
        root,
        0.0,
        0.0,
        viewport_w,
        viewport_h,
        BlockOverrides::default(),
        &mut ctx,
    ))
}

/// Compatibility wrapper for callers that don't render text. Builds a
/// throw-away `TextContext` (no fonts registered → text leaves shape
/// to zero size) at scale 1.0.
pub fn layout(tree: &CascadedTree, viewport_w: f32, viewport_h: f32) -> Option<LayoutBox> {
    let mut text_ctx = TextContext::new(64);
    layout_with_text(tree, &mut text_ctx, viewport_w, viewport_h, 1.0)
}

pub(crate) struct Ctx<'a> {
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub scale: f32,
    pub text: TextCtx<'a>,
}

/// Wrapper so `Ctx` can borrow a `&mut TextContext` without forcing
/// every caller to thread a lifetime through. Passes shaping calls
/// through to the underlying context.
pub(crate) struct TextCtx<'a> {
    pub ctx: &'a mut TextContext,
}

// ---------------------------------------------------------------------------
// Block layout
// ---------------------------------------------------------------------------

/// Optional caller-supplied content-box overrides for the recursive
/// block layout. Used by the flex layer to drive an item to a
/// pre-computed main / cross extent without mutating its style.
///
/// When a field is `Some`, that axis is sized exactly to the value
/// (already in *content-box* pixels — `box-sizing` and `min-*` /
/// `max-*` clamping have already been applied by the caller).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BlockOverrides {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

/// Lay out one node as a block at `(origin_x, origin_y)` inside the
/// given container, with optional content-box overrides. Used by the
/// flex layer to drive an item to a precomputed main / cross extent.
pub(crate) fn layout_block_at_with(
    node: &CascadedNode,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    container_h: f32,
    overrides: BlockOverrides,
    ctx: &mut Ctx,
) -> LayoutBox {
    layout_block(
        node,
        origin_x,
        origin_y,
        container_w,
        container_h,
        overrides,
        ctx,
    )
}

fn layout_block(
    node: &CascadedNode,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    container_h: f32,
    overrides: BlockOverrides,
    ctx: &mut Ctx,
) -> LayoutBox {
    // `display: none` removes the element and its subtree from the
    // box tree entirely. Returning a zero-sized box means the parent
    // contributes nothing for this child — no painting, no
    // descendants, no advance.
    if matches!(node.style.display, Some(Display::None)) {
        return empty_box(origin_x, origin_y);
    }

    // Text leaves: shape with the first registered font (T3
    // simplification — proper font-family inheritance lands in T4).
    // If no font is registered, the run is `None` and the box has
    // zero size, matching pre-text behaviour.
    if let Element::Text(s) = &node.element {
        // Block-flow text leaf — uses the parent's content width as
        // the soft-wrap budget so paragraphs that are *direct* text
        // children of a block (rare, but legal) wrap rather than
        // overflow.
        let (box_, _w, _h, _ascent) = make_text_leaf(
            s,
            &node.style,
            origin_x,
            origin_y,
            Some(container_w),
            ctx,
        );
        return box_;
    }

    // <img> replaced element: load the image and use its intrinsic
    // dimensions (or the HTML width/height attributes) as the
    // content size. CSS width/height override below as usual.
    let img_data = if let Element::Img(img) = &node.element {
        load_image(img)
    } else {
        None
    };

    let style = &node.style;

    let mut margin = resolve_insets_margin(style, container_w, ctx);
    let border = Insets {
        top: length::resolve(style.border_top_width.as_ref(), container_w, ctx).unwrap_or(0.0),
        right: length::resolve(style.border_right_width.as_ref(), container_w, ctx).unwrap_or(0.0),
        bottom: length::resolve(style.border_bottom_width.as_ref(), container_w, ctx)
            .unwrap_or(0.0),
        left: length::resolve(style.border_left_width.as_ref(), container_w, ctx).unwrap_or(0.0),
    };
    let padding = resolve_insets_padding(style, container_w, ctx);

    let box_sizing = style.box_sizing.clone().unwrap_or(BoxSizing::ContentBox);

    // Inner width: caller-supplied override, then explicit `width`,
    // then fill the parent. Min/max clamping is applied to the
    // cascade-derived size; overrides are taken at face value (the
    // flex algorithm has already clamped).
    let frame_w = margin.horizontal() + border.horizontal() + padding.horizontal();
    let inner_width = match overrides.width {
        Some(w) => w,
        None => {
            let base = match length::resolve(style.width.as_ref(), container_w, ctx) {
                Some(specified) => match box_sizing {
                    BoxSizing::ContentBox => specified,
                    BoxSizing::BorderBox => {
                        (specified - border.horizontal() - padding.horizontal()).max(0.0)
                    }
                },
                None => {
                    // Replaced elements (<img>) use intrinsic width
                    // when no CSS width is specified.
                    if let Some(ref id) = img_data {
                        id.width as f32
                    } else {
                        (container_w - frame_w).max(0.0)
                    }
                }
            };
            clamp_axis(
                base,
                style.min_width.as_ref(),
                style.max_width.as_ref(),
                container_w,
                border.horizontal() + padding.horizontal(),
                box_sizing.clone(),
                ctx,
            )
        }
    };

    // Auto horizontal margins on a block with an explicit width center
    // (or push) the block within its container, matching the standard
    // CSS `margin: 0 auto` idiom. Only kicks in when there's a
    // non-zero `width` and free space remains.
    //
    // Skipped when `overrides` is non-default — that signals the call
    // came from the flex layer, which handles its own auto-margin
    // redistribution before placing items. Running the block-level
    // pass on top would double-consume free space and push flex items
    // off the line.
    let from_flex = overrides.width.is_some() || overrides.height.is_some();
    let auto_left = is_auto_margin(&style.margin_left, &style.margin);
    let auto_right = is_auto_margin(&style.margin_right, &style.margin);
    let has_explicit_width = style.width.is_some();
    if !from_flex && has_explicit_width && (auto_left || auto_right) {
        let used =
            margin.horizontal() + border.horizontal() + padding.horizontal() + inner_width;
        let free = (container_w - used).max(0.0);
        match (auto_left, auto_right) {
            (true, true) => {
                let half = free * 0.5;
                margin.left += half;
                margin.right += half;
            }
            (true, false) => margin.left += free,
            (false, true) => margin.right += free,
            (false, false) => {}
        }
    }

    // Lay out children inside the content box, dispatching on display.
    let content_x = origin_x + margin.left + border.left + padding.left;
    let content_y_top = origin_y + margin.top + border.top + padding.top;

    // Pre-resolve an explicit height (used for `align-items: stretch`
    // and as the override target). Caller-supplied override wins; then
    // an explicit `height` style; otherwise unknown until children lay
    // out. Min/max clamping happens after content height is known
    // (when no explicit height) so it can extend a too-short block.
    let inner_height_explicit = match overrides.height {
        Some(h) => Some(h),
        None => {
            // Replaced elements use intrinsic height when no CSS
            // height is specified.
            let css_h = length::resolve(style.height.as_ref(), container_h, ctx);
            let effective_h = css_h.or_else(|| img_data.as_ref().map(|id| id.height as f32));
            effective_h.map(|specified| {
                let raw = match box_sizing {
                    BoxSizing::ContentBox => specified,
                    BoxSizing::BorderBox => {
                        (specified - border.vertical() - padding.vertical()).max(0.0)
                    }
                };
                clamp_axis(
                    raw,
                    style.min_height.as_ref(),
                    style.max_height.as_ref(),
                    container_h,
                    border.vertical() + padding.vertical(),
                    box_sizing.clone(),
                    ctx,
                )
            })
        }
    };

    let display = style.display.clone().unwrap_or(Display::Block);
    let (children, content_h_from_children) = match display {
        Display::Flex | Display::InlineFlex => {
            let (kids, _content_w_used, content_h_used) = flex::layout_flex_children(
                node,
                style,
                content_x,
                content_y_top,
                inner_width,
                inner_height_explicit,
                ctx,
            );
            (kids, content_h_used)
        }
        Display::Grid | Display::InlineGrid => {
            let (kids, _content_w_used, content_h_used) = grid::layout_grid_children(
                node,
                style,
                content_x,
                content_y_top,
                inner_width,
                inner_height_explicit,
                ctx,
            );
            (kids, content_h_used)
        }
        _ => {
            // Inline formatting context: when every child of this
            // block is inline-level (text, <strong>, <em>, …), pack
            // them onto a single line box at the parent's content
            // origin. Otherwise fall back to the block flow that
            // stacks children vertically.
            if all_children_inline_level(node) {
                let (kids, _w_used, h_used) = layout_inline_block_children(
                    node,
                    content_x,
                    content_y_top,
                    inner_width,
                    ctx,
                );
                (kids, h_used)
            } else {
                let mut children = Vec::with_capacity(node.children.len());
                let mut cursor = 0.0_f32;
                for child in &node.children {
                    let child_box = layout_block(
                        child,
                        content_x,
                        content_y_top + cursor,
                        inner_width,
                        container_h,
                        BlockOverrides::default(),
                        ctx,
                    );
                    cursor += child_box.margin_rect.h;
                    children.push(child_box);
                }
                (children, cursor)
            }
        }
    };

    // Final inner height: explicit / override wins; otherwise content
    // size, then clamped by min/max (so a too-short content can be
    // extended by `min-height` and a too-tall content by `max-height`).
    let inner_height = match inner_height_explicit {
        Some(h) => h,
        None => clamp_axis(
            content_h_from_children,
            style.min_height.as_ref(),
            style.max_height.as_ref(),
            container_h,
            border.vertical() + padding.vertical(),
            box_sizing.clone(),
            ctx,
        ),
    };

    // Compose the rects.
    let border_rect = Rect::new(
        origin_x + margin.left,
        origin_y + margin.top,
        border.horizontal() + padding.horizontal() + inner_width,
        border.vertical() + padding.vertical() + inner_height,
    );
    let content_rect = Rect::new(content_x, content_y_top, inner_width, inner_height);
    let margin_rect = Rect::new(
        origin_x,
        origin_y,
        margin.horizontal() + border_rect.w,
        margin.vertical() + border_rect.h,
    );

    let background = style.background_color.as_ref().and_then(resolve_color);
    let border_colors = BorderColors {
        top: style.border_top_color.as_ref().and_then(resolve_color),
        right: style.border_right_color.as_ref().and_then(resolve_color),
        bottom: style.border_bottom_color.as_ref().and_then(resolve_color),
        left: style.border_left_color.as_ref().and_then(resolve_color),
    };
    let border_styles = BorderStyles {
        top: style.border_top_style.clone(),
        right: style.border_right_style.clone(),
        bottom: style.border_bottom_style.clone(),
        left: style.border_left_style.clone(),
    };
    let resolve_corner = |h: Option<&CssLength>, v: Option<&CssLength>, ctx: &mut Ctx| -> Radius {
        let h_px = length::resolve(h, container_w, ctx).unwrap_or(0.0).max(0.0);
        // Vertical resolves against the box height when known, else
        // viewport height (Ctx). When the v field is unset, fall back
        // to the same value as h (CSS default).
        let v_px = match v {
            Some(_) => length::resolve(v, container_h, ctx).unwrap_or(0.0).max(0.0),
            None => h_px,
        };
        Radius { h: h_px, v: v_px }
    };
    let mut border_radius = CornerRadii {
        top_left: resolve_corner(
            style.border_top_left_radius.as_ref(),
            style.border_top_left_radius_v.as_ref(),
            ctx,
        ),
        top_right: resolve_corner(
            style.border_top_right_radius.as_ref(),
            style.border_top_right_radius_v.as_ref(),
            ctx,
        ),
        bottom_right: resolve_corner(
            style.border_bottom_right_radius.as_ref(),
            style.border_bottom_right_radius_v.as_ref(),
            ctx,
        ),
        bottom_left: resolve_corner(
            style.border_bottom_left_radius.as_ref(),
            style.border_bottom_left_radius_v.as_ref(),
            ctx,
        ),
    };
    clamp_corner_radii(&mut border_radius, border_rect.w, border_rect.h);

    let (background_rect, background_radii) = compute_background_box(
        style,
        border_rect,
        content_rect,
        border,
        padding,
        &border_radius,
    );

    let background_image = resolve_background_image(style, background_rect);

    LayoutBox {
        margin_rect,
        border_rect,
        content_rect,
        background,
        background_rect,
        background_radii,
        border,
        border_colors,
        border_styles,
        border_radius,
        kind: BoxKind::Block,
        text_run: None,
        text_color: None,
        text_decorations: Vec::new(),
        overflow: effective_overflow(style),
        image: img_data,
        background_image,
        children,
    }
}

/// Collapse `overflow` / `overflow-x` / `overflow-y` to a single
/// effective value. v1 doesn't honour per-axis hidden / visible
/// mismatch — when either axis is non-`Visible`, both axes clip.
fn effective_overflow(style: &Style) -> Overflow {
    let pick = style
        .overflow_x
        .as_ref()
        .filter(|v| !matches!(v, Overflow::Visible))
        .or_else(|| {
            style
                .overflow_y
                .as_ref()
                .filter(|v| !matches!(v, Overflow::Visible))
        })
        .or(style.overflow.as_ref());
    pick.cloned().unwrap_or(Overflow::Visible)
}

/// Shape a text-node string against the current `TextContext`. Reads
/// `font-size` and `line-height` from the cascaded style (which the
/// inheritance pass filled in from the nearest ancestor that set
/// them); falls back to 16px / 1.25× when unset. Picks the first
/// registered font for now — proper `font-family` matching is T4.
fn shape_text_run(
    text: &str,
    style: &Style,
    max_width_px: Option<f32>,
    ctx: &mut Ctx,
) -> (Option<ShapedRun>, f32, f32, f32) {
    if text.is_empty() {
        return (None, 0.0, 0.0, 0.0);
    }

    // CSS `white-space: normal` (the default) collapses runs of
    // whitespace — including embedded `\n` from raw HTML source —
    // into a single space. Apply that first so cosmic-text doesn't
    // see a leading newline and produce an empty first layout run
    // (we only consume `layout_runs().next()` until line breaking
    // lands in T7). Without this, a text leaf like "\n    Plain, "
    // shapes to width 0 and paints nothing.
    let collapsed = collapse_whitespace(text);
    if collapsed.is_empty() {
        return (None, 0.0, 0.0, 0.0);
    }

    // `text-transform` re-cases the *visible* text before shaping. Do
    // it once here so `font-feature` style ligatures still apply to
    // the transformed forms.
    let transformed = apply_text_transform(&collapsed, style.text_transform.as_ref());
    let display_text: &str = match transformed.as_ref() {
        Some(s) => s.as_str(),
        None => collapsed.as_str(),
    };

    // Family / weight / style come from the cascaded style. The text
    // node itself has no rules applied, but cascade inheritance has
    // already pulled these from the nearest ancestor that set them
    // (or from UA defaults for `<b>`, `<strong>`, `<em>`, …).
    let families = parse_family_list(style.font_family.as_deref());
    let family_refs: Vec<&str> = families.iter().map(String::as_str).collect();
    let weight = font_weight_value(style.font_weight.as_ref());
    let axis = font_style_axis(style.font_style.as_ref());

    let Some(handle) = ctx.text.ctx.pick_font(&family_refs, weight, axis) else {
        return (None, 0.0, 0.0, 0.0);
    };

    let size_css = font_size_px(style).unwrap_or(16.0);
    let line_h_css = line_height_px(style, size_css);
    let size_px = size_css * ctx.scale;
    let line_height = line_h_css * ctx.scale;
    let letter_spacing = letter_spacing_px(style, size_css) * ctx.scale;
    let color = style
        .color
        .as_ref()
        .and_then(resolve_color)
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
    match ctx.text.ctx.shape_and_pack(
        display_text,
        handle,
        size_px,
        line_height,
        letter_spacing,
        weight,
        axis,
        max_width_px.map(|w| w * ctx.scale),
        color,
    ) {
        Some(run) => {
            let w = run.width;
            let h = run.height;
            let a = run.ascent;
            (Some(run), w, h, a)
        }
        None => (None, 0.0, 0.0, 0.0),
    }
}

/// Resolve `letter-spacing` to CSS pixels. `Px` is literal; `Em` /
/// `Rem` multiply against `font_size`. Anything else (percent,
/// unset) is treated as zero.
fn letter_spacing_px(style: &Style, font_size: f32) -> f32 {
    use wgpu_html_models::common::css_enums::CssLength;
    match style.letter_spacing.as_ref() {
        Some(CssLength::Px(v)) => *v,
        Some(CssLength::Em(v)) | Some(CssLength::Rem(v)) => v * font_size,
        _ => 0.0,
    }
}

/// Apply CSS `text-transform`. Returns `None` when no transform is
/// set (caller can keep using the original `&str` without an extra
/// allocation), otherwise the transformed string.
fn apply_text_transform(
    text: &str,
    tt: Option<&wgpu_html_models::common::css_enums::TextTransform>,
) -> Option<String> {
    use wgpu_html_models::common::css_enums::TextTransform as Tt;
    match tt {
        Some(Tt::Uppercase) => Some(text.to_uppercase()),
        Some(Tt::Lowercase) => Some(text.to_lowercase()),
        Some(Tt::Capitalize) => Some(capitalize_words(text)),
        // None / FullWidth / FullSizeKana — pass through unchanged.
        _ => None,
    }
}

/// CSS `white-space: normal` whitespace collapsing: every run of
/// ASCII / Unicode whitespace (including `\n`, `\t`, `\r`) becomes a
/// single ASCII space. Leading and trailing whitespace are preserved
/// (as a single space) — line-boundary trimming is part of the
/// inline pass, not the shaper. Returns an owned `String` because
/// the typical input differs from the output.
fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

/// `text-transform: capitalize` — uppercase the first letter of each
/// run of non-whitespace characters; pass everything else through.
fn capitalize_words(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for ch in s.chars() {
        if ch.is_whitespace() {
            at_word_start = true;
            out.push(ch);
        } else if at_word_start {
            for u in ch.to_uppercase() {
                out.push(u);
            }
            at_word_start = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Zero-area `LayoutBox` for elements whose effective `display` is
/// `none`. The parent treats it as contributing no width / height /
/// children, so the subtree disappears from the box tree completely.
fn empty_box(origin_x: f32, origin_y: f32) -> LayoutBox {
    let r = Rect::new(origin_x, origin_y, 0.0, 0.0);
    LayoutBox {
        margin_rect: r,
        border_rect: r,
        content_rect: r,
        background: None,
        background_rect: r,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Block,
        text_run: None,
        text_color: None,
        text_decorations: Vec::new(),
        overflow: Overflow::Visible,
        image: None,
        background_image: None,
        children: Vec::new(),
    }
}

/// Build a text-leaf `LayoutBox` for an `Element::Text`. Used both
/// from block-flow (text as a degenerate "block" of one line) and
/// from the inline-formatting context.
fn make_text_leaf(
    text: &str,
    style: &Style,
    origin_x: f32,
    origin_y: f32,
    max_width_px: Option<f32>,
    ctx: &mut Ctx,
) -> (LayoutBox, f32, f32, f32) {
    let (run, w, h, ascent) = shape_text_run(text, style, max_width_px, ctx);
    let text_color = style
        .color
        .as_ref()
        .and_then(resolve_color)
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let decorations = resolve_text_decorations(style);
    let r = Rect::new(origin_x, origin_y, w, h);
    let box_ = LayoutBox {
        margin_rect: r,
        border_rect: r,
        content_rect: r,
        background: None,
        background_rect: r,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Text,
        text_run: run,
        text_color: Some(text_color),
        text_decorations: decorations,
        overflow: Overflow::Visible,
        image: None,
        background_image: None,
        children: Vec::new(),
    };
    (box_, w, h, ascent)
}

/// Parse the raw `text-decoration` shorthand string into the set of
/// active decoration lines. Whitespace-separated tokens; `none`
/// resets any previously-collected lines.
fn parse_text_decorations(s: &str) -> Vec<TextDecorationLine> {
    let mut out = Vec::new();
    for tok in s.split_ascii_whitespace() {
        match tok.to_ascii_lowercase().as_str() {
            "underline" => out.push(TextDecorationLine::Underline),
            "line-through" => out.push(TextDecorationLine::LineThrough),
            "overline" => out.push(TextDecorationLine::Overline),
            "none" => out.clear(),
            // Other tokens (colour names, "wavy", "solid", …) — the
            // value parser hands them along as part of the same raw
            // string; we ignore everything but the line keywords for
            // now.
            _ => {}
        }
    }
    out
}

fn resolve_text_decorations(style: &Style) -> Vec<TextDecorationLine> {
    style
        .text_decoration
        .as_deref()
        .map(parse_text_decorations)
        .unwrap_or_default()
}

/// Inline-level test: an element whose default formatting puts it on
/// a line with its siblings. Honours an explicit `display` override
/// (`inline / inline-block / inline-flex`) but otherwise defaults by
/// HTML element kind. Block-by-default elements like `<div>` /
/// `<p>` / headings are *not* inline-level.
fn is_inline_level(node: &CascadedNode) -> bool {
    if let Some(d) = node.style.display.as_ref() {
        use wgpu_html_models::common::css_enums::Display::*;
        return matches!(d, Inline | InlineBlock | InlineFlex);
    }
    matches!(
        &node.element,
        Element::Text(_)
            | Element::Span(_)
            | Element::A(_)
            | Element::Strong(_)
            | Element::B(_)
            | Element::Em(_)
            | Element::I(_)
            | Element::U(_)
            | Element::S(_)
            | Element::Small(_)
            | Element::Mark(_)
            | Element::Code(_)
            | Element::Kbd(_)
            | Element::Samp(_)
            | Element::Var(_)
            | Element::Abbr(_)
            | Element::Cite(_)
            | Element::Dfn(_)
            | Element::Sub(_)
            | Element::Sup(_)
            | Element::Time(_)
            | Element::Br(_)
            | Element::Wbr(_)
            | Element::Bdi(_)
            | Element::Bdo(_)
            | Element::Ins(_)
            | Element::Del(_)
            | Element::Label(_)
            | Element::Output(_)
            | Element::Data(_)
            | Element::Ruby(_)
            | Element::Rt(_)
            | Element::Rp(_)
    )
}

/// True when every child of `node` is an inline-level box, so the
/// whole block becomes one inline formatting context. Empty parents
/// stay in block-flow (with zero content) — they have nothing to
/// flow.
fn all_children_inline_level(node: &CascadedNode) -> bool {
    !node.children.is_empty() && node.children.iter().all(is_inline_level)
}

/// Result of laying out one inline-level subtree at a temporary
/// origin. The caller composes these on a horizontal cursor and
/// re-aligns each on the line's baseline by adjusting `box_.y` after
/// the fact.
///
/// Currently unused — the IFC switched to the rich-text paragraph
/// path (`layout_inline_paragraph`), which feeds cosmic-text's
/// `set_rich_text` and never goes through `layout_inline_subtree`.
/// The struct is left in place so future work that needs to compose
/// layouts horizontally on a custom path (e.g. inline-block content)
/// can re-use it without reinventing the shape.
#[allow(dead_code)]
struct InlineLayout {
    box_: LayoutBox,
    width: f32,
    ascent: f32,
    descent: f32,
}

/// Lay out one inline-level subtree starting at `(origin_x, origin_y)`.
/// Text leaves shape into a single `BoxKind::Text` with the run +
/// foreground colour. Inline elements recurse, position their
/// children on a baseline (so a `<small>` and a `<strong>` flow on
/// the same line), and wrap the result in a `BoxKind::Block` whose
/// background — if any — covers the inline element's content extent
/// (this is what makes `<mark>` paintable).
///
/// Currently unused — see the note on [`InlineLayout`].
#[allow(dead_code)]
fn layout_inline_subtree(
    node: &CascadedNode,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    ctx: &mut Ctx,
) -> InlineLayout {
    // `display: none` removes the inline subtree from the line —
    // zero width, zero ascent / descent, no paintable box.
    if matches!(node.style.display, Some(Display::None)) {
        return InlineLayout {
            box_: empty_box(origin_x, origin_y),
            width: 0.0,
            ascent: 0.0,
            descent: 0.0,
        };
    }

    if let Element::Text(s) = &node.element {
        // IFC text leaf: shape on a single line. Wrapping per-leaf
        // against the *remaining* container width turns short late
        // leaves (text after a `<mark>` etc.) into vertical columns
        // of single characters when the cursor's already near the
        // right edge — each glyph is wider than the leftover width
        // so cosmic-text breaks at every character. Real cross-leaf
        // paragraph wrapping — where the IFC itself stacks lines and
        // the next leaf moves onto a fresh line — is the proper fix
        // and is tracked as a T7 follow-up. The trade-off today: a
        // very long single-leaf paragraph will overflow to the right
        // instead of breaking. Block-flow text leaves (rare;
        // direct text under a block) still get container-width wrap
        // via `layout_block`'s text path.
        let (box_, w, h, ascent) = make_text_leaf(
            s,
            &node.style,
            origin_x,
            origin_y,
            None,
            ctx,
        );
        let _ = container_w; // eliminated — IFC doesn't wrap leaves itself
        let descent = (h - ascent).max(0.0);
        return InlineLayout {
            box_,
            width: w,
            ascent,
            descent,
        };
    }

    // Inline element: walk children at a horizontal cursor, then
    // baseline-align them inside this element.
    let mut cursor_x = 0.0_f32;
    let mut max_ascent = 0.0_f32;
    let mut max_descent = 0.0_f32;
    let mut child_layouts: Vec<InlineLayout> = Vec::new();
    for child in &node.children {
        let cl = layout_inline_subtree(
            child,
            origin_x + cursor_x,
            origin_y,
            (container_w - cursor_x).max(0.0),
            ctx,
        );
        if cl.ascent > max_ascent {
            max_ascent = cl.ascent;
        }
        if cl.descent > max_descent {
            max_descent = cl.descent;
        }
        cursor_x += cl.width;
        child_layouts.push(cl);
    }

    let line_h = max_ascent + max_descent;
    let baseline_y = origin_y + max_ascent;
    let mut final_children: Vec<LayoutBox> = Vec::with_capacity(child_layouts.len());
    for cl in child_layouts {
        let cur_top = cl.box_.margin_rect.y;
        let target_top = baseline_y - cl.ascent;
        let dy = target_top - cur_top;
        let mut b = cl.box_;
        translate_box_y_in_place(&mut b, dy);
        final_children.push(b);
    }

    let bg = node.style.background_color.as_ref().and_then(resolve_color);
    let r = Rect::new(origin_x, origin_y, cursor_x, line_h);
    let box_ = LayoutBox {
        margin_rect: r,
        border_rect: r,
        content_rect: r,
        background: bg,
        background_rect: r,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Block,
        text_run: None,
        text_color: None,
        // Decorations live on text leaves (cascade inheritance has
        // already propagated `text-decoration` down to every text
        // descendant). The inline wrapper itself draws nothing.
        text_decorations: Vec::new(),
        overflow: Overflow::Visible,
        image: None,
        background_image: None,
        children: final_children,
    };
    InlineLayout {
        box_,
        width: cursor_x,
        ascent: max_ascent,
        descent: max_descent,
    }
}

/// Lay out a block's inline-level children as a stack of line boxes
/// at `(origin_x, origin_y)`. Returns the final children (already
/// positioned absolutely) plus the paragraph's used width (max line
/// width) and height (sum of line heights).
///
/// Behaviour:
/// - **Single text-leaf child** — shapes the leaf with cosmic-text's
///   soft-wrap (`Some(container_w)`) so the paragraph breaks at
///   actual word boundaries inside the run.
/// - **Multiple inline children** — greedy element-boundary wrap.
///   Each child is shaped on its own line at a scratch origin; the
///   IFC accumulates them onto the current line and rolls over to
///   a new line when `cursor_x + child.width > container_w`. Breaks
///   land between elements (a `<strong>` either fits on the line or
///   moves whole to the next line); breaks *inside* a multi-leaf
///   sentence are still pending — that's the cross-leaf rich-text
///   shape pass tracked under T7.
///
/// Each completed line is baseline-aligned independently (its max
/// ascent over its own children) and shifted by `horizontal_align_offset`
/// for `text-align`. `justify` falls through to `left`.
fn layout_inline_block_children(
    node: &CascadedNode,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
    let text_align = node.style.text_align.as_ref();

    // Single-text-leaf fast path: cosmic-text's word-boundary wrap
    // gives the right answer for plain paragraphs.
    if node.children.len() == 1 {
        if let Element::Text(s) = &node.children[0].element {
            let child_style = &node.children[0].style;
            let (box_, w, h, _ascent) = make_text_leaf(
                s,
                child_style,
                origin_x,
                origin_y,
                Some(container_w),
                ctx,
            );
            // Heuristic text-align: the wrapped run's `width` is the
            // *widest* line, so right / center align by shifting the
            // whole box. Multi-line per-line align (the proper
            // CSS behaviour) lands with the rich-text path.
            let align_dx = horizontal_align_offset(text_align, container_w, w);
            let mut b = box_;
            if align_dx != 0.0 {
                translate_box_x_in_place(&mut b, align_dx);
            }
            return (vec![b], w, h);
        }
    }

    // Multi-child IFC: rich-text paragraph shape. We flatten the
    // inline subtree into a list of `(text, attrs)` spans (one per
    // source text leaf), feed cosmic-text via `set_rich_text` so its
    // word-boundary breaks land *between* spans without losing per-
    // span attributes, then re-expand the result into anonymous Block
    // boxes for per-line backgrounds (`<mark>`) and decoration bars
    // (`<a>` / `<u>` / `<s>`), plus one `BoxKind::Text` containing
    // every glyph (with each glyph's source colour baked in by
    // `shape_paragraph`).
    layout_inline_paragraph(node, origin_x, origin_y, container_w, text_align, ctx)
}

// ---------------------------------------------------------------------------
// Rich-text paragraph path
// ---------------------------------------------------------------------------

/// One source text leaf the paragraph plan has flattened from the
/// inline subtree. Attributes are already resolved against the
/// cascade (font matched to a concrete family name, colour reduced
/// to linear RGBA, sizes converted to physical pixels).
struct SpanData {
    text: String,
    family: String,
    weight: u16,
    style_axis: wgpu_html_text::FontStyleAxis,
    size_px: f32,
    line_height_px: f32,
    color: Color,
}

/// One inline element the paragraph plan crossed. `leaf_range` is
/// the half-open interval of `SpanData` indices the element wraps —
/// used to assemble per-line backgrounds and decoration bars after
/// shaping.
struct InlineBlockSpan {
    leaf_range: (u32, u32),
    background: Option<Color>,
    decorations: Vec<TextDecorationLine>,
    decoration_color: Color,
}

#[derive(Default)]
struct ParagraphPlan {
    spans: Vec<SpanData>,
    inline_blocks: Vec<InlineBlockSpan>,
}

/// Walk one inline-level subtree depth-first, appending to `plan`.
/// `Element::Text` becomes a span; an inline element wrapping
/// children that contributed any spans is recorded as an
/// `InlineBlockSpan` if it has a background or decoration, so its
/// per-line bounds can be reconstructed after shaping.
fn collect_paragraph_spans(
    node: &CascadedNode,
    plan: &mut ParagraphPlan,
    ctx: &mut Ctx,
) {
    if matches!(node.style.display, Some(Display::None)) {
        return;
    }

    if let Element::Text(s) = &node.element {
        let collapsed = collapse_whitespace(s);
        if collapsed.is_empty() {
            return;
        }
        let display = match apply_text_transform(
            &collapsed,
            node.style.text_transform.as_ref(),
        ) {
            Some(t) => t,
            None => collapsed,
        };

        let families = parse_family_list(node.style.font_family.as_deref());
        let family_refs: Vec<&str> = families.iter().map(String::as_str).collect();
        let weight = font_weight_value(node.style.font_weight.as_ref());
        let axis = font_style_axis(node.style.font_style.as_ref());
        let family = ctx
            .text
            .ctx
            .resolve_family(&family_refs, weight, axis)
            .unwrap_or_default();

        let size_css = font_size_px(&node.style).unwrap_or(16.0);
        let line_h_css = line_height_px(&node.style, size_css);
        let color = node
            .style
            .color
            .as_ref()
            .and_then(resolve_color)
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);

        plan.spans.push(SpanData {
            text: display,
            family,
            weight,
            style_axis: axis,
            size_px: size_css * ctx.scale,
            line_height_px: line_h_css * ctx.scale,
            color,
        });
        return;
    }

    let leaf_start = plan.spans.len() as u32;
    for child in &node.children {
        collect_paragraph_spans(child, plan, ctx);
    }
    let leaf_end = plan.spans.len() as u32;
    if leaf_end > leaf_start {
        let bg = node
            .style
            .background_color
            .as_ref()
            .and_then(resolve_color);
        let decos = resolve_text_decorations(&node.style);
        if bg.is_some() || !decos.is_empty() {
            let decoration_color = node
                .style
                .color
                .as_ref()
                .and_then(resolve_color)
                .unwrap_or([0.0, 0.0, 0.0, 1.0]);
            plan.inline_blocks.push(InlineBlockSpan {
                leaf_range: (leaf_start, leaf_end),
                background: bg,
                decorations: decos,
                decoration_color,
            });
        }
    }
}

/// Build a `LayoutBox` whose only purpose is to paint a solid
/// background fill — used for inline-element backgrounds (`<mark>`)
/// and decoration bars (underline / line-through / overline).
fn make_anon_bg_box(rect: Rect, color: Color) -> LayoutBox {
    LayoutBox {
        margin_rect: rect,
        border_rect: rect,
        content_rect: rect,
        background: Some(color),
        background_rect: rect,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Block,
        text_run: None,
        text_color: None,
        text_decorations: Vec::new(),
        overflow: Overflow::Visible,
        image: None,
        background_image: None,
        children: Vec::new(),
    }
}

fn layout_inline_paragraph(
    node: &CascadedNode,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    text_align: Option<&wgpu_html_models::common::css_enums::TextAlign>,
    ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
    // 1. Flatten the inline subtree into spans + recorded inline
    //    blocks (the elements with bg / decoration whose per-line
    //    bounds we'll need after shaping).
    let mut plan = ParagraphPlan::default();
    for child in &node.children {
        collect_paragraph_spans(child, &mut plan, ctx);
    }
    if plan.spans.is_empty() {
        return (Vec::new(), 0.0, 0.0);
    }

    // 2. Hand the paragraph to cosmic-text. Each span's `leaf_id`
    //    matches its index in `plan.spans`, which is what
    //    `inline_blocks.leaf_range` indexes into.
    let paragraph_spans: Vec<ParagraphSpan<'_>> = plan
        .spans
        .iter()
        .enumerate()
        .map(|(i, sd)| ParagraphSpan {
            text: &sd.text,
            family: &sd.family,
            weight: sd.weight,
            style: sd.style_axis,
            size_px: sd.size_px,
            line_height_px: sd.line_height_px,
            color: sd.color,
            leaf_id: i as u32,
        })
        .collect();
    let para = match ctx
        .text
        .ctx
        .shape_paragraph(&paragraph_spans, Some(container_w))
    {
        Some(p) => p,
        None => return (Vec::new(), 0.0, 0.0),
    };

    // 3. Per-line `text-align` shift.
    let line_align_dx: Vec<f32> = para
        .lines
        .iter()
        .map(|line| horizontal_align_offset(text_align, container_w, line.line_width))
        .collect();

    let mut boxes: Vec<LayoutBox> = Vec::new();

    // 4. Inline-element backgrounds (`<mark>` and friends). One
    //    anonymous Block per (line × span-in-element-range) so a
    //    span that wraps gets a background bar on each line it
    //    occupies.
    for inline in &plan.inline_blocks {
        let Some(bg) = inline.background else {
            continue;
        };
        for leaf_id in inline.leaf_range.0..inline.leaf_range.1 {
            let Some(segs) = para.leaf_segments.get(&leaf_id) else {
                continue;
            };
            for seg in segs {
                let line = &para.lines[seg.line_index];
                let dx = line_align_dx[seg.line_index];
                let r = Rect::new(
                    origin_x + seg.x_start + dx,
                    origin_y + line.top,
                    seg.x_end - seg.x_start,
                    line.height,
                );
                if r.w > 0.0 && r.h > 0.0 {
                    boxes.push(make_anon_bg_box(r, bg));
                }
            }
        }
    }

    // 5. Decoration bars. Underline below baseline, line-through
    //    through the x-height, overline at line top. Thickness
    //    scales with line ascent so big text gets a beefier line.
    for inline in &plan.inline_blocks {
        if inline.decorations.is_empty() {
            continue;
        }
        for leaf_id in inline.leaf_range.0..inline.leaf_range.1 {
            let Some(segs) = para.leaf_segments.get(&leaf_id) else {
                continue;
            };
            for seg in segs {
                let line = &para.lines[seg.line_index];
                let dx = line_align_dx[seg.line_index];
                let ascent = (line.baseline - line.top).max(1.0);
                let thickness = (ascent / 12.0).max(1.0);
                for deco in &inline.decorations {
                    let y = match deco {
                        TextDecorationLine::Underline => line.baseline + thickness,
                        TextDecorationLine::LineThrough => {
                            line.baseline - ascent * 0.30
                        }
                        TextDecorationLine::Overline => line.top,
                    };
                    let r = Rect::new(
                        origin_x + seg.x_start + dx,
                        origin_y + y,
                        seg.x_end - seg.x_start,
                        thickness,
                    );
                    if r.w > 0.0 && r.h > 0.0 {
                        boxes.push(make_anon_bg_box(r, inline.decoration_color));
                    }
                }
            }
        }
    }

    // 6. The single `BoxKind::Text` for the whole paragraph. Apply
    //    each line's text-align dx to its glyph slice. Per-glyph
    //    colour was baked in by `shape_paragraph` so the paint side
    //    just reads `g.color`.
    let mut positioned: Vec<PositionedGlyph> = Vec::with_capacity(para.glyphs.len());
    for (li, line) in para.lines.iter().enumerate() {
        let dx = line_align_dx[li];
        for g in &para.glyphs[line.glyph_range.0..line.glyph_range.1] {
            positioned.push(PositionedGlyph {
                x: g.x + dx,
                y: g.y,
                w: g.w,
                h: g.h,
                uv_min: g.uv_min,
                uv_max: g.uv_max,
                color: g.color,
            });
        }
    }
    let run = ShapedRun {
        glyphs: positioned,
        width: para.width,
        height: para.height,
        ascent: para.first_line_ascent,
    };

    let r = Rect::new(origin_x, origin_y, para.width, para.height);
    let text_box = LayoutBox {
        margin_rect: r,
        border_rect: r,
        content_rect: r,
        background: None,
        background_rect: r,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Text,
        text_run: Some(run),
        text_color: None,
        text_decorations: Vec::new(),
        overflow: Overflow::Visible,
        image: None,
        background_image: None,
        children: Vec::new(),
    };
    boxes.push(text_box);

    (boxes, para.width, para.height)
}

/// CSS `text-align` → number of pixels to shift each child of the
/// line. `start`/`end` follow `dir: ltr` (no bidi yet). `justify`
/// falls through to `left`.
fn horizontal_align_offset(
    text_align: Option<&wgpu_html_models::common::css_enums::TextAlign>,
    container_w: f32,
    line_w: f32,
) -> f32 {
    use wgpu_html_models::common::css_enums::TextAlign as Ta;
    let free = (container_w - line_w).max(0.0);
    match text_align {
        Some(Ta::Center) => free * 0.5,
        Some(Ta::Right) | Some(Ta::End) => free,
        // Left, Start, Justify, None — flush to the inline-start edge.
        _ => 0.0,
    }
}

/// Recursively shift every rect on `b` and its descendants by `dx`
/// pixels along the x axis. Used to apply `text-align` after the
/// inline pass has positioned children at the line's left edge.
pub(crate) fn translate_box_x_in_place(b: &mut LayoutBox, dx: f32) {
    b.margin_rect.x += dx;
    b.border_rect.x += dx;
    b.content_rect.x += dx;
    b.background_rect.x += dx;
    if let Some(bgi) = b.background_image.as_mut() {
        for tile in &mut bgi.tiles {
            tile.x += dx;
        }
    }
    for child in &mut b.children {
        translate_box_x_in_place(child, dx);
    }
}

/// Recursively shift every rect on `b` and its descendants by `dy`
/// pixels along the y axis. Used by the inline pass to baseline-
/// align children after laying them all out at the line's top.
pub(crate) fn translate_box_y_in_place(b: &mut LayoutBox, dy: f32) {
    b.margin_rect.y += dy;
    b.border_rect.y += dy;
    b.content_rect.y += dy;
    b.background_rect.y += dy;
    if let Some(bgi) = b.background_image.as_mut() {
        for tile in &mut bgi.tiles {
            tile.y += dy;
        }
    }
    for child in &mut b.children {
        translate_box_y_in_place(child, dy);
    }
}

/// Split a CSS `font-family` value into individual family names.
/// Each entry is trimmed and stripped of surrounding quotes; empty
/// entries are dropped. The empty list means "no family preference".
fn parse_family_list(s: Option<&str>) -> Vec<String> {
    let Some(raw) = s else { return Vec::new() };
    raw.split(',')
        .map(|p| {
            p.trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// CSS `font-weight` → numeric weight in [100, 900]. `None` falls
/// back to 400 (CSS initial). `Lighter` / `Bolder` are treated as
/// fixed shifts here (no parent context yet) — 300 / 700.
fn font_weight_value(fw: Option<&wgpu_html_models::common::css_enums::FontWeight>) -> u16 {
    use wgpu_html_models::common::css_enums::FontWeight as Fw;
    match fw {
        Some(Fw::Bold) => 700,
        Some(Fw::Lighter) => 300,
        Some(Fw::Bolder) => 700,
        Some(Fw::Weight(n)) => *n,
        Some(Fw::Normal) | None => 400,
    }
}

/// CSS `font-style` → font-registry style axis.
fn font_style_axis(
    fs: Option<&wgpu_html_models::common::css_enums::FontStyle>,
) -> wgpu_html_text::FontStyleAxis {
    use wgpu_html_models::common::css_enums::FontStyle as Fs;
    use wgpu_html_text::FontStyleAxis as A;
    match fs {
        Some(Fs::Italic) => A::Italic,
        Some(Fs::Oblique) => A::Oblique,
        Some(Fs::Normal) | None => A::Normal,
    }
}

/// Resolve `font-size` to CSS pixels. `Em` and `Rem` use 16px as the
/// reference (the T3 placeholder — proper `em` against the parent's
/// computed font size lands in T4 once the cascade tracks computed
/// values). `Percent`, viewport-relative units, and `auto` aren't
/// meaningful here yet and fall through.
fn font_size_px(style: &Style) -> Option<f32> {
    use wgpu_html_models::common::css_enums::CssLength;
    match style.font_size.as_ref()? {
        CssLength::Px(v) => Some(*v),
        CssLength::Em(v) | CssLength::Rem(v) => Some(v * 16.0),
        _ => None,
    }
}

/// Resolve `line-height` to CSS pixels. CSS allows a unitless number
/// (multiplier of font size); we currently parse line-height as
/// `CssLength`, so a `Px` value is the literal height and `Em` /
/// `Rem` multiply against `font_size_px`. Falls back to 1.25× the
/// font size.
fn line_height_px(style: &Style, font_size: f32) -> f32 {
    use wgpu_html_models::common::css_enums::CssLength;
    match style.line_height.as_ref() {
        Some(CssLength::Px(v)) => *v,
        Some(CssLength::Em(v)) | Some(CssLength::Rem(v)) => v * font_size,
        _ => font_size * 1.25,
    }
}

/// Pick the rectangle and corner radii that the background fills, based
/// on `background-clip`. The default `border-box` keeps the outer
/// rectangle and radii. `padding-box` shrinks by the border thickness;
/// `content-box` shrinks by border + padding. Inner radii are reduced
/// in step so the curvature stays concentric with the outer edge.
fn compute_background_box(
    style: &Style,
    border_rect: Rect,
    content_rect: Rect,
    border: Insets,
    padding: Insets,
    radii: &CornerRadii,
) -> (Rect, CornerRadii) {
    use wgpu_html_models::common::css_enums::BackgroundClip;
    match style.background_clip.clone().unwrap_or(BackgroundClip::BorderBox) {
        BackgroundClip::BorderBox => (border_rect, radii.clone()),
        BackgroundClip::PaddingBox => {
            let inset_top = border.top;
            let inset_right = border.right;
            let inset_bottom = border.bottom;
            let inset_left = border.left;
            let r = inset_radii(
                radii,
                inset_top,
                inset_right,
                inset_bottom,
                inset_left,
            );
            let rect = Rect::new(
                border_rect.x + inset_left,
                border_rect.y + inset_top,
                (border_rect.w - inset_left - inset_right).max(0.0),
                (border_rect.h - inset_top - inset_bottom).max(0.0),
            );
            (rect, r)
        }
        BackgroundClip::ContentBox => {
            let inset_top = border.top + padding.top;
            let inset_right = border.right + padding.right;
            let inset_bottom = border.bottom + padding.bottom;
            let inset_left = border.left + padding.left;
            let r = inset_radii(
                radii,
                inset_top,
                inset_right,
                inset_bottom,
                inset_left,
            );
            (content_rect, r)
        }
    }
}

/// Reduce each corner's radius by the matching adjacent insets,
/// clamped at zero. The horizontal component shrinks by the inset of
/// the side it shares an x-edge with; the vertical component shrinks
/// by the inset of its y-edge. A tight border eats into the curvature
/// until the inner edge is straight.
fn inset_radii(r: &CornerRadii, top: f32, right: f32, bottom: f32, left: f32) -> CornerRadii {
    let shrink = |corner: Radius, dh: f32, dv: f32| Radius {
        h: (corner.h - dh).max(0.0),
        v: (corner.v - dv).max(0.0),
    };
    CornerRadii {
        top_left: shrink(r.top_left, left, top),
        top_right: shrink(r.top_right, right, top),
        bottom_right: shrink(r.bottom_right, right, bottom),
        bottom_left: shrink(r.bottom_left, left, bottom),
    }
}

fn resolve_insets_margin(style: &Style, container_w: f32, ctx: &mut Ctx) -> Insets {
    Insets {
        top: side(&style.margin_top, &style.margin, container_w, ctx),
        right: side(&style.margin_right, &style.margin, container_w, ctx),
        bottom: side(&style.margin_bottom, &style.margin, container_w, ctx),
        left: side(&style.margin_left, &style.margin, container_w, ctx),
    }
}

fn resolve_insets_padding(style: &Style, container_w: f32, ctx: &mut Ctx) -> Insets {
    Insets {
        top: side(&style.padding_top, &style.padding, container_w, ctx),
        right: side(&style.padding_right, &style.padding, container_w, ctx),
        bottom: side(&style.padding_bottom, &style.padding, container_w, ctx),
        left: side(&style.padding_left, &style.padding, container_w, ctx),
    }
}

fn side(
    specific: &Option<CssLength>,
    shorthand: &Option<CssLength>,
    container_w: f32,
    ctx: &mut Ctx,
) -> f32 {
    length::resolve(specific.as_ref(), container_w, ctx)
        .or_else(|| length::resolve(shorthand.as_ref(), container_w, ctx))
        .unwrap_or(0.0)
}

/// True when the effective value of a margin side (specific longhand
/// falling through to shorthand) is `auto`. Used by the flex layer to
/// detect items that want to absorb free space on the main / cross
/// axis.
pub(crate) fn is_auto_margin(specific: &Option<CssLength>, shorthand: &Option<CssLength>) -> bool {
    fn is_auto(v: &Option<CssLength>) -> bool {
        matches!(v, Some(CssLength::Auto))
    }
    if specific.is_some() {
        is_auto(specific)
    } else {
        is_auto(shorthand)
    }
}

/// Apply CSS `min-*` / `max-*` clamping to a content-box dimension.
///
/// Both bounds are interpreted in `box_sizing` semantics (so a
/// `border-box` `min-width: 100px` clamps the *border-box* width to
/// 100px just like browsers do). `frame` is the matching axis's
/// border + padding, used to convert between content-box and
/// border-box. `Auto` resolves to "no constraint", matching CSS.
///
/// `max` is applied first, then `min` (per CSS-Sizing-3 §5.2: "min
/// wins ties"), so an over-eager `min` always wins against `max`.
pub(crate) fn clamp_axis(
    size: f32,
    min: Option<&CssLength>,
    max: Option<&CssLength>,
    container: f32,
    frame: f32,
    box_sizing: BoxSizing,
    ctx: &mut Ctx,
) -> f32 {
    let convert = |raw: f32| -> f32 {
        match box_sizing {
            BoxSizing::ContentBox => raw,
            BoxSizing::BorderBox => (raw - frame).max(0.0),
        }
    };
    let mut out = size;
    if let Some(m) = length::resolve(max, container, ctx) {
        out = out.min(convert(m));
    }
    if let Some(m) = length::resolve(min, container, ctx) {
        out = out.max(convert(m));
    }
    out.max(0.0)
}

/// Per CSS 3 border-radius spec: when the sum of radii on any side
/// exceeds that side's length, *all* radii are scaled down by the
/// same factor (the smallest of the per-side ratios) so adjacent
/// corners no longer overlap. The horizontal and vertical components
/// are checked independently — overflow on either axis triggers a
/// uniform scale of every corner on every axis.
fn clamp_corner_radii(r: &mut CornerRadii, width: f32, height: f32) {
    let mut scale: f32 = 1.0;
    let limit = |edge_len: f32, sum: f32, scale: &mut f32| {
        if sum > 0.0 && edge_len > 0.0 && sum > edge_len {
            *scale = scale.min(edge_len / sum);
        }
    };
    // Horizontal axis: the h-components on each horizontal edge.
    limit(width, r.top_left.h + r.top_right.h, &mut scale);
    limit(width, r.bottom_left.h + r.bottom_right.h, &mut scale);
    // Vertical axis: the v-components on each vertical edge.
    limit(height, r.top_left.v + r.bottom_left.v, &mut scale);
    limit(height, r.top_right.v + r.bottom_right.v, &mut scale);
    if scale < 1.0 {
        for c in [
            &mut r.top_left,
            &mut r.top_right,
            &mut r.bottom_right,
            &mut r.bottom_left,
        ] {
            c.h *= scale;
            c.v *= scale;
        }
    }
}

#[cfg(test)]
mod tests;
