use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct DeviceMotionEventInit {
  pub base: EventInit,
  pub acceleration: Option<DeviceAccel>,
  pub acceleration_including_gravity: Option<DeviceAccel>,
  pub rotation_rate: Option<DeviceRotation>,
  pub interval: f64,
}
impl Default for DeviceMotionEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      acceleration: None,
      acceleration_including_gravity: None,
      rotation_rate: None,
      interval: 0.0,
    }
  }
}

#[derive(Debug, Clone)]
pub struct DeviceAccel {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

#[derive(Debug, Clone)]
pub struct DeviceRotation {
  pub alpha: f64,
  pub beta: f64,
  pub gamma: f64,
}

#[derive(Debug, Clone)]
pub struct DeviceOrientationEventInit {
  pub base: EventInit,
  pub alpha: Option<f64>,
  pub beta: Option<f64>,
  pub gamma: Option<f64>,
  pub absolute: bool,
}
impl Default for DeviceOrientationEventInit {
  fn default() -> Self {
    Self {
      base: EventInit::default(),
      alpha: None,
      beta: None,
      gamma: None,
      absolute: false,
    }
  }
}
