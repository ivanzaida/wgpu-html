//! Font context — cosmic-text `FontSystem` bridge with custom font registry.

use std::{collections::HashMap, sync::Arc};

use cosmic_text::FontSystem;
use fontdb::{self, Source};

use crate::{
  font_face::{FontFace, FontHandle, FontStyleAxis},
  font_registry::FontRegistry,
};

// ── Internal bridge state ───────────────────────────────────────────────

struct LoadedFace {
  data_ptr: *const u8,
  fontdb_id: fontdb::ID,
}

unsafe impl Send for LoadedFace {}
unsafe impl Sync for LoadedFace {}

struct SharedBytes(Arc<[u8]>);

impl AsRef<[u8]> for SharedBytes {
  fn as_ref(&self) -> &[u8] {
    &self.0
  }
}

// ── FontContext ─────────────────────────────────────────────────────────

/// Owns the cosmic-text `FontSystem` (for system fonts and shaping) and a
/// bridge from the custom `FontRegistry` of registered faces.
pub struct FontContext {
  pub(crate) system: FontSystem,
  registry: FontRegistry,
  loaded: HashMap<FontHandle, LoadedFace>,
  last_sync_gen: u64,
}

impl FontContext {
  pub fn new() -> Self {
    let system = FontSystem::new();
    Self {
      system,
      registry: FontRegistry::default(),
      loaded: HashMap::new(),
      last_sync_gen: 0,
    }
  }

  pub fn registry(&self) -> &FontRegistry {
    &self.registry
  }

  /// Register a custom font face. Syncs into cosmic-text immediately.
  pub fn register_font(&mut self, face: FontFace) -> FontHandle {
    let handle = self.registry.register(face);
    self.sync_registry();
    handle
  }

  /// Reconcile the cosmic-text font system against the registry.
  pub fn sync_registry(&mut self) {
    if self.registry.generation() == self.last_sync_gen && self.registry.len() == self.loaded.len() {
      return;
    }
    self.last_sync_gen = self.registry.generation();

    let mut still_present: HashMap<FontHandle, ()> = HashMap::new();
    for (handle, face) in self.registry.iter() {
      still_present.insert(handle, ());
      let new_ptr = Arc::as_ptr(&face.data) as *const u8;
      let needs_load = match self.loaded.get(&handle) {
        Some(prev) => prev.data_ptr != new_ptr,
        None => true,
      };
      if !needs_load {
        continue;
      }

      if let Some(prev) = self.loaded.remove(&handle) {
        self.system.db_mut().remove_face(prev.fontdb_id);
      }

      let source = Source::Binary(Arc::new(SharedBytes(face.data.clone())));
      let ids = self.system.db_mut().load_font_source(source);
      let Some(&fontdb_id) = ids.first() else {
        continue;
      };

      self.loaded.insert(
        handle,
        LoadedFace {
          data_ptr: new_ptr,
          fontdb_id,
        },
      );
    }

    let stale: Vec<FontHandle> = self
      .loaded
      .keys()
      .copied()
      .filter(|h| !still_present.contains_key(h))
      .collect();
    for h in stale {
      if let Some(prev) = self.loaded.remove(&h) {
        self.system.db_mut().remove_face(prev.fontdb_id);
      }
    }
  }

  /// Resolve a CSS family list + weight + style to a `FontHandle`.
  pub fn pick_font(&self, families: &[&str], weight: u16, style: FontStyleAxis) -> Option<FontHandle> {
    self
      .registry
      .find_first(families, weight, style)
      .or_else(|| self.loaded.keys().copied().min())
  }

  /// Get the cosmic-text `fontdb::ID` for a handle.
  pub fn fontdb_id(&self, handle: FontHandle) -> Option<fontdb::ID> {
    self.loaded.get(&handle).map(|f| f.fontdb_id)
  }

  /// Resolve a CSS family list to the concrete family name cosmic-text
  /// needs in its `Attrs`.
  pub fn resolve_family(&mut self, families: &[&str], weight: u16, style: FontStyleAxis) -> Option<String> {
    let handle = self.pick_font(families, weight, style)?;
    let fontdb_id = self.fontdb_id(handle)?;
    let db = self.system.db_mut();
    let face = db.face(fontdb_id)?;
    face.families.first().map(|(name, _)| name.clone())
  }

  pub fn font_system_mut(&mut self) -> &mut FontSystem {
    &mut self.system
  }
}

impl Default for FontContext {
  fn default() -> Self {
    Self::new()
  }
}
