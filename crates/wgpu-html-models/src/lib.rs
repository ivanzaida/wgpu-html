use std::sync::Arc;

pub type ArcStr = Arc<str>;

pub mod common;
pub mod css;
pub use css::*;
pub mod html;
pub use html::*;
