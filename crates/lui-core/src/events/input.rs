use super::ui::UiEventInit;

#[derive(Debug, Clone)]
pub struct InputEventInit {
    pub ui: UiEventInit,
    pub data: Option<String>, pub is_composing: bool, pub input_type: String,
}

impl Default for InputEventInit {
    fn default() -> Self {
        Self { ui: UiEventInit::default(), data: None, is_composing: false, input_type: String::new() }
    }
}
