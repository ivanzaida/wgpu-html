use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct HashChangeEventInit {
    pub base: EventInit,
    pub old_url: String, pub new_url: String,
}

impl Default for HashChangeEventInit {
    fn default() -> Self { Self { base: EventInit::default(), old_url: String::new(), new_url: String::new() } }
}
