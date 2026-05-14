use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct MessageEventInit {
  pub base: EventInit,
  pub data: String,
  pub origin: String,
  pub last_event_id: String,
  pub source: Option<String>,
  pub ports: Vec<String>,
}

impl Default for MessageEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      data: String::new(),
      origin: String::new(),
      last_event_id: String::new(),
      source: None,
      ports: Vec::new(),
    }
  }
}
