use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct StorageEventInit {
  pub base: EventInit,
  pub key: Option<String>,
  pub old_value: Option<String>,
  pub new_value: Option<String>,
  pub url: String,
  pub storage_area: Option<String>,
}
impl Default for StorageEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      key: None,
      old_value: None,
      new_value: None,
      url: String::new(),
      storage_area: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct SubmitEventInit {
  pub base: EventInit,
  pub submitter: Option<String>,
}
impl Default for SubmitEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      submitter: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct FormDataEventInit {
  pub base: EventInit,
  pub form_data: Option<String>,
}
impl Default for FormDataEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      form_data: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ToggleEventInit {
  pub base: EventInit,
  pub old_state: String,
  pub new_state: String,
}
impl Default for ToggleEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      old_state: String::new(),
      new_state: String::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct CommandEventInit {
  pub base: EventInit,
  pub command: String,
}
impl Default for CommandEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      command: String::new(),
    }
  }
}
