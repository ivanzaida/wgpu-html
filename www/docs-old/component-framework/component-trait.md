---
title: Component Trait
---

# Component Trait

The `Component` trait defines the lifecycle and API of a component.

## Definition

```rust
pub trait Component: 'static {
    type Props: Clone + 'static;
    type Msg: Clone + Send + Sync + 'static;

    fn create(props: &Self::Props) -> Self;
    fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;
    fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>) -> El;

    // Optional hooks (defaults shown):
    fn props_changed(&mut self, _old: &Self::Props, _new: &Self::Props) -> ShouldRender { Yes }
    fn mounted(&mut self, _sender: MsgSender<Self::Msg>) {}
    fn subscribe(&self, _sender: &MsgSender<Self::Msg>, _subs: &mut Subscriptions) {}
    fn updated(&mut self, _props: &Self::Props) {}
    fn destroyed(&mut self) {}
    fn styles() -> Stylesheet { Stylesheet::empty() }
}
```

## Lifecycle

```
create(props)
    ↓
view(props, ctx)        ← first render
    ↓
mounted(sender)         ← called once; store sender for async work
    ↓
subscribe(sender, subs) ← set up Observable subscriptions (auto-cleaned)
    ↓
┌─ update(msg, props)   ← handle a message
│      ↓ ShouldRender::Yes
│  view(props, ctx)     ← re-render
│      ↓
│  updated(props)       ← post-render hook (not on initial mount)
└──────┘
    ↓ (parent removes component)
destroyed()             ← subscriptions already cancelled
```

## create(props)

Factory method. Called once when the component is first mounted:

```rust
fn create(props: &Props) -> Self {
    Counter { count: props.initial_count }
}
```

No side effects here — use `mounted()` for subscriptions and async work.

## update(msg, props)

Handle a message. Return `ShouldRender::Yes` to trigger `view()`:

```rust
fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
    match msg {
        Msg::Increment => { self.count += 1; ShouldRender::Yes }
        Msg::Reset => { self.count = 0; ShouldRender::Yes }
    }
}
```

## view(props, ctx)

Produce the element tree. Use `ctx.scoped()` for scoped class names (cached across renders — zero allocation after the first frame):

```rust
fn view(&self, props: &Props, ctx: &Ctx<Msg>) -> El {
    el::div().class(ctx.scoped("root")).children([
        el::span().text(&format!("{}: {}", props.label, self.count)),
        el::button().text("+").on_click_cb(ctx.on_click(Msg::Increment)),
    ])
}
```

## props_changed(old, new)

Called when the parent passes new props. Default always re-renders. Override to skip:

```rust
fn props_changed(&mut self, old: &Props, new: &Props) -> ShouldRender {
    if old.label == new.label { ShouldRender::No } else { ShouldRender::Yes }
}
```

## mounted(sender)

Called once after the initial render. Store the `MsgSender` for background threads or manual subscriptions:

```rust
fn mounted(&mut self, sender: MsgSender<Msg>) {
    self.sender = Some(sender);
}
```

## subscribe(sender, subs)

Set up `Observable` subscriptions that live as long as the component. Called once after `mounted()`. Subscriptions added to `subs` are automatically cancelled on destroy — no need to store them as struct fields:

```rust
fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
    subs.add(self.theme.subscribe_msg(sender, |v| Msg::ThemeChanged(v.clone())));
    subs.add(self.count.subscribe_msg(sender, |v| Msg::CountSync(*v)));
}
```

`Subscription` is marked `#[must_use]` — forgetting to store or add it to `subs` produces a compiler warning.

## updated(props)

Called after every re-render triggered by `update()` returning `Yes`. Not called after the initial mount.

## destroyed()

Called before the component is removed. Runtime-managed subscriptions (from `subscribe()`) are already cancelled at this point.

## styles() → Stylesheet

Scoped CSS registered once when the component type is first mounted. Use `.scoped("prefix")` on the stylesheet; `ctx.scoped()` in `view()` produces matching class names:

```rust
fn styles() -> Stylesheet {
    style::sheet([
        style::rule(".root").padding(px(16)).border_radius(px(8)),
        style::rule(".root:hover").background_color("#2a2a2a"),
    ]).scoped("card")
}
```

Generates: `.card-root { padding: 16px; ... }` `.card-root:hover { ... }`

## Complete Example

```rust
use wgpu_html_ui::{Component, Ctx, El, MsgSender, Observable, ShouldRender, Subscriptions, el, style};
use wgpu_html_models::common::Display;

struct Counter {
    count: i32,
    shared: Observable<i32>,
}

#[derive(Clone)]
struct Props { label: String, shared: Observable<i32> }

#[derive(Clone)]
enum Msg { Inc, Dec, Synced(i32) }

impl Component for Counter {
    type Props = Props;
    type Msg = Msg;

    fn create(props: &Props) -> Self {
        Counter { count: 0, shared: props.shared.clone() }
    }

    fn update(&mut self, msg: Msg, _: &Props) -> ShouldRender {
        match msg {
            Msg::Inc => { self.count += 1; self.shared.set(self.count); }
            Msg::Dec => { self.count -= 1; self.shared.set(self.count); }
            Msg::Synced(v) => { self.count = v; }
        }
        ShouldRender::Yes
    }

    fn view(&self, props: &Props, ctx: &Ctx<Msg>) -> El {
        el::div().class(ctx.scoped("root")).children([
            el::span().text(&props.label),
            el::button().text("-").on_click_cb(ctx.on_click(Msg::Dec)),
            el::span().text(&self.count.to_string()),
            el::button().text("+").on_click_cb(ctx.on_click(Msg::Inc)),
        ])
    }

    fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
        subs.add(self.shared.subscribe_msg(sender, |v| Msg::Synced(*v)));
    }

    fn styles() -> style::Stylesheet {
        style::sheet([
            style::rule(".root")
                .display(Display::Flex)
                .gap(style::px(8)),
        ]).scoped("counter")
    }
}
```
