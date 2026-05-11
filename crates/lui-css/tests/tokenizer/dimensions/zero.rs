use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_zero_dimension_with_deg_unit() {
    assert_eq!(
        tokenize("0deg"),
        vec![Token::Dimension { value: 0.0, unit: "deg".into() }]
    );
}
