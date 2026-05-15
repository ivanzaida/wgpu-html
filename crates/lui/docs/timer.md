# Timers

Lui provides `set_timeout`, `set_interval`, and `set_immediate` for scheduling deferred work. Callbacks receive `&mut HtmlDocument` and can mutate the DOM.

## Direct API

When you have `&mut Lui`:

```rust
use std::time::Duration;

// Fire once after 500ms
let handle = lui.set_timeout(Duration::from_millis(500), |lui| {
    if let Some(node) = lui.doc.root.query_selector_mut("#status") {
        node.children.clear();
        node.children.push(HtmlNode::text("Done"));
    }
});

// Fire every second
let handle = lui.set_interval(Duration::from_secs(1), |lui| {
    // full access to Lui: DOM, scroll state, selection, other timers...
});

// Fire on the next frame
lui.set_immediate(|lui| {
    // deferred work with full Lui access
});

// Cancel
lui.clear_timer(handle);
```

## Channel API

`timer_sender()` returns a `TimerSender` that is `Clone + Send`. Pass it to other threads, store it in closures, or hand it to async code.

```rust
let sender = lui.timer_sender();

// Move to another thread
std::thread::spawn(move || {
    std::thread::sleep(Duration::from_secs(2));
    sender.set_immediate(|doc| {
        // runs on next render_frame
    });
});
```

The sender enqueues requests via an `mpsc` channel. `Lui` drains the channel at the start of each `render_frame`.

## Timing

Timer resolution is tied to the frame rate. A 16ms frame interval means timers fire within ~16ms of their deadline. This matches browser behavior — `setTimeout(fn, 0)` doesn't fire instantly, it fires on the next event loop tick.

Timers fire **before** layout and paint within `render_frame`:

```
render_frame()
  1. timers.tick()          ← expired timers fire here
  2. cascade + layout
  3. paint
  4. hover transitions
  5. render to GPU
```

If a timer fires and mutates the DOM, the change is visible in the same frame's layout and paint.

## Interval behavior

`set_interval` re-enqueues itself after each fire. The next fire time is `now + interval`, not `previous_fire + interval`, so intervals don't accumulate drift from slow frames.

## API reference

| Method | Description |
|---|---|
| `lui.set_timeout(delay, cb)` | Fire `cb` once after `delay` |
| `lui.set_interval(interval, cb)` | Fire `cb` every `interval` |
| `lui.set_immediate(cb)` | Fire `cb` on the next frame |
| `lui.clear_timer(handle)` | Cancel a pending timer |
| `lui.timer_sender()` | Get a cloneable `Send` handle for cross-thread use |

All methods return a `TimerHandle` for cancellation. Callbacks are `FnMut(&mut Lui) + Send + 'static` — full access to the engine state.
