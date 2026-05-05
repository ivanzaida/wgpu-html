# wgpu-html-ui

Elm-architecture component framework for `wgpu-html`. No JSX, no macros — just Rust traits and a chainable builder DSL.

---

## Quick start

```rust
use wgpu_html_ui::{el, App, Component, Ctx, El, ShouldRender};

struct Counter { count: i32 }

#[derive(Clone)]
struct Props { label: String }

#[derive(Clone)]
enum Msg { Inc, Dec }

impl Component for Counter {
    type Props = Props;
    type Msg = Msg;
    type Env = ();

    fn create(_props: &Props) -> Self { Counter { count: 0 } }

    fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
        match msg {
            Msg::Inc => self.count += 1,
            Msg::Dec => self.count -= 1,
        }
        ShouldRender::Yes
    }

    fn view(&self, props: &Props, ctx: &Ctx<Msg>, _env: &()) -> El {
        el::div().class("counter").children([
            el::span().text(&props.label),
            el::button().text("-").on_click_cb(ctx.on_click(Msg::Dec)),
            el::span().text(&self.count.to_string()),
            el::button().text("+").on_click_cb(ctx.on_click(Msg::Inc)),
        ])
    }
}

fn main() {
    App::new::<Counter>(Props { label: "Count".into() })
        .stylesheet("body { font-family: sans-serif; }")
        .title("Counter")
        .size(400, 300)
        .run()
        .unwrap();
}
```

---

## Component trait

```rust
pub trait Component: 'static {
    type Props: Clone + 'static;
    type Msg:   Clone + Send + Sync + 'static;
    type Env:   'static;   // () for standalone; custom type for shared context

    fn create(props: &Self::Props) -> Self;
    fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;
    fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El;

    // Optional overrides:
    fn props_changed(&mut self, old: &Self::Props, new: &Self::Props) -> ShouldRender { Yes }
    fn mounted(&mut self, sender: MsgSender<Self::Msg>) {}  // receive sender for subscriptions
    fn updated(&mut self, props: &Self::Props) {}            // called after each self-triggered re-render
    fn destroyed(&mut self) {}
    fn scope()  -> &'static str  { "" }       // CSS scope prefix
    fn styles() -> Stylesheet    { empty() }  // scoped component stylesheet
}
```

- **Props** — immutable configuration from the parent; must be `Clone`.
- **Msg** — events the component handles; must be `Clone + Send + Sync`.
- **Env** — external data injected at the mount site (`&Env` in every `view` call). Use `()` for standalone apps, a custom struct (e.g. `Arc<AppState>`) for read-only shared context. For mutable shared state, use `Observable<T>`.
- **State** — the struct itself. Private, mutable, owned by the component.

---

## El builder DSL

Every HTML element has a constructor function in `wgpu_html_ui::el`. Chain methods to build the tree.

```rust
use wgpu_html_ui::el;

el::div().id("app").class("container").children([
    el::h1().text("Hello"),
    el::button()
        .class("btn")
        .text("Click me")
        .on_click(|_| println!("clicked")),
    el::input().configure(|i: &mut wgpu_html_models::Input| {
        i.placeholder = Some("Type here".into());
    }),
])
```

### El methods

| Method | Description |
|--------|-------------|
| `.id(v)` | Set the `id` attribute |
| `.class(v)` | Set the `class` attribute |
| `.style(v)` | Set inline `style` |
| `.attr_title(v)` | Set the `title` attribute |
| `.hidden(bool)` | Set `hidden` |
| `.tabindex(i32)` | Set `tabindex` |
| `.data(key, val)` | Set a `data-*` attribute |
| `.attribute(name, val)` | Set a raw attribute and reflect common input fields |
| `.custom_property(name, val)` | Set a CSS custom property |
| `.text(t)` | Append a text child node |
| `.child(el)` | Append a single child |
| `.children(iter)` | Append multiple children |
| `.bind(observable)` / `.bind_value(observable)` | Two-way bind an input/textarea string value |
| `.bind_checked(observable)` | Two-way bind checkbox/radio checked state |
| `.on_click(f)` | Attach a click handler (closure) |
| `.on_click_cb(cb)` | Attach a pre-built `MouseCallback` (from `ctx.on_click()`) |
| `.on_mouse_down/up/enter/leave(f/_cb)` | Other mouse events |
| `.on_event(f/_cb)` | General event handler (keyboard, focus, wheel, …) |
| `.configure(f)` | Mutate the underlying model struct for element-specific fields |
| `.into_node()` | Unwrap into a raw `Node` |

`El` implements `Clone` — it can be stored in `Props` for named-slot / content-projection patterns.

### Element constructors (73 total)

`div`, `span`, `p`, `a`, `button`, `input`, `textarea`, `select`, `label`, `form`,
`h1`–`h6`, `ul`, `ol`, `li`, `table`, `tr`, `td`, `th`, `img`, `nav`, `header`,
`footer`, `section`, `article`, `aside`, `main_el`, `pre`, `code`, `strong`, `em`,
and every other standard HTML element. See `el.rs` for the full list.

---

## Ctx — callback factory

`Ctx<Msg>` is passed to `view()`:

```rust
// Fixed message on click (returns MouseCallback; use with .on_click_cb)
ctx.on_click(Msg::Inc)

// Map event → message
ctx.callback(|ev| Msg::ClickedAt(ev.pos))

// General event handler (keyboard, focus, etc.)
ctx.event_callback(|ev| match ev {
    HtmlEvent::KeyDown { key, .. } if key == "Enter" => Some(Msg::Submit),
    _ => None,
})

// Raw sender for custom closures in props
ctx.sender()  // → MsgSender<Msg>

// Scoped class name (uses Component::scope())
ctx.scoped("card")  // → "mycomp-card"
```

---

## Child components

### Positional children (`ctx.child`)

```rust
fn view(&self, _: &(), ctx: &Ctx<DashMsg>, _env: &()) -> El {
    el::div().children([
        ctx.child::<Counter>(Props { label: "Users".into() }),
        ctx.child::<Counter>(Props { label: "Score".into() }),
    ])
}
```

Identity is `(call-site position, TypeId)` — stable as long as call order doesn't change.

### Keyed children (`ctx.keyed_child`)

For dynamic lists where items can be reordered or removed, pass an explicit string key so state survives reordering:

```rust
fn view(&self, _: &(), ctx: &Ctx<Msg>, _env: &()) -> El {
    let rows = self.items.iter().map(|item|
        ctx.keyed_child::<ItemRow>(
            item.id.to_string(),
            ItemProps { data: item.clone() },
        )
    );
    el::div().children(rows)
}
```

---

## Children — named slots / content projection

`El: Clone` lets you pass whole subtrees in props. `Children` is a cloneable `Vec<El>` newtype for variadic slots:

```rust
#[derive(Clone)]
struct CardProps {
    header: El,        // single named slot
    body:   Children,  // variadic slot
}

// In the parent:
ctx.child::<Card>(CardProps {
    header: el::h2().text("Title"),
    body: Children::from([
        el::p().text("paragraph 1"),
        el::p().text("paragraph 2"),
    ]),
})

// Inside Card::view:
fn view(&self, props: &CardProps, _ctx: &Ctx<Msg>, _env: &()) -> El {
    el::div().class("card").children([
        el::div().class("card-header").child(props.header.clone()),
        el::div().class("card-body").children(props.body.iter()),
    ])
}
```

---

## Observable — shared reactive state

`Observable<T>` wraps a value in `Arc<Mutex<T>>` and notifies subscribers on mutation. `Store<T>` remains as a compatibility alias.

```rust
use std::sync::LazyLock;
use wgpu_html_ui::Observable;

static THEME: LazyLock<Observable<String>> =
    LazyLock::new(|| Observable::new("dark"));

// Write from anywhere:
THEME.set("light".to_string());
THEME.update(|t| *t = "high-contrast".to_string());

// Read:
let current = THEME.get();   // → cloned value
```

### Subscribing from a component

Subscribe inside `mounted` — the `MsgSender` is already wired to the runtime wake function, so mutations trigger redraws automatically:

```rust
fn mounted(&mut self, sender: MsgSender<Msg>) {
    self.theme_sub = Some(THEME.subscribe_msg(&sender, |val| {
        Msg::ThemeChanged(val.clone())
    }));
}

fn update(&mut self, msg: Msg, _: &()) -> ShouldRender {
    match msg {
        Msg::ThemeChanged(t) => { self.theme = t; ShouldRender::Yes }
    }
}
```

Keep the returned `Subscription` in the component; dropping it unsubscribes.

### Binding form controls

```rust
use wgpu_html_ui::{Observable, el};

let name = Observable::new("");
let enabled = Observable::new(false);

el::div().children([
    el::input()
        .attribute("type", "text")
        .bind(name.clone()),
    el::input()
        .attribute("type", "checkbox")
        .bind_checked(enabled.clone()),
])
```

---

## Background tasks (`ctx.spawn`)

Spawn a blocking task on an OS thread; the return value arrives as a message:

```rust
fn view(&self, _: &Props, ctx: &Ctx<Msg>, _: &()) -> El {
    el::button()
        .text("Load file")
        .on_click_cb({
            let s = ctx.sender();
            std::sync::Arc::new(move |_: &_| {
                // or use ctx.spawn from within view if called inline:
            })
        })
}

// Inline in view (e.g. triggered by a flag set in update):
if self.should_load {
    ctx.spawn(|| {
        let data = std::fs::read_to_string("data.txt").unwrap_or_default();
        Msg::Loaded(data)
    });
}
```

`spawn` runs `f` on a new OS thread and enqueues the returned `Msg` via `MsgSender::send` when done.

---

## Lifecycle hooks

| Hook | When called |
|------|------------|
| `create(props)` | Component first instantiated |
| `mounted(sender)` | After first render; use to subscribe to `Observable`s or start background work |
| `update(msg, props)` | Every time a message arrives |
| `updated(props)` | After each re-render triggered by this component's own `update` returning `Yes` |
| `props_changed(old, new)` | When parent passes new props; return `No` to skip re-render |
| `destroyed()` | Just before the component is removed from the tree |

---

## Reconciliation / render caching

The runtime uses a **three-path** model to minimise work per frame:

### Path 1 — Clean fast-path
`!needs_render && !subtree_dirty` → return `last_node` immediately.  
No allocations, no `view()` call, zero work.

### Path 2 — Patch path *(new)*
`!needs_render && subtree_dirty` (parent clean, descendant dirty):
- Parent's `view()` is **skipped entirely**.
- Clone `skeleton_node` — the raw `view()` output stored from the last full render.  
  Skeletons contain tiny placeholder `<div>` nodes instead of full child subtrees, so cloning is cheap.
- Re-substitute every child: dirty children re-render recursively (path 3), clean children return their `last_node` (path 1).
- Result is cached as the new `last_node`.

This saves one `view()` call per **ancestor** of every updated leaf. In a dashboard with a root, a row component, and 10 leaf counters, clicking one counter triggers only that counter's `view()` — not the row's or the root's.

### Path 3 — Full render
`needs_render` (or first render):
- Call `view()` → reconcile child set (add/remove/update via key matching).
- Store raw output as `skeleton_node` (for future patch-path passes).
- Substitute children (dirty → re-render, clean → `last_node`).
- Cache as `last_node`.

### Per-component state

| Field | Role |
|-------|------|
| `last_node` | Fully-resolved cached output (children substituted) |
| `skeleton_node` | Raw `view()` output (placeholder divs, no child content) |
| `needs_render` | Set when `update()` returns `Yes` or props changed |
| `subtree_dirty` | Set when any descendant is dirty; propagated upward |

---

## Scoped component CSS

```rust
impl Component for MyCard {
    fn scope() -> &'static str { "card" }

    fn styles() -> Stylesheet {
        style::sheet([
            style::rule(".root").padding(px(16)).border_radius(px(8)),
            style::rule(".title").font_size(px(18)).font_weight(FontWeight::Bold),
            style::rule(".root:hover").background_color("#2a2a2a"),
        ])
    }

    fn view(&self, _: &(), ctx: &Ctx<Msg>, _: &()) -> El {
        el::div().class(ctx.scoped("root")).children([
            el::h3().class(ctx.scoped("title")).text("Hello"),
        ])
    }
}
// Generates CSS: .card-root { … }  .card-title { … }  .card-root:hover { … }
```

---

## Child-to-parent communication

Pass a callback in props:

```rust
#[derive(Clone)]
struct CounterProps {
    label:     String,
    on_change: Arc<dyn Fn(i32) + Send + Sync>,
}

// In Counter::update():
(props.on_change)(self.count);

// In the parent's view():
let sender = ctx.sender();
ctx.child::<Counter>(CounterProps {
    label:     "Score".into(),
    on_change: Arc::new(move |val| sender.send(AppMsg::ScoreChanged(val))),
})
```

---

## App — standalone applications

```rust
// Stateless (Env = ()):
App::new::<MyRoot>(my_props)
    .title("My App")
    .size(1280, 720)
    .stylesheet(include_str!("style.css"))
    .run()
    .unwrap();

// With shared read-only context (Env = MyState):
App::with_state::<Dashboard>(my_state, my_props)
    .title("Dashboard")
    .run()
    .unwrap();
```

`App<State>` holds `Arc<State>` and passes `&State` to every `view()` call. For mutable shared state use `Observable<T>`.

---

## Mount — embedding in existing trees

For secondary windows or non-App contexts (e.g. devtools panels):

```rust
let mut mount = Mount::<MyComponent>::new(props);

// Initial render into an existing Tree:
mount.render(&mut tree, &env);

// Each frame, drain messages and re-render if needed:
mount.process(&mut tree, &env);

// When env changes externally:
mount.force_render(&mut tree, &env);
```

---

## How it works

```
callback fires
  → MsgSender::send(msg)
  → wake() → window.request_redraw()
  → on_frame hook
  → Runtime::process()
    → drain messages → Component::update() → ShouldRender
    → if any dirty → render_component()
        → path 1: !needs_render && !subtree_dirty → return last_node (zero work)
        → path 2: !needs_render && subtree_dirty  → clone skeleton, re-patch children
                                                     (parent view() skipped)
        → path 3: needs_render → call view(), reconcile, store skeleton, cache node
    → replace tree.root → tree.generation += 1
  → pipeline detects generation change
  → re-cascade → re-layout → re-paint
```

No virtual-DOM diff of the `Node` tree. Each component replaces its own subtree on re-render. The per-component `last_node` cache prevents calling `view()` on clean subtrees.

---

## File layout

```
crates/wgpu-html-ui/src/
  lib.rs           public API re-exports
  el.rs            El (Clone), 73 constructors, attrs/callbacks, Children type
  style.rs         Stylesheet / Rule builder DSL
  core/observable.rs Observable<T> reactive shared state + subscriptions
  core/
    component.rs   Component trait, ShouldRender
    ctx.rs         Ctx<Msg> (child/keyed_child/spawn), MsgSender channel, ChildSlot
    runtime.rs     AnyComponent erasure, MountedComponent (cached), Runtime loop
  app/
    app.rs         App<State> builder, UiHook, SecondaryWindow trait
    mount.rs       Mount<C> for embedding in existing trees
```
