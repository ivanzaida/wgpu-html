use crate::ArcStr;

#[derive(Debug, Clone, Default)]
pub struct ClassList {
  classes: Vec<ArcStr>,
  dirty: bool,
}

impl ClassList {
  pub fn new() -> Self {
    Self { classes: Vec::new(), dirty: false }
  }

  pub fn contains(&self, class: &str) -> bool {
    self.classes.iter().any(|c| c.as_ref() == class)
  }

  pub fn add(&mut self, class: &str) {
    if !self.contains(class) {
      self.classes.push(ArcStr::from(class));
      self.dirty = true;
    }
  }

  pub fn remove(&mut self, class: &str) -> bool {
    let before = self.classes.len();
    self.classes.retain(|c| c.as_ref() != class);
    let removed = self.classes.len() < before;
    if removed {
      self.dirty = true;
    }
    removed
  }

  pub fn toggle(&mut self, class: &str) -> bool {
    if self.contains(class) {
      self.remove(class);
      false
    } else {
      self.add(class);
      true
    }
  }

  pub fn set(&mut self, class_string: &str) {
    self.classes = class_string.split_ascii_whitespace().map(ArcStr::from).collect();
    self.dirty = true;
  }

  pub fn clear(&mut self) {
    if !self.classes.is_empty() {
      self.classes.clear();
      self.dirty = true;
    }
  }

  pub fn iter(&self) -> impl Iterator<Item = &ArcStr> {
    self.classes.iter()
  }

  pub fn len(&self) -> usize {
    self.classes.len()
  }

  pub fn is_empty(&self) -> bool {
    self.classes.is_empty()
  }

  pub fn is_dirty(&self) -> bool {
    self.dirty
  }

  pub fn clear_dirty(&mut self) {
    self.dirty = false;
  }

  pub fn as_slice(&self) -> &[ArcStr] {
    &self.classes
  }
}

impl PartialEq for ClassList {
  fn eq(&self, other: &Self) -> bool {
    self.classes == other.classes
  }
}
