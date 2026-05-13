use super::base::EventInit;

#[derive(Debug, Clone)]
pub struct AnimationEventInit {
    pub base: EventInit,
    pub animation_name: String, pub elapsed_time: f32, pub pseudo_element: String,
}

impl Default for AnimationEventInit {
    fn default() -> Self { Self { base: EventInit::default(), animation_name: String::new(), elapsed_time: 0.0, pseudo_element: String::new() } }
}
