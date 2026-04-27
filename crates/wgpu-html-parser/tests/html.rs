//! Integration tests for the HTML parser.
//!
//! - `single` — one tag at a time, asserts the parsed `Element` variant.
//! - `tree`   — multi-element trees, attributes, text, nesting, auto-close.

mod html {
    automod::dir!("tests/html");
}
