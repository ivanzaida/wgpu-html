use std::cell::RefCell;
use std::sync::Arc;

pub(crate) type WatchSubscribeFn =
    Box<dyn FnOnce(Arc<dyn Fn() + Send + Sync>) -> Box<dyn std::any::Any + Send + Sync> + Send + Sync>;

struct TrackingEntry {
    signal_id: u64,
    subscribe: WatchSubscribeFn,
}

thread_local! {
    static TRACKING: RefCell<Option<Vec<TrackingEntry>>> = const { RefCell::new(None) };
}

pub(crate) fn start_tracking() {
    TRACKING.with(|t| {
        *t.borrow_mut() = Some(Vec::new());
    });
}

pub(crate) fn stop_tracking() -> Vec<(u64, WatchSubscribeFn)> {
    TRACKING.with(|t| {
        t.borrow_mut()
            .take()
            .unwrap_or_default()
            .into_iter()
            .map(|e| (e.signal_id, e.subscribe))
            .collect()
    })
}

pub(crate) fn track_signal(signal_id: u64, subscribe: WatchSubscribeFn) {
    TRACKING.with(|t| {
        let mut borrow = t.borrow_mut();
        if let Some(ref mut deps) = *borrow {
            if !deps.iter().any(|e| e.signal_id == signal_id) {
                deps.push(TrackingEntry {
                    signal_id,
                    subscribe,
                });
            }
        }
    });
}
