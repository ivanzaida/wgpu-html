---
title: Component Framework (lui-ui)
---

# Component Framework (`lui-ui`)

`lui-ui` is an Elm-architecture component framework for building interactive UI with lui. It provides state management, scoped CSS, render caching, and a builder DSL — all in Rust.

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

    fn create(props: &Self::Props) -> Self;
    fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;
    fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>) -> El;

    // Optional hooks:
    fn props_changed(&mut self, old: &Self::Props, new: &Self::Props) -> ShouldRender { Yes }
    fn mounted(&mut self, sender: MsgSender<Self::Msg>) {}
    fn subscribe(&self, sender: &MsgSender<Self::Msg>, subs: &mut Subscriptions) {}
    fn updated(&mut self, props: &Self::Props) {}
    fn destroyed(&mut self) {}
    fn styles() -> Stylesheet { Stylesheet::empty() }
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

`Ctx<Msg>` is passed to `view()`:

```rust
el::button().on_click_cb(ctx.on_click(Msg::Increment));

// Scoped class names (cached across renders — zero alloc after first frame)
el::div().class(ctx.scoped("card"))  // → "mycomp-card"
```

## Observable — Reactive Shared State

```rust
use lui_ui::Observable;

let theme = Observable::new("dark");

// Write (notifies all subscribers)
theme.set("light");

// Read (cheap ArcStr clone)
let current = theme.get();

// Subscribe from a component (auto-cleaned on destroy)
fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
    subs.add(self.theme.subscribe_msg(sender, |v| Msg::ThemeChanged(v.clone())));
}
```

## Context

Provide values at a point in the tree that any descendant can consume — no prop-drilling:

```rust
// Parent
ctx.provide_context(Theme { dark: true, accent: "#3b82f6".into() });

// Any descendant
let theme = ctx.use_context::<Theme>().unwrap();
```

Type-keyed via `TypeId` — compile-time safe, no string keys. Context is an injection mechanism; for reactive cross-tree state, provide an `Observable` via context.

## Three-Path Render Model

Per mounted component, the runtime selects one of three paths:

1. **Clean fast-path**: Zero work when neither the component nor any descendant needs re-render.
2. **Patch path**: Skip `view()`, clone the skeleton node, re-substitute only dirty children.
3. **Full render**: Call `view()`, reconcile children, cache output for future patches.

The DOM is patched in-place (not replaced wholesale), preserving form control values and interaction state. Context values are propagated through the tree during rendering — merged from inherited (parent) and provided (this component) at each level.

## Entry Points

```rust
// Standalone mount (e.g. devtools, secondary windows)
use lui_ui::Mount;
let mount = Mount::<MyComponent>::new(props);
mount.render(&mut tree);
mount.process(&mut tree);
```

## Sub-Pages

- [Component Trait](./component-trait) — lifecycle, hooks, styles
- [El Builder DSL](./el-dsl) — element constructors, children, callbacks
- [Ctx Callback Factory](./ctx) — message senders, child embedding, background tasks
- [Context](./context) — tree-scoped dependency injection
- [Observable — Reactive State](./store) — shared state, subscriptions
- [Rendering Model](./rendering) — three-path render, DOM patching
