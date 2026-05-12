use lui_html_parser::tokenizer::{Token, tokenize};

#[test]
fn simple_tag() {
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
fn self_closing() {
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
fn attributes() {
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
fn boolean_attribute() {
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
fn comment() {
    let tokens = tokenize("<!-- hello -->");
    assert_eq!(tokens, vec![Token::Comment(" hello ".into())]);
}

#[test]
fn doctype() {
    let tokens = tokenize("<!DOCTYPE html>");
    assert_eq!(tokens, vec![Token::Doctype("DOCTYPE html".into())]);
}

#[test]
fn text_and_elements() {
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
fn script_raw_text() {
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

#[test]
fn raw_text_style() {
    let tokens = tokenize("<style>div { color: red; }</style>");
    assert_eq!(tokens.len(), 3);
    assert!(matches!(tokens[0], Token::OpenTag { ref name, .. } if name == "style"));
    assert_eq!(tokens[1], Token::Text("div { color: red; }".into()));
}

#[test]
fn raw_text_textarea() {
    let tokens = tokenize("<textarea>hello world</textarea>");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[1], Token::Text("hello world".into()));
}

#[test]
fn html_entities_in_attrs() {
    let tokens = tokenize(r#"<div title="a &amp; b">"#);
    match &tokens[0] {
        Token::OpenTag { attrs, .. } => {
            assert_eq!(attrs[0], ("title".into(), "a & b".into()));
        }
        _ => panic!("expected OpenTag"),
    }
}

#[test]
fn empty_string() {
    assert_eq!(tokenize(""), vec![]);
}

#[test]
fn text_only() {
    assert_eq!(tokenize("hello"), vec![Token::Text("hello".into())]);
}

#[test]
fn void_element_implicit() {
    assert_eq!(
        tokenize("<br>"),
        vec![Token::OpenTag {
            name: "br".into(),
            attrs: vec![],
            self_closing: false,
        }]
    );
}

#[test]
fn doctype_lowercase() {
    assert_eq!(
        tokenize("<!doctype html>"),
        vec![Token::Doctype("doctype html".into())]
    );
}
