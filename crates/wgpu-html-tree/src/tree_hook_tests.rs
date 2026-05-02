use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::Node;

struct CountingHook {
    count: Arc<AtomicUsize>,
    stop: bool,
}

impl TreeHook for CountingHook {
    fn on_render(&mut self, _tree: &mut Tree, _event: &TreeRenderEvent<'_>) -> TreeHookResponse {
        self.count.fetch_add(1, Ordering::Relaxed);
        if self.stop {
            TreeHookResponse::Stop
        } else {
            TreeHookResponse::Continue
        }
    }
}

#[test]
fn emit_render_calls_registered_hooks_until_stop() {
    let first = Arc::new(AtomicUsize::new(0));
    let second = Arc::new(AtomicUsize::new(0));
    let third = Arc::new(AtomicUsize::new(0));

    let mut tree = Tree::new(Node::new("root"));
    tree.add_hook(CountingHook {
        count: first.clone(),
        stop: false,
    });
    tree.add_hook(CountingHook {
        count: second.clone(),
        stop: true,
    });
    tree.add_hook(CountingHook {
        count: third.clone(),
        stop: false,
    });

    let event = TreeRenderEvent::new(Duration::from_millis(16));
    assert_eq!(tree.emit_render(&event), TreeHookResponse::Stop);
    assert_eq!(first.load(Ordering::Relaxed), 1);
    assert_eq!(second.load(Ordering::Relaxed), 1);
    assert_eq!(third.load(Ordering::Relaxed), 0);
}

#[test]
fn remove_hook_drops_matching_handle() {
    let mut tree = Tree::new(Node::new("root"));
    let handle = tree.add_hook(CountingHook {
        count: Arc::new(AtomicUsize::new(0)),
        stop: false,
    });
    assert_eq!(tree.hook_count(), 1);
    assert!(tree.remove_hook(&handle));
    assert_eq!(tree.hook_count(), 0);
    assert!(!tree.remove_hook(&handle));
}
