use super::ui::UiEventInit;

#[derive(Debug, Clone)]
pub struct CompositionEventInit {
    pub ui: UiEventInit,
    pub data: String,
}

impl Default for CompositionEventInit {
    fn default() -> Self { Self { ui: UiEventInit::default(), data: String::new() } }
}
