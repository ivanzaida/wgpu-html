use lui::live::{Runtime, Ctx};
use lui::live::builder::el::*;
use lui::live::builder::style::*;

fn styled_component(ctx: &Ctx, _: ()) -> El {
    ctx.styles(scoped_sheet("sc", [
        rule(".box").width(px(100)).height(px(100)),
    ]));
    div().class(ctx.scoped("box"))
}

#[test]
fn scoped_returns_prefixed_class() {
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        ctx.styles(scoped_sheet("counter", [
            rule(".wrapper").width(px(200)),
        ]));
        let class = ctx.scoped("wrapper");
        assert_eq!(class, "counter-wrapper");
        div().class(class)
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);
}

#[test]
fn two_instances_register_styles_without_panic() {
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        div().children([
            ctx.component(styled_component, ()),
            ctx.component(styled_component, ()),
        ])
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let app = lui.doc.root.query_selector("#app").unwrap();
    let children = app.children()[0].children();
    assert_eq!(children.len(), 2);
}

#[test]
fn scoped_class_applied_to_element() {
    let mut rt = Runtime::new("#app", move |ctx: &Ctx| {
        ctx.component(styled_component, ())
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let app = lui.doc.root.query_selector("#app").unwrap();
    let component_div = &app.children()[0];
    assert!(component_div.class_list().contains("sc-box"));
}
