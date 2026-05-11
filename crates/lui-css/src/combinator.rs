/// CSS selector combinators and the nesting selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssCombinator {
    /// ` ` — descendant combinator
    Descendant,
    /// `>` — child combinator
    Child,
    /// `+` — next-sibling combinator
    NextSibling,
    /// `~` — subsequent-sibling combinator
    SubsequentSibling,
    /// `||` — column combinator
    Column,
    /// `&` — nesting selector (refers to the parent rule's selector)
    Nesting,
}

impl CssCombinator {
    pub fn name(&self) -> &'static str {
        match self {
            CssCombinator::Descendant => " ",
            CssCombinator::Child => ">",
            CssCombinator::NextSibling => "+",
            CssCombinator::SubsequentSibling => "~",
            CssCombinator::Column => "||",
            CssCombinator::Nesting => "&",
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '>' => Some(CssCombinator::Child),
            '+' => Some(CssCombinator::NextSibling),
            '~' => Some(CssCombinator::SubsequentSibling),
            _ => None,
        }
    }
}
