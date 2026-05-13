use super::mouse::MouseEventInit;

#[derive(Debug, Clone)]
pub struct DragEventInit {
    pub mouse: MouseEventInit,
    pub data_transfer: Option<String>,
}

impl Default for DragEventInit {
    fn default() -> Self { Self { mouse: MouseEventInit::default(), data_transfer: None } }
}
