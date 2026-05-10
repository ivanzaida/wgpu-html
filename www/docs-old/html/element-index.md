---
title: Element Index
---

# Element Index

This page lists every HTML element type recognized by lui. Each element is parsed into a typed Rust struct with per-element attribute parsing. Unknown tags are silently dropped.

> **Total:** 98 element types (plus `Text` for text nodes). `<script>` and `<noscript>` are parsed but entirely ignored at runtime.

## Document Structure

| Element | Description | Category | Notes |
|---|---|---|---|
| `html` | Root element | Document | Stores `xmlns` attribute |
| `head` | Document metadata container | Document | `display: none` in UA |
| `body` | Document body | Document | Default `margin: 8px` in UA |
| `title` | Document title | Document | Raw-text element; content captured |
| `link` | External resource link | Document | `<link rel="stylesheet">` parsed but not auto-fetched — host must call `tree.register_linked_stylesheet()` |
| `meta` | Document metadata | Document | `name`, `content`, `charset`, `http-equiv` parsed |
| `style` | Embedded CSS stylesheet | Document | Raw-text element; CSS parsed by stylesheet parser |

## Sections & Structure

| Element | Description | Category | Notes |
|---|---|---|---|
| `h1`–`h6` | Headings (1–6) | Section | UA defaults: `h1`=2em bold, `h2`=1.5em bold, `h3`=1.17em bold, `h4`=1em bold, `h5`=0.83em bold, `h6`=0.67em bold; all have `margin-block: 0.67em` |
| `header` | Introductory content | Section | Block element |
| `footer` | Footer content | Section | Block element |
| `nav` | Navigation section | Section | Block element |
| `main` | Main content area | Section | Block element |
| `section` | Generic document section | Section | Block element |
| `article` | Self-contained composition | Section | Block element |
| `aside` | Tangential content | Section | Block element |
| `div` | Generic container | Section | Block element |
| `span` | Generic inline container | Section | Inline element |

## Text & Block Content

| Element | Description | Category | Notes |
|---|---|---|---|
| `p` | Paragraph | Block text | Auto-close on next `p`, `div`, heading, etc. |
| `br` | Line break | Inline | Void element |
| `hr` | Thematic break / horizontal rule | Block text | Void element; painted as a line |
| `pre` | Preformatted text | Block text | Preserves whitespace |
| `blockquote` | Block quotation | Block text | UA `margin: 1em 40px` |
| `address` | Contact information | Block text | Usually italic |

## Lists

| Element | Description | Category | Notes |
|---|---|---|---|
| `ul` | Unordered list | List | Block; UA `margin: 1em 0`, `padding-left: 40px` |
| `ol` | Ordered list | List | Block; same UA margins as `ul` |
| `li` | List item | List | Auto-close on next `li` |
| `dl` | Description list | List | Block |
| `dt` | Description term | List | Auto-close on next `dt`/`dd` |
| `dd` | Description details | List | Auto-close on next `dt`/`dd` |

## Tables

| Element | Description | Category | Notes |
|---|---|---|---|
| `table` | Table root | Table | Parsed; falls through to block layout (no table layout yet) |
| `caption` | Table caption | Table | |
| `colgroup` | Column group | Table | |
| `col` | Column definition | Table | Void element |
| `thead` | Table header group | Table | Auto-close on `tbody`/`tfoot` |
| `tbody` | Table body group | Table | Auto-close on `thead`/`tfoot` |
| `tfoot` | Table footer group | Table | Auto-close on `tbody` |
| `tr` | Table row | Table | Auto-close on next `tr` |
| `th` | Table header cell | Table | Auto-close on next `th`/`td` |
| `td` | Table data cell | Table | Auto-close on next `th`/`td` |

## Forms

| Element | Description | Category | Notes |
|---|---|---|---|
| `form` | Form container | Form | `name`, `action`, `method`, `target` attributes; no submission yet |
| `input` | Form input (22 types) | Form | Void element; `type`, `value`, `placeholder`, `name`, `checked`, `disabled`, `readonly`, `required`, `min`, `max`, `step`, `pattern`, `multiple`, `autofocus` parsed. Types: `text`, `password`, `email`, `url`, `tel`, `search`, `number`, `range`, `date`, `time`, `datetime-local`, `month`, `week`, `color`, `checkbox`, `radio`, `file`, `hidden`, `image`, `submit`, `reset`, `button` |
| `textarea` | Multi-line text input | Form | Raw-text element; full text editing with caret |
| `select` | Dropdown select | Form | `name`, `disabled`, `required`, `multiple`, `autofocus`; no dropdown UI yet |
| `option` | Select option | Form | `value`, `selected`, `disabled` |
| `optgroup` | Option group | Form | `disabled` |
| `label` | Label for form control | Form | `for` attribute |
| `button` | Button | Form | `type` (button/submit/reset), `value`, `name`, `disabled`, `autofocus`; focusable |
| `fieldset` | Form control group | Form | `name`, `disabled` |
| `legend` | Fieldset caption | Form | |
| `datalist` | Predefined input options | Form | |
| `output` | Calculation output | Form | `name`, `for` |
| `progress` | Progress bar | Form | `value` parsed as `f32` |
| `meter` | Scalar measurement | Form | `value` parsed as `f32` |

## Media

| Element | Description | Category | Notes |
|---|---|---|---|
| `img` | Image | Media | `src`, `alt`, `width`, `height`; async loading with HTTP(S)/file/data-URI; GIF/WebP animation; two-level cache with TTL |
| `svg` | Inline SVG | Media | Rasterized via `resvg` to a GPU texture |
| `source` | Media source | Media | Void element; `src`, `type` |
| `picture` | Responsive image container | Media | |
| `figure` | Figure with caption | Media | |
| `figcaption` | Figure caption | Media | |
| `area` | Image map area | Media | |
| `map` | Image map | Media | |
| `canvas` | Bitmap canvas | Media | GPU-backed via wgpu texture |
| `video` | Video element | Media | `src`; not rendered |
| `audio` | Audio element | Media | `src`; not rendered |
| `track` | Media text track | Media | Void element; `src` |
| `iframe` | Nested browsing context | Media | `src`, `name`; not rendered |

## Interactive

| Element | Description | Category | Notes |
|---|---|---|---|
| `a` | Hyperlink / anchor | Interactive | `href`, `type`; focusable when `href` is set; inline element |
| `details` | Disclosure widget | Interactive | `name` |
| `summary` | Details summary/legend | Interactive | Focusable; triggers details open/close |
| `dialog` | Dialog box | Interactive | |

## Inline Text Semantics

| Element | Description | Category | Notes |
|---|---|---|---|
| `strong` | Strong importance | Inline | UA: `font-weight: bold` |
| `b` | Bring attention | Inline | UA: `font-weight: bold` |
| `em` | Stress emphasis | Inline | UA: `font-style: italic` |
| `i` | Idiomatic text | Inline | UA: `font-style: italic` |
| `u` | Unarticulated annotation | Inline | UA: `text-decoration: underline` |
| `s` | Strikethrough | Inline | UA: `text-decoration: line-through` |
| `del` | Deleted text | Inline | UA: `text-decoration: line-through` |
| `ins` | Inserted text | Inline | UA: `text-decoration: underline` |
| `code` | Inline code | Inline | UA: `font-family: monospace` |
| `kbd` | Keyboard input | Inline | UA: `font-family: monospace` |
| `samp` | Sample output | Inline | UA: `font-family: monospace` |
| `var` | Variable | Inline | Usually italic |
| `mark` | Marked/highlighted text | Inline | |
| `small` | Small print | Inline | UA: `font-size: smaller` |
| `sub` | Subscript | Inline | UA: `vertical-align: sub`, `font-size: smaller` |
| `sup` | Superscript | Inline | UA: `vertical-align: super`, `font-size: smaller` |
| `abbr` | Abbreviation | Inline | |
| `cite` | Citation | Inline | UA: `font-style: italic` |
| `q` | Inline quotation | Inline | Displays with quotes |
| `dfn` | Defining instance | Inline | Usually italic |
| `time` | Date/time | Inline | |
| `wbr` | Line break opportunity | Inline | Void element |
| `bdi` | Bidirectional isolate | Inline | |
| `bdo` | Bidirectional override | Inline | |
| `data` | Machine-readable data | Inline | `value` attribute |
| `ruby` | Ruby annotation | Inline | |
| `rt` | Ruby text | Inline | Auto-close on next `rt`/`rp` |
| `rp` | Ruby fallback parenthesis | Inline | Auto-close on next `rt`/`rp` |

## Ignored Elements

| Element | Description | Notes |
|---|---|---|
| `script` | JavaScript | Parsed but entirely ignored; raw-text element |
| `noscript` | Fallback for no-script | Parsed but ignored |
| `template` | Content template | Parsed inertly; content clonable via `tree.clone_template_content_by_id()` |
| `slot` | Shadow DOM slot | Parsed; `name` attribute |

## Global Attributes

All elements (except `Text`) support these global attributes:

| Attribute | Type | Description |
|---|---|---|
| `id` | `Option<String>` | Element identifier; used by `#id` selectors and `get_element_by_id()` |
| `class` | `Option<String>` | Whitespace-separated CSS class list |
| `style` | `Option<String>` | Inline CSS declarations |
| `title` | `Option<String>` | Advisory tooltip text |
| `lang` | `Option<String>` | Language code |
| `dir` | `Option<HtmlDirection>` | Text direction (`ltr` / `rtl` / `auto`) |
| `hidden` | `Option<bool>` | Hide element (`display: none` semantics via UA cascade) |
| `tabindex` | `Option<i32>` | Tab order; `>=0` makes focusable via keyboard |
| `contenteditable` | (parsed, not consumed) | |
| `draggable` | (parsed, not consumed) | |
| `spellcheck` | (parsed, not consumed) | |
| `translate` | (parsed, not consumed) | |
| `accesskey` | (parsed, not consumed) | |
| `role` | `Option<String>` | ARIA role |
| `aria-*` | `HashMap<String, String>` | ARIA attributes |
| `data-*` | `HashMap<String, String>` | Custom data attributes |
