use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct ProgressEventInit {
  pub base: EventInit,
  pub length_computable: bool,
  pub loaded: u64,
  pub total: u64,
}

impl Default for ProgressEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      length_computable: false,
      loaded: 0,
      total: 0,
    }
  }
}
