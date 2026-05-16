use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use lui::live::{Runtime, Ctx};
use lui::live::builder::el::*;

#[test]
fn on_mounted_fires_once_on_first_render() {
    let count = Arc::new(AtomicI32::new(0));

    let c = count.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let c = c.clone();
        ctx.on_mounted(move || {
            c.fetch_add(1, Ordering::Relaxed);
        });
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
    assert_eq!(count.load(Ordering::Relaxed), 1);

    rt.process(&mut lui);
    rt.process(&mut lui);
    assert_eq!(count.load(Ordering::Relaxed), 1);
}
