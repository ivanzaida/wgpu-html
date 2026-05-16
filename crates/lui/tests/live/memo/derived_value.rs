use lui::live::{Signal, Runtime, Ctx};
use lui::live::builder::el::*;

#[test]
fn memo_computes_initial_value() {
    let result: parking_lot::Mutex<Option<Signal<i32>>> = parking_lot::Mutex::new(None);
    let result = std::sync::Arc::new(result);

    let r = result.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let count = ctx.signal(10i32);
        let doubled = ctx.memo(move || count.get() * 2);
        *r.lock() = Some(doubled.clone());
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let sig = result.lock().clone().unwrap();
    assert_eq!(sig.get(), 20);
}

#[test]
fn memo_updates_when_dependency_changes() {
    let source: std::sync::Arc<parking_lot::Mutex<Option<Signal<i32>>>> = Default::default();
    let derived: std::sync::Arc<parking_lot::Mutex<Option<Signal<i32>>>> = Default::default();

    let s = source.clone();
    let d = derived.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let count = ctx.signal(5i32);
        let doubled = ctx.memo({
            let count = count.clone();
            move || count.get() * 2
        });
        *s.lock() = Some(count);
        *d.lock() = Some(doubled);
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    assert_eq!(derived.lock().as_ref().unwrap().get(), 10);

    source.lock().as_ref().unwrap().set(7);
    assert_eq!(derived.lock().as_ref().unwrap().get(), 14);
}
