use lui_css::tokenizer::{tokenize, Token};
use lui_css::CssUnit;

#[test]
fn tokenizes_calc_expression_with_operators_and_dimensions() {
    assert_eq!(tokenize("calc(100% - 20px)"), vec![
        Token::Function("calc".into()), Token::Delim('('),
        Token::Percentage(100.0), Token::Delim('-'),
        Token::Dimension { value: 20.0, unit: CssUnit::Px },
        Token::Delim(')'),
    ]);
}
