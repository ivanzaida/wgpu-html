use crate::ArcStr;

#[derive(Debug, Clone)]
pub struct EventInit {
  pub event_type: ArcStr,
  pub bubbles: bool,
  pub cancelable: bool,
  pub default_prevented: bool,
}

impl Default for EventInit {
  fn default() -> Self {
    Self {
      event_type: ArcStr::from(""),
      bubbles: false,
      cancelable: false,
      default_prevented: false,
    }
  }
}
