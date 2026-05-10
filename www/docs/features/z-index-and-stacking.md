---
sidebar_position: 10
---

# Z-Index and Stacking

## Z-Index Support

`z-index` is parsed and stored per element. Paint order is influenced by z-index values:

```css
.modal { position: absolute; z-index: 100; }
.tooltip { position: relative; z-index: 50; }
```

## Paint Order Layers

Siblings are sorted into three layers for paint order:

1. **Negative layer** — items with `z-index < 0`
2. **Auto layer** — non-positioned items and items with `z-index: auto`
3. **Non-negative layer** — items with `z-index >= 0`

Within each layer, items are sorted by their integer z-index value. Source order is preserved for equal z-index values.

## Positioning Requirement

`z-index` only applies to positioned elements (`position: relative | absolute | fixed`). Static elements are always in the auto layer.

## Current Limitations

- **No independent stacking contexts** — cross-branch ordering is still tree DFS. A deeply nested `z-index: 999` paints behind a shallow `z-index: 1` in a different subtree.
- Sibling sort works correctly; full stacking context isolation is the next step.
- `opacity < 1`, `transform`, and other stacking-context triggers don't create new stacking contexts.

## Paint Walk Behavior

The paint walk visits:
1. Element's own background and borders
2. Negative z-index children
3. Auto z-index children (in-flow + non-negative)
4. Non-negative z-index children
5. Element's text and foreground content

This matches the CSS 2.1 Appendix E painting order within a single stacking context.
