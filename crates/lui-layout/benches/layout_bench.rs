use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_layout::{
  engine::{layout_tree, layout_tree_with},
  incremental::layout_tree_incremental_with,
  text::TextContext,
};
use lui_parse::{parse, parse_stylesheet};

const UA_CSS: &str = include_str!("../../lui/ua/ua_whatwg.css");

fn setup(html: &str) -> (lui_parse::HtmlDocument, CascadeContext) {
  let full = format!("<html><body>{}</body></html>", html);
  let doc = parse(&full);
  let ua = parse_stylesheet(UA_CSS).unwrap();
  let reset = parse_stylesheet("* { margin: 0; padding: 0; border-width: 0; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[ua, reset]);
  (doc, ctx)
}

// ── HTML generators ──────────────────────────────────────────────────

fn block_stack(n: usize) -> String {
  (0..n)
    .map(|i| format!(r#"<div style="height:20px; width:100%; padding:2px; margin:4px" class="b{i}">block {i}</div>"#))
    .collect()
}

fn nested_blocks(depth: usize, breadth: usize) -> String {
  fn nest(depth: usize, breadth: usize) -> String {
    if depth == 0 {
      return "<div style=\"height:10px\">leaf</div>".to_string();
    }
    let children: String = (0..breadth)
      .map(|_| {
        format!(
          r#"<div style="padding:2px; margin:1px">{}</div>"#,
          nest(depth - 1, breadth)
        )
      })
      .collect();
    children
  }
  nest(depth, breadth)
}

fn flex_row(n: usize) -> String {
  let items: String = (0..n)
    .map(|i| format!(r#"<div style="flex:1; height:40px; padding:4px">item {i}</div>"#))
    .collect();
  format!(r#"<div style="display:flex; width:800px; gap:8px">{items}</div>"#)
}

fn flex_wrap(rows: usize, cols: usize) -> String {
  let items: String = (0..rows * cols)
    .map(|i| {
      format!(
        r#"<div style="flex:0 0 {}px; height:30px">item {i}</div>"#,
        800 / cols - 8
      )
    })
    .collect();
  format!(r#"<div style="display:flex; flex-wrap:wrap; width:800px; gap:8px">{items}</div>"#)
}

fn flex_nested(depth: usize, items_per: usize) -> String {
  fn nest(depth: usize, items_per: usize, row: bool) -> String {
    if depth == 0 {
      return r#"<div style="flex:1; height:20px">leaf</div>"#.to_string();
    }
    let dir = if row { "row" } else { "column" };
    let children: String = (0..items_per)
      .map(|_| format!(r#"<div style="flex:1">{}</div>"#, nest(depth - 1, items_per, !row)))
      .collect();
    format!(r#"<div style="display:flex; flex-direction:{dir}; gap:4px">{children}</div>"#)
  }
  nest(depth, items_per, true)
}

fn grid_fixed(rows: usize, cols: usize) -> String {
  let col_def: String = (0..cols).map(|_| "1fr ").collect();
  let items: String = (0..rows * cols)
    .map(|i| format!(r#"<div style="height:30px; padding:2px">cell {i}</div>"#))
    .collect();
  format!(r#"<div style="display:grid; grid-template-columns:{col_def}; gap:4px; width:800px">{items}</div>"#)
}

fn grid_auto_placement(n: usize) -> String {
  let items: String = (0..n)
    .map(|i| format!(r#"<div style="height:30px">item {i}</div>"#))
    .collect();
  format!(
    r#"<div style="display:grid; grid-template-columns:repeat(4, 1fr); grid-auto-rows:40px; gap:4px; width:800px">{items}</div>"#
  )
}

fn table_simple(rows: usize, cols: usize) -> String {
  let header: String = (0..cols)
    .map(|c| format!("<th style=\"height:25px\">H{c}</th>"))
    .collect();
  let body_rows: String = (0..rows)
    .map(|r| {
      let cells: String = (0..cols)
        .map(|c| format!("<td style=\"height:20px\">R{r}C{c}</td>"))
        .collect();
      format!("<tr>{cells}</tr>")
    })
    .collect();
  format!(
    r#"<table style="width:800px; border-spacing:2px"><thead><tr>{header}</tr></thead><tbody>{body_rows}</tbody></table>"#
  )
}

fn table_with_spans(rows: usize, cols: usize) -> String {
  let body_rows: String = (0..rows)
    .map(|r| {
      let cells: String = if r % 3 == 0 && cols >= 2 {
        let mut s = format!(r#"<td colspan="2" style="height:20px">span</td>"#);
        for c in 2..cols {
          s.push_str(&format!("<td style=\"height:20px\">R{r}C{c}</td>"));
        }
        s
      } else {
        (0..cols)
          .map(|c| format!("<td style=\"height:20px\">R{r}C{c}</td>"))
          .collect()
      };
      format!("<tr>{cells}</tr>")
    })
    .collect();
  format!(r#"<table style="width:800px; border-spacing:2px">{body_rows}</table>"#)
}

fn inline_text(n: usize) -> String {
  let words: String = (0..n).map(|i| format!("<span>word{i} </span>")).collect();
  format!(r#"<div style="width:400px">{words}</div>"#)
}

fn positioned_tree(n: usize) -> String {
  let children: String = (0..n)
    .map(|i| {
      format!(
        r#"<div style="position:absolute; top:{}px; left:{}px; width:50px; height:50px">abs {i}</div>"#,
        i * 10,
        i * 15
      )
    })
    .collect();
  format!(r#"<div style="position:relative; width:800px; height:600px">{children}</div>"#)
}

fn mixed_layout() -> String {
  format!(
    r#"<div style="display:flex; width:800px; gap:16px">
            <div style="flex:0 0 200px">
                <div style="padding:8px">
                    <h2 style="height:24px">Sidebar</h2>
                    <ul><li style="height:20px">Item 1</li><li style="height:20px">Item 2</li><li style="height:20px">Item 3</li></ul>
                </div>
            </div>
            <div style="flex:1">
                <div style="display:grid; grid-template-columns:1fr 1fr 1fr; gap:8px">
                    {cards}
                </div>
                <table style="width:100%; border-spacing:4px; margin-top:16px">
                    <tr><th style="height:25px">Name</th><th style="height:25px">Value</th><th style="height:25px">Status</th></tr>
                    <tr><td style="height:20px">Alpha</td><td style="height:20px">100</td><td style="height:20px">Active</td></tr>
                    <tr><td style="height:20px">Beta</td><td style="height:20px">200</td><td style="height:20px">Pending</td></tr>
                </table>
            </div>
        </div>"#,
    cards = (0..9)
      .map(|i| format!(r#"<div style="padding:8px; height:80px">Card {i}</div>"#))
      .collect::<String>(),
  )
}

// ── Benchmark groups ─────────────────────────────────────────────────

fn bench_block_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("block_layout");
  let mut text_ctx = TextContext::new();

  let html = block_stack(50);
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("50_stacked_divs", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = block_stack(200);
  let (doc, ctx) = setup(&html);

  group.bench_function("200_stacked_divs", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = nested_blocks(4, 3);
  let (doc, ctx) = setup(&html);

  group.bench_function("nested_4_deep_3_wide", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = nested_blocks(3, 8);
  let (doc, ctx) = setup(&html);

  group.bench_function("nested_3_deep_8_wide", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_flex_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("flex_layout");
  let mut text_ctx = TextContext::new();

  let html = flex_row(10);
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("row_10_items", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = flex_row(50);
  let (doc, ctx) = setup(&html);

  group.bench_function("row_50_items", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = flex_wrap(5, 4);
  let (doc, ctx) = setup(&html);

  group.bench_function("wrap_5x4", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = flex_wrap(10, 8);
  let (doc, ctx) = setup(&html);

  group.bench_function("wrap_10x8", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = flex_nested(3, 3);
  let (doc, ctx) = setup(&html);

  group.bench_function("nested_3_deep", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = flex_nested(4, 3);
  let (doc, ctx) = setup(&html);

  group.bench_function("nested_4_deep", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_grid_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("grid_layout");
  let mut text_ctx = TextContext::new();

  let html = grid_fixed(4, 4);
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("4x4_fixed", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = grid_fixed(10, 6);
  let (doc, ctx) = setup(&html);

  group.bench_function("10x6_fixed", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = grid_auto_placement(24);
  let (doc, ctx) = setup(&html);

  group.bench_function("auto_24_items", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = grid_auto_placement(100);
  let (doc, ctx) = setup(&html);

  group.bench_function("auto_100_items", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_table_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("table_layout");
  let mut text_ctx = TextContext::new();

  let html = table_simple(5, 4);
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("5x4_simple", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = table_simple(20, 6);
  let (doc, ctx) = setup(&html);

  group.bench_function("20x6_simple", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = table_simple(50, 8);
  let (doc, ctx) = setup(&html);

  group.bench_function("50x8_simple", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = table_with_spans(20, 6);
  let (doc, ctx) = setup(&html);

  group.bench_function("20x6_with_colspan", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_inline_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("inline_layout");
  let mut text_ctx = TextContext::new();

  let html = inline_text(20);
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("20_spans", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = inline_text(100);
  let (doc, ctx) = setup(&html);

  group.bench_function("100_spans", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = inline_text(500);
  let (doc, ctx) = setup(&html);

  group.bench_function("500_spans", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_positioned_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("positioned_layout");
  let mut text_ctx = TextContext::new();

  let html = positioned_tree(20);
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("20_absolute", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html = positioned_tree(100);
  let (doc, ctx) = setup(&html);

  group.bench_function("100_absolute", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_mixed_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("mixed_layout");
  let mut text_ctx = TextContext::new();

  let html = mixed_layout();
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("dashboard_page", |b| {
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
  let mut group = c.benchmark_group("end_to_end");
  let mut text_ctx = TextContext::new();

  let html = mixed_layout();
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  group.bench_function("cascade_plus_layout", |b| {
    b.iter(|| {
      let styled = ctx.cascade(&doc.root, &media, &interaction);
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html_large = format!(
    "<div>{}{}{}</div>",
    nested_blocks(3, 5),
    flex_nested(3, 3),
    grid_fixed(6, 4),
  );
  let (doc, ctx) = setup(&html_large);

  group.bench_function("large_mixed_tree", |b| {
    b.iter(|| {
      let styled = ctx.cascade(&doc.root, &media, &interaction);
      let lt = layout_tree_with(&styled, 1024.0, 768.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

fn bench_incremental_layout(c: &mut Criterion) {
  let mut group = c.benchmark_group("incremental_layout");
  let mut text_ctx = TextContext::new();

  let html = mixed_layout();
  let (doc, ctx) = setup(&html);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);

  group.bench_function("full_baseline", |b| {
    b.iter(|| {
      let lt = layout_tree_with(&styled, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.bench_function("incremental_0_dirty", |b| {
    let prev = layout_tree(&styled, 800.0, 600.0);
    b.iter(|| {
      let lt = layout_tree_incremental_with(&styled, &prev, &[], 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.bench_function("incremental_1_dirty_leaf", |b| {
    let prev = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0, 1, 0, 0]];
    b.iter(|| {
      let lt = layout_tree_incremental_with(&styled, &prev, &dirty, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.bench_function("incremental_1_dirty_near_root", |b| {
    let prev = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0]];
    b.iter(|| {
      let lt = layout_tree_incremental_with(&styled, &prev, &dirty, 800.0, 600.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  let html_large = format!(
    "<div>{}{}{}</div>",
    nested_blocks(3, 5),
    flex_nested(3, 3),
    grid_fixed(6, 4),
  );
  let (doc_l, ctx_l) = setup(&html_large);
  let styled_l = ctx_l.cascade(&doc_l.root, &media, &interaction);

  group.bench_function("large_full_baseline", |b| {
    b.iter(|| {
      let lt = layout_tree(&styled_l, 1024.0, 768.0);
      black_box(&lt);
    });
  });

  group.bench_function("large_incremental_1_leaf", |b| {
    let prev = layout_tree(&styled_l, 1024.0, 768.0);
    let dirty = vec![vec![0, 0, 0, 0, 0]];
    b.iter(|| {
      let lt = layout_tree_incremental_with(&styled_l, &prev, &dirty, 1024.0, 768.0, &mut text_ctx);
      black_box(&lt);
    });
  });

  group.finish();
}

criterion_group!(
  benches,
  bench_block_layout,
  bench_flex_layout,
  bench_grid_layout,
  bench_table_layout,
  bench_inline_layout,
  bench_positioned_layout,
  bench_mixed_layout,
  bench_end_to_end,
  bench_incremental_layout,
);
criterion_main!(benches);
