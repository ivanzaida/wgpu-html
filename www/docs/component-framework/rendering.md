---
title: Component Rendering Model
---

# Component Rendering Model

The runtime manages mounted components and selects one of three render paths per frame to minimize work.

## MountedComponent

Each mounted component is tracked as:

```rust
struct MountedComponent {
    state: Box<dyn AnyComponent>,
    last_node: Option<Node>,       // full output from last view()
    skeleton_node: Option<Node>,   // skeleton with placeholder children
    needs_render: bool,
    subtree_dirty: bool,
    subscriptions: Subscriptions,  // auto-cancelled on destroy
    child_context: ContextMap,     // inherited + provided context for children
}
```

## Three Render Paths

### 1. Clean Fast-Path

**Condition**: `!needs_render && !subtree_dirty`

Returns the cached `last_node` directly. No `view()` call, no allocation, zero work. This is the common steady-state path.

### 2. Patch Path

**Condition**: `!needs_render && subtree_dirty`

The component itself is unchanged but a descendant is dirty:

1. Skip the parent's `view()` entirely.
2. Clone `skeleton_node` (contains placeholder divs, not full child subtrees — cheap).
3. Re-substitute children: dirty ones re-render recursively, clean ones return their `last_node`.
4. Reuse the stored `child_context` (context is unchanged since the parent didn't re-render).
5. Cache the result as the new `last_node`.

### 3. Full Render

**Condition**: `needs_render` (or first render)

1. Call `view(props, ctx)` to produce a new `El` tree.
2. Merge inherited context (from parent) with any values provided via `ctx.provide_context()` during this `view()` call. Store the result as `child_context`.
3. Reconcile child slots by `(key, TypeId)`:
   - Matched children: call `props_changed()`, optionally re-render.
   - New children: `create()` → `view()` → `mounted()` → `subscribe()`.
   - Removed children: `destroyed()` (subscriptions auto-cancelled first).
4. Render children, passing `child_context` as their inherited context.
5. Cache `skeleton_node` and `last_node`.

## DOM Patching

When `apply_node` writes the component output into the `Tree`, it uses **in-place patching** (`patch_node`) instead of wholesale replacement. This preserves:

- **Form control values**: Input/textarea values typed by the user are kept when the component's stale cached node doesn't explicitly set a new value.
- **Layout rects**: `Node.rect` from the previous layout pass is preserved on same-tag nodes.
- **Interaction state**: `focus_path`, `hover_path`, `scroll_offsets` remain valid because nodes are updated in-place, not swapped.

The patch algorithm walks old and new trees in parallel:

- **Same tag**: Update attributes, replace event handlers, recurse into children.
- **Different tag**: Replace the node wholesale.
- **Children added**: Appended.
- **Children removed**: Truncated.

Event handlers are always replaced — components create fresh closures each render capturing current state.

## Keyed Children

Child identity is `(String, TypeId)`:

```rust
// Auto-generated key (positional)
ctx.child::<MyComp>(props)                     // key = "__pos_0"

// User-specified key (stable across reordering)
ctx.keyed_child::<MyComp>("item-1", props)     // key = "item-1"
```

Keyed children survive reordering, insertion, and removal. Without keys, positional children are rebuilt on reorder.

## Message Processing

```
callback fires → MsgSender::send(msg) → wake() → request_redraw()
    → Runtime::process()
    → drain messages → Component::update() → ShouldRender
    → if dirty → render_component() (path 1/2/3)
    → patch_node() into Tree → tree.generation += 1
    → pipeline detects change → cascade → layout → paint
```

Messages are batched — multiple messages arriving before the next frame produce a single re-render.

## Scoped Class Caching

`ctx.scoped("class")` returns `ArcStr`. Results are cached in a per-component `HashMap<&'static str, ArcStr>` that persists across renders. After the first frame, repeated calls are a hash lookup + refcount bump — zero allocation.

## Performance Characteristics

| Path | view() calls | Child reconcile | DOM writes |
|---|---|---|---|
| Clean fast-path | 0 | 0 | 0 |
| Patch | 0 (parent) | Only dirty children | Minimal |
| Full render | 1 | All children | Full subtree |
