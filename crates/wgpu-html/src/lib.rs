//! Top-level facade for the wgpu-html stack.
//!
//! Re-exports the model types and the renderer so downstream apps only need
//! one dependency.

pub use wgpu_html_layout as layout;
pub use wgpu_html_models as models;
pub use wgpu_html_parser as parser;
pub use wgpu_html_renderer as renderer;
pub use wgpu_html_style as style;
pub use wgpu_html_tree as tree;

pub use wgpu_html_text as text;

pub mod paint;
pub use paint::{paint_tree, paint_tree_with_text};
