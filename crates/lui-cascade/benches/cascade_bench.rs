use criterion::{criterion_group, criterion_main, Criterion, black_box};
use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_css_parser::parse_stylesheet;
use lui_html_parser::parse;

const UA_CSS: &str = include_str!("../../../.data/ua_whatwg_html.css");

fn build_deep_html(depth: usize, breadth: usize) -> String {
    fn nest(depth: usize, breadth: usize) -> String {
        if depth == 0 {
            return "<span>leaf</span>".to_string();
        }
        let children: String = (0..breadth)
            .map(|i| format!(r#"<div class="d{depth} c{i}">{}</div>"#, nest(depth - 1, breadth)))
            .collect();
        children
    }
    format!("<html><body>{}</body></html>", nest(depth, breadth))
}

fn build_stylesheet(num_rules: usize) -> String {
    let mut css = String::new();
    for i in 0..num_rules {
        css.push_str(&format!(".c{} {{ color: red; padding: {}px; margin: {}px; }}\n", i % 10, i, i * 2));
    }
    css.push_str("div { display: block; font-family: Arial; }\n");
    css.push_str("span { display: inline; }\n");
    css.push_str(".d1 { font-size: 14px; }\n");
    css.push_str(".d2 { font-size: 16px; }\n");
    css.push_str(".d3 { font-size: 18px; }\n");
    css
}

fn bench_full_cascade(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_cascade");

    let ua = parse_stylesheet(UA_CSS).unwrap();
    let author_css = build_stylesheet(50);
    let author = parse_stylesheet(&author_css).unwrap();

    // Small tree: 3 levels, 3 children each = ~40 nodes
    let html_small = build_deep_html(3, 3);
    let doc_small = parse(&html_small);

    group.bench_function("small_tree_40_nodes", |b| {
        let mut ctx = CascadeContext::new();
        ctx.set_stylesheets(&[ua.clone(), author.clone()]);
        let media = MediaContext::default();
        let interaction = InteractionState::default();
        b.iter(|| {
            let styled = ctx.cascade(&doc_small.root, &media, &interaction);
            black_box(&styled);
        });
    });

    // Medium tree: 4 levels, 4 children each = ~340 nodes
    let html_med = build_deep_html(4, 4);
    let doc_med = parse(&html_med);

    group.bench_function("medium_tree_340_nodes", |b| {
        let mut ctx = CascadeContext::new();
        ctx.set_stylesheets(&[ua.clone(), author.clone()]);
        let media = MediaContext::default();
        let interaction = InteractionState::default();
        b.iter(|| {
            let styled = ctx.cascade(&doc_med.root, &media, &interaction);
            black_box(&styled);
        });
    });

    // Large tree: 5 levels, 4 children each = ~1365 nodes
    let html_large = build_deep_html(5, 4);
    let doc_large = parse(&html_large);

    group.bench_function("large_tree_1365_nodes", |b| {
        let mut ctx = CascadeContext::new();
        ctx.set_stylesheets(&[ua.clone(), author.clone()]);
        let media = MediaContext::default();
        let interaction = InteractionState::default();
        b.iter(|| {
            let styled = ctx.cascade(&doc_large.root, &media, &interaction);
            black_box(&styled);
        });
    });

    group.finish();
}

fn bench_incremental_cascade(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_cascade");

    let html = build_deep_html(5, 4);
    let doc = parse(&html);
    let ua = parse_stylesheet(UA_CSS).unwrap();
    let author_css = build_stylesheet(50);
    let author = parse_stylesheet(&author_css).unwrap();
    let media = MediaContext::default();
    let interaction = InteractionState::default();

    // Baseline: two full cascades (what you'd pay without incremental)
    group.bench_function("two_full_cascades", |b| {
        let mut ctx = CascadeContext::new();
        ctx.set_stylesheets(&[ua.clone(), author.clone()]);
        b.iter(|| {
            let _first = ctx.cascade(&doc.root, &media, &interaction);
            let second = ctx.cascade(&doc.root, &media, &interaction);
            black_box(&second);
        });
    });

    // Full + incremental pure clone (best case: nothing dirty)
    group.bench_function("full_plus_clone_0_dirty", |b| {
        let mut ctx = CascadeContext::new();
        ctx.set_stylesheets(&[ua.clone(), author.clone()]);
        b.iter(|| {
            let prev = ctx.cascade(&doc.root, &media, &interaction);
            let styled = ctx.cascade_dirty(
                &doc.root, &prev, &[],
                &media, &interaction,
            );
            black_box(&styled);
        });
    });

    // Full + incremental with 1 dirty leaf
    group.bench_function("full_plus_dirty_1_leaf", |b| {
        let mut ctx = CascadeContext::new();
        ctx.set_stylesheets(&[ua.clone(), author.clone()]);
        b.iter(|| {
            let prev = ctx.cascade(&doc.root, &media, &interaction);
            let styled = ctx.cascade_dirty(
                &doc.root, &prev, &[vec![0, 0, 0, 0, 0]],
                &media, &interaction,
            );
            black_box(&styled);
        });
    });

    // Full + incremental with 1 dirty subtree near root
    group.bench_function("full_plus_dirty_1_subtree", |b| {
        let mut ctx = CascadeContext::new();
        ctx.set_stylesheets(&[ua.clone(), author.clone()]);
        b.iter(|| {
            let prev = ctx.cascade(&doc.root, &media, &interaction);
            let styled = ctx.cascade_dirty(
                &doc.root, &prev, &[vec![0, 0]],
                &media, &interaction,
            );
            black_box(&styled);
        });
    });

    group.finish();
}

fn bench_set_stylesheets(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_stylesheets");

    let css_small = build_stylesheet(10);
    let sheet_small = parse_stylesheet(&css_small).unwrap();

    group.bench_function("10_rules", |b| {
        let mut ctx = CascadeContext::new();
        b.iter(|| {
            ctx.set_stylesheets(&[sheet_small.clone()]);
            black_box(&ctx);
        });
    });

    let css_large = build_stylesheet(500);
    let sheet_large = parse_stylesheet(&css_large).unwrap();

    group.bench_function("500_rules", |b| {
        let mut ctx = CascadeContext::new();
        b.iter(|| {
            ctx.set_stylesheets(&[sheet_large.clone()]);
            black_box(&ctx);
        });
    });

    group.finish();
}

fn bench_selector_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("selector_matching");

    let html = r#"<div id="main" class="container large">
        <ul class="list">
            <li class="item active">one</li>
            <li class="item">two</li>
            <li class="item">three</li>
        </ul>
    </div>"#;
    let doc = parse(html);

    group.bench_function("query_selector_all", |b| {
        b.iter(|| {
            let results = lui_cascade::query::query_selector_all(&doc.root, ".item");
            std::hint::black_box(results);
        });
    });

    group.bench_function("query_selector_complex", |b| {
        b.iter(|| {
            let results = lui_cascade::query::query_selector_all(&doc.root, ".container > .list .item.active");
            std::hint::black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_full_cascade,
    bench_incremental_cascade,
    bench_set_stylesheets,
    bench_selector_matching,
);
criterion_main!(benches);
