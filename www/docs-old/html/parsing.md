---
title: HTML Parser Internals
---

# HTML Parser Internals

The lui parser converts raw HTML strings into typed `Tree<Node<Element>>` structures. It operates in two phases: **tokenization** and **tree building**.

## Architecture

```
"<div class='card'>Hello</div>"
         │
         ▼  tokenizer::tokenize()
[OpenTag("div", [("class","card")], false), Text("Hello"), CloseTag("div")]
         │
         ▼  tree_builder::build()
Tree { root: Some(Node::new(Div { class: "card", .. }).with_children([
         Node::new(Element::Text("Hello"))
       ])) }
```

## Tokenizer

The tokenizer (`lui-parser/src/tokenizer.rs`) scans the input character-by-character and emits a flat list of `Token` values:

```rust
pub enum Token {
    Doctype(String),
    OpenTag {
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    CloseTag(String),
    Text(String),
    Comment(String),
}
```

### Tag Recognition

- `<tag>` — Open tag with optional attributes
- `</tag>` — Close tag
- `<tag/>` — Self-closing tag (sets `self_closing: true`)
- `<!-- ... -->` — Comment (tokenized, discarded by tree builder)
- `<!doctype ...>` — DOCTYPE (tokenized, discarded by tree builder)

### Attribute Parsing

Attributes are parsed inside open and self-closing tags. Three forms are supported:

```html
<!-- Quoted attributes -->
<input type="text" value="hello">

<!-- Unquoted attributes -->
<input type=text>

<!-- Boolean attributes (value = empty string) -->
<input disabled required>
```

Attribute values undergo entity decoding (see below).

### Raw-Text Elements

For `<style>`, `<script>`, `<textarea>`, and `<title>`, the tokenizer captures **everything** between the open and close tags as a single `Text` token — no further tokenization occurs inside:

```html
<style>
  /* This entire block is one Text token */
  .card { color: red; }
</style>

<textarea>
  Line 1
  Line 2
  This is <strong>not a tag</strong> — it's all text
</textarea>
```

## Entity Decoding

The tokenizer decodes HTML entities in text content and attribute values:

| Entity | Decoded |
|---|---|
| `&amp;amp;` | `&` |
| `&amp;lt;` | `<` |
| `&amp;gt;` | `>` |
| `&amp;quot;` | `"` |
| `&amp;apos;` | `'` |
| `&amp;nbsp;` | `\u{00A0}` (non-breaking space) |
| `&amp;#NN;` | Unicode codepoint `NN` (decimal) |
| `&amp;#xNN;` | Unicode codepoint `NN` (hex) |

```html
<p>Hello &amp;amp; welcome &amp;mdash; click &amp;lt;here&amp;gt;</p>
```

Result: `Hello & welcome — click <here>`

Other named entities beyond `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;` are **not decoded** — the parser recognizes only these five named entities plus numeric character references.

## Tree Builder

The tree builder (`lui-parser/src/tree_builder.rs`) consumes the token stream and constructs the DOM tree.

### Void Elements

14 elements are void (cannot have children, never need a closing tag):

```
area, base, br, col, embed, hr, img, input,
link, meta, param, source, track, wbr
```

When a void element is opened (or self-closed), it is immediately pushed and popped — no children are collected.

```html
<br>       <!-- immediately popped, no children -->
<img src="x.png">  <!-- immediately popped, no children -->
<hr/>      <!-- self-closing, also immediately popped -->
```

### Self-Closing Recognition

Any tag ending with `/>` sets `self_closing: true`. For void elements this is redundant; for non-void elements it functions as an immediate close:

```html
<div/>  <!-- treated as <div></div> -->
<span/> <!-- treated as <span></span> -->
```

### Auto-Close Rules

The tree builder implements auto-close for several element groups to handle HTML where closing tags are omitted:

| Opened Tag | Auto-closes on |
|---|---|
| `<p>` | Next `<p>`, `<div>`, heading, `<ul>`, `<ol>`, `<dl>`, `<table>`, `<form>`, `<header>`, `<footer>`, `<nav>`, `<section>`, `<article>`, `<aside>`, `<main>`, `<details>`, `<fieldset>`, `<figure>`, `<hr>`, `<pre>`, `<blockquote>`, `<address>`, or end of parent |
| `<li>` | Next `<li>` |
| `<dt>`, `<dd>` | Next `<dt>` or `<dd>` |
| `<thead>` | Next `<tbody>` or `<tfoot>` |
| `<tbody>` | Next `<thead>` or `<tfoot>` |
| `<tfoot>` | Next `<tbody>` |
| `<tr>` | Next `<tr>` |
| `<th>`, `<td>` | Next `<th>` or `<td>` |
| `<option>` | Next `<option>` or `<optgroup>` |
| `<optgroup>` | Next `<optgroup>` |
| `<rt>`, `<rp>` | Next `<rt>` or `<rp>` |

At end-of-file, all remaining open elements are auto-closed.

### Unknown Tags

Tags not matching any of the ~98 recognized element types are **dropped silently** along with their entire subtree. The tree builder pushes a `None` slot on the stack, collects children normally (for nested recognized elements), and discards everything on close:

```html
<custom-element>
  <p>This paragraph survives</p>  <!-- Actually, it won't →
</custom-element>
<!-- Neither <custom-element> nor <p> appear in the tree -->
```

### Synthetic `<body>` Wrapping

If the token stream produces zero or one top-level node, it becomes the tree root directly. If multiple top-level nodes are produced, they are wrapped in a synthetic `<body>`:

```html
<!-- Input: -->
<h1>Title</h1>
<p>Paragraph</p>
<!-- Tree root: <body><h1>Title</h1><p>Paragraph</p></body> -->
```

If one of the top-level nodes is already a `<body>`, siblings are merged into it instead of creating a second wrapper.

## Inline CSS Extraction

### `style` Attributes

The value of every `style="..."` attribute is parsed by `parse_inline_style_decls()`:

```rust
use lui_parser::parse_inline_style;
use lui_models::Style;

let style: Style = parse_inline_style("color: red; font-size: 16px; display: flex;");
```

`!important` declarations are recognized and respected in cascade ordering. Custom property references (`var(--x)`) are resolved during cascade.

### `<style>` Blocks

The text content of `<style>` elements is parsed by `parse_stylesheet()`:

```rust
use lui_parser::Stylesheet;

let css = r#"
  .card { background: #fff; border-radius: 8px; }
  .card.active { border-color: blue; }
"#;
let sheet: Stylesheet = lui_parser::parse_stylesheet(css);
```

The parser supports:
- Tag, `#id`, `.class`, universal `*`, and comma-separated selector lists
- **Descendant combinator** (space) — e.g., `.card p`
- `/* CSS comments */`
- Specificity calculation: `(id << 16) | (class << 8) | tag`
- `!important` flag

Child (`>`), sibling (`+`, `~`), and attribute selectors (`[attr]`) are **not yet supported** in the stylesheet parser (they work in the `query_selector` API).

## Serialization

The tree can be serialized back to HTML for debugging:

```rust
use lui_parser::parse;

let tree = parse("<div id='main'><p>Hello</p></div>");

// Full document with <!DOCTYPE> prefix
let html: String = tree.to_html();
// => <!DOCTYPE html>\n<div id="main"><p>Hello</p></div>

// Single node as HTML fragment
let node_html: Option<String> = tree.node_to_html(&[0, 0]);
// => Some("<p>Hello</p>")

// Any Node can serialize itself
let p_node = &tree.root.as_ref().unwrap().children[0];
let p_html: String = p_node.to_html();
// => <p>Hello</p>
```

Serialization escapes `&`, `<`, `>` in text content, and escapes `"`, `&`, `<`, `>` in attribute values. Void elements omit closing tags. `data-*` and `aria-*` attributes are included. Raw-text elements (`<style>`, `<script>`) serialize their content unescaped.

## See Also

- [HTML Overview](./overview) — How parsing fits into the full pipeline
- [Element Index](./element-index) — Complete list of recognized elements
- [CSS documentation](../css/overview) — Stylesheet parsing and cascade engine
