use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use lui::live::{Runtime, Ctx};
use lui::live::builder::el::*;

#[test]
fn on_unmounted_fires_when_ctx_drops() {
    let cleaned = Arc::new(AtomicBool::new(false));

    let c = cleaned.clone();
    {
        let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
            let c = c.clone();
            ctx.on_unmounted(move || {
                c.store(true, Ordering::Relaxed);
            });
            div()
        });

        let mut lui = lui::Lui::new();
        lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
        rt.render(&mut lui);

        assert!(!cleaned.load(Ordering::Relaxed));
    }

    assert!(cleaned.load(Ordering::Relaxed));
}
