---
title: Component Trait
---

# Component Trait

The `Component` trait defines the lifecycle and API of a component. It's the core of the Elm-architecture model.

## Full Definition

```rust
pub trait Component: 'static {
    type Props: Clone + 'static;
    type Msg: Clone + Send + Sync + 'static;
    type Env: 'static;

    fn create(props: &Self::Props) -> Self;
    fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;
    fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El;

    fn props_changed(&mut self, old: &Self::Props, new: &Self::Props) -> ShouldRender {
        ShouldRender::Yes  // default: always re-render on new props
    }
    fn mounted(&mut self, sender: MsgSender<Self::Msg>) {}
    fn updated(&mut self, props: &Self::Props) {}
    fn destroyed(&mut self) {}

    fn scope() -> &'static str { "" }
    fn styles() -> Stylesheet { Stylesheet::empty() }
}
```

## Lifecycle

```
create(props)
    │
    ▼
mounted(sender)       ← Called once after first mount.
    │                   Store the sender for Store::subscribe.
    │
    ▼
view()                ← Called on each render.
    │
    ├─ update(msg) ──→ ShouldRender::Yes → view() → updated()
    │
    ├─ props_changed  → ShouldRender::Yes → view()
    │
    ▼
destroyed()           ← Called before removal.
```

## create(props)

Factory method. Called once when the component is first mounted. Receive initial props, return a new instance:

```rust
fn create(props: &Props) -> Self {
    Counter {
        count: props.initial_count,
    }
}
```

## update(msg, props)

Handle a message. Return `ShouldRender::Yes` to trigger `view()`:

```rust
enum Msg { Increment, Decrement, Reset }

fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
    match msg {
        Msg::Increment => { self.count += 1; ShouldRender::Yes }
        Msg::Decrement => { self.count -= 1; ShouldRender::Yes }
        Msg::Reset => { self.count = 0; ShouldRender::Yes }
    }
}
```

## view(props, ctx, env)

Produce the element tree for the current state:

```rust
fn view(&self, props: &Props, ctx: &Ctx<Msg>, _env: &()) -> El {
    el::div().class("counter").children([
        el::span().text(&format!("{}: {}", props.label, self.count)),
        el::button().text("+").on_click_cb(ctx.on_click(Msg::Increment)),
        el::button().text("-").on_click_cb(ctx.on_click(Msg::Decrement)),
    ])
}
```

The `env` parameter carries external data provided by the mount site. Standalone apps use `Env = ()`.

## props_changed(old, new)

Called when the parent passes new props. Default always re-renders. Override to skip unnecessary renders:

```rust
fn props_changed(&mut self, old: &Props, _new: &Props) -> ShouldRender {
    if old.label == _new.label && old.initial_count == _new.initial_count {
        ShouldRender::No
    } else {
        ShouldRender::Yes
    }
}
```

## mounted(sender)

Called once after first mount. Store the `MsgSender` for lifecycle-scoped subscriptions:

```rust
fn mounted(&mut self, sender: MsgSender<Msg>) {
    GLOBAL_STORE.subscribe(&sender, |data| Msg::DataChanged(data.clone()));
}
```

## updated(props)

Called after every re-render triggered by `update()`. Analogous to React's `componentDidUpdate`. Not called after the initial mount render.

## destroyed()

Called before the component is removed from the tree. Clean up resources.

## scope() → &'static str

CSS scope prefix. When non-empty, all class names in `styles()` and `Ctx::scoped()` are auto-prefixed:

```rust
fn scope() -> &'static str { "my-component" }  // .btn → .my-component-btn
```

## styles() → Stylesheet

Scoped CSS registered once when the component type is first mounted:

```rust
fn styles() -> Stylesheet {
    Stylesheet::parse(r#"
        .btn { background: #4a90d9; color: white; padding: 8px 16px; border-radius: 4px; }
        .btn:hover { background: #357abd; }
    "#)
}
```

If `scope()` returns `"my-component"`, the `.btn` selector is auto-scoped to `.my-component-btn`.
