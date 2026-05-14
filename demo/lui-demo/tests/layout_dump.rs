use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_glyph::TextContext;
use lui_layout::engine::LayoutEngine;
use lui_layout::LayoutBox;
use lui_parse::Stylesheet;
use std::sync::LazyLock;

const HTML: &str = include_str!("../html/test.html");
const UA_CSS: &str = include_str!("../../../crates/lui/ua/ua_whatwg.css");

static UA_SHEET: LazyLock<Stylesheet> =
    LazyLock::new(|| lui_parse::parse_stylesheet(UA_CSS).unwrap_or_default());

fn dump_css_val(v: Option<&lui_parse::CssValue>) -> String {
    match v {
        None => "None".to_string(),
        Some(v) => format!("{:?}", v),
    }
}

fn dump(b: &LayoutBox, indent: usize) {
    let tag = b.node.element.tag_name();
    let kind = format!("{:?}", b.kind);
    let text = if let lui_parse::HtmlElement::Text(t) = &b.node.element {
        let s = t.to_string();
        let escaped: String = s.chars().take(30)
            .map(|c| if c == '\n' { '↵' } else { c })
            .collect();
        format!(" {:?}", escaped.trim())
    } else {
        String::new()
    };
    let pad = "  ".repeat(indent);
    let pad_info = format!(
        "pad=[{:.1},{:.1},{:.1},{:.1}] bdr=[{:.1},{:.1},{:.1},{:.1}]",
        b.padding.top, b.padding.right, b.padding.bottom, b.padding.left,
        b.border.top, b.border.right, b.border.bottom, b.border.left,
    );
    let font_size = dump_css_val(b.style.font_size);
    let border_raw = if b.border.top > 0.01 && !b.node.element.is_text() {
        format!("  border_top_width_raw={}", dump_css_val(b.style.border_top_width))
    } else {
        String::new()
    };
    let font_info = if b.node.element.tag_name() == "table"
        || b.kind == lui_layout::BoxKind::TableCell
        || b.node.element.is_text()
    {
        format!("  font_size={font_size}")
    } else {
        String::new()
    };
    println!(
        "{pad}{kind} <{tag}>{text}  x={:.1} y={:.1} w={:.1} h={:.1}  m=[{:.1},{:.1},{:.1},{:.1}] {pad_info}  children={}{border_raw}{font_info}",
        b.content.x, b.content.y, b.content.width, b.content.height,
        b.margin.top, b.margin.right, b.margin.bottom, b.margin.left,
        b.children.len(),
    );
    for child in &b.children {
        dump(child, indent + 1);
    }
}

#[test]
fn dump_demo_layout() {
    let doc = lui_parse::parse(HTML);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[UA_SHEET.clone()]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let mut text_ctx = TextContext::new();
    let mut engine = LayoutEngine::new();
    let tree = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);
    println!("\n=== WITH UA STYLESHEET ===\n");
    dump(&tree.root, 0);
}

#[test]
fn dump_table_structure() {
    let html = HTML; // use the actual demo test.html
    let doc = lui_parse::parse(html);
    let ctx = CascadeContext::new();
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let mut text_ctx = TextContext::new();
    let mut engine = LayoutEngine::new();
    let tree = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);

    // Find the table
    fn find<'a>(b: &'a LayoutBox<'a>, kind: lui_layout::BoxKind) -> Option<&'a LayoutBox<'a>> {
        if b.kind == kind { return Some(b); }
        for c in &b.children { if let Some(f) = find(c, kind) { return Some(f); } }
        None
    }
    let table = find(&tree.root, lui_layout::BoxKind::Table).unwrap();
    println!("Table: w={} h={}", table.content.width, table.content.height);

    let row = find(table, lui_layout::BoxKind::TableRow).unwrap();
    println!("Row children: {}", row.children.len());
    for (i, child) in row.children.iter().enumerate() {
        println!("  child[{}]: kind={:?} tag={} w={} h={} x={} y={}",
            i, child.kind, child.node.element.tag_name(),
            child.content.width, child.content.height,
            child.content.x, child.content.y);
    }
}

#[test]
fn dump_demo_no_ua() {
    let doc = lui_parse::parse(HTML);
    let ctx = CascadeContext::new();
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let mut text_ctx = TextContext::new();
    let mut engine = LayoutEngine::new();
    let tree = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);
    println!("\n=== WITHOUT UA STYLESHEET ===\n");
    dump(&tree.root, 0);
}
