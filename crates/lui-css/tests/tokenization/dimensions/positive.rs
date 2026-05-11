use lui_css::tokenizer::{tokenize, Token};
use lui_css::CssUnit;

#[test]
fn tokenizes_px_dimension() {
    assert_eq!(tokenize("10px"), vec![Token::Dimension { value: 10.0, unit: CssUnit::Px }]);
}

#[test]
fn tokenizes_em_dimension() {
    assert_eq!(tokenize("1.5em"), vec![Token::Dimension { value: 1.5, unit: CssUnit::Em }]);
}
