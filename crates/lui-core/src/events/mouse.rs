use super::ui::UiEventInit;

#[derive(Debug, Clone)]
pub struct MouseEventInit {
    pub ui: UiEventInit,
    pub screen_x: f64, pub screen_y: f64,
    pub client_x: f64, pub client_y: f64,
    pub ctrl_key: bool, pub shift_key: bool, pub alt_key: bool, pub meta_key: bool,
    pub button: i16, pub buttons: u16,
    pub related_target: Option<String>,
}

impl Default for MouseEventInit {
    fn default() -> Self {
        Self {
            ui: UiEventInit::default(),
            screen_x: 0.0, screen_y: 0.0, client_x: 0.0, client_y: 0.0,
            ctrl_key: false, shift_key: false, alt_key: false, meta_key: false,
            button: 0, buttons: 0, related_target: None,
        }
    }
}
