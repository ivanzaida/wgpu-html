use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::mpsc,
};

#[derive(Clone, Debug)]
pub enum ResourceData {
  Bytes(Vec<u8>),
  Text(String),
}

#[derive(Clone, Debug)]
pub enum ResourceState {
  Loading,
  Ready(ResourceData),
  Failed(String),
}

impl ResourceState {
  pub fn is_loading(&self) -> bool {
    matches!(self, Self::Loading)
  }

  pub fn is_ready(&self) -> bool {
    matches!(self, Self::Ready(_))
  }

  pub fn is_failed(&self) -> bool {
    matches!(self, Self::Failed(_))
  }

  pub fn data(&self) -> Option<&ResourceData> {
    match self {
      Self::Ready(data) => Some(data),
      _ => None,
    }
  }

  pub fn bytes(&self) -> Option<&[u8]> {
    match self {
      Self::Ready(ResourceData::Bytes(b)) => Some(b),
      _ => None,
    }
  }

  pub fn text(&self) -> Option<&str> {
    match self {
      Self::Ready(ResourceData::Text(t)) => Some(t),
      _ => None,
    }
  }

  pub fn error(&self) -> Option<&str> {
    match self {
      Self::Failed(e) => Some(e),
      _ => None,
    }
  }
}

struct Completed {
  key: String,
  result: Result<ResourceData, String>,
}

pub struct ResourceManager {
  cache: HashMap<String, ResourceState>,
  rx: mpsc::Receiver<Completed>,
  tx: mpsc::Sender<Completed>,
  base_path: Option<PathBuf>,
}

impl ResourceManager {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel();
    Self {
      cache: HashMap::new(),
      rx,
      tx,
      base_path: None,
    }
  }

  pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
    self.base_path = Some(path.into());
  }

  pub fn load_bytes(&mut self, key: impl Into<String>, path: impl AsRef<Path>) {
    let key = key.into();
    if self.cache.contains_key(&key) {
      return;
    }

    self.cache.insert(key.clone(), ResourceState::Loading);

    let resolved = self.resolve_path(path.as_ref());
    let tx = self.tx.clone();
    rayon::spawn(move || {
      let result = std::fs::read(&resolved)
        .map(ResourceData::Bytes)
        .map_err(|e| format!("{}: {}", resolved.display(), e));
      let _ = tx.send(Completed { key, result });
    });
  }

  pub fn load_text(&mut self, key: impl Into<String>, path: impl AsRef<Path>) {
    let key = key.into();
    if self.cache.contains_key(&key) {
      return;
    }

    self.cache.insert(key.clone(), ResourceState::Loading);

    let resolved = self.resolve_path(path.as_ref());
    let tx = self.tx.clone();
    rayon::spawn(move || {
      let result = std::fs::read_to_string(&resolved)
        .map(ResourceData::Text)
        .map_err(|e| format!("{}: {}", resolved.display(), e));
      let _ = tx.send(Completed { key, result });
    });
  }

  pub fn load_bytes_from_memory(&mut self, key: impl Into<String>, data: Vec<u8>) {
    self.cache.insert(key.into(), ResourceState::Ready(ResourceData::Bytes(data)));
  }

  pub fn load_text_from_memory(&mut self, key: impl Into<String>, data: String) {
    self.cache.insert(key.into(), ResourceState::Ready(ResourceData::Text(data)));
  }

  pub fn get(&self, key: &str) -> Option<&ResourceState> {
    self.cache.get(key)
  }

  pub fn state(&self, key: &str) -> ResourceState {
    self.cache.get(key).cloned().unwrap_or(ResourceState::Failed("not requested".into()))
  }

  pub fn is_all_ready(&self) -> bool {
    self.cache.values().all(|s| !s.is_loading())
  }

  pub fn poll(&mut self) -> bool {
    let mut any_completed = false;
    while let Ok(completed) = self.rx.try_recv() {
      let state = match completed.result {
        Ok(data) => ResourceState::Ready(data),
        Err(e) => ResourceState::Failed(e),
      };
      self.cache.insert(completed.key, state);
      any_completed = true;
    }
    any_completed
  }

  pub fn remove(&mut self, key: &str) {
    self.cache.remove(key);
  }

  pub fn clear(&mut self) {
    self.cache.clear();
  }

  fn resolve_path(&self, path: &Path) -> PathBuf {
    if path.is_absolute() {
      path.to_path_buf()
    } else if let Some(base) = &self.base_path {
      base.join(path)
    } else {
      path.to_path_buf()
    }
  }
}
