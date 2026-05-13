use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct TransitionEventInit {
    pub base: EventInit,
    pub property_name: String, pub elapsed_time: f32, pub pseudo_element: String,
}

impl Default for TransitionEventInit {
    fn default() -> Self { Self { base: EventInit::default(), property_name: String::new(), elapsed_time: 0.0, pseudo_element: String::new() } }
}
