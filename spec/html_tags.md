# HTML Tags Support

This table tracks current element support from the parser/model/layout pipeline.
Unknown tags currently drop their entire subtree in `tree_builder`.

Status meanings:

- `Yes`: parsed and consumed by layout/paint/text behavior.
- `Partial`: parsed and some behavior works, but browser semantics are incomplete.
- `Parsed only`: stored as a typed element, but not consumed by runtime behavior.
- `Deferred`: recognized in shorthand/deferred-longhand storage, but not typed/consumed. This is mainly relevant to CSS docs; no HTML tags currently use it.
- `No`: not recognized by the parser today.

| Tag | Supported | Notes |
|---|---:|---|
| `a` | Yes | Parsed with link attributes. Focusable when `href` is present; navigation/download/ping are not implemented. |
| `abbr` | Yes | Parsed with globals; UA style gives abbreviation decoration. |
| `address` | Yes | Parsed and styled as block content. |
| `area` | No | Unknown tag; image maps are not implemented. |
| `article` | Yes | Parsed and styled as block/sectioning content. |
| `aside` | Yes | Parsed and styled as block/sectioning content. |
| `audio` | Partial | Parsed with media attributes; no audio playback or controls implementation. |
| `b` | Yes | Parsed; UA/style handles bold semantics via CSS. |
| `base` | No | Unknown tag; base URL handling is not implemented. |
| `bdi` | Yes | Parsed; bidi isolation behavior is not implemented beyond stored attributes/text flow. |
| `bdo` | Yes | Parsed; `dir` can be stored, but full bidi override behavior is limited. |
| `blockquote` | Yes | Parsed with `cite`; rendered as normal block content with UA defaults. |
| `body` | Yes | Parsed; also used as synthetic wrapper when multiple roots are produced. |
| `br` | Yes | Parsed as void element; inline layout treats it as a forced line break. |
| `button` | Partial | Parsed, focusable, and styled as inline-block; no native form submission. |
| `canvas` | Partial | Parsed with dimensions; no canvas drawing API. |
| `caption` | Partial | Parsed with table UA display; table layout itself is not implemented. |
| `cite` | Yes | Parsed as phrasing content. |
| `code` | Yes | Parsed; UA/style can apply monospace defaults. |
| `col` | Partial | Parsed with `span`; table layout is not implemented. |
| `colgroup` | Partial | Parsed with `span`; table layout is not implemented. |
| `data` | Yes | Parsed with `value`; no special runtime behavior. |
| `datalist` | Partial | Parsed; no autocomplete popup/list behavior. |
| `dd` | Yes | Parsed and styled as block/list content. |
| `del` | Yes | Parsed with `cite`/`datetime`; no edit-history behavior. |
| `details` | Partial | Parsed with `open`/`name`; native toggle behavior is not complete. |
| `dfn` | Yes | Parsed as phrasing content. |
| `dialog` | Partial | Parsed with `open`; closed dialogs get UA `display: none`, but modal/top-layer behavior is not implemented. |
| `div` | Yes | Parsed and rendered as generic block content. |
| `dl` | Yes | Parsed and styled as block/list content. |
| `dt` | Yes | Parsed and styled as block/list content. |
| `em` | Yes | Parsed; UA/style handles emphasis via CSS. |
| `embed` | No | Unknown tag; embedded plugin/content loading is not implemented. |
| `fieldset` | Partial | Parsed with form attributes; no native grouped-control behavior. |
| `figcaption` | No | Unknown tag; subtree is dropped. |
| `figure` | No | Unknown tag; subtree is dropped. |
| `footer` | Yes | Parsed and styled as block/sectioning content. |
| `form` | Partial | Parsed with form attributes; no form submission or validation pipeline. |
| `h1` | Yes | Parsed and styled with heading UA defaults. |
| `h2` | Yes | Parsed and styled with heading UA defaults. |
| `h3` | Yes | Parsed and styled with heading UA defaults. |
| `h4` | Yes | Parsed and styled with heading UA defaults. |
| `h5` | Yes | Parsed and styled with heading UA defaults. |
| `h6` | Yes | Parsed and styled with heading UA defaults. |
| `head` | Partial | Parsed; non-rendered metadata container. Child metadata support varies by tag. |
| `header` | Yes | Parsed and styled as block/sectioning content. |
| `hgroup` | No | Unknown tag; subtree is dropped. |
| `hr` | Yes | Parsed as void element and styled as block/rule-like content. |
| `html` | Yes | Parsed as document root element. |
| `i` | Yes | Parsed; UA/style handles italic semantics via CSS. |
| `iframe` | Partial | Parsed with frame attributes; no nested browsing context or document loading. |
| `img` | Partial | Parsed with image attributes; layout loads/renders images and uses intrinsic/declared size, but image maps and full responsive-source selection are limited. |
| `input` | Partial | Parsed with many input attributes; text-like editing, focus, placeholder, checked/disabled/read-only selectors exist, but many input types and validation are not complete. |
| `ins` | Yes | Parsed with `cite`/`datetime`; no edit-history behavior. |
| `kbd` | Yes | Parsed; UA/style can apply monospace defaults. |
| `label` | Partial | Parsed with `for`; native label-to-control activation is not complete. |
| `legend` | Partial | Parsed; fieldset/legend native layout behavior is limited. |
| `li` | Partial | Parsed with `value`; list-item display exists, but marker rendering/numbering is limited. |
| `link` | Parsed only | Parsed with link attributes; stylesheet/resource link loading is not implemented here. |
| `main` | Yes | Parsed and styled as block/sectioning content. |
| `map` | No | Unknown tag; image maps are not implemented. |
| `mark` | Yes | Parsed; UA/style handles highlight-like defaults where present. |
| `menu` | No | Unknown tag; subtree is dropped. |
| `meta` | Parsed only | Parsed with metadata attributes; metadata does not affect host/document behavior. |
| `meter` | Partial | Parsed with numeric attributes; native meter rendering is not implemented. |
| `nav` | Yes | Parsed and styled as block/sectioning content. |
| `noscript` | Partial | Parsed; JavaScript is permanently out of scope, so browser-equivalent scripting fallback semantics are not meaningful. |
| `object` | No | Unknown tag; embedded object loading is not implemented. |
| `ol` | Partial | Parsed with `reversed`/`start`/`type`; list layout exists, marker numbering is limited. |
| `optgroup` | Partial | Parsed with `label`/`disabled`; native select popup behavior is not implemented. |
| `option` | Partial | Parsed with `value`/`label`/`selected`/`disabled`; native select popup behavior is not implemented. |
| `output` | Partial | Parsed with form attributes; no form calculation/submission behavior. |
| `p` | Yes | Parsed; tree builder has basic paragraph auto-close handling. |
| `picture` | Partial | Parsed; full source selection is limited. |
| `pre` | Yes | Parsed; UA/style supports preformatted whitespace behavior. |
| `progress` | Partial | Parsed with `value`/`max`; native progress rendering is not implemented. |
| `q` | No | Unknown tag; subtree is dropped. |
| `rp` | Partial | Parsed; UA sets `display: none`, but full ruby layout is limited. |
| `rt` | Partial | Parsed with ruby text display; full ruby layout is limited. |
| `ruby` | Partial | Parsed with ruby display; full ruby layout is limited. |
| `s` | Yes | Parsed; UA/style handles strike-through semantics where present. |
| `samp` | Yes | Parsed; UA/style can apply monospace defaults. |
| `script` | Parsed only | Parsed as raw text with attributes, but JavaScript execution is permanently out of scope. |
| `search` | No | Unknown tag; subtree is dropped. |
| `section` | Yes | Parsed and styled as block/sectioning content. |
| `select` | Partial | Parsed, focusable, and styled as inline-block; native popup/list interaction is not implemented. |
| `slot` | Parsed only | Parsed with `name`; shadow DOM distribution is not implemented. |
| `small` | Yes | Parsed; UA/style handles smaller text where present. |
| `source` | Parsed only | Parsed with media/source attributes; full `<picture>`/media source selection is limited. |
| `span` | Yes | Parsed and rendered as generic inline content. |
| `strong` | Yes | Parsed; UA/style handles strong emphasis via CSS. |
| `style` | Yes | Parsed as raw text; inline stylesheet content is collected by cascade. |
| `sub` | Partial | Parsed; vertical-align/style support is limited. |
| `summary` | Partial | Parsed and focusable; native details disclosure behavior is not complete. |
| `sup` | Partial | Parsed; vertical-align/style support is limited. |
| `table` | Partial | Parsed with table UA display, but table layout is not implemented. |
| `tbody` | Partial | Parsed with table-row-group UA display; table layout is not implemented. |
| `td` | Partial | Parsed with table-cell attributes; table layout is not implemented. |
| `template` | Parsed only | Parsed with `shadowrootmode`; inert template content and shadow roots are not implemented. |
| `textarea` | Partial | Parsed, focusable, editable, and supports placeholder/text editing; form submission and full native behavior are incomplete. |
| `tfoot` | Partial | Parsed with table-footer-group UA display; table layout is not implemented. |
| `th` | Partial | Parsed with table-header attributes; table layout is not implemented. |
| `thead` | Partial | Parsed with table-header-group UA display; table layout is not implemented. |
| `time` | Yes | Parsed with `datetime`; no date/time semantics beyond stored attribute. |
| `title` | Parsed only | Parsed as raw text; title metadata is stored in tree, not applied to host window automatically. |
| `tr` | Partial | Parsed with table-row UA display; table layout is not implemented. |
| `track` | Parsed only | Parsed with track attributes; no media text-track behavior. |
| `u` | Yes | Parsed; UA/style handles underline semantics where present. |
| `ul` | Partial | Parsed; list layout exists, marker rendering is limited. |
| `var` | Yes | Parsed as phrasing content. |
| `video` | Partial | Parsed with media attributes; no video playback or controls implementation. |
| `wbr` | Partial | Parsed as void element; inline layout treats it as a word-break opportunity. |
| `svg` | Partial | Parsed as a top-level `Svg` element with a few attributes; foreign-content parsing for nested SVG nodes is not implemented. |
| `math` | No | MathML foreign content is not implemented. |
