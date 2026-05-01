//! User-agent stylesheet — browser-default CSS applied before author
//! rules.
//!
//! Specificity: every UA rule uses tag selectors only, so they sit at
//! the bottom of the author-normal cascade band. An author tag rule
//! with the same name wins on source order (the UA rules are emitted
//! first). An author class / id rule wins on specificity as expected.
//! Higher-origin (UA `!important`) ordering isn't tracked — we don't
//! mark UA defaults important on purpose, so any author rule overrides.

use std::sync::OnceLock;

use wgpu_html_parser::{Stylesheet, parse_stylesheet};

/// Source for the UA stylesheet. Parsed once and cached.
const UA_CSS: &str = r#"
/* Document */

html {
  display: block;
}

body {
  display: block;
  margin: 8px;
}

head {
  display: none;
}

title,
base,
link,
meta,
style,
script,
noscript,
template {
  display: none;
}

/* Sections */

article,
aside,
footer,
header,
hgroup,
main,
nav,
section {
  display: block;
}

address {
  display: block;
  font-style: italic;
}

blockquote {
  display: block;
  margin-block-start: 1em;
  margin-block-end: 1em;
  margin-inline-start: 40px;
  margin-inline-end: 40px;
}

div {
  display: block;
}

/* Headings */

h1 {
  display: block;
  font-size: 2em;
  font-weight: bold;
  margin-block-start: 0.67em;
  margin-block-end: 0.67em;
}

h2 {
  display: block;
  font-size: 1.5em;
  font-weight: bold;
  margin-block-start: 0.83em;
  margin-block-end: 0.83em;
}

h3 {
  display: block;
  font-size: 1.17em;
  font-weight: bold;
  margin-block-start: 1em;
  margin-block-end: 1em;
}

h4 {
  display: block;
  font-size: 1em;
  font-weight: bold;
  margin-block-start: 1.33em;
  margin-block-end: 1.33em;
}

h5 {
  display: block;
  font-size: 0.83em;
  font-weight: bold;
  margin-block-start: 1.67em;
  margin-block-end: 1.67em;
}

h6 {
  display: block;
  font-size: 0.67em;
  font-weight: bold;
  margin-block-start: 2.33em;
  margin-block-end: 2.33em;
}

/* Text blocks */

p {
  display: block;
  margin-block-start: 1em;
  margin-block-end: 1em;
}

hr {
  display: block;
  margin-block-start: 0.5em;
  margin-block-end: 0.5em;
  margin-inline-start: auto;
  margin-inline-end: auto;
  border-style: inset;
  border-width: 1px;
}

pre {
  display: block;
  font-family: monospace;
  white-space: pre;
  margin-block-start: 1em;
  margin-block-end: 1em;
}

/* Lists */

ul,
menu,
dir {
  display: block;
  list-style-type: disc;
  margin-block-start: 1em;
  margin-block-end: 1em;
  padding-inline-start: 40px;
}

ol {
  display: block;
  list-style-type: decimal;
  margin-block-start: 1em;
  margin-block-end: 1em;
  padding-inline-start: 40px;
}

li {
  display: list-item;
}

ul ul,
ol ul {
  list-style-type: circle;
}

ul ul ul,
ol ul ul,
ul ol ul,
ol ol ul {
  list-style-type: square;
}

dd {
  display: block;
  margin-inline-start: 40px;
}

dl {
  display: block;
  margin-block-start: 1em;
  margin-block-end: 1em;
}

dt {
  display: block;
}

/* Inline text */

a {
  color: blue;
  text-decoration: underline;
  cursor: pointer;
}

a:visited {
  color: purple;
}

abbr[title],
acronym[title] {
  text-decoration: underline dotted;
}

b,
strong {
  font-weight: bold;
}

i,
cite,
em,
var,
dfn {
  font-style: italic;
}

code,
kbd,
samp,
tt {
  font-family: monospace;
}

small {
  font-size: smaller;
}

big {
  font-size: larger;
}

sub {
  vertical-align: sub;
  font-size: smaller;
}

sup {
  vertical-align: super;
  font-size: smaller;
}

s,
strike,
del {
  text-decoration: line-through;
}

u,
ins {
  text-decoration: underline;
}

mark {
  background-color: yellow;
  color: black;
}

q {
  display: inline;
}

br {
  display: inline;
}

wbr {
  display: inline;
}

/* Replaced / media */

img {
  display: inline;
}

iframe {
  display: inline-block;
  border: 2px inset;
}

embed,
object {
  display: inline-block;
}

video,
canvas {
  display: inline-block;
}

audio {
  display: inline-block;
}

/* Tables */

table {
  display: table;
  border-collapse: separate;
  border-spacing: 2px;
  border-color: gray;
}

caption {
  display: table-caption;
  text-align: center;
}

thead {
  display: table-header-group;
  vertical-align: middle;
  border-color: inherit;
}

tbody {
  display: table-row-group;
  vertical-align: middle;
  border-color: inherit;
}

tfoot {
  display: table-footer-group;
  vertical-align: middle;
  border-color: inherit;
}

tr {
  display: table-row;
  vertical-align: inherit;
  border-color: inherit;
}

td,
th {
  display: table-cell;
  vertical-align: inherit;
}

th {
  font-weight: bold;
  text-align: center;
}

col {
  display: table-column;
}

colgroup {
  display: table-column-group;
}

table {
  box-sizing: border-box;
}

/* Forms */

form {
  display: block;
  margin-block-end: 1em;
}

fieldset {
  display: block;
  margin-inline-start: 2px;
  margin-inline-end: 2px;
  padding-block-start: 0.35em;
  padding-inline-start: 0.75em;
  padding-inline-end: 0.75em;
  padding-block-end: 0.625em;
  border: 2px groove;
}

legend {
  display: block;
  padding-inline-start: 2px;
  padding-inline-end: 2px;
}

label {
  cursor: default;
}

input,
textarea,
select,
button {
  font: initial;
  color: initial;
  letter-spacing: normal;
  word-spacing: normal;
  line-height: normal;
  text-transform: none;
  text-indent: 0;
  text-shadow: none;
  display: inline-block;
  text-align: start;
  margin: 0;
}

button,
input[type="button"],
input[type="submit"],
input[type="reset"] {
  display: inline-block;
  text-align: center;
  cursor: default;
  box-sizing: border-box;
  padding-block: 2px;
  padding-inline: 6px;
  border: 2px outset;
  background-color: buttonface;
  color: buttontext;
}

input {
  padding-block: 1px;
  padding-inline: 2px;
  border: 2px inset;
  background-color: field;
  color: fieldtext;
}

input[type="hidden"] {
  display: none;
}

input[type="checkbox"],
input[type="radio"] {
  box-sizing: border-box;
  padding: 0;
}

textarea {
  display: inline-block;
  white-space: pre-wrap;
  overflow-wrap: break-word;
  overflow: auto;
  resize: both;
  border: 2px inset;
  padding: 2px;
  background-color: field;
  color: fieldtext;
}

select {
  display: inline-block;
  box-sizing: border-box;
  border: 1px solid;
  background-color: field;
  color: fieldtext;
}

option {
  display: block;
}

optgroup {
  display: block;
  font-weight: bold;
}

/* Interactive */

details {
  display: block;
}

summary {
  display: list-item;
  cursor: default;
}

dialog {
  position: absolute;
  display: none;
  inset-inline-start: 0;
  inset-inline-end: 0;
  width: fit-content;
  height: fit-content;
  margin: auto;
  border: solid;
  padding: 1em;
  background: canvas;
  color: canvastext;
}

dialog[open] {
  display: block;
}

/* Semantic / ruby */

ruby {
  display: ruby;
}

rt {
  display: ruby-text;
}

rp {
  display: none;
}

/* Bidi */

bdi {
  unicode-bidi: isolate;
}

bdo {
  unicode-bidi: bidi-override;
}

[dir="ltr"] {
  direction: ltr;
}

[dir="rtl"] {
  direction: rtl;
}

/* Hidden */

[hidden] {
  display: none;
}

template {
  display: none;
}

/* Default focus */

:focus {
  outline: auto;
}
"#;

/// The lazily-parsed UA stylesheet.
pub fn ua_stylesheet() -> &'static Stylesheet {
  static SHEET: OnceLock<Stylesheet> = OnceLock::new();
  SHEET.get_or_init(|| parse_stylesheet(UA_CSS))
}
