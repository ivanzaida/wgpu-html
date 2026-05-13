use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct ErrorEventInit {
    pub base: EventInit,
    pub message: String, pub filename: String,
    pub lineno: u32, pub colno: u32, pub error: Option<String>,
}

impl Default for ErrorEventInit {
    fn default() -> Self {
        Self { base: EventInit::default(), message: String::new(), filename: String::new(), lineno: 0, colno: 0, error: None }
    }
}
