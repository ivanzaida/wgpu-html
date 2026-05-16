# lui::live — Reactive UI Framework

A Solid.js/React-inspired reactive component system for the lui wgpu renderer. Enabled via the `live` feature flag.

```bash
cargo run -p lui --features all --example counter
```

## Quick example

```rust
use lui::live::{Runtime, Ctx, batch};
use lui::live::builder::el::*;

fn counter(ctx: &Ctx) -> El {
    let count = ctx.signal(0i32);

    ctx.on_effect({
        let count = count.clone();
        move || println!("Count: {}", count.get())
    });

    div().children([
        p().text(&format!("Count: {}", count.get())),
        button().text("+1").on_click({
            let count = count.clone();
            move |_| count.update(|n| *n += 1)
        }),
    ])
}

fn main() {
    let mut lui = lui::Lui::new();
    lui.set_html(r#"<html><body><div id="app"></div></body></html>"#);

    let mut rt = Runtime::new("#app", counter);
    rt.render(&mut lui);

    // in frame loop:
    rt.process(&mut lui);
}
```

## Primitives

| Primitive | API | Description |
|---|---|---|
| Signal | `ctx.signal(initial)` | Reactive value, persists across re-renders |
| Memo | `ctx.memo(\|\| ...)` | Auto-tracked derived signal, skips if equal |
| Store | `ctx.store(value)` | Reactive wrapper over a struct |
| Lens | `store.lens(\|s\| s.field)` | Projected sub-signal from a store |
| Effect | `ctx.on_effect(\|\| ...)` | Auto-tracked side effect |
| Watch | `ctx.watch(&signal, \|val\| ...)` | Explicit single-signal watcher |
| Batch | `batch(\|\| { ... })` | Coalesce writes — effects + re-render deferred |
| Mounted | `ctx.on_mounted(\|\| ...)` | Fires once after first render |
| Unmounted | `ctx.on_unmounted(\|\| ...)` | Fires when component is dropped |
| Component | `ctx.component(fn, props)` | Child with own Ctx / hooks / lifecycle |
| Keyed | `ctx.keyed(key, fn, props)` | Keyed child for list reconciliation |
| ForEach | `ctx.for_each(items, key_fn, render)` | Keyed list helper |
| Show | `show(cond, \|\| el)` | Conditional rendering |
| Context | `ctx.provide(val)` / `ctx.use_context::<T>()` | DI down the tree |
| Styles | `ctx.styles(sheet)` / `ctx.scoped("class")` | Scoped CSS, deduped |
| NodeRef | `ctx.node_ref()` | Stable DOM handle |
| Untrack | `signal.get_untracked()` | Read without tracking |
| Command | `ctx.cmd(\|lui\| ...)` | Queue imperative DOM mutation |
| ErrorBoundary | `ctx.error_boundary(fn, fallback)` | Catch panics |

## Component identity

Every child has a `ChildKey(key, ComponentId, render_order)`:

- **Keyed** (key != ""): matched by `(key, ComponentId)` — survives reorder
- **Unkeyed**: matched by full key including position

Pruned children fire `on_unmounted`.

## Batch + render coalescing

`batch(|| { ... })` defers effect/watcher execution until the closure returns. Value-subscribers (dirty flag) fire immediately but are idempotent. Stale watchers are rejected via generation counter.

Re-render coalescing: `Runtime::process()` checks the dirty flag once per frame. Multiple signal writes between frames result in a single re-render regardless of batch.

## Scoped styles

- `ctx.styles(scoped_sheet("name", [...]))` registers once per ComponentId
- `ctx.scoped("class")` returns `"name-class"`
- `StyleRegistry` flushes adds/removes via `Lui::add_stylesheet`/`remove_stylesheet`
- Styles auto-removed when all instances of a component type unmount

## Architecture

- Feature-gated: `lui` crate, `live` feature, `lui::live::*`
- Hooks are positional (same order every render)
- Children stored in `HashMap<ChildKey, Arc<Ctx>>`
- Auto-tracking via thread-local; `signal.get()` registers watchers
- Watchers fire after value lock released (no re-entrancy issues)
- `TrackedEffect` uses generation counter to invalidate stale callbacks
- Shared across tree: dirty flag, command queue, style registry
- `parking_lot` for all locks

## Running tests

```bash
cargo test -p lui --features live --test live
```
