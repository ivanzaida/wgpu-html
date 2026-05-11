use lui_css_parser::tokenizer::{tokenize, Token};
use lui_css_parser::ArcStr;
use lui_css_parser::CssUnit;

#[test]
fn tokenizes_zero_deg_dimension() {
    assert_eq!(tokenize("0deg"), vec![Token::Dimension { value: 0.0, unit: CssUnit::Deg }]);
}

#[test]
fn tokenizes_zero_ms_dimension() {
    assert_eq!(tokenize("0ms"), vec![Token::Dimension { value: 0.0, unit: CssUnit::Ms }]);
}