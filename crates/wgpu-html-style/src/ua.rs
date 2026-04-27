//! User-agent stylesheet — the small set of "browser default" rules
//! that every author implicitly relies on.
//!
//! Scope today: inline emphasis (`<b>`, `<strong>`, `<em>`, `<i>`,
//! `<u>`, `<s>`, `<code>`, `<a>`, `<mark>`, `<small>`) plus heading
//! sizes (`h1`–`h6`). Block-level resets (default margins on `<p>`,
//! `<ul>`, etc.) are deliberately *not* included — they would change
//! existing layouts that don't expect browser-style spacing. Add
//! those once block-flow rendering is happy with them.
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
const UA_CSS: &str = "
/* Document-machinery elements: never rendered. Without these the
   contents of <style> / <script> / <head> / <noscript> / <meta> /
   <link> / <title> would flow through the IFC as visible text. */
head, style, script, meta, link, title, noscript,
template, source, track, base, param, col, colgroup
    { display: none; }

/* Document body — browsers ship with a small inset by default. */
body { margin: 8px; }

/* Block-level vertical rhythm. Pixel values match the typical
   browser computation of `<n>em` against the element's own
   font-size: e.g. `h1 { margin: 0.67em 0 }` ≈ 21px at 32px.
   Using px directly keeps the cascade simple — em-against-parent
   resolution is still pending. */
p, blockquote, pre, ul, ol, dl, address { margin: 16px 0; }
h1 { margin: 21px 0; font-size: 32px; font-weight: bold; }
h2 { margin: 20px 0; font-size: 24px; font-weight: bold; }
h3 { margin: 19px 0; font-size: 19px; font-weight: bold; }
h4 { margin: 21px 0; font-weight: bold; }
h5 { margin: 22px 0; font-size: 13px; font-weight: bold; }
h6 { margin: 26px 0; font-size: 11px; font-weight: bold; }
hr { margin: 8px 0; border-top: 1px solid gray; }
ul, ol { padding-left: 40px; }
dd { margin-left: 40px; }
blockquote { margin: 16px 40px; }

/* Inline emphasis */
b, strong { font-weight: bold; }
i, em { font-style: italic; }
u, ins { text-decoration: underline; }
s, del, strike { text-decoration: line-through; }
code, kbd, samp { font-family: monospace; }
pre { font-family: monospace; }
a { color: blue; text-decoration: underline; }
mark { background-color: yellow; color: black; }
small { font-size: 13px; }

/* <sub> / <sup>: the smaller font is shipped today; the actual
   baseline shift (vertical-align: super/sub) is wired in via
   the dedicated CSS property below once the layout supports it. */
sub { font-size: 13px; vertical-align: sub; }
sup { font-size: 13px; vertical-align: super; }
";

/// The lazily-parsed UA stylesheet.
pub fn ua_stylesheet() -> &'static Stylesheet {
    static SHEET: OnceLock<Stylesheet> = OnceLock::new();
    SHEET.get_or_init(|| parse_stylesheet(UA_CSS))
}
