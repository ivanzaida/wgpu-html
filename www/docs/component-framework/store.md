---
title: Store — Reactive Shared State
---

# Store — Reactive Shared State

`Store<T>` wraps a value of type `T` behind `Arc<Mutex<T>>` with a listener list. Any number of components can share a clone of the same store. When the value is mutated, all subscribers are notified.

## Types

```rust
pub struct Store<T> {
    inner: StoreInner<T>,
    // Arc<Mutex<T>> value + Mutex<Vec<Box<dyn Fn(&T)>>
}
```

`Store<T>` is cheap to clone — all clones share the same underlying value via `Arc`.

## Creating a Store

```rust
use wgpu_html_ui::Store;

#[derive(Clone)]
struct Theme { primary: String, background: String }

static THEME_STORE: Store<Theme> = Store::new(Theme {
    primary: "#4a90d9".into(),
    background: "#ffffff".into(),
});
```

Or use `once_cell` / `LazyLock` for lazy initialization:

```rust
use std::sync::LazyLock;
static THEME_STORE: LazyLock<Store<Theme>> = LazyLock::new(|| {
    Store::new(Theme::default())
});
```

Requirements: `T: Send + Sync + 'static`.

## Reading

```rust
let theme = THEME_STORE.get();  // Returns a clone of T
```

`get()` requires `T: Clone`. It locks the mutex, clones the value, and unlocks.

## Writing

```rust
// Replace the entire value
THEME_STORE.set(Theme {
    primary: "#ff0000".into(),
    background: "#1a1a1a".into(),
});

// Mutate in-place
THEME_STORE.update(|theme| {
    theme.primary = "#ff0000".into();
});
```

Both `set()` and `update()` notify all subscriber callbacks synchronously on the calling thread. Notifications happen outside the lock.

## Subscribing from a Component

Subscribe inside `Component::mounted()`:

```rust
fn mounted(&mut self, sender: MsgSender<Msg>) {
    THEME_STORE.subscribe(&sender, |theme: &Theme| {
        Msg::ThemeChanged(theme.clone())
    });
}
```

`subscribe()` registers a callback that maps `&T` to a message and sends it via `MsgSender`. The subscription is active for the component's lifetime.

## Raw Listeners

```rust
THEME_STORE.on_change(|theme: &Theme| {
    println!("Theme changed to: {:?}", theme.primary);
});
```

`on_change()` registers a raw callback without a `MsgSender`. This is useful for non-component code.

## Subscription Limitations

- Subscriptions are never automatically removed. If a component is destroyed while a store it subscribed to still lives, the `MsgSender` clone keeps its queue alive. Messages sent to the orphaned queue are silently discarded on the next `process()` cycle.
- This is a minor memory overhead, not a crash. A future version will add `SubscriptionHandle` with automatic cleanup on drop.

## Complete Example

```rust
use wgpu_html_ui::{Component, Ctx, ShouldRender, Store, el};

static THEME: Store<String> = Store::new("light".into());

struct MyComponent { current_theme: String }

enum Msg { ToggleTheme, ThemeChanged(String) }

impl Component for MyComponent {
    type Props = ();
    type Msg = Msg;
    type Env = ();

    fn create(_: &()) -> Self {
        Self { current_theme: THEME.get() }
    }

    fn mounted(&mut self, sender: MsgSender<Msg>) {
        THEME.subscribe(&sender, |theme| Msg::ThemeChanged(theme.clone()));
    }

    fn update(&mut self, msg: Msg, _: &()) -> ShouldRender {
        match msg {
            Msg::ToggleTheme => {
                let new = if self.current_theme == "light" { "dark" } else { "light" };
                THEME.set(new.into());
                ShouldRender::No  // Subscribe will trigger the re-render
            }
            Msg::ThemeChanged(theme) => {
                self.current_theme = theme;
                ShouldRender::Yes
            }
        }
    }

    fn view(&self, _: &(), ctx: &Ctx<Msg>, _: &()) -> El {
        el::div()
            .class(&format!("app-{}", self.current_theme))
            .children([
                el::p().text(&format!("Current theme: {}", self.current_theme)),
                el::button()
                    .text("Toggle")
                    .on_click_cb(ctx.on_click(Msg::ToggleTheme)),
            ])
    }
}
```
