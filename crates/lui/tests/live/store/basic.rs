use std::sync::Arc;
use lui::live::{Runtime, Ctx, Store};
use lui::live::builder::el::*;

#[derive(Clone, Debug, PartialEq)]
struct AppState {
    count: i32,
    name: String,
}

#[test]
fn store_get_set() {
    let stored: Arc<parking_lot::Mutex<Option<Store<AppState>>>> = Default::default();

    let s = stored.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let store = ctx.store(AppState { count: 0, name: "test".into() });
        *s.lock() = Some(store.clone());
        let state = store.get();
        div().text(&format!("{}: {}", state.name, state.count))
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let store = stored.lock().clone().unwrap();
    assert_eq!(store.get().count, 0);

    store.update(|s| s.count = 42);
    assert_eq!(store.get().count, 42);
}

#[test]
fn store_update_triggers_rerender() {
    let stored: Arc<parking_lot::Mutex<Option<Store<AppState>>>> = Default::default();

    let s = stored.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let store = ctx.store(AppState { count: 0, name: "x".into() });
        *s.lock() = Some(store.clone());
        div().text(&format!("{}", store.get().count))
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    stored.lock().as_ref().unwrap().update(|s| s.count = 7);
    assert!(rt.process(&mut lui));
}

#[test]
fn store_lens_projects_field() {
    let stored: Arc<parking_lot::Mutex<Option<Store<AppState>>>> = Default::default();

    let s = stored.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let store = ctx.store(AppState { count: 10, name: "hi".into() });
        *s.lock() = Some(store.clone());
        let count_sig = store.lens(|s| s.count);
        assert_eq!(count_sig.get(), 10);
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
}

#[test]
fn store_lens_updates_when_field_changes() {
    let stored: Arc<parking_lot::Mutex<Option<Store<AppState>>>> = Default::default();
    let lens_store: Arc<parking_lot::Mutex<Option<lui::live::Lens<i32>>>> = Default::default();

    let s = stored.clone();
    let l = lens_store.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let store = ctx.store(AppState { count: 0, name: "a".into() });
        let count = store.lens(|s| s.count);
        *s.lock() = Some(store);
        *l.lock() = Some(count.clone());
        div().text(&format!("{}", count.get()))
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    stored.lock().as_ref().unwrap().update(|s| s.count = 99);
    let count_val = lens_store.lock().as_ref().unwrap().get();
    assert_eq!(count_val, 99);
}
