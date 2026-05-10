---
title: Devtools
---

# Devtools

The `wgpu-html-devtools` crate provides a visual inspector panel built with `wgpu-html-ui` components. It opens in a secondary OS window for inspecting the host tree.

## Features

- **Component tree browser**: Expand/collapse the DOM tree, see element tag names and IDs.
- **Styles inspector**: View the cascaded style properties for the selected element.
- **Breadcrumb bar**: Navigate the element hierarchy with a breadcrumb trail.
- **Element picker**: Click the picker button, then hover elements in the host window to highlight them. Click to select.
- **Search/filter**: Filter the tree by tag name, class, or ID.
- **Live updates**: The tree reflects changes in the host document in real time.

## Visual Design

The devtools window uses a dark theme:
- Component tree on the left (resizable).
- Styles panel on the right.
- Breadcrumb bar at the top.
- Element picker button in the toolbar.

Built entirely with HTML/CSS rendered by wgpu-html — the devtools are a wgpu-html application inspecting another wgpu-html application.

## Attaching to a Host Tree

```rust
use wgpu_html_devtools::Devtools;

let devtools = Devtools::new();
devtools.attach(&mut host_tree);
```

`Devtools::attach()` registers hooks on the host tree so the devtools receive update events. The devtools panel opens in its own OS window and runs on the same event loop.

## API

```rust
pub struct Devtools {
    window: Option<HtmlWindow>,
    host_tree_hook: Option<TreeHookHandle>,
    // ...
}

impl Devtools {
    pub fn new() -> Self;

    /// Attach the devtools to a host tree. Opens the inspector window
    /// and starts receiving tree updates.
    pub fn attach(&mut self, host_tree: &mut Tree);

    /// Detach from the host tree and close the inspector window.
    pub fn detach(&mut self, host_tree: &mut Tree);

    /// Check if the devtools window is currently open.
    pub fn is_open(&self) -> bool;
}
```

## Implementation

The devtools are built with `wgpu-html-ui` components:

```rust
struct DevtoolsComponent {
    // Internal state: expanded nodes, selected path, filter, picker mode
}

impl Component for DevtoolsComponent {
    type Props = DevtoolsProps;
    type Msg = DevtoolsMsg;
    type Env = DevtoolsEnv;  // carries reference to host tree

    fn view(&self, props: &Props, ctx: &Ctx<Msg>, env: &Env) -> El {
        el::div().class("devtools-root").children([
            // Toolbar with breadcrumb + filter + picker
            self.view_toolbar(ctx),
            // Main area: tree panel + divider + styles panel
            el::div().class("main").children([
                self.view_tree(env),
                self.view_divider(),
                self.view_styles(props, env),
            ]),
        ])
    }
}
```

## Tree Hooks

The devtools use `TreeHook` to receive updates from the host:

```rust
host_tree.hooks.push(TreeHookHandle::new(
    move |event: &TreeLifecycleEvent| {
        match event {
            TreeLifecycleEvent::Render { viewport, layout } => {
                // Update devtools tree with new layout info
            }
            _ => {}
        }
        TreeHookResponse::Continue
    }
));
```

## Fonts

The devtools embed the Lucide icon font at compile time (`include_bytes!("../fonts/lucide.ttf")`) for toolbar icons (picker, expand/collapse arrows, search).

## Usage with Winit Harness

In the winit harness, devtools integration is manual:

```rust
use wgpu_html_winit::{create_window, AppHook};
use wgpu_html_devtools::Devtools;

struct MyApp { devtools: Devtools }

impl AppHook for MyApp {
    fn on_frame(&mut self, ctx: HookContext<'_>, _timings: &FrameTimings) {
        // Devtools hooks fire automatically
    }

    fn on_window_event(&mut self, _ctx: HookContext<'_>, window_id: WindowId, event: &WindowEvent) -> bool {
        // Forward events to the devtools window
        false
    }
}

let mut app = MyApp { devtools: Devtools::new() };
app.devtools.attach(&mut tree);

create_window(&mut tree)
    .with_title("My App")
    .with_hook(app)
    .run()
    .unwrap();
```

## Limitations

- **Single devtools instance per process**: winit supports only one event loop, so only one devtools window can be open.
- **No computed styles view**: Only cascaded property values are shown; the "computed" final values are not yet exposed.
- **No layout overlay**: The devtools don't overlay margin/padding/border rectangles on the host window.
- **No style editing**: Styles are read-only for now.
