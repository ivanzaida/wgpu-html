use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::CssUnit;

#[test]
fn tokenizes_negative_em_dimension() {
    assert_eq!(tokenize("-5em"), vec![Token::Dimension { value: -5.0, unit: CssUnit::Em }]);
}

#[test]
fn tokenizes_negative_rem_dimension() {
    assert_eq!(tokenize("-2rem"), vec![Token::Dimension { value: -2.0, unit: CssUnit::Rem }]);
}
