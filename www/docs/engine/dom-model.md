---
sidebar_position: 3
---

# DOM Model

lui maintains its own DOM-like tree in `lui-tree`. This is not a web DOM — there is no JavaScript, no `document`, no live `NodeList`. It is a Rust-native tree of elements.

## Tree Structure

```rust
pub struct Tree {
    pub root: Node,
    pub fonts: FontDatabase,
    pub interaction: InteractionState,
    pub generation: u64,
    pub dirty_paths: Vec<Vec<usize>>,
    pub preload_queue: Vec<String>,
}

pub struct Node {
    pub element: Element,
    pub children: Vec<Node>,
    pub on_click: Option<Callback>,
    pub on_mouse_enter: Option<Callback>,
    pub on_mouse_leave: Option<Callback>,
    pub on_mouse_down: Option<Callback>,
    pub on_mouse_up: Option<Callback>,
    pub on_focus: Option<Callback>,
    pub on_blur: Option<Callback>,
    pub on_event: Option<TypedEventCallback>,
}
```

## Element Types

Each HTML element type has its own Rust struct with typed attributes. ~100 element types are modeled, including:

- **Content**: `Div`, `P`, `Span`, `A`, `H1`-`H6`, `Pre`, `Br`, `Hr`
- **Sections**: `Body`, `Header`, `Footer`, `Nav`, `Main`, `Section`, `Article`, `Aside`
- **Forms**: `Input`, `Textarea`, `Button`, `Select`, `Option`, `Label`, `Fieldset`
- **Tables**: `Table`, `Tr`, `Td`, `Th`, `Thead`, `Tbody`, `Tfoot`
- **Media**: `Img`, `Video`, `Audio`, `Picture`, `Source`
- **Lists**: `Ul`, `Ol`, `Li`, `Dl`, `Dt`, `Dd`
- **Metadata**: `Head`, `Title`, `Meta`, `Link`, `Style`, `Script`

## Building the Tree

The tree is typically built via parsing:

```rust
let mut tree = lui_parser::parse(html_string);
```

It can also be constructed programmatically by building `Node` structures directly.

## InteractionState

The tree carries all runtime interaction state:

```rust
pub struct InteractionState {
    pub hover_path: Vec<usize>,
    pub active_path: Vec<usize>,
    pub focus_path: Option<Vec<usize>>,
    pub focus_visible: bool,
    pub text_selection: Option<TextSelection>,
    pub scroll_offsets_y: BTreeMap<Vec<usize>, f32>,
    pub scroll_offsets_x: BTreeMap<Vec<usize>, f32>,
    pub modifiers: Modifiers,
    pub mouse_buttons: u8,
    pub time_origin: Instant,
}
```

## Event Callbacks

Elements carry optional Rust closures for interactivity:

```rust
// Legacy per-event-type callbacks
node.on_click = Some(Box::new(|| { println!("clicked!"); }));

// Typed event dispatch
node.on_event = Some(Box::new(|event: &HtmlEvent| {
    match &event.event_type {
        HtmlEventType::Click(_) => { /* ... */ }
        _ => {}
    }
}));
```

## Query Selectors

DOM-style element lookup:

```rust
tree.get_element_by_id("my-id");
tree.get_elements_by_class_name("my-class");
tree.query_selector("div.container > p");
tree.query_selector_all("input[type=text]:focus");
```

The query engine supports full CSS Level 4 selectors: all combinators, attribute selectors, pseudo-classes (`:is()`, `:where()`, `:not()`, `:has()`, `:nth-child()`, etc.).

## Node Paths

Every node in the tree is identified by a `Vec<usize>` path — the sequence of child indices from root to the target node. Paths are used for:
- Hit-testing (returning the deepest matching path)
- Event dispatch (targeting the correct element)
- Focus tracking (storing the focused element's path)
- Scroll offset storage (per-element scroll positions)
