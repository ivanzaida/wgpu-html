use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct UiEventInit {
    pub base: EventInit,
    pub view: Option<String>,
    pub detail: i32,
}

impl Default for UiEventInit {
    fn default() -> Self {
        Self { base: EventInit::default(), view: None, detail: 0 }
    }
}
