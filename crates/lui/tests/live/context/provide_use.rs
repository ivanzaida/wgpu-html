use lui::live::{Runtime, Ctx};
use lui::live::builder::el::*;

#[derive(Clone, Debug, PartialEq)]
struct Theme {
    dark: bool,
}

fn themed_child(ctx: &Ctx, _: ()) -> El {
    let theme = ctx.use_context::<Theme>();
    assert!(theme.is_some());
    assert!(theme.unwrap().dark);
    div()
}

#[test]
fn child_receives_parent_context() {
    let mut rt = Runtime::new("#app", |ctx: &Ctx| {
        ctx.provide(Theme { dark: true });
        div().child(ctx.component(themed_child, ()))
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
}

fn override_child(ctx: &Ctx, _: ()) -> El {
    ctx.provide(Theme { dark: false });
    let theme = ctx.use_context::<Theme>().unwrap();
    assert!(!theme.dark);
    div()
}

#[test]
fn child_can_override_context() {
    let mut rt = Runtime::new("#app", |ctx: &Ctx| {
        ctx.provide(Theme { dark: true });
        div().children([
            ctx.component(override_child, ()),
            ctx.component(themed_child, ()),
        ])
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
}

#[test]
fn use_context_returns_none_when_not_provided() {
    let mut rt = Runtime::new("#app", |ctx: &Ctx| {
        let theme = ctx.use_context::<Theme>();
        assert!(theme.is_none());
        div()
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
}
