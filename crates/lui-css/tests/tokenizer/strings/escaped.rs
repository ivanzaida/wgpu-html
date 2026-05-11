use lui_css::tokenizer::{Token, tokenize};

#[test]
fn unescapes_backslash_escaped_quotes_inside_double_quoted_string() {
    assert_eq!(
        tokenize("\"say \\\"hi\\\"\""),
        vec![Token::String("say \"hi\"".into())]
    );
}

#[test]
fn unescapes_backslash_escaped_quotes_inside_single_quoted_string() {
    assert_eq!(
        tokenize("'it\\'s working'"),
        vec![Token::String("it's working".into())]
    );
}
