use lui::live::{Signal, Runtime, Ctx};
use lui::live::builder::el::*;

#[test]
fn initial_render_inserts_into_doc() {
    let mut rt = Runtime::new("#app", |_ctx: &Ctx| {
        div().text("hello")
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let app = lui.doc.root.query_selector("#app").unwrap();
    assert_eq!(app.children().len(), 1);
}

#[test]
fn process_returns_false_when_clean() {
    let mut rt = Runtime::new("#app", |_ctx: &Ctx| { div() });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    assert!(!rt.process(&mut lui));
}

#[test]
fn process_returns_true_and_rerenders_when_dirty() {
    let sig_store: std::sync::Arc<parking_lot::Mutex<Option<Signal<i32>>>> = Default::default();

    let ss = sig_store.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let count = ctx.signal(0i32);
        *ss.lock() = Some(count.clone());
        div().text(&format!("{}", count.get()))
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    sig_store.lock().as_ref().unwrap().set(42);
    assert!(rt.process(&mut lui));
}

#[test]
fn hooks_persist_across_rerenders() {
    let sig_store: std::sync::Arc<parking_lot::Mutex<Option<Signal<i32>>>> = Default::default();

    let ss = sig_store.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let count = ctx.signal(0i32);
        *ss.lock() = Some(count.clone());
        div().text(&format!("{}", count.get()))
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let sig = sig_store.lock().clone().unwrap();
    sig.set(10);
    rt.process(&mut lui);
    assert_eq!(sig.get(), 10);

    sig.set(20);
    rt.process(&mut lui);
    assert_eq!(sig.get(), 20);
}
