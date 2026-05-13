use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct CustomEventInit {
    pub base: EventInit,
    pub detail: String,
}

impl Default for CustomEventInit {
    fn default() -> Self { Self { base: EventInit::default(), detail: String::new() } }
}
