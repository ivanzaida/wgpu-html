use crate::ArcStr;

#[derive(Debug, Clone)]
pub struct EventInit {
  pub event_type: ArcStr,
  pub bubbles: bool,
  pub cancelable: bool,
  pub default_prevented: bool,
  pub propagation_stopped: bool,
  pub immediate_propagation_stopped: bool,
  pub target_path: Vec<usize>,
}

impl Default for EventInit {
  fn default() -> Self {
    Self {
      event_type: ArcStr::from(""),
      bubbles: false,
      cancelable: false,
      default_prevented: false,
      propagation_stopped: false,
      immediate_propagation_stopped: false,
      target_path: Vec::new(),
    }
  }
}
