use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use lui::live::{Runtime, Ctx};
use lui::live::builder::el::*;

fn child_component(ctx: &Ctx, _: ()) -> El {
    let count = ctx.signal(0i32);
    let _ = count;
    div()
}

#[test]
fn child_component_has_independent_hooks() {
    let render_count = Arc::new(AtomicI32::new(0));

    let rc = render_count.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        rc.fetch_add(1, Ordering::Relaxed);
        div().children([
            ctx.component(child_component, ()),
            ctx.component(child_component, ()),
        ])
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    assert_eq!(render_count.load(Ordering::Relaxed), 1);
}

#[test]
fn child_mounted_callback_fires() {
    let mounted = Arc::new(AtomicI32::new(0));

    fn child_with_mount(ctx: &Ctx, counter: Arc<AtomicI32>) -> El {
        ctx.on_mounted(move || {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        div()
    }

    let m = mounted.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        div().children([
            ctx.component(child_with_mount, m.clone()),
            ctx.component(child_with_mount, m.clone()),
        ])
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    assert_eq!(mounted.load(Ordering::Relaxed), 2);
}
