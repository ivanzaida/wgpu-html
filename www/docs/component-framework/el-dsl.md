---
title: El Builder DSL
---

# El Builder DSL

The `El` builder provides chainable methods for constructing HTML element trees. Each builder method returns `Self` for ergonomic chaining.

## Element Constructors

73 element constructor functions are available in the `el` module, covering all standard HTML elements:

```rust
use wgpu_html_ui::el;

el::div()       el::span()      el::p()         el::h1()..el::h6()
el::a()         el::button()    el::input()     el::textarea()
el::select()    el::option()    el::label()     el::form()
el::ul()        el::li()        el::ol()        el::table()
el::img()       el::video()     el::audio()     el::canvas()
el::header()    el::footer()    el::nav()       el::main()
el::article()   el::section()   el::aside()     el::figure()
el::code()      el::pre()       el::blockquote() el::hr()
el::br()        el::em()        el::strong()    el::small()
el::mark()      el::del()       el::ins()       el::sup()  el::sub()
el::details()   el::summary()   el::dialog()
el::progress()  el::meter()
// ... and more
```

Each returns an `El` builder for chainable configuration.

## Global Attributes

```rust
el::div()
    .id("main-panel")                     // id attribute
    .class("container dark")              // CSS class(es)
    .style("background: #f0f0f0")         // inline style string
    .hidden(true)                          // hidden attribute
    .tabindex(0)                           // tabindex
    .data("role", "sidebar")              // data-role="sidebar"
    .attr_title("Tooltip text")           // title attribute
    .custom_property("--accent", "#ff0000")  // CSS custom property
```

## Child Management

```rust
// Single child
el::div().child(el::span().text("Hello"))

// Multiple children
el::div().children([
    el::h1().text("Title"),
    el::p().text("Paragraph"),
    el::button().text("Click me"),
])

// Children from an iterator
el::ul().children(items.iter().map(|item|
    el::li().text(&item.name)
))

// Text content
el::span().text("Hello World")
```

## Callbacks

```rust
use wgpu_html_tree::{MouseEvent, HtmlEvent};

// Inline closure
el::button().on_click(|ev: &MouseEvent| {
    println!("clicked at {:?}", ev.pos);
})

// Ctx-based (sends a message)
el::button().on_click_cb(ctx.on_click(Msg::Clicked))

// Mapped callback
el::button().on_click_cb(ctx.callback(|ev| Msg::Position(ev.pos)))

// General event callback
el::input().on_event(|ev: &HtmlEvent| {
    println!("event: {:?}", ev.event_type);
})

// _cb variants take Arc<dyn Fn>
el::button().on_click_cb(ctx.on_click(Msg::Submit))
el::input().on_event_cb(ctx.event_callback(|ev| Some(Msg::Input(ev.data.clone()))))
```

## Element Mutation

`.configure()` gives mutable access to the underlying `Node`'s `Element`:

```rust
el::input().configure(|element| {
    if let Element::Input(input) = element {
        input.value = "default text".into();
        input.type_ = "email".into();
    }
})
```

## El: Clone for Named-Slot Patterns

`El` is `Clone`, enabling named-slot / content-projection patterns:

```rust
#[derive(Clone)]
struct CardProps {
    header: El,
    body: Children,
}

// In the parent's view():
let card = el::div().children([
    props.header,
    el::div().class("card-body").children(props.body.iter()),
]);
```

## Children Type

```rust
pub struct Children(Vec<El>);

impl Children {
    pub fn empty() -> Self;
    pub fn from(iter: impl IntoIterator<Item = El>) -> Self;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = El>;
}
```

Cloneable `Vec<El>` wrapper for content projection — pass it as a prop and render via `.children(props.body.iter())`.
