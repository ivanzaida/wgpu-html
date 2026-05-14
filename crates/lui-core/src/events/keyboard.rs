use super::ui::UiEventInit;

#[derive(Debug, Clone)]
pub struct KeyboardEventInit {
  pub ui: UiEventInit,
  pub key: String,
  pub code: String,
  pub location: u32,
  pub repeat: bool,
  pub is_composing: bool,
}

impl Default for KeyboardEventInit {
  fn default() -> Self {
    Self {
      ui: UiEventInit::default(),
      key: String::new(),
      code: String::new(),
      location: 0,
      repeat: false,
      is_composing: false,
    }
  }
}
