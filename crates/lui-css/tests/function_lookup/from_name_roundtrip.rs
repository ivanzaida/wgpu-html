use lui_css::CssFunction;

#[test]
fn resolves_acos_from_string_name() {
    let f = CssFunction::from_name("acos").unwrap();
    assert_eq!(f, CssFunction::Acos);
    assert_eq!(f.name(), "acos");
}

#[test]
fn resolves_rgb_from_string_name() {
    let f = CssFunction::from_name("rgb").unwrap();
    assert_eq!(f, CssFunction::Rgb);
    assert_eq!(f.name(), "rgb");
}

#[test]
fn resolves_webkit_alias_from_hyphenated_name() {
    let f = CssFunction::from_name("-webkit-image-set").unwrap();
    assert_eq!(f, CssFunction::WebkitImageSet);
    assert_eq!(f.name(), "-webkit-image-set");
}

#[test]
fn resolves_hyphenated_function_from_string_name() {
    let f = CssFunction::from_name("drop-shadow").unwrap();
    assert_eq!(f, CssFunction::DropShadow);
    assert_eq!(f.name(), "drop-shadow");
}

#[test]
fn roundtrip_name_from_name_is_identity_for_all_tested_functions() {
    let functions = [
        CssFunction::Abs,
        CssFunction::Acos,
        CssFunction::Calc,
        CssFunction::Var,
        CssFunction::Rgb,
        CssFunction::Rgba,
        CssFunction::Hsl,
        CssFunction::Url,
    ];
    for func in functions {
        assert_eq!(CssFunction::from_name(func.name()), Some(func));
    }
}
