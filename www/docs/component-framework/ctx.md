---
title: Ctx Callback Factory
---

# Ctx Callback Factory

`Ctx<Msg>` is the callback factory passed to `Component::view()`. It creates event handlers that send messages to the component's update loop.

## Creating a Ctx

`Ctx` is created by the runtime — you never create one manually. It's passed to `view()`:

```rust
fn view(&self, props: &Props, ctx: &Ctx<Msg>, env: &Env) -> El { ... }
```

## Mouse Callbacks

### on_click(msg)

Sends a pre-built message on click:

```rust
el::button()
    .text("Increment")
    .on_click_cb(ctx.on_click(Msg::Increment))
```

### callback(|ev| Msg)

Maps an event to a message:

```rust
el::div()
    .on_click_cb(ctx.callback(|ev: &MouseEvent| {
        Msg::ClickedAt(ev.pos.0, ev.pos.1)
    }))
```

### Other mouse targets

```rust
el::button().on_mouse_down_cb(ctx.on_click(Msg::Pressed))
el::button().on_mouse_up_cb(ctx.on_click(Msg::Released))
el::div().on_mouse_enter_cb(ctx.on_click(Msg::Hovered))
el::div().on_mouse_leave_cb(ctx.on_click(Msg::Unhovered))
```

All `_cb` variants accept `Arc<dyn Fn(&MouseEvent)>` returned by Ctx methods.

## Event Callbacks

### event_callback(|ev| Option<Msg>)

Maps any `HtmlEvent` to an optional message. Return `None` to ignore the event:

```rust
el::input().on_event_cb(ctx.event_callback(|ev: &HtmlEvent| {
    if let HtmlEventType::Input { data } = &ev.event_type {
        Some(Msg::InputChanged(data.clone()))
    } else {
        None
    }
}))
```

### on_event (raw closure)

```rust
el::div().on_event(|ev: &HtmlEvent| {
    // Handle any event type
})
```

## MsgSender

```rust
pub struct MsgSender<M> {
    queue: Arc<Mutex<Vec<M>>>,
    wake: Arc<dyn Fn() + Send + Sync>,
}

impl<M> MsgSender<M> {
    pub fn send(&self, msg: M);
}
```

Obtain via `Ctx::sender()`:

```rust
let sender = ctx.sender();
// Store sender, use later from another thread
std::thread::spawn(move || {
    let result = fetch_data();
    sender.send(Msg::DataLoaded(result));
});
```

## Scoped CSS Classes

```rust
let scoped_class = ctx.scoped("highlight");
// If scope() == "my-component" → "my-component-highlight"
// If scope() == "" → "highlight"

el::div().class(&ctx.scoped("card"))
```

## scoped() Helper

When `Component::scope()` returns a prefix, `Ctx::scoped("class")` prepends it:

```rust
fn scope() -> &'static str { "my-component" }

// In view():
el::div().class(&ctx.scoped("btn"))  // → class="my-component-btn"
```

This ensures CSS isolation without manual class name management.

## Child Component Embedding

### child::<C>(props)

Embed a child component at the current position:

```rust
el::div().children([
    el::h2().text("User List"),
    ctx.child::<UserList>(UserListProps { filter: "active".into() }),
])
```

Returns an `El` containing a marker node — the runtime replaces it with the child's rendered output.

### keyed_child::<C>(key, props)

Keyed variant for stable identity across re-renders:

```rust
items.iter().enumerate().map(|(i, item)| {
    ctx.keyed_child::<ItemRow>(
        &format!("item-{}", item.id),  // stable key
        ItemRowProps { item: item.clone() },
    )
})
```

Keyed children survive reordering and insertion without unmounting. The key is `(String, TypeId)`.

## Background Thread → Message

```rust
fn view(&self, _props: &Props, ctx: &Ctx<Msg>, _env: &()) -> El {
    let sender = ctx.sender();

    el::button()
        .text("Load Data")
        .on_click(move |_| {
            let sender = sender.clone();
            std::thread::spawn(move || {
                let data = fetch_from_api();
                sender.send(Msg::DataLoaded(data));
            });
        })
}
```

`MsgSender::send()` enqueues the message and calls `wake()` to trigger a re-render. The runtime drains all pending messages during `process()`.
