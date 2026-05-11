use lui_css::tokenizer::{Token, tokenize};

#[test]
fn tokenizes_negative_dimension_with_em_unit() {
    assert_eq!(
        tokenize("-5em"),
        vec![Token::Dimension { value: -5.0, unit: "em".into() }]
    );
}
