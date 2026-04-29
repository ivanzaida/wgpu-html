//! Integration test: every HTML input type in a parsed `<form>`.
//!
//! Parses a form containing all 22 input types, focuses each in turn,
//! types "X", and asserts that only text-editable types accept the
//! character.

use wgpu_html_tree::{self, Element, FontFace, FontStyleAxis};

/// Parse a `<form>` containing every HTML input type, focus each
/// editable field in turn, type "X" into it, and verify the value
/// was set. Non-editable types (button, submit, reset, checkbox,
/// radio, file, hidden, image, color, range) must reject the input.
#[test]
fn each_input_type_gets_its_own_value_via_text_input() {
    let html = r#"
        <form style="display: flex; flex-direction: column; gap: 10px; width: 300px">
            <input type="button" value="Button" />
            <input type="checkbox" />
            <input type="color" />
            <input type="date" />
            <input type="datetime-local" />
            <input type="email" placeholder="email@example.com" />
            <input type="file" />
            <input type="hidden" value="hidden-value" />
            <input type="image" src="submit.png" alt="Submit" />
            <input type="month" />
            <input type="number" />
            <input type="password" placeholder="Password" />
            <input type="radio" />
            <input type="range" />
            <input type="reset" />
            <input type="search" placeholder="Search" />
            <input type="submit" />
            <input type="tel" placeholder="+380..." />
            <input type="text" placeholder="Text" />
            <input type="time" />
            <input type="url" placeholder="https://example.com" />
            <input type="week" />
        </form>
    "#;
    let mut tree = wgpu_html_parser::parse(html);

    // Non-text types (button, submit, reset, checkbox, radio,
    // file, hidden, image, color, range) are blocked by
    // `read_editable_value` — they don't accept typed text.
    let cases: &[(&str, bool)] = &[
        ("button", false),
        ("checkbox", false),
        ("color", false),
        ("date", true),
        ("datetime-local", true),
        ("email", true),
        ("file", false),
        ("hidden", false),
        ("image", false),
        ("month", true),
        ("number", true),
        ("password", true),
        ("radio", false),
        ("range", false),
        ("reset", false),
        ("search", true),
        ("submit", false),
        ("tel", true),
        ("text", true),
        ("time", true),
        ("url", true),
        ("week", true),
    ];

    for &(type_name, expect_editable) in cases {
        // Use querySelector to find the input by its type
        // attribute — avoids hardcoding child indices that
        // shift when the parser inserts whitespace text nodes.
        let selector = format!("input[type={type_name}]");
        let input_path = tree
            .query_selector_path(&selector)
            .unwrap_or_else(|| panic!("selector {selector:?} should match"));

        // Focus the input.
        tree.focus(Some(&input_path));

        // Try typing "X".
        let mutated = wgpu_html_tree::text_input(&mut tree, "X");

        // Read back the value.
        let node = tree.root.as_ref().unwrap().at_path(&input_path);
        let value = node.and_then(|n| match &n.element {
            Element::Input(inp) => inp.value.clone(),
            _ => None,
        });

        if expect_editable {
            assert!(
                mutated,
                "type={type_name}: text_input should have mutated the value"
            );
            let v = value.unwrap_or_default();
            assert!(
                v.contains('X'),
                "type={type_name}: value should contain 'X', got {v:?}"
            );
        } else {
            assert!(
                !mutated,
                "type={type_name}: text_input should NOT mutate (editable=false)"
            );
        }

        // Clear for next iteration: blur, then reset value.
        tree.blur();
        if let Some(n) = tree.root.as_mut().unwrap().at_path_mut(&input_path) {
            if let Element::Input(inp) = &mut n.element {
                inp.value = None;
            }
        }
    }
}

/// Type "secret" into a `<input type="password">`, then verify:
/// 1. The tree stores the cleartext value "secret".
/// 2. The shaped text run in layout contains only U+2022 bullets.
/// 3. The number of bullets equals the number of characters typed.
/// 4. The actual characters never appear in the shaped run.
/// 5. A neighbouring `<input type="text">` with the same typed text
///    stores and displays its value in cleartext.
#[test]
fn password_input_stores_cleartext_but_renders_bullets() {
    let html = r#"
        <body style="margin: 0;">
            <input id="pw" type="password" style="font-size: 14px;" />
            <input id="txt" type="text" style="font-size: 14px;" />
        </body>
    "#;
    let mut tree = wgpu_html_parser::parse(html);

    // Register a font so layout can shape the value text.
    let font_bytes = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf")
        .or_else(|_| std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"))
        .expect("need a system font for this test");
    tree.register_font(FontFace {
        family: "sans-serif".into(),
        weight: 400,
        style: FontStyleAxis::Normal,
        data: std::sync::Arc::from(font_bytes.into_boxed_slice()),
    });

    let typed = "secret";

    // ── Type into the password field ────────────────────────────
    let pw_path = tree
        .query_selector_path("#pw")
        .expect("#pw should exist");
    tree.focus(Some(&pw_path));
    for ch in typed.chars() {
        wgpu_html_tree::text_input(&mut tree, &ch.to_string());
    }

    // 1. Cleartext value stored on the element.
    let pw_node = tree.root.as_ref().unwrap().at_path(&pw_path).unwrap();
    let pw_value = match &pw_node.element {
        Element::Input(inp) => inp.value.clone().unwrap_or_default(),
        _ => panic!("expected Input"),
    };
    assert_eq!(pw_value, typed, "password element stores cleartext");

    // ── Type the same text into the plain text field ────────────
    tree.blur();
    let txt_path = tree
        .query_selector_path("#txt")
        .expect("#txt should exist");
    tree.focus(Some(&txt_path));
    for ch in typed.chars() {
        wgpu_html_tree::text_input(&mut tree, &ch.to_string());
    }
    tree.blur();

    // ── Layout both fields and inspect the shaped runs ─────────
    let mut text_ctx = wgpu_html_text::TextContext::new(2048);
    text_ctx.sync_fonts(&tree.fonts);
    let mut image_cache = wgpu_html_layout::ImageCache::new();
    let layout = wgpu_html_layout::layout_with_text(
        &wgpu_html_style::cascade(&tree),
        &mut text_ctx,
        &mut image_cache,
        800.0,
        600.0,
        1.0,
    )
    .expect("layout should produce a root");

    // Walk the layout tree to find the two inputs' text runs.
    // body → children; each input is a direct child of body.
    let pw_box = wgpu_html::layout_at_path(&layout, &pw_path)
        .expect("password layout box");
    let txt_box = wgpu_html::layout_at_path(&layout, &txt_path)
        .expect("text layout box");

    let pw_run = pw_box.text_run.as_ref().expect("password should have a text run");
    let txt_run = txt_box.text_run.as_ref().expect("text should have a text run");

    // 2. Password run contains only bullets.
    let bullet = '\u{2022}';
    assert!(
        pw_run.text.chars().all(|c| c == bullet),
        "password run should be all bullets, got {:?}",
        pw_run.text
    );

    // 3. Number of bullets = number of typed chars.
    assert_eq!(
        pw_run.text.chars().count(),
        typed.chars().count(),
        "bullet count should match typed char count"
    );

    // 4. Cleartext never appears in the shaped text.
    assert!(
        !pw_run.text.contains(typed),
        "cleartext must not leak into the password shaped run"
    );

    // 5. Plain text field shows cleartext.
    assert_eq!(
        txt_run.text, typed,
        "text input should shape the actual value"
    );
}
