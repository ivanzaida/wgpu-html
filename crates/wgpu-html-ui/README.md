# wgpu-html-ui

Component framework for `wgpu-html`. Elm-architecture components with a chainable builder DSL — no JSX, no macros for users.

## Quick start

```rust
use wgpu_html_ui::{el, App, Component, Ctx, El, ShouldRender};

struct Counter { count: i32 }

#[derive(Clone)]
struct Props { label: String }

#[derive(Clone)]
enum Msg { Inc, Dec }

impl Component for Counter {
    type Props = Props;
    type Msg = Msg;
    type Env = ();

    fn create(_props: &Props) -> Self { Counter { count: 0 } }

    fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
        match msg {
            Msg::Inc => self.count += 1,
            Msg::Dec => self.count -= 1,
        }
        ShouldRender::Yes
    }

    fn view(&self, props: &Props, ctx: &Ctx<Msg>, _env: &()) -> El {
        el::div().class("counter").children([
            el::span().text(&props.label),
            el::button().text("-").on_click_cb(ctx.msg(Msg::Dec)),
            el::span().text(&self.count.to_string()),
            el::button().text("+").on_click_cb(ctx.msg(Msg::Inc)),
        ])
    }
}

fn main() {
    App::new::<Counter>(Props { label: "Count".into() })
        .stylesheet("body { font-family: sans-serif; }")
        .title("Counter")
        .size(400, 300)
        .run()
        .unwrap();
}
```

## Builder DSL

Every HTML element has a constructor in the `el` module. Chain methods to set attributes, add children, and attach callbacks.

```rust
use wgpu_html_ui::el;

let ui = el::div().id("app").class("container").children([
    el::h1().text("Hello"),
    el::p().class("lead").text("Welcome to my app"),
    el::button()
        .class("btn btn-primary")
        .text("Click me")
        .on_click(|_| println!("clicked")),
    el::input().configure(|i: &mut wgpu_html_models::Input| {
        i.placeholder = Some("Type here".into());
    }),
]);
```

### Available methods on `El`

| Method | Description |
|--------|-------------|
| `.id(v)` | Set the `id` attribute |
| `.class(v)` | Set the `class` attribute |
| `.style(v)` | Set inline `style` |
| `.attr_title(v)` | Set the `title` attribute |
| `.hidden(bool)` | Set `hidden` |
| `.tabindex(i32)` | Set `tabindex` |
| `.data(key, val)` | Set a `data-*` attribute |
| `.text(t)` | Append a text child node |
| `.child(el)` | Append a single child |
| `.children(iter)` | Append multiple children (arrays, vecs, iterators) |
| `.on_click(f)` | Attach a click handler (closure) |
| `.on_click_cb(cb)` | Attach a pre-built `MouseCallback` (from `ctx.msg()`) |
| `.on_mouse_down(f)` | Attach a mouse-down handler |
| `.on_mouse_up(f)` | Attach a mouse-up handler |
| `.on_mouse_enter(f)` | Attach a mouse-enter handler |
| `.on_mouse_leave(f)` | Attach a mouse-leave handler |
| `.on_event(f)` | Attach a general event handler (keyboard, focus, etc.) |
| `.on_event_cb(cb)` | Attach a pre-built `EventCallback` |
| `.configure(f)` | Mutate the underlying model struct for element-specific fields |
| `.custom_property(name, val)` | Set a CSS custom property |
| `.into_node()` | Unwrap into a raw `Node` |

Each `on_*` callback method has a `_cb` variant that accepts a pre-built `Arc<dyn Fn>` (returned by `ctx.msg()`, `ctx.callback()`, etc.) instead of a closure.

### Element constructors

`div`, `span`, `p`, `a`, `button`, `input`, `textarea`, `select`, `label`, `form`,
`h1`-`h6`, `ul`, `ol`, `li`, `table`, `tr`, `td`, `th`, `img`, `nav`, `header`,
`footer`, `section`, `article`, `aside`, `main_el`, `pre`, `code`, `strong`, `em`,
and every other HTML element (73 total). See `el.rs` for the full list.

## Component trait

```rust
pub trait Component: 'static {
    type Props: Clone + 'static;
    type Msg: Clone + Send + Sync + 'static;
    type Env: 'static;  // () for standalone, or custom state/context

    fn create(props: &Self::Props) -> Self;
    fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;
    fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El;

    // Optional:
    fn props_changed(&mut self, old: &Self::Props, new: &Self::Props) -> ShouldRender { .. }
    fn mounted(&mut self) { .. }
    fn destroyed(&mut self) { .. }
}
```

- **Props** -- immutable data passed from parent. Must be `Clone`.
- **Msg** -- enum of events the component handles. Must be `Clone + Send + Sync`.
- **Env** -- external data provided by the mount site at render time. Use `()` for standalone apps, or a custom type (e.g. `Tree`) for embedded components.
- **State** -- the struct itself (`self`). Private, mutable, owned by the component.

## Ctx -- callback factory

`Ctx<Msg>` is passed to `view()` and provides ways to create event handlers:

```rust
// Send a fixed message on click (returns MouseCallback — use with on_click_cb)
ctx.msg(Msg::Inc)

// Map the event to a message
ctx.callback(|ev| Msg::ClickedAt(ev.pos))

// General event handler (keyboard, focus, etc.)
ctx.event_callback(|ev| match ev {
    HtmlEvent::KeyDown { key, .. } if key == "Enter" => Some(Msg::Submit),
    _ => None,
})

// Get a raw sender for custom callbacks in props
ctx.sender()
```

## Child components

Embed child components with `ctx.child::<C>(props)`:

```rust
impl Component for Dashboard {
    type Props = ();
    type Msg = DashMsg;
    type Env = ();

    fn view(&self, _: &(), ctx: &Ctx<DashMsg>, _env: &()) -> El {
        el::div().children([
            el::h1().text("Dashboard"),
            ctx.child::<Counter>(Props { label: "Users".into() }),
            ctx.child::<Counter>(Props { label: "Score".into() }),
        ])
    }
    // ...
}
```

Children are keyed by `(position_index, TypeId)` -- stable as long as the order of `ctx.child` calls doesn't change.

## Child-to-parent communication

Pass a callback closure in props:

```rust
#[derive(Clone)]
struct CounterProps {
    label: String,
    on_change: Arc<dyn Fn(i32) + Send + Sync>,
}

// In Counter::update():
fn update(&mut self, msg: Msg, props: &CounterProps) -> ShouldRender {
    match msg {
        Msg::Inc => self.count += 1,
        Msg::Dec => self.count -= 1,
    }
    (props.on_change)(self.count);
    ShouldRender::No  // parent will re-render us
}

// In the parent's view():
fn view(&self, _: &(), ctx: &Ctx<AppMsg>, _env: &()) -> El {
    let sender = ctx.sender();
    ctx.child::<Counter>(CounterProps {
        label: "Score".into(),
        on_change: Arc::new(move |val| {
            sender.send(AppMsg::ScoreChanged(val));
        }),
    })
}
```

## App -- standalone applications

```rust
// Stateless (Env = ()):
App::new::<MyRoot>(my_props)
    .title("My App")
    .size(1280, 720)
    .stylesheet(include_str!("style.css"))
    .run()
    .unwrap();

// With shared state (Env = State):
App::with_state::<Dashboard>(my_state, my_props)
    .title("Dashboard")
    .run()
    .unwrap();
```

`App` wraps the winit harness, creates the tree, registers stylesheets, and drives the component update loop. `App<State>` holds `Arc<State>` and passes `&State` as the `Env` to `view()` each frame.

## Mount -- embedding in existing trees

For secondary windows or non-App contexts (e.g. devtools), use `Mount<C>`:

```rust
let mut mount = Mount::<MyComponent>::new(props);

// Initial render into an existing Tree:
mount.render(&mut tree, &env);

// Each frame, drain messages and re-render if needed:
mount.process(&mut tree, &env);

// When env changes externally:
mount.force_render(&mut tree, &env);
```

## How it works

```
callback fires
  -> MsgSender::send(msg)
  -> wake() -> window.request_redraw()
  -> on_frame hook
  -> Runtime::process()
  -> drain messages -> Component::update()
  -> ShouldRender::Yes -> Component::view()
  -> replace tree.root -> tree.generation += 1
  -> pipeline detects generation change
  -> re-cascade -> re-layout -> re-paint
```

No virtual DOM. Components do full subtree replacement on re-render. The pipeline cache avoids redundant work -- only re-cascading and re-laying-out when the DOM generation counter changes.

## Architecture

```
wgpu-html-ui/src/
  lib.rs          -- public API re-exports
  el.rs           -- El builder type, 73 element constructors, attribute/callback setters
  component.rs    -- Component trait, ShouldRender enum
  ctx.rs          -- Ctx<Msg> callback factory, MsgSender<Msg> channel
  runtime.rs      -- AnyComponent type erasure, MountedComponent tree, Runtime update loop
  mount.rs        -- Mount<C> for embedding components in existing trees
  app.rs          -- App<State> builder, UiHook (AppHook impl)
```
