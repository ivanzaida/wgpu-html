use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct ClipboardEventInit {
  pub base: EventInit,
  pub clipboard_data: Option<String>,
}

impl Default for ClipboardEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      clipboard_data: None,
    }
  }
}
