use super::ui::UiEventInit;

#[derive(Debug, Clone)]
pub struct TouchInit {
    pub identifier: i32, pub target: String,
    pub client_x: f64, pub client_y: f64,
    pub screen_x: f64, pub screen_y: f64,
    pub radius_x: f32, pub radius_y: f32,
    pub rotation_angle: f32, pub force: f32,
}

#[derive(Debug, Clone)]
pub struct TouchEventInit {
    pub ui: UiEventInit,
    pub touches: Vec<TouchInit>,
    pub target_touches: Vec<TouchInit>,
    pub changed_touches: Vec<TouchInit>,
    pub ctrl_key: bool, pub shift_key: bool, pub alt_key: bool, pub meta_key: bool,
}

impl Default for TouchEventInit {
    fn default() -> Self {
        Self {
            ui: UiEventInit::default(),
            touches: Vec::new(), target_touches: Vec::new(), changed_touches: Vec::new(),
            ctrl_key: false, shift_key: false, alt_key: false, meta_key: false,
        }
    }
}
