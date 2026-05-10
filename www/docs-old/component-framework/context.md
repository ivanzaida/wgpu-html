---
title: Context
---

# Context

Context lets a parent component provide values that any descendant can access without prop-drilling. Values are keyed by Rust type (`TypeId`), so lookups are type-safe at compile time.

## Providing Context

Call `ctx.provide_context()` inside `view()`. The value is available to all children rendered in that same `view()` call (and their descendants):

```rust
fn view(&self, _props: &(), ctx: &Ctx<Msg>) -> El {
    ctx.provide_context(ThemeData { dark: self.is_dark, accent: "#3b82f6".into() });
    ctx.provide_context(AuthInfo { user_id: self.user_id.clone() });

    el::div().children([
        ctx.child::<Toolbar>(()),
        ctx.child::<MainContent>(()),
    ])
}
```

If the same type is provided multiple times in a chain of ancestors, the **nearest** ancestor wins.

## Consuming Context

Call `ctx.use_context::<T>()` from any descendant's `view()`:

```rust
fn view(&self, _props: &(), ctx: &Ctx<Msg>) -> El {
    let theme = ctx.use_context::<ThemeData>().unwrap();
    let bg = if theme.dark { "#1e1e1e" } else { "#ffffff" };

    el::div().style(&format!("background: {bg}")).children([
        el::span().text("Hello"),
    ])
}
```

Returns `Option<Arc<T>>` — `None` if no ancestor provided a value of that type.

## Type Keying

Context is keyed by `TypeId`, not by string. This means:

- No typos or name collisions.
- The compiler ensures you read back the same type you provided.
- To provide multiple values of the same underlying type, wrap them in newtypes:

```rust
struct PrimaryColor(String);
struct AccentColor(String);

// Provider
ctx.provide_context(PrimaryColor("#fff".into()));
ctx.provide_context(AccentColor("#3b82f6".into()));

// Consumer
let primary = ctx.use_context::<PrimaryColor>().unwrap();
let accent = ctx.use_context::<AccentColor>().unwrap();
```

## Context vs Observable

| | Context | Observable |
|---|---|---|
| **Scope** | Tree-scoped (parent to descendants) | Global (any component can subscribe) |
| **Reactivity** | Read at render time; not reactive on its own | Reactive — mutations trigger subscriber re-renders |
| **Use case** | Dependency injection (theme, config, auth) | Shared mutable state (counters, form data) |
| **Setup** | Zero boilerplate — provide and consume | Requires `subscribe()` wiring |

Context is an **injection mechanism**, not a reactive one. When the provider re-renders with a new value, children that re-render (for any reason) will see the updated value. Children that don't re-render continue using the value from their last render.

For reactive cross-tree state, combine both: provide an `Observable` via context, and subscribe in the consumer:

```rust
// Provider
ctx.provide_context(AppState {
    theme: self.theme_observable.clone(),
    locale: self.locale_observable.clone(),
});

// Consumer
fn create(props: &()) -> Self {
    Self { theme: None, current_theme: String::new() }
}

fn view(&self, _props: &(), ctx: &Ctx<Msg>) -> El {
    let state = ctx.use_context::<AppState>().unwrap();
    // Store the observable for subscribing in mounted()
    // Use self.current_theme for rendering
    el::div().text(&self.current_theme)
}

fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
    if let Some(theme) = &self.theme {
        subs.add(theme.subscribe_msg(sender, |v| Msg::ThemeChanged(v.clone())));
    }
}
```

## Complete Example

```rust
use std::sync::Arc;
use wgpu_html_ui::{Component, Ctx, El, ShouldRender, el};

// ── Context types ──────────────────────────────────────────────

#[derive(Clone)]
struct Theme {
    dark: bool,
    accent: String,
}

// ── Provider (root component) ──────────────────────────────────

struct App { is_dark: bool }

#[derive(Clone)]
enum AppMsg { ToggleTheme }

impl Component for App {
    type Props = ();
    type Msg = AppMsg;

    fn create(_: &()) -> Self { App { is_dark: true } }

    fn update(&mut self, msg: AppMsg, _: &()) -> ShouldRender {
        match msg {
            AppMsg::ToggleTheme => { self.is_dark = !self.is_dark; ShouldRender::Yes }
        }
    }

    fn view(&self, _: &(), ctx: &Ctx<AppMsg>) -> El {
        ctx.provide_context(Theme {
            dark: self.is_dark,
            accent: if self.is_dark { "#60a5fa" } else { "#2563eb" }.into(),
        });

        el::div().children([
            el::button().text("Toggle theme").on_click_cb(ctx.on_click(AppMsg::ToggleTheme)),
            ctx.child::<Card>(CardProps { title: "Settings".into() }),
        ])
    }
}

// ── Consumer (nested child) ────────────────────────────────────

struct Card;

#[derive(Clone)]
struct CardProps { title: String }

#[derive(Clone)]
enum CardMsg {}

impl Component for Card {
    type Props = CardProps;
    type Msg = CardMsg;

    fn create(_: &CardProps) -> Self { Card }

    fn update(&mut self, _: CardMsg, _: &CardProps) -> ShouldRender { ShouldRender::No }

    fn view(&self, props: &CardProps, ctx: &Ctx<CardMsg>) -> El {
        let theme = ctx.use_context::<Theme>().unwrap();
        let (bg, fg) = if theme.dark { ("#1e1e1e", "#e5e5e5") } else { ("#fff", "#111") };

        el::div()
            .style(&format!("background:{bg}; color:{fg}; padding:16px; border-left:3px solid {}", theme.accent))
            .children([
                el::h3().text(&props.title),
                el::p().text(if theme.dark { "Dark mode active" } else { "Light mode active" }),
            ])
    }
}
```

## How It Works

Context values are stored as `HashMap<TypeId, Arc<dyn Any + Send + Sync>>` and propagated through the mounted component tree:

1. During `view()`, calls to `provide_context()` collect values in the `Ctx`.
2. After `view()` returns, the runtime merges provided values with inherited context from ancestors.
3. When rendering children, the merged context is passed as their inherited context.
4. `use_context()` checks the component's own provided values first, then inherited.

The three render paths handle context as follows:

| Path | Context behavior |
|---|---|
| **Clean fast-path** | No-op — context unchanged |
| **Patch path** | Reuses stored context (parent didn't re-render) |
| **Full render** | Recomputes merged context from inherited + provided |
