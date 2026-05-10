---
title: Ctx Callback Factory
---

# Ctx Callback Factory

`Ctx<Msg>` is passed to `Component::view()`. It creates event handlers that send messages, embeds child components, and produces scoped class names.

## Mouse Callbacks

```rust
// Fixed message
el::button().on_click_cb(ctx.on_click(Msg::Increment))

// Map event to message
el::div().on_click_cb(ctx.callback(|ev: &MouseEvent| {
    Msg::ClickedAt(ev.pos.0, ev.pos.1)
}))
```

All mouse targets: `on_click_cb`, `on_mouse_down_cb`, `on_mouse_up_cb`, `on_mouse_enter_cb`, `on_mouse_leave_cb`.

## Event Callbacks

```rust
// Map any HtmlEvent; return None to ignore
el::input().on_event_cb(ctx.event_callback(|ev| {
    if let HtmlEvent::Input(input) = ev {
        Some(Msg::Typed(input.value.clone()))
    } else {
        None
    }
}))
```

## MsgSender

Thread-safe message queue. Obtain via `ctx.sender()`:

```rust
let sender = ctx.sender();
std::thread::spawn(move || {
    let result = fetch_data();
    sender.send(Msg::DataLoaded(result));
});
```

`send()` enqueues the message and calls `wake()` to trigger a re-render on the next frame.

## Scoped Class Names

`ctx.scoped("class")` returns an `ArcStr` with the component's scope prefix:

```rust
el::div().class(ctx.scoped("card"))
// If styles().scoped("mycomp") → class="mycomp-card"
// If no scope                  → class="card"
```

Results are **cached across renders** in a per-component hash map. After the first frame, repeated calls are a lookup + refcount bump — zero allocation.

## Child Components

### Positional (`ctx.child`)

```rust
el::div().children([
    ctx.child::<UserList>(UserListProps { filter: "active".into() }),
    ctx.child::<StatusBar>(StatusBarProps),
])
```

Identity is `(position, TypeId)` — stable as long as call order doesn't change.

### Keyed (`ctx.keyed_child`)

For dynamic lists where items can be reordered:

```rust
let rows = items.iter().map(|item| {
    ctx.keyed_child::<ItemRow>(
        &item.id.to_string(),
        ItemRowProps { data: item.clone() },
    )
});
el::div().children(rows)
```

Keyed children survive reordering and insertion without unmounting. Identity is `(key, TypeId)`.

## Background Tasks

`ctx.spawn()` runs a closure on a background OS thread. When it returns a message, that message is enqueued via `MsgSender`:

```rust
fn view(&self, _props: &Props, ctx: &Ctx<Msg>) -> El {
    el::button()
        .text("Load file")
        .on_click_cb(ctx.on_click(Msg::StartLoad))
}

fn update(&mut self, msg: Msg, _: &Props) -> ShouldRender {
    match msg {
        Msg::StartLoad => {
            // spawn is available via the sender
            // typically triggered from update, not view
            ShouldRender::No
        }
        Msg::Loaded(data) => {
            self.data = data;
            ShouldRender::Yes
        }
    }
}
```

Inline in `view()` (when a flag triggers a one-shot load):

```rust
if self.should_load {
    ctx.spawn(|| {
        let data = std::fs::read_to_string("data.txt").unwrap_or_default();
        Msg::Loaded(data)
    });
}
```

## Child-to-Parent Communication

Pass a callback in props:

```rust
#[derive(Clone)]
struct CounterProps {
    label: String,
    on_change: Arc<dyn Fn(i32) + Send + Sync>,
}

// In Counter::update:
(props.on_change)(self.count);

// In the parent's view:
let sender = ctx.sender();
ctx.child::<Counter>(CounterProps {
    label: "Score".into(),
    on_change: Arc::new(move |val| sender.send(AppMsg::ScoreChanged(val))),
})
```

## Context

`ctx.provide_context()` and `ctx.use_context()` allow passing data to descendants without prop-drilling. See [Context](./context) for full documentation.
