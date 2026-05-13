use super::ui::UiEventInit;

#[derive(Debug, Clone)]
pub struct FocusEventInit {
    pub ui: UiEventInit,
    pub related_target: Option<String>,
}

impl Default for FocusEventInit {
    fn default() -> Self { Self { ui: UiEventInit::default(), related_target: None } }
}
