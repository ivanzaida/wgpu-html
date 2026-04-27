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
b, strong { font-weight: bold; }
i, em { font-style: italic; }
u, ins { text-decoration: underline; }
s, del, strike { text-decoration: line-through; }
code, kbd, samp { font-family: monospace; }
a { color: blue; text-decoration: underline; }
mark { background-color: yellow; color: black; }
small { font-size: 13px; }
sub { font-size: 13px; }
sup { font-size: 13px; }
h1 { font-size: 32px; font-weight: bold; }
h2 { font-size: 24px; font-weight: bold; }
h3 { font-size: 19px; font-weight: bold; }
h4 { font-weight: bold; }
h5 { font-size: 13px; font-weight: bold; }
h6 { font-size: 11px; font-weight: bold; }
";

/// The lazily-parsed UA stylesheet.
pub fn ua_stylesheet() -> &'static Stylesheet {
    static SHEET: OnceLock<Stylesheet> = OnceLock::new();
    SHEET.get_or_init(|| parse_stylesheet(UA_CSS))
}
