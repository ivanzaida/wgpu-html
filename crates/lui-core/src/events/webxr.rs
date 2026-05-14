use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct XRInputSourceEventInit {
  pub base: EventInit,
  pub frame: Option<String>,
  pub input_source: Option<String>,
}
impl Default for XRInputSourceEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      frame: None,
      input_source: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct XRInputSourcesChangeEventInit {
  pub base: EventInit,
  pub session: Option<String>,
  pub added: Vec<String>,
  pub removed: Vec<String>,
}
impl Default for XRInputSourcesChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      session: None,
      added: Vec::new(),
      removed: Vec::new(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct XRLayerEventInit {
  pub base: EventInit,
  pub layer: Option<String>,
}
impl Default for XRLayerEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      layer: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct XRReferenceSpaceEventInit {
  pub base: EventInit,
  pub reference_space: Option<String>,
  pub transform: Option<String>,
}
impl Default for XRReferenceSpaceEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      reference_space: None,
      transform: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct XRSessionEventInit {
  pub base: EventInit,
  pub session: Option<String>,
}
impl Default for XRSessionEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      session: None,
    }
  }
}

#[derive(Debug, Clone)]
pub struct XRVisibilityMaskChangeEventInit {
  pub base: EventInit,
  pub session: Option<String>,
}
impl Default for XRVisibilityMaskChangeEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      session: None,
    }
  }
}
