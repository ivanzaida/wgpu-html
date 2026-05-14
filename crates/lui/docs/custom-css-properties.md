# Custom CSS Properties & Pseudo-Elements

Lui defines engine-specific CSS properties and pseudo-elements for scrollbar styling. It also implements a subset of standard CSS scrollbar properties.

---

## Scrollbar Pseudo-Elements

### `::lui-scrollbar`

Represents the scrollbar container. Controls layout behavior and high-level scrollbar configuration.

| Property | Values | Default | Inherited |
|---|---|---|---|
| `scrollbar-mode` | `auto` \| `classic` \| `overlay` \| `none` | `auto` | no |
| `scrollbar-width` | `auto` \| `thin` \| `none` | `auto` | no |
| `scrollbar-inset` | `<length>{1,4}` | `0px` | no |
| `scrollbar-min-thumb-size` | `<length>` | `20px` | no |

```css
.panel::lui-scrollbar {
  scrollbar-mode: overlay;
  scrollbar-width: 10px;
  scrollbar-inset: 4px;
  scrollbar-min-thumb-size: 24px;
}
```

### `::lui-scrollbar-thumb`

Represents the draggable thumb.

| Property | Values | Default |
|---|---|---|
| `background-color` | `<color>` | `#888` |
| `border-radius` | `<length>{1,4}` | `4px` |
| `opacity` | `<number>` | `1` |

```css
.panel::lui-scrollbar-thumb {
  background-color: rgba(255, 255, 255, 0.35);
  border-radius: 999px;
}
```

### `::lui-scrollbar-track`

Represents the track behind the thumb.

| Property | Values | Default |
|---|---|---|
| `background-color` | `<color>` | `#222` |
| `border-radius` | `<length>{1,4}` | `0px` |
| `opacity` | `<number>` | `1` |

```css
.panel::lui-scrollbar-track {
  background-color: rgba(0, 0, 0, 0.1);
  border-radius: 999px;
}
```

### `::lui-scrollbar-corner`

Represents the corner where vertical and horizontal scrollbars meet.

| Property | Values | Default |
|---|---|---|
| `background-color` | `<color>` | `#222` |
| `opacity` | `<number>` | `1` |

```css
.panel::lui-scrollbar-corner {
  background-color: transparent;
}
```

---

## Custom Properties

### `scrollbar-mode`

Controls whether the scrollbar reserves layout space.

| Value | Meaning |
|---|---|
| `auto` | Engine/platform default |
| `classic` | Scrollbar reserves layout space |
| `overlay` | Scrollbar floats above content, no layout impact |
| `none` | Scrollbar hidden; scrolling still works |

### `scrollbar-inset`

Inset from the scrollbar container edges. Uses the same 1-to-4-value expansion as `margin`/`padding`:

```css
scrollbar-inset: 4px;             /* all sides */
scrollbar-inset: 4px 8px;         /* block inline */
scrollbar-inset: 2px 4px 6px 8px; /* top right bottom left */
```

### `scrollbar-min-thumb-size`

Minimum visual length of the thumb in pixels. Prevents the thumb from becoming too small on very long content.

```css
scrollbar-min-thumb-size: 24px;
```

---

## Standard Properties (implemented)

### `scrollbar-width`

Standard CSS property ([CSS Scrollbars Level 1](https://drafts.csswg.org/css-scrollbars-1/#propdef-scrollbar-width)).

| Value | Resolved width |
|---|---|
| `auto` | 15px |
| `thin` | 8px |
| `none` | 0px (hidden) |

### `scrollbar-color`

Standard CSS property ([CSS Scrollbars Level 1](https://drafts.csswg.org/css-scrollbars-1/#propdef-scrollbar-color)). Inherited.

```css
scrollbar-color: auto;            /* engine defaults */
scrollbar-color: dark;            /* dark theme */
scrollbar-color: light;           /* light theme */
scrollbar-color: #888 #222;       /* thumb-color track-color */
```

When both `scrollbar-color` and pseudo-element styles are present, pseudo-element styles take precedence for element scrollbars.

### `scrollbar-gutter`

Standard CSS property ([CSS Overflow 3](https://drafts.csswg.org/css-overflow-3/#propdef-scrollbar-gutter)).

```css
scrollbar-gutter: auto;           /* default */
scrollbar-gutter: stable;         /* always reserve space */
scrollbar-gutter: stable both-edges; /* reserve on both sides */
```

---

## UA Defaults

The engine applies these defaults via the UA stylesheet when the `ua_whatwg` feature is enabled:

```css
html, body, * {
  scrollbar-width: auto;
  scrollbar-color: #888 #222;
}

*::lui-scrollbar {
  scrollbar-mode: auto;
  scrollbar-width: auto;
  scrollbar-inset: 0px;
  scrollbar-min-thumb-size: 20px;
}

*::lui-scrollbar-track {
  background-color: #222;
  border-radius: 0px;
  opacity: 1;
}

*::lui-scrollbar-thumb {
  background-color: #888;
  border-radius: 4px;
  opacity: 1;
}

*::lui-scrollbar-corner {
  background-color: #222;
  opacity: 1;
}
```

---

## Interaction

Scrollbar thumbs are draggable via mouse. Clicking on the track (outside the thumb) jumps the scroll position to that location. Both element scrollbars and viewport scrollbars support drag interaction.
