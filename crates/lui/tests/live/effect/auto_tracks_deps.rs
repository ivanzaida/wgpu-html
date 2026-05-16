use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use lui::live::{Signal, Runtime};
use lui::live::builder::el::*;
use lui::live::Ctx;

#[test]
fn effect_runs_on_first_render() {
    let run_count = Arc::new(AtomicI32::new(0));

    let rc = run_count.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let count = ctx.signal(0i32);
        let rc = rc.clone();
        ctx.on_effect(move || {
            let _ = count.get();
            rc.fetch_add(1, Ordering::Relaxed);
        });
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    assert_eq!(run_count.load(Ordering::Relaxed), 1);
}

#[test]
fn effect_reruns_when_tracked_signal_changes() {
    let run_count = Arc::new(AtomicI32::new(0));
    let stored_signal: Arc<parking_lot::Mutex<Option<Signal<i32>>>> = Arc::new(parking_lot::Mutex::new(None));

    let rc = run_count.clone();
    let ss = stored_signal.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let count = ctx.signal(0i32);
        *ss.lock() = Some(count.clone());
        let rc = rc.clone();
        ctx.on_effect(move || {
            let _ = count.get();
            rc.fetch_add(1, Ordering::Relaxed);
        });
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
    assert_eq!(run_count.load(Ordering::Relaxed), 1);

    let sig = stored_signal.lock().clone().unwrap();
    sig.set(5);
    assert_eq!(run_count.load(Ordering::Relaxed), 2);
}

#[test]
fn effect_does_not_rerun_for_unread_signals() {
    let run_count = Arc::new(AtomicI32::new(0));
    let stored_signal: Arc<parking_lot::Mutex<Option<Signal<i32>>>> = Arc::new(parking_lot::Mutex::new(None));

    let rc = run_count.clone();
    let ss = stored_signal.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let unread = ctx.signal(0i32);
        *ss.lock() = Some(unread.clone());
        let rc = rc.clone();
        ctx.on_effect(move || {
            rc.fetch_add(1, Ordering::Relaxed);
        });
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
    assert_eq!(run_count.load(Ordering::Relaxed), 1);

    let sig = stored_signal.lock().clone().unwrap();
    sig.set(5);
    assert_eq!(run_count.load(Ordering::Relaxed), 1);
}
