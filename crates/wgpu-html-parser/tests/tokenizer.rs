use wgpu_html_parser::tokenizer::{tokenize, Token};

#[test]
fn test_simple_tag() {
    let tokens = tokenize("<div></div>");
    assert_eq!(
        tokens,
        vec![
            Token::OpenTag {
                name: "div".into(),
                attrs: vec![],
                self_closing: false
            },
            Token::CloseTag("div".into()),
        ]
    );
}

#[test]
fn test_self_closing() {
    let tokens = tokenize("<br/>");
    assert_eq!(
        tokens,
        vec![Token::OpenTag {
            name: "br".into(),
            attrs: vec![],
            self_closing: true
        },]
    );
}

#[test]
fn test_attributes() {
    let tokens = tokenize(r#"<a href="http://example.com" target="_blank">link</a>"#);
    assert_eq!(
        tokens,
        vec![
            Token::OpenTag {
                name: "a".into(),
                attrs: vec![
                    ("href".into(), "http://example.com".into()),
                    ("target".into(), "_blank".into()),
                ],
                self_closing: false,
            },
            Token::Text("link".into()),
            Token::CloseTag("a".into()),
        ]
    );
}

#[test]
fn test_boolean_attribute() {
    let tokens = tokenize("<input disabled>");
    assert_eq!(
        tokens,
        vec![Token::OpenTag {
            name: "input".into(),
            attrs: vec![("disabled".into(), String::new())],
            self_closing: false,
        },]
    );
}

#[test]
fn test_comment() {
    let tokens = tokenize("<!-- hello -->");
    assert_eq!(tokens, vec![Token::Comment(" hello ".into())]);
}

#[test]
fn test_doctype() {
    let tokens = tokenize("<!DOCTYPE html>");
    assert_eq!(tokens, vec![Token::Doctype("DOCTYPE html".into())]);
}

#[test]
fn test_text_and_elements() {
    let tokens = tokenize("<p>Hello <b>world</b></p>");
    assert_eq!(
        tokens,
        vec![
            Token::OpenTag {
                name: "p".into(),
                attrs: vec![],
                self_closing: false
            },
            Token::Text("Hello ".into()),
            Token::OpenTag {
                name: "b".into(),
                attrs: vec![],
                self_closing: false
            },
            Token::Text("world".into()),
            Token::CloseTag("b".into()),
            Token::CloseTag("p".into()),
        ]
    );
}

#[test]
fn test_script_raw_text() {
    let tokens = tokenize("<script>var x = '<div>';</script>");
    assert_eq!(
        tokens,
        vec![
            Token::OpenTag {
                name: "script".into(),
                attrs: vec![],
                self_closing: false
            },
            Token::Text("var x = '<div>';".into()),
            Token::CloseTag("script".into()),
        ]
    );
}
