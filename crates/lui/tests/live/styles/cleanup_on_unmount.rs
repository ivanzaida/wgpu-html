use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use lui::live::{Signal, Runtime, Ctx};
use lui::live::builder::el::*;
use lui::live::builder::style::*;

fn styled_child(ctx: &Ctx, _: ()) -> El {
    ctx.styles(scoped_sheet("child", [
        rule(".inner").width(px(50)),
    ]));
    div().class(ctx.scoped("inner"))
}

#[test]
fn styles_removed_when_all_instances_unmounted() {
    let show_child: Arc<parking_lot::Mutex<Option<Signal<bool>>>> = Default::default();

    let sc = show_child.clone();
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        let visible = ctx.signal(true);
        *sc.lock() = Some(visible.clone());
        if visible.get() {
            div().child(ctx.component(styled_child, ()))
        } else {
            div()
        }
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    show_child.lock().as_ref().unwrap().set(false);
    rt.process(&mut lui);
}
