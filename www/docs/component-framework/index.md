---
title: Component Framework (wgpu-html-ui)
---

# Component Framework (`wgpu-html-ui`)

`wgpu-html-ui` is an Elm-architecture component framework for building interactive UI with wgpu-html. It provides state management, scoped CSS, render caching, and a builder DSL — all in Rust.

## Elm Architecture

```
┌─────────────────┐
│     Props       │  ← Immutable config from parent
├─────────────────┤
│    Component    │
│   (self.state)  │  ← Private mutable state
├─────────────────┤
│  view() → El    │  ← Produces element tree
├─────────────────┤
│  update(msg)    │  ← Message → state change → re-render?
└─────────────────┘
        │
   MsgSender → callbacks fire → Msg enqueued → update() called
```

## Component Trait

```rust
pub trait Component: 'static {
    type Props: Clone + 'static;
    type Msg: Clone + Send + Sync + 'static;
    type Env: 'static;

    fn create(props: &Self::Props) -> Self;
    fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;
    fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El;
}
```

## El Builder DSL

73 element constructor functions with chainable builders:

```rust
el::div().id("app").class("container").children([
    el::h1().text("Hello"),
    el::button().text("Click").on_click_cb(ctx.on_click(Msg::Clicked)),
])
```

## Ctx Callback Factory

`Ctx<Msg>` creates callbacks that send messages to the component's update loop:

```rust
el::button().on_click_cb(ctx.on_click(Msg::Increment));
el::input().on_event_cb(ctx.event_callback(|ev| Some(Msg::Input(ev.data.clone()))));
```

## Store — Reactive Shared State

```rust
let theme_store: Store<Theme> = Store::new(Theme::default());

// Read
let t = theme_store.get();

// Write (notifies all subscribers)
theme_store.set(Theme::dark());

// Subscribe from a component
theme_store.subscribe(&sender, |theme| Msg::ThemeChanged(theme.clone()));
```

## Three-Path Render Model

Per mounted component, the runtime selects one of three paths:

1. **Clean fast-path**: Zero work when neither the component nor any descendant needs re-render.
2. **Patch path**: Skip `view()`, clone the skeleton node, re-substitute only dirty children.
3. **Full render**: Call `view()`, reconcile children, cache output for future patches.

## Entry Points

```rust
// Mount a root component into a Tree
use wgpu_html_ui::App;

let app = App::new::<MyRootComponent>(props);
app.mount(&mut tree);  // runs create() → view() → build tree

// Programmatic mount
use wgpu_html_ui::Mount;
Mount::new::<MyComponent>(props).at("#my-element", &mut tree);
```

## Sub-Pages

- [Component Trait](./component-trait) — lifecycle, hooks, styles
- [El Builder DSL](./el-dsl) — element constructors, children, callbacks
- [Ctx Callback Factory](./ctx) — message senders, child embedding
- [Store — Reactive State](./store) — shared state, subscriptions
- [Rendering Model](./rendering) — three-path render, keyed children
