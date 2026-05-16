use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use lui::live::{Signal, Runtime, Ctx, batch};
use lui::live::builder::el::*;

#[test]
fn batch_runs_effect_once_for_multiple_writes() {
    let effect_count = Arc::new(AtomicI32::new(0));
    let stored: Arc<parking_lot::Mutex<Option<(Signal<i32>, Signal<i32>)>>> = Default::default();

    let ec = effect_count.clone();
    let ss = stored.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
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
    let before = effect_count.load(Ordering::Relaxed);

    let (a, b) = stored.lock().clone().unwrap();
    batch(|| {
        a.set(1);
        b.set(2);
    });

    let after = effect_count.load(Ordering::Relaxed);
    assert_eq!(after - before, 1, "batch should run the effect exactly once");
}

#[test]
fn without_batch_effects_run_per_signal() {
    let effect_count = Arc::new(AtomicI32::new(0));
    let stored: Arc<parking_lot::Mutex<Option<(Signal<i32>, Signal<i32>)>>> = Default::default();

    let ec = effect_count.clone();
    let ss = stored.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
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
    let before = effect_count.load(Ordering::Relaxed);

    let (a, b) = stored.lock().clone().unwrap();
    a.set(1);
    b.set(2);

    let after = effect_count.load(Ordering::Relaxed);
    assert!(after - before >= 2, "without batch, effects should run multiple times");
}
