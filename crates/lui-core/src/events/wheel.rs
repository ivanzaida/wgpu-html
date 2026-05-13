use super::mouse::MouseEventInit;

#[derive(Debug, Clone)]
pub struct WheelEventInit {
    pub mouse: MouseEventInit,
    pub delta_x: f64, pub delta_y: f64, pub delta_z: f64,
    pub delta_mode: u32,
}

impl Default for WheelEventInit {
    fn default() -> Self {
        Self { mouse: MouseEventInit::default(), delta_x: 0.0, delta_y: 0.0, delta_z: 0.0, delta_mode: 0 }
    }
}
