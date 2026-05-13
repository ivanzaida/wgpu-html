use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct PopStateEventInit {
    pub base: EventInit,
    pub state: Option<String>,
}

impl Default for PopStateEventInit {
    fn default() -> Self { Self { base: EventInit::default(), state: None } }
}
