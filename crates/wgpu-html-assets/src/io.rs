use std::{
  collections::{HashMap, HashSet},
  sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
  },
  time::{Duration, Instant},
};

use crate::{
  fetcher::{FetchResponse, Fetcher},
  images::{self, DecodedAsset, ImageData, SizedKey},
  AssetStatus, FetchConfig,
};

const DEFAULT_WORKERS: usize = 4;
const SWEEP_INTERVAL: Duration = Duration::from_secs(10);

type ResultMap = Arc<Mutex<HashMap<String, Option<FetchResponse>>>>;

pub struct AssetIo<F: Fetcher> {
  // Raw bytes layer
  cache: HashMap<String, CacheEntry>,
  ttl: Duration,
  generation: u64,
  tx: Sender<Job>,
  results: ResultMap,
  submitted: HashSet<String>,
  last_sweep: Instant,

  // Image decode/resize layer
  decoded_images: HashMap<String, DecodedAsset>,
  sized_images: HashMap<SizedKey, Option<ImageData>>,

  _marker: std::marker::PhantomData<F>,
}

struct CacheEntry {
  status: EntryStatus,
  last_access: Instant,
  max_age: Option<Duration>,
  ttl_override: Option<Duration>,
  fetched_at: Option<Instant>,
}

enum EntryStatus {
  Pending,
  Ready(Arc<[u8]>),
  Failed,
}

struct Job {
  url: String,
  config: FetchConfig,
}

#[cfg(feature = "blocking")]
impl Default for AssetIo<crate::blocking::BlockingFetcher> {
  fn default() -> Self {
    Self::new(crate::blocking::BlockingFetcher::new())
  }
}

#[cfg(feature = "blocking")]
impl AssetIo<crate::blocking::BlockingFetcher> {
  pub fn blocking() -> Self {
    Self::default()
  }
}

impl<F: Fetcher + 'static> AssetIo<F> {
  pub fn new(fetcher: F) -> Self {
    Self::with_workers(fetcher, DEFAULT_WORKERS)
  }

  pub fn with_workers(fetcher: F, n: usize) -> Self {
    let n = n.max(1);
    let fetcher = Arc::new(fetcher);
    let (tx, rx) = channel::<Job>();
    let rx = Arc::new(Mutex::new(rx));
    let results: ResultMap = Arc::new(Mutex::new(HashMap::new()));

    for _ in 0..n {
      let worker_rx = Arc::clone(&rx);
      let worker_results = Arc::clone(&results);
      let worker_fetcher = Arc::clone(&fetcher);
      std::thread::spawn(move || worker_loop(worker_rx, worker_results, worker_fetcher));
    }

    Self {
      cache: HashMap::new(),
      ttl: Duration::from_secs(300),
      generation: 0,
      tx,
      results,
      submitted: HashSet::new(),
      last_sweep: Instant::now(),
      decoded_images: HashMap::new(),
      sized_images: HashMap::new(),
      _marker: std::marker::PhantomData,
    }
  }

  pub fn with_ttl(mut self, ttl: Duration) -> Self {
    self.ttl = ttl;
    self
  }

  pub fn set_ttl(&mut self, ttl: Duration) {
    self.ttl = ttl;
  }

  pub fn generation(&self) -> u64 {
    self.generation
  }

  // -------------------------------------------------------------------------
  // Raw file loading (CSS, etc.)
  // -------------------------------------------------------------------------

  pub fn load_file(&mut self, url: &str) -> AssetStatus {
    self.load_file_with(url, &FetchConfig::default())
  }

  pub fn load_file_with(&mut self, url: &str, config: &FetchConfig) -> AssetStatus {
    self.maybe_sweep();
    self.load_raw(url, config)
  }

  // -------------------------------------------------------------------------
  // Image loading (fetch + decode + resize)
  // -------------------------------------------------------------------------

  pub fn load_image(&mut self, img: &wgpu_html_models::Img) -> Option<ImageData> {
    let src = img.src.as_deref()?;
    self.load_image_url(src, None, None)
  }

  pub fn load_image_url(&mut self, src: &str, declared_w: Option<u32>, declared_h: Option<u32>) -> Option<ImageData> {
    self.maybe_sweep();

    let sized_key: SizedKey = (src.to_owned(), declared_w, declared_h);
    if let Some(entry) = self.sized_images.get(&sized_key) {
      return entry.as_ref().map(images::current_frame);
    }

    if !self.decoded_images.contains_key(src) {
      match self.load_raw(src, &FetchConfig::default()) {
        AssetStatus::Ready(bytes) => {
          if let Some(asset) = images::decode_asset(&bytes) {
            self.decoded_images.insert(src.to_owned(), asset);
          } else {
            self.sized_images.insert(sized_key, None);
            return None;
          }
        }
        AssetStatus::Pending => return None,
        AssetStatus::Failed => {
          self.sized_images.insert(sized_key, None);
          return None;
        }
      }
    }

    let asset = self.decoded_images.get(src)?;
    let full = images::build_sized(src, asset, declared_w, declared_h);
    self.sized_images.insert(sized_key, full.clone());
    full.as_ref().map(images::current_frame)
  }

  pub fn preload_image(&mut self, src: &str) {
    self.preload(src);
  }

  // -------------------------------------------------------------------------
  // Font loading (fetch raw bytes for font files)
  // -------------------------------------------------------------------------

  pub fn load_font(&mut self, url: &str) -> AssetStatus {
    self.load_file(url)
  }

  pub fn has_pending(&mut self) -> bool {
    self.drain_completed();
    self.cache.values().any(|e| matches!(e.status, EntryStatus::Pending))
  }

  fn drain_completed(&mut self) {
    let Ok(mut map) = self.results.lock() else {
      return;
    };
    let finished: Vec<(String, Option<FetchResponse>)> = map.drain().collect();
    drop(map);
    let now = Instant::now();
    for (url, resp) in finished {
      if let Some(entry) = self.cache.get_mut(&url) {
        if matches!(entry.status, EntryStatus::Pending) {
          match resp {
            Some(r) => {
              entry.status = EntryStatus::Ready(r.bytes.into());
              entry.max_age = r.max_age;
              entry.fetched_at = Some(now);
            }
            None => {
              entry.status = EntryStatus::Failed;
            }
          }
          self.generation += 1;
        }
      }
    }
  }

  pub fn has_animated(&self) -> bool {
    self
      .decoded_images
      .values()
      .any(|d| matches!(d, DecodedAsset::Animated { .. }))
  }

  pub fn has_animated_images(&self) -> bool {
    self.has_animated()
  }

  // -------------------------------------------------------------------------
  // Common
  // -------------------------------------------------------------------------

  pub fn preload(&mut self, url: &str) {
    self.preload_with(url, &FetchConfig::default());
  }

  pub fn preload_with(&mut self, url: &str, config: &FetchConfig) {
    if self.cache.contains_key(url) {
      return;
    }
    self.submit(url, config);
    self.cache.insert(
      url.to_owned(),
      CacheEntry {
        status: EntryStatus::Pending,
        last_access: Instant::now(),
        max_age: None,
        ttl_override: config.ttl,
        fetched_at: None,
      },
    );
  }

  pub fn invalidate(&mut self, url: &str) {
    if self.cache.remove(url).is_some() {
      self.submitted.remove(url);
      self.decoded_images.remove(url);
      self.sized_images.retain(|k, _| k.0 != url);
      self.generation += 1;
    }
  }

  pub fn clear(&mut self) {
    if !self.cache.is_empty() {
      self.cache.clear();
      self.submitted.clear();
      self.decoded_images.clear();
      self.sized_images.clear();
      self.generation += 1;
    }
  }

  pub fn sweep(&mut self) {
    let now = Instant::now();
    let global_ttl = self.ttl;
    self.cache.retain(|url, entry| {
      let effective_ttl = entry.ttl_override.or(entry.max_age).unwrap_or(global_ttl);
      let age = match entry.fetched_at {
        Some(t) => now.duration_since(t),
        None => now.duration_since(entry.last_access),
      };
      let keep = age < effective_ttl;
      if !keep {
        self.submitted.remove(url);
        self.decoded_images.remove(url);
      }
      keep
    });
    self.sized_images.retain(|k, _| self.cache.contains_key(&k.0));
  }

  // -------------------------------------------------------------------------
  // Internal
  // -------------------------------------------------------------------------

  fn load_raw(&mut self, url: &str, config: &FetchConfig) -> AssetStatus {
    let now = Instant::now();

    if let Some(entry) = self.cache.get_mut(url) {
      entry.last_access = now;
      return match &entry.status {
        EntryStatus::Ready(data) => AssetStatus::Ready(Arc::clone(data)),
        EntryStatus::Pending => match self.poll(url) {
          PollResult::Ready(resp) => {
            let data: Arc<[u8]> = resp.bytes.into();
            let entry = self.cache.get_mut(url).unwrap();
            entry.status = EntryStatus::Ready(Arc::clone(&data));
            entry.max_age = resp.max_age;
            entry.ttl_override = config.ttl;
            entry.fetched_at = Some(now);
            self.generation += 1;
            AssetStatus::Ready(data)
          }
          PollResult::Failed => {
            let entry = self.cache.get_mut(url).unwrap();
            entry.status = EntryStatus::Failed;
            self.generation += 1;
            AssetStatus::Failed
          }
          PollResult::Pending => AssetStatus::Pending,
        },
        EntryStatus::Failed => AssetStatus::Failed,
      };
    }

    self.submit(url, config);
    self.cache.insert(
      url.to_owned(),
      CacheEntry {
        status: EntryStatus::Pending,
        last_access: now,
        max_age: None,
        ttl_override: config.ttl,
        fetched_at: None,
      },
    );
    AssetStatus::Pending
  }

  fn maybe_sweep(&mut self) {
    let now = Instant::now();
    if now.duration_since(self.last_sweep) < SWEEP_INTERVAL {
      return;
    }
    self.last_sweep = now;
    self.sweep();
  }

  fn submit(&mut self, url: &str, config: &FetchConfig) {
    if !self.submitted.insert(url.to_owned()) {
      return;
    }
    let _ = self.tx.send(Job {
      url: url.to_owned(),
      config: config.clone(),
    });
  }

  fn poll(&mut self, url: &str) -> PollResult {
    if let Ok(mut map) = self.results.lock() {
      if let Some(entry) = map.remove(url) {
        return match entry {
          Some(resp) => PollResult::Ready(resp),
          None => PollResult::Failed,
        };
      }
    }
    PollResult::Pending
  }
}

enum PollResult {
  Ready(FetchResponse),
  Failed,
  Pending,
}

fn worker_loop<F: Fetcher>(rx: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>, results: ResultMap, fetcher: Arc<F>) {
  loop {
    let job = match rx.lock() {
      Ok(guard) => match guard.recv() {
        Ok(j) => j,
        Err(_) => break,
      },
      Err(_) => break,
    };

    let outcome = fetcher.fetch(&job.url, &job.config);

    if let Ok(mut map) = results.lock() {
      map.insert(job.url, outcome);
    }
  }
}
