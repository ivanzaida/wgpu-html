---
sidebar_position: 1
---

# Supported HTML

lui supports **96 HTML tags** with typed element structs. Unknown tags are silently dropped (subtree discarded).

## Fully Supported

| Element | Notes |
|---|---|
| `html`, `body` | Document root; `body` also used as synthetic wrapper for multi-root docs |
| `div`, `span` | Generic block/inline containers |
| `p` | Paragraph with auto-close handling |
| `h1`–`h6` | Headings with UA font-size/weight/margin |
| `pre` | Preformatted text, monospace |
| `blockquote`, `address` | Block content with UA styles |
| `hr` | Horizontal rule, void element |
| `br` | Forced line break, void element |
| `wbr` | Word-break opportunity |
| `header`, `nav`, `main`, `section`, `article`, `aside`, `footer` | Sectioning content |
| `a` | Inline anchor, focusable with `href` |
| `strong`, `b` | Bold via CSS |
| `em`, `i` | Italic via CSS |
| `u`, `ins`, `s`, `del` | Underline/strikethrough via CSS |
| `small`, `mark` | Smaller text / highlight |
| `code`, `kbd`, `samp` | Monospace |
| `cite`, `dfn`, `var` | Italic phrasing |
| `abbr` | Abbreviation with dotted underline |
| `sub`, `sup` | Subscript/superscript |
| `time`, `data` | Phrasing with `datetime`/`value` |
| `bdi`, `bdo` | Bidi isolation/override |
| `ul`, `ol`, `li` | Lists with basic marker support |
| `dl`, `dt`, `dd` | Description lists |
| `label` | Form label |
| `style` | Inline stylesheet (collected by cascade) |
| `template` | Inert content container |

## Partially Supported

| Element | What Works | Gaps |
|---|---|---|
| `img` | Loads/renders from `src`, `srcset`; uses intrinsic dimensions | Image maps, full `<picture>` source selection |
| **`table`** | **Full table layout**: `table`/`table-row-group`/`table-row`/`table-cell`/`table-caption` display, `colspan`/`rowspan`, column distribution, `border-spacing` | Border-collapse limited |
| `tr`, `td`, `th` | Table row/cell layout, `colspan`/`rowspan` | — |
| `thead`, `tbody`, `tfoot` | `table-header-group`/`row-group`/`footer-group` | — |
| `colgroup`, `col` | Parsed with `span` | Column styling not propagated |
| `input` | See [Forms and Inputs](forms-and-inputs) for type breakdown | Varies by type |
| `textarea` | Multiline editing, placeholder, resize | No form submission |
| `button` | Focusable, styled as button | No form submission |
| `select`, `option` | Parsed, styled | No popup list |
| `form` | Parsed with attributes | No submission/validation |
| `fieldset`, `legend` | Block layout, border | Limited native behavior |
| `details`, `summary` | Parsed | No toggle behavior |
| `dialog` | Parsed with `open` | No modal/top-layer |
| `video`, `audio` | Parsed with media attributes | No playback |
| `svg` | Rasterized as replaced element | Limited nested SVG |
| `ruby`, `rt`, `rp` | Ruby display | Limited layout |
| `canvas`, `meter`, `progress` | Parsed | No native rendering |

## Input Type Support

| Type | Level | Notes |
|---|---|---|
| `text`, `password` | **Full** | Editing, placeholder, caret, selection, masking |
| `checkbox`, `radio` | **Full** | Click toggle, native paint (checkmark/dot), `accent-color` |
| `range` | **Full** | Track+thumb, `min`/`max`/`step`, drag adjust |
| `color` | **Full** | Swatch + full color picker overlay |
| `date`, `datetime-local` | **Full** | Segmented editing, locale formatting, calendar picker |
| `file` | **Full** | Native file dialog (rfd), filenames display |
| `submit`, `reset`, `button` | **Full** | Button styling, static |
| `hidden` | **Full** | `display: none` |
| `email`, `number`, `search`, `tel`, `url` | **Partial** | Like `text`, no validation |
| `time`, `week`, `month` | **Partial** | Like `text`, no picker |
| `image` | **Partial** | Like `text`, no image submit |

## Parsed Only (no runtime behavior)

`head`, `title`, `meta`, `link`, `script` (JS out of scope), `slot`, `track`, `source`, `noscript`, `datalist`, `output`

## Not Supported (dropped)

`figure`, `figcaption`, `hgroup`, `menu`, `search`, `q`, `area`, `base`, `embed`, `map`, `object`, `math`
