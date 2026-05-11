use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_positive_dimension_with_px_unit() {
    assert_eq!(
        tokenize("10px"),
        vec![Token::Dimension { value: 10.0, unit: "px".into() }]
    );
}
