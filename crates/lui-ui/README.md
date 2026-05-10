# lui-ui

Elm-architecture component framework for `lui`. No JSX, no macros — just Rust traits and a chainable builder DSL.

## Quick Start

```rust
use lui_ui::{el, Component, Ctx, El, ShouldRender};

struct Counter { count: i32 }

#[derive(Clone)]
struct Props { label: String }

#[derive(Clone)]
enum Msg { Inc, Dec }

impl Component for Counter {
    type Props = Props;
    type Msg = Msg;

    fn create(_props: &Props) -> Self { Counter { count: 0 } }

    fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
        match msg {
            Msg::Inc => self.count += 1,
            Msg::Dec => self.count -= 1,
        }
        ShouldRender::Yes
    }

    fn view(&self, props: &Props, ctx: &Ctx<Msg>) -> El {
        el::div().class("counter").children([
            el::span().text(&props.label),
            el::button().text("-").on_click_cb(ctx.on_click(Msg::Dec)),
            el::span().text(&self.count.to_string()),
            el::button().text("+").on_click_cb(ctx.on_click(Msg::Inc)),
        ])
    }
}
```

## Features

- **Component trait** — props, messages, lifecycle hooks, scoped CSS
- **El builder DSL** — 73 HTML element constructors with chainable API
- **Ctx callback factory** — type-safe event handlers, child embedding, background tasks
- **Context** — tree-scoped dependency injection via `provide_context` / `use_context`
- **Observable** — reactive shared state with automatic subscription management
- **Three-path rendering** — clean fast-path, patch path, and full render for minimal per-frame work
- **DOM patching** — in-place updates preserving form state and layout rects

## Documentation

Full documentation is available in the [Component Framework](../../www/docs/component-framework/index.md) section of the Docusaurus site:

- [Component Trait](../../www/docs/component-framework/component-trait.md) — lifecycle, hooks, styles
- [El Builder DSL](../../www/docs/component-framework/el-dsl.md) — element constructors, children, callbacks
- [Ctx Callback Factory](../../www/docs/component-framework/ctx.md) — message senders, child embedding, background tasks
- [Context](../../www/docs/component-framework/context.md) — tree-scoped dependency injection
- [Observable — Reactive State](../../www/docs/component-framework/store.md) — shared state, subscriptions
- [Rendering Model](../../www/docs/component-framework/rendering.md) — three-path render, DOM patching
