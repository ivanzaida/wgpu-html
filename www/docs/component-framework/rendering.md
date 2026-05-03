---
title: Component Rendering Model
---

# Component Rendering Model

The runtime (`wgpu-html-ui::core::runtime`) manages mounted components and selects one of three render paths per frame to minimize work.

## MountedComponent

Each mounted component is tracked as a `MountedComponent`:

```rust
struct MountedComponent {
    state: Box<dyn AnyComponent>,     // Type-erased Component
    last_node: Option<Node>,           // Full output from last view()
    skeleton_node: Option<Node>,       // Skeleton with placeholder children
    key: String,
    needs_render: bool,
    subtree_dirty: bool,
    // ...
}
```

## Three Render Paths

### 1. Clean Fast-Path

**Condition**: `!needs_render && !subtree_dirty`

**Work**: Nothing. The component's cached `last_node` is used as-is.

This is the common case when a component has no pending messages, props haven't changed, and no descendant needs rendering. Zero allocation, zero DOM manipulation.

### 2. Patch Path

**Condition**: `needs_render && skeleton_node.is_some() && !props_changed`

**Work**:
1. Skip the parent's `view()`.
2. Clone `skeleton_node` (the DOM structure without children).
3. Only re-substitute children whose `needs_render` is true.
4. Update `last_node` with the patched version.

This avoids expensive `view()` calls for the parent when only a single child changed. The parent's element structure is reused, and only dirty children are re-inserted.

### 3. Full Render

**Condition**: `needs_render && (props_changed || no skeleton)`

**Work**:
1. Call `view(props, ctx, env)` to produce a new `El` tree.
2. Reconcile child slots — match new children against existing mounted components by key.
3. For matched children: call `props_changed()` and optionally `update()` + `view()`.
4. For new children: call `create()`, `mounted()`, then `view()`.
5. For removed children: call `destroyed()`.
6. Build the output `Node` tree from the `El` tree and child components.
7. Cache `last_node` and `skeleton_node` for future patch paths.

## Keyed Children

Child identity is `(String, TypeId)`:

```rust
// Auto-generated key (positional)
ctx.child::<MyComp>(props)         → key = "__pos_{n}"

// User-specified key (stable across reordering)
ctx.keyed_child::<MyComp>("item-1", props)  → key = "item-1"
```

Keyed children survive:
- Reordering: the runtime matches by key, not position.
- Insertion: new keys create new components; existing keys keep their state.
- Removal: keys not present in the new view are destroyed.

Without keys, positional children are rebuilt on reorder (destroy + create).

## Message Processing

```rust
impl MsgSender<M> {
    pub fn send(&self, msg: M) {
        self.queue.lock().unwrap().push(msg);
        (self.wake)();  // triggers re-render loop
    }
}
```

`MsgSender::send()` enqueues a message and calls `wake()`, which the host maps to `request_redraw()`. The runtime's `process()` method drains all pending messages until the queue stabilizes:

```rust
while let Some(msg) = next_message() {
    component.update(msg, props)?;
    if component.needs_render() {
        component.render();
    }
}
```

## Per-Component Caching

- **last_node**: The complete `Node` tree from the previous `view()`. Used directly on the clean fast-path.
- **skeleton_node**: The `Node` structure from `view()` with all child component slots replaced by markers. Used for the patch path — the parent's structure is reused, only dirty children are substituted.

Both are invalidated on `props_changed` or when `view()` returns a structurally different tree.

## Performance Characteristics

| Path | view() calls | Child reconcile | DOM writes |
|---|---|---|---|
| Clean fast-path | 0 | 0 | 0 |
| Patch | 0 (parent) | Only dirty children | Minimal |
| Full render | 1 | All children | Full rebuild |

For a typical interactive app (hovering, typing, clicking), most frames hit the clean fast-path or patch path. Full renders only happen on prop changes, initial mount, or structural DOM mutations.
