use lui_core::ArcStr;
use lui_parse::HtmlNode;
use bumpalo::Bump;
pub use style::ComputedStyle;

pub mod bloom;
pub mod cascade;
pub mod index;
pub mod inline;
pub mod matching;
pub mod media;
pub mod pseudo;
pub mod query;
pub mod style;
pub mod var_resolution;

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
    /// Thread-local bump arenas from parallel child cascades.
    /// Kept alive so `CssValue` references remain valid.
    #[doc(hidden)]
    pub _arenas: Vec<Bump>,
}

impl<'a> Default for StyledNode<'a> {
    fn default() -> Self {
        // SAFETY: node is a zero-sized reference in a Default that is
        // immediately overwritten before any read.
        #[allow(invalid_value)]
        let node = unsafe { std::mem::zeroed() };
        StyledNode {
            node,
            style: ComputedStyle::default(),
            children: Vec::new(),
            before: None, after: None, first_line: None, first_letter: None,
            placeholder: None, selection: None, marker: None,
            _arenas: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct PseudoElementStyle<'a> {
    pub style: ComputedStyle<'a>,
    pub content_text: ArcStr,
}
