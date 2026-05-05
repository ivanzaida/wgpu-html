//! Bridge from `wgpu_html_tree::FontRegistry` to a `cosmic_text::FontSystem`.
//!
//! The bridge is keyed by the byte-array identities (`Arc<[u8]>`
//! pointer values) of the registered faces. Reusing the same
//! `FontRegistry` across frames doesn't reload anything; a freshly-
//! registered face is loaded into the `FontSystem` on the next
//! `sync()`; a face whose underlying `Arc<[u8]>` was swapped (different
//! identity, same `(family, weight, style)` triple) gets re-loaded.
//!
//! No system fonts are loaded — this is the whole point of the
//! Tree-owned constraint (`docs/text.md` §3). The host's registered
//! faces are the only fonts cosmic-text sees.

use std::{collections::HashMap, sync::Arc};

use cosmic_text::fontdb;
use wgpu_html_tree::{FontHandle, FontRegistry};

#[derive(Debug, Clone)]
struct LoadedFace {
  /// Identity of the source `Arc<[u8]>`; used as the cache key on
  /// re-`sync`.
  data_ptr: *const u8,
  /// Handle into cosmic-text's font database.
  fontdb_id: fontdb::ID,
}

// `*const u8` is just a cache key; we never deref it. Promise Send+Sync
// so `FontDb` can move between threads if a future caller needs it.
unsafe impl Send for LoadedFace {}
unsafe impl Sync for LoadedFace {}

/// Cache + cosmic-text bridge. Owns the `FontSystem` consulted by
/// shaping; sync against a `FontRegistry` to populate it.
///
/// The `FontSystem` is built with an empty database (no system fonts),
/// so only host-registered faces are visible to the shaper.
pub struct FontDb {
  system: cosmic_text::FontSystem,
  loaded: HashMap<FontHandle, LoadedFace>,
}

impl std::fmt::Debug for FontDb {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FontDb")
      .field("loaded_faces", &self.loaded.len())
      .finish_non_exhaustive()
  }
}

impl Default for FontDb {
  fn default() -> Self {
    Self::new()
  }
}

impl FontDb {
  /// Empty database. Use `sync(registry)` to populate.
  pub fn new() -> Self {
    // Empty fontdb → no system fonts. Locale "en-US" is just a
    // shaping-time hint; the registry is the source of truth.
    let db = fontdb::Database::new();
    let system = cosmic_text::FontSystem::new_with_locale_and_db("en-US".to_string(), db);
    Self {
      system,
      loaded: HashMap::new(),
    }
  }

  /// Build directly from a registry.
  pub fn from_registry(registry: &FontRegistry) -> Self {
    let mut db = Self::new();
    db.sync(registry);
    db
  }

  /// Borrow the underlying cosmic-text `FontSystem` for shaping
  /// callers (`Buffer::set_text`, etc.).
  pub fn font_system_mut(&mut self) -> &mut cosmic_text::FontSystem {
    &mut self.system
  }

  /// Reconcile the cache against `registry`. Returns the number of
  /// faces freshly loaded or replaced during this call.
  ///
  /// - Faces present in `registry` but not in this cache are loaded.
  /// - Faces whose `Arc<[u8]>` identity changed (re-registration in place) are re-loaded; the previous fontdb entry is
  ///   removed.
  /// - Faces in this cache that are no longer in the registry are removed from both the local map and the fontdb
  ///   (leaving any shape-cache entries pointing at them stale — callers that care should drop their `Buffer`s).
  pub fn sync(&mut self, registry: &FontRegistry) -> usize {
    let mut updated = 0;

    // First pass: load / refresh entries that the registry has.
    let mut still_present: HashMap<FontHandle, ()> = HashMap::new();
    for (handle, face) in registry.iter() {
      still_present.insert(handle, ());
      let new_ptr = Arc::as_ptr(&face.data) as *const u8;
      let needs_load = match self.loaded.get(&handle) {
        Some(prev) => prev.data_ptr != new_ptr,
        None => true,
      };
      if !needs_load {
        continue;
      }

      // Drop the previous fontdb entry, if any.
      if let Some(prev) = self.loaded.remove(&handle) {
        self.system.db_mut().remove_face(prev.fontdb_id);
      }

      // cosmic-text expects an `Arc<dyn AsRef<[u8]> + ...>`. We
      // wrap our `Arc<[u8]>` so the underlying bytes are shared
      // (no copy), then load. `load_font_source` returns 0..n
      // ids depending on how many faces the file contains; for
      // a single-face TTF this is one entry.
      let source = fontdb::Source::Binary(Arc::new(SharedBytes(face.data.clone())));
      let ids = self.system.db_mut().load_font_source(source);
      let Some(&fontdb_id) = ids.first() else {
        // Not a parseable font — skip. The host gets to see the
        // empty `is_loaded(handle) == false` if they ask.
        continue;
      };

      self.loaded.insert(
        handle,
        LoadedFace {
          data_ptr: new_ptr,
          fontdb_id,
        },
      );
      updated += 1;
    }

    // Second pass: drop entries the registry no longer carries.
    let stale: Vec<FontHandle> = self
      .loaded
      .keys()
      .copied()
      .filter(|h| !still_present.contains_key(h))
      .collect();
    for h in stale {
      if let Some(prev) = self.loaded.remove(&h) {
        self.system.db_mut().remove_face(prev.fontdb_id);
        updated += 1;
      }
    }

    updated
  }

  pub fn is_loaded(&self, handle: FontHandle) -> bool {
    self.loaded.contains_key(&handle)
  }

  pub fn len(&self) -> usize {
    self.loaded.len()
  }

  pub fn is_empty(&self) -> bool {
    self.loaded.is_empty()
  }

  /// Resolve a `FontHandle` to its cosmic-text fontdb ID, useful
  /// when constructing `Attrs::new().family(...)` for shaping.
  pub fn fontdb_id(&self, handle: FontHandle) -> Option<fontdb::ID> {
    self.loaded.get(&handle).map(|f| f.fontdb_id)
  }

  /// First registered (lowest-numbered) handle still loaded. T3 uses
  /// this as a fallback when the cascade hasn't told us which family
  /// to pick. Real font-family matching arrives with inheritance in
  /// T4.
  pub fn first_handle(&self) -> Option<FontHandle> {
    // Handle indices are sequential, so `min` returns the
    // earliest-registered face that hasn't been removed.
    self.loaded.keys().copied().min()
  }
}

/// Adapter that lets cosmic-text consume our `Arc<[u8]>` without
/// copying. cosmic-text's `Source::Binary` wants
/// `Arc<dyn AsRef<[u8]> + Send + Sync>`; we wrap once here.
struct SharedBytes(Arc<[u8]>);

impl AsRef<[u8]> for SharedBytes {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}

