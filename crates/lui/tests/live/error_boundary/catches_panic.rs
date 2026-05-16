use lui::live::{Runtime, Ctx};
use lui::live::builder::el::*;

fn panicking_component(ctx: &Ctx) -> El {
    panic!("intentional test panic");
}

#[test]
fn error_boundary_catches_panic_and_renders_fallback() {
    let mut rt = Runtime::new("#app", |ctx: &Ctx| {
        ctx.error_boundary(
            panicking_component,
            |msg| div().text(&format!("Error: {msg}")),
        )
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let app = lui.doc.root.query_selector("#app").unwrap();
    let error_div = &app.children()[0];
    let text = error_div.text_content();
    assert!(text.contains("intentional test panic"), "got: {text}");
}

fn ok_component(ctx: &Ctx) -> El {
    div().text("all good")
}

#[test]
fn error_boundary_passes_through_on_success() {
    let mut rt = Runtime::new("#app", |ctx: &Ctx| {
        ctx.error_boundary(
            ok_component,
            |msg| div().text(&format!("Error: {msg}")),
        )
    });

    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);
    rt.render(&mut lui);

    let app = lui.doc.root.query_selector("#app").unwrap();
    let content = &app.children()[0];
    assert_eq!(content.text_content(), "all good");
}
