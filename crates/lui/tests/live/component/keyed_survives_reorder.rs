use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use lui::live::{Signal, Runtime, Ctx};
use lui::live::builder::el::*;

fn item_view(ctx: &Ctx, label: &str) -> El {
    let render_count = ctx.signal(0i32);
    render_count.update(|n| *n += 1);
    div().text(label)
}

#[test]
fn keyed_children_persist_across_reorder() {
    let order: Arc<parking_lot::Mutex<Option<Signal<Vec<String>>>>> = Default::default();

    let o = order.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let items = ctx.signal(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        *o.lock() = Some(items.clone());
        div().children(
            items.get().iter().map(|item| {
                ctx.keyed(item.clone(), item_view, item.as_str())
            }).collect::<Vec<_>>()
        )
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let sig = order.lock().clone().unwrap();
    sig.set(vec!["c".to_string(), "a".to_string(), "b".to_string()]);
    rt.process(&mut lui);
}

#[test]
fn removed_keyed_child_fires_unmounted() {
    let unmounted = Arc::new(AtomicI32::new(0));

    fn tracked_item(ctx: &Ctx, (label, counter): (&str, Arc<AtomicI32>)) -> El {
        ctx.on_unmounted(move || {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        div().text(label)
    }

    let items_sig: Arc<parking_lot::Mutex<Option<Signal<Vec<String>>>>> = Default::default();

    let is = items_sig.clone();
    let um = unmounted.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let items = ctx.signal(vec!["a".to_string(), "b".to_string()]);
        *is.lock() = Some(items.clone());
        let um = um.clone();
        div().children(
            items.get().iter().map(|item| {
                ctx.keyed(item.clone(), tracked_item, (item.as_str(), um.clone()))
            }).collect::<Vec<_>>()
        )
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
    assert_eq!(unmounted.load(Ordering::Relaxed), 0);

    items_sig.lock().as_ref().unwrap().set(vec!["a".to_string()]);
    rt.process(&mut lui);
    assert_eq!(unmounted.load(Ordering::Relaxed), 1);
}
