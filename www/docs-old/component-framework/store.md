---
title: Observable — Reactive Shared State
---

# Observable — Reactive Shared State

`Observable<T>` wraps a value behind `Arc<Mutex<T>>` with a subscriber list. Any number of components can share clones of the same observable. Mutations notify all subscribers synchronously.

## Creating

```rust
use wgpu_html_ui::Observable;

// From a value
let theme = Observable::new("dark");

// Default (requires T: Default)
let count: Observable<i32> = Observable::default();
```

`Observable<T>` is cheap to clone — all clones share the same inner value via `Arc`.

Requirements: `T: Send + Sync + 'static`.

## Reading

```rust
let current = theme.get();  // clones T out of the mutex
```

For `ArcStr`-backed observables (`Observable<ArcStr>`), `get()` is a refcount bump — no deep copy.

## Writing

```rust
// Replace
theme.set("light");

// Mutate in place
counter.update(|n| *n += 1);
```

Both `set()` and `update()` notify all subscribers synchronously after releasing the lock.

## Subscribing from a Component

Use the `subscribe()` lifecycle hook. Subscriptions added to `subs` are automatically cancelled when the component is destroyed:

```rust
fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
    subs.add(self.theme.subscribe_msg(sender, |v| Msg::ThemeChanged(v.clone())));
}
```

`subscribe_msg` maps `&T` to a message and sends it via `MsgSender`, which wakes the component runtime to process the update.

### Raw subscriptions

For non-component code:

```rust
let sub = theme.subscribe(|value| {
    println!("Theme changed: {}", value);
});
// sub is a Subscription — dropping it unsubscribes.
```

`Subscription` is `#[must_use]` — the compiler warns if you forget to store it.

## Subscription Bag

`Subscriptions` is a type-erased bag that holds any `Subscription<T>`. Used by the runtime to manage component subscriptions, but can also be used standalone:

```rust
use wgpu_html_ui::Subscriptions;

let mut subs = Subscriptions::new();
subs.add(theme.subscribe(|v| println!("{}", v)));
subs.add(count.subscribe(|v| println!("{}", v)));

// Drop the bag to cancel all subscriptions
subs.clear();
```

## Two-Way Form Binding

`Observable<ArcStr>` integrates with form controls via `El::bind()`:

```rust
let name = Observable::new(ArcStr::from(""));

el::input()
    .bind(name.clone())       // user input → observable
    .placeholder("Your name")
```

`bind()` sets the initial value from the observable and updates it on every `input` event. The DOM node's value is preserved across re-renders via in-place patching.

## Complete Example

```rust
use wgpu_html_ui::{Component, Ctx, El, MsgSender, Observable, ShouldRender, Subscriptions, el};
use wgpu_html_models::ArcStr;

struct ThemeToggle {
    theme: Observable<ArcStr>,
    current: ArcStr,
}

#[derive(Clone)]
struct Props { theme: Observable<ArcStr> }

#[derive(Clone)]
enum Msg { Toggle, Changed(ArcStr) }

impl Component for ThemeToggle {
    type Props = Props;
    type Msg = Msg;

    fn create(props: &Props) -> Self {
        Self { theme: props.theme.clone(), current: props.theme.get() }
    }

    fn update(&mut self, msg: Msg, _: &Props) -> ShouldRender {
        match msg {
            Msg::Toggle => {
                let next = if &*self.current == "light" { "dark" } else { "light" };
                self.theme.set(ArcStr::from(next));
                ShouldRender::No  // subscription handles the re-render
            }
            Msg::Changed(v) => {
                self.current = v;
                ShouldRender::Yes
            }
        }
    }

    fn view(&self, _: &Props, ctx: &Ctx<Msg>) -> El {
        el::div().children([
            el::span().text(&format!("Theme: {}", self.current)),
            el::button().text("Toggle").on_click_cb(ctx.on_click(Msg::Toggle)),
        ])
    }

    fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
        subs.add(self.theme.subscribe_msg(sender, |v| Msg::Changed(v.clone())));
    }
}
```
