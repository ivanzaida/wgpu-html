#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl ParseError {
    pub fn new(msg: impl Into<String>, pos: usize) -> Self {
        ParseError { message: msg.into(), position: pos }
    }
}
