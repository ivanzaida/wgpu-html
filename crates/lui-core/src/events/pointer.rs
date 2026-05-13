use super::mouse::MouseEventInit;

#[derive(Debug, Clone)]
pub struct PointerEventInit {
    pub mouse: MouseEventInit,
    pub pointer_id: i32,
    pub width: f64, pub height: f64,
    pub pressure: f32, pub tangential_pressure: f32,
    pub tilt_x: i32, pub tilt_y: i32, pub twist: i32,
    pub pointer_type: String, pub is_primary: bool,
}

impl Default for PointerEventInit {
    fn default() -> Self {
        Self {
            mouse: MouseEventInit::default(),
            pointer_id: 0, width: 1.0, height: 1.0,
            pressure: 0.0, tangential_pressure: 0.0,
            tilt_x: 0, tilt_y: 0, twist: 0,
            pointer_type: String::new(), is_primary: false,
        }
    }
}
