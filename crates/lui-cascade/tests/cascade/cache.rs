use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_css_parser::{parse_stylesheet, parse_value};
use lui_html_parser::parse;

fn val(css: &str) -> lui_css_parser::CssValue { parse_value(css).unwrap() }

#[test]
fn identical_siblings_get_same_style() {
    // 5 middle siblings share (first=false, last=false) → 4 cache hits
    let doc = parse(r#"<div>
        <span class="item">a</span>
        <span class="item">b</span>
        <span class="item">c</span>
        <span class="item">d</span>
        <span class="item">e</span>
        <span class="item">f</span>
        <span class="item">g</span>
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(".item { color: red; padding: 8px; }").unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let div = &styled.children[0];
    for child in &div.children {
        if child.node.element.is_text() { continue; }
        assert!(child.style.color.is_some(), "cached style should have color");
        assert!(child.style.padding_top.is_some(), "cached style should have padding");
    }

    let stats = ctx.cache_stats();
    // 7 spans: first(miss), 5 middle(1 miss + 4 hits), last(miss) = 4 hits minimum
    assert!(stats.hits >= 4, "expected at least 4 cache hits, got {}", stats.hits);
}

#[test]
fn identical_elements_in_different_parents_get_same_base_style() {
    let doc = parse(r#"
        <div><p class="x">a</p></div>
        <section><p class="x">b</p></section>
    "#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(".x { color: blue; font-size: 14px; }").unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let p1 = &styled.children[0].children[0];
    let p2 = &styled.children[1].children[0];
    assert_eq!(p1.style.color, p2.style.color);
    assert_eq!(p1.style.font_size, p2.style.font_size);
}

#[test]
fn cache_distinguishes_different_classes() {
    let doc = parse(r#"<div>
        <span class="a">x</span>
        <span class="b">y</span>
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        ".a { color: red; } .b { color: blue; }"
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let div = &styled.children[0];
    let a = div.children.iter().find(|c| !c.node.element.is_text() && c.node.class_list.iter().any(|cl| cl.as_ref() == "a")).unwrap();
    let b = div.children.iter().find(|c| !c.node.element.is_text() && c.node.class_list.iter().any(|cl| cl.as_ref() == "b")).unwrap();
    assert_ne!(a.style.color.unwrap(), b.style.color.unwrap());
}

#[test]
fn cache_distinguishes_different_ids() {
    let doc = parse(r#"<div>
        <span id="x">x</span>
        <span id="y">y</span>
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        "#x { color: red; } #y { color: blue; }"
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let div = &styled.children[0];
    let x = div.children.iter().find(|c| c.node.id.as_deref() == Some("x")).unwrap();
    let y = div.children.iter().find(|c| c.node.id.as_deref() == Some("y")).unwrap();
    assert_ne!(x.style.color.unwrap(), y.style.color.unwrap());
}

#[test]
fn cache_distinguishes_inline_styles() {
    let doc = parse(r#"<div>
        <span style="color: red">a</span>
        <span style="color: blue">b</span>
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("").unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let div = &styled.children[0];
    let spans: Vec<_> = div.children.iter().filter(|c| !c.node.element.is_text()).collect();
    assert_ne!(spans[0].style.color.unwrap(), spans[1].style.color.unwrap());
}

#[test]
fn cache_distinguishes_hover_state() {
    let doc = parse(r#"<div>
        <span class="btn">a</span>
        <span class="btn">b</span>
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        ".btn { color: red; } .btn:hover { color: blue; }"
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState {
        hover_path: Some(vec![0, 0]),
        ..Default::default()
    };
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let div = &styled.children[0];
    let spans: Vec<_> = div.children.iter().filter(|c| !c.node.element.is_text()).collect();
    // First span is hovered, second is not — they should have different colors
    assert_ne!(spans[0].style.color.unwrap(), spans[1].style.color.unwrap());
}

#[test]
fn many_identical_list_items_all_styled() {
    let items: String = (0..50).map(|i| format!(r#"<li class="item">{}</li>"#, i)).collect();
    let html = format!("<ul>{}</ul>", items);
    let doc = parse(&html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        ".item { color: red; padding: 4px; margin: 2px; display: block; }"
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let ul = &styled.children[0];
    let items: Vec<_> = ul.children.iter().filter(|c| !c.node.element.is_text()).collect();
    assert_eq!(items.len(), 50);
    for item in &items {
        assert!(item.style.color.is_some());
        assert!(item.style.padding_top.is_some());
        assert!(item.style.margin_top.is_some());
        assert!(item.style.display.is_some());
    }

    let _stats = ctx.cache_stats();
    // With parallel cascade (≥16 siblings), per-thread caches are isolated
    // so sibling-to-sibling cache sharing is reduced.
    // Styles must still be correct (verified above).
}

#[test]
fn cache_respects_first_child_last_child() {
    let doc = parse(r#"<ul>
        <li class="item">first</li>
        <li class="item">middle</li>
        <li class="item">last</li>
    </ul>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        ".item { color: red; } .item:first-child { color: blue; } .item:last-child { color: green; }"
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let ul = &styled.children[0];
    let items: Vec<_> = ul.children.iter().filter(|c| !c.node.element.is_text()).collect();
    assert_eq!(items.len(), 3);
    // first, middle, last should all have color but potentially different values
    // because :first-child and :last-child affect the cache key
    assert!(items[0].style.color.is_some());
    assert!(items[1].style.color.is_some());
    assert!(items[2].style.color.is_some());
    // first and last should differ from middle
    assert_ne!(items[0].style.color.unwrap(), items[1].style.color.unwrap());
    assert_ne!(items[2].style.color.unwrap(), items[1].style.color.unwrap());
}

#[test]
fn inheritance_applied_after_cache_hit() {
    let doc = parse(r#"
        <div style="color: red"><span class="x">a</span></div>
        <div style="color: blue"><span class="x">b</span></div>
    "#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("").unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let span1 = &styled.children[0].children[0];
    let span2 = &styled.children[1].children[0];
    // Both spans have class="x" so they'd cache-hit on the same pre-inheritance style.
    // But after inheritance, they should have different colors from their parents.
    assert_eq!(*span1.style.color.unwrap(), val("red"));
    assert_eq!(*span2.style.color.unwrap(), val("blue"));

    // No cache hit expected: parents differ (different inline styles),
    // so ancestor hash differs — correctly preventing false sharing.
}

#[test]
fn cache_distinguishes_ancestor_context() {
    let doc = parse(r#"
        <div class="container"><span class="item">a</span></div>
        <div class="sidebar"><span class="item">b</span></div>
    "#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        ".container .item { color: red; } .sidebar .item { color: blue; }"
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let item1 = &styled.children[0].children[0];
    let item2 = &styled.children[1].children[0];
    assert_ne!(
        item1.style.color.unwrap(), item2.style.color.unwrap(),
        "items in different ancestors must get different styles despite same class"
    );

    let stats = ctx.cache_stats();
    // Two spans with same class but different parents → 0 hits
    assert_eq!(stats.hits, 0, "different ancestor context must not cache-hit");
}

#[test]
fn cache_distinguishes_attribute_selectors() {
    // 5 inputs so the 3 middle text inputs share positional state (first=false, last=false)
    let doc = parse(r#"<div>
        <input type="text">
        <input type="text">
        <input type="email">
        <input type="text">
        <input type="text">
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        r#"[type="text"] { color: red; } [type="email"] { color: blue; }"#
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let div = &styled.children[0];
    let inputs: Vec<_> = div.children.iter().filter(|c| !c.node.element.is_text()).collect();
    assert_eq!(inputs.len(), 5);
    // text and email inputs must get different colors
    assert_ne!(inputs[1].style.color.unwrap(), inputs[2].style.color.unwrap());
    // middle text inputs should have same color
    assert_eq!(inputs[1].style.color.unwrap(), inputs[3].style.color.unwrap());

    let stats = ctx.cache_stats();
    // middle text inputs share same key → at least 1 hit
    assert!(stats.hits >= 1, "identical middle inputs should cache-hit, got {} hits", stats.hits);
}

#[test]
fn cache_distinguishes_data_attributes() {
    let doc = parse(r#"<div>
        <span data-theme="dark">a</span>
        <span data-theme="light">b</span>
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(
        r#"[data-theme="dark"] { color: red; } [data-theme="light"] { color: blue; }"#
    ).unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let div = &styled.children[0];
    let spans: Vec<_> = div.children.iter().filter(|c| !c.node.element.is_text()).collect();
    assert_ne!(spans[0].style.color.unwrap(), spans[1].style.color.unwrap());
}

#[test]
fn identical_items_in_same_ancestor_still_cache() {
    let doc = parse(r#"<div class="list">
        <span class="item">a</span>
        <span class="item">b</span>
        <span class="item">c</span>
        <span class="item">d</span>
        <span class="item">e</span>
    </div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(".list .item { color: red; }").unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let _styled = ctx.cascade(&doc.root, &media, &interaction);

    let stats = ctx.cache_stats();
    // 5 items in same parent, middle 3 share (first=false, last=false) → 2 hits
    assert!(stats.hits >= 2, "identical items in same ancestor should cache-hit, got {} hits", stats.hits);
}

#[test]
fn stats_zero_hits_when_all_unique() {
    let doc = parse(r#"<div><span id="a">x</span><span id="b">y</span><span id="c">z</span></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("#a { color: red; } #b { color: blue; } #c { color: green; }").unwrap()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let _styled = ctx.cascade(&doc.root, &media, &interaction);

    let stats = ctx.cache_stats();
    // All elements have different ids → 0 cache hits among the spans
    // (html + div might share if they're both "no-class no-id" but spans are unique)
    assert_eq!(stats.hits, 0, "expected 0 hits for all-unique elements, got {}", stats.hits);
}
