use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use lui::live::{Signal, Runtime, Ctx, batch};
use lui::live::builder::el::*;

#[test]
fn multiple_signal_writes_produce_single_rerender() {
    let render_count = Arc::new(AtomicI32::new(0));
    let stored: Arc<parking_lot::Mutex<Option<(Signal<i32>, Signal<i32>, Signal<i32>)>>> = Default::default();

    let rc = render_count.clone();
    let ss = stored.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        rc.fetch_add(1, Ordering::Relaxed);
        let a = ctx.signal(0i32);
        let b = ctx.signal(0i32);
        let c = ctx.signal(0i32);
        *ss.lock() = Some((a.clone(), b.clone(), c.clone()));
        div().text(&format!("{} {} {}", a.get(), b.get(), c.get()))
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
    assert_eq!(render_count.load(Ordering::Relaxed), 1);

    let (a, b, c) = stored.lock().clone().unwrap();
    a.set(1);
    b.set(2);
    c.set(3);

    let processed = rt.process(&mut lui);
    assert!(processed);
    assert_eq!(render_count.load(Ordering::Relaxed), 2);
}

#[test]
fn batch_with_render_coalescing() {
    let render_count = Arc::new(AtomicI32::new(0));
    let effect_count = Arc::new(AtomicI32::new(0));
    let stored: Arc<parking_lot::Mutex<Option<(Signal<i32>, Signal<i32>)>>> = Default::default();

    let rc = render_count.clone();
    let ec = effect_count.clone();
    let ss = stored.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        rc.fetch_add(1, Ordering::Relaxed);
        let a = ctx.signal(0i32);
        let b = ctx.signal(0i32);
        *ss.lock() = Some((a.clone(), b.clone()));
        let ec = ec.clone();
        ctx.on_effect(move || {
            let _ = a.get();
            let _ = b.get();
            ec.fetch_add(1, Ordering::Relaxed);
        });
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let renders_before = render_count.load(Ordering::Relaxed);
    let effects_before = effect_count.load(Ordering::Relaxed);

    let (a, b) = stored.lock().clone().unwrap();
    batch(|| {
        a.set(10);
        b.set(20);
    });

    rt.process(&mut lui);

    let renders_after = render_count.load(Ordering::Relaxed);
    let effects_after = effect_count.load(Ordering::Relaxed);

    assert_eq!(renders_after - renders_before, 1, "one re-render");
    assert_eq!(effects_after - effects_before, 1, "one effect re-run from batch");
}
