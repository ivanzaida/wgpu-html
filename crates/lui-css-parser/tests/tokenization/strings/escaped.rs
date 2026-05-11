use lui_css_parser::tokenizer::{tokenize, Token};

#[test]
fn resolves_escaped_quotes_inside_double_quoted_string() {
    assert_eq!(tokenize("\"say \\\"hi\\\"\""), vec![Token::String("say \"hi\"".into())]);
}
