use lui_css_parser::ArcStr;
use lui_html_parser::HtmlNode;

pub mod cascade;
pub mod index;
pub mod inline;
pub mod matching;
pub mod media;
pub mod query;
pub mod style;
pub mod var_resolution;

pub use style::ComputedStyle;

/// A fully cascaded tree, borrowing from the input `HtmlDocument` and
/// stylesheets. All `CssValue` references point into either the original
/// stylesheet data or the bump arena owned by the caller.
#[derive(Debug)]
pub struct StyledTree<'a> {
    pub root: StyledNode<'a>,
}

/// A node with its computed style. Borrows the original `HtmlNode` for
/// element type and attributes — no cloning.
#[derive(Debug)]
pub struct StyledNode<'a> {
    pub node: &'a HtmlNode,
    pub style: ComputedStyle<'a>,
    pub children: Vec<StyledNode<'a>>,
    pub before: Option<Box<PseudoElementStyle<'a>>>,
    pub after: Option<Box<PseudoElementStyle<'a>>>,
    pub first_line: Option<Box<ComputedStyle<'a>>>,
    pub first_letter: Option<Box<ComputedStyle<'a>>>,
    pub placeholder: Option<Box<ComputedStyle<'a>>>,
    pub selection: Option<Box<ComputedStyle<'a>>>,
    pub marker: Option<Box<PseudoElementStyle<'a>>>,
}

#[derive(Debug)]
pub struct PseudoElementStyle<'a> {
    pub style: ComputedStyle<'a>,
    pub content_text: ArcStr,
}
