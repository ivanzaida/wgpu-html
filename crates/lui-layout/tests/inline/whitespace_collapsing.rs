use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_layout::engine::layout_tree;
use crate::helpers::*;

/// In `white-space: normal`, newlines in the HTML source between tags
/// should be collapsed into a single space. Text like:
///   "\n        Hello World\n    "
/// should be treated as " Hello World " (then leading/trailing stripped).
///
/// The height of a div containing newline-indented text should equal
/// the height of the same text without newlines (single line).
#[test]
fn newlines_collapsed_to_space_in_normal_mode() {
    let media = MediaContext::default();
    let interaction = InteractionState::default();

    // Text with newlines and indentation (mimics formatted HTML source)
    let (doc, ctx) = flex_lt(r#"<div style="width:800px; font-size:16px">
        Hello World
    </div>"#, 800.0);
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div_newlines = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();

    // Same text without newlines — should produce identical layout
    let (doc2, ctx2) = flex_lt(
        r#"<div style="width:800px; font-size:16px">Hello World</div>"#, 800.0,
    );
    let styled2 = ctx2.cascade(&doc2.root, &media, &interaction);
    let lt2 = layout_tree(&styled2, 800.0, 600.0);
    let div_clean = find_by_tag(&lt2.root, "body").unwrap().children.first().unwrap();

    assert!(
        (div_newlines.content.height - div_clean.content.height).abs() < 1.0,
        "newline-indented text should be single line like clean text: \
         newlines_h={}, clean_h={}",
        div_newlines.content.height,
        div_clean.content.height,
    );
}

/// Multiple consecutive spaces in `white-space: normal` should collapse
/// to a single space. "Hello     World" should render the same width
/// as "Hello World".
#[test]
fn multiple_spaces_collapsed_to_single() {
    let media = MediaContext::default();
    let interaction = InteractionState::default();

    let (doc, ctx) = flex_lt(
        r#"<div style="width:800px; font-size:16px">Hello     World</div>"#, 800.0,
    );
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div_multi = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let w_multi = div_multi.children[0].content.width;

    let (doc2, ctx2) = flex_lt(
        r#"<div style="width:800px; font-size:16px">Hello World</div>"#, 800.0,
    );
    let styled2 = ctx2.cascade(&doc2.root, &media, &interaction);
    let lt2 = layout_tree(&styled2, 800.0, 600.0);
    let div_single = find_by_tag(&lt2.root, "body").unwrap().children.first().unwrap();
    let w_single = div_single.children[0].content.width;

    assert!(
        (w_multi - w_single).abs() < 2.0,
        "multiple spaces should collapse to one: multi_w={}, single_w={}",
        w_multi,
        w_single,
    );
}

/// Leading and trailing whitespace in a text node should be stripped
/// under `white-space: normal`. The text "  Hello  " should render the
/// same as "Hello".
#[test]
fn leading_trailing_whitespace_stripped() {
    let media = MediaContext::default();
    let interaction = InteractionState::default();

    let (doc, ctx) = flex_lt(
        r#"<div style="width:800px; font-size:16px">  Hello  </div>"#, 800.0,
    );
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div_padded = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let w_padded = div_padded.children[0].content.width;

    let (doc2, ctx2) = flex_lt(
        r#"<div style="width:800px; font-size:16px">Hello</div>"#, 800.0,
    );
    let styled2 = ctx2.cascade(&doc2.root, &media, &interaction);
    let lt2 = layout_tree(&styled2, 800.0, 600.0);
    let div_clean = find_by_tag(&lt2.root, "body").unwrap().children.first().unwrap();
    let w_clean = div_clean.children[0].content.width;

    assert!(
        (w_padded - w_clean).abs() < 2.0,
        "leading/trailing whitespace should be stripped: padded_w={}, clean_w={}",
        w_padded,
        w_clean,
    );
}

/// Tab characters should also be collapsed to a single space under
/// `white-space: normal`.
#[test]
fn tabs_collapsed_to_single_space() {
    let media = MediaContext::default();
    let interaction = InteractionState::default();

    let (doc, ctx) = flex_lt(
        r#"<div style="width:800px; font-size:16px">Hello&#9;&#9;World</div>"#, 800.0,
    );
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div_tabs = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let w_tabs = div_tabs.children[0].content.width;

    let (doc2, ctx2) = flex_lt(
        r#"<div style="width:800px; font-size:16px">Hello World</div>"#, 800.0,
    );
    let styled2 = ctx2.cascade(&doc2.root, &media, &interaction);
    let lt2 = layout_tree(&styled2, 800.0, 600.0);
    let div_space = find_by_tag(&lt2.root, "body").unwrap().children.first().unwrap();
    let w_space = div_space.children[0].content.width;

    assert!(
        (w_tabs - w_space).abs() < 2.0,
        "tabs should collapse to single space: tabs_w={}, space_w={}",
        w_tabs,
        w_space,
    );
}

/// Whitespace-only text nodes between block elements (e.g. blank lines
/// in formatted HTML) should produce zero-height anonymous blocks so
/// they don't add extra vertical spacing.
#[test]
fn whitespace_between_blocks_adds_no_vertical_space() {
    let media = MediaContext::default();
    let interaction = InteractionState::default();

    // Formatted HTML: blank lines between h1 and p
    let (doc, ctx) = flex_lt(r#"<div style="width:800px">
        <div style="height:20px">A</div>

        <div style="height:20px">B</div>
    </div>"#, 800.0);
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();

    // Compact version: no whitespace between elements
    let (doc2, ctx2) = flex_lt(
        r#"<div style="width:800px"><div style="height:20px">A</div><div style="height:20px">B</div></div>"#,
        800.0,
    );
    let styled2 = ctx2.cascade(&doc2.root, &media, &interaction);
    let lt2 = layout_tree(&styled2, 800.0, 600.0);
    let container2 = find_by_tag(&lt2.root, "body").unwrap().children.first().unwrap();

    assert!(
        (container.content.height - container2.content.height).abs() < 1.0,
        "blank lines between blocks should not add height: formatted={}, compact={}",
        container.content.height,
        container2.content.height,
    );
}
