mod support;

use support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn flex_item_anon_block_no_background_bleed() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="display:flex; width:300px">
        <div style="background:red; padding:0 20px">X</div>
      </div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let red_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[0] > 0.9 && q.color[1] < 0.1 && q.color[2] < 0.1 && q.color[3] > 0.9)
    .collect();

  assert!(!red_quads.is_empty(), "expected at least one red quad");
  assert_eq!(red_quads.len(), 1, "anonymous block should not paint its own background (got {} red quads)", red_quads.len());
}

#[test]
fn flex_item_anon_block_no_bleed_with_shell_styles() {
  let spy = support::RenderSpy::default();
  let mut lui = lui::Lui::new();
  lui.set_stylesheets(&[
    lui_parse::parse_stylesheet("* { margin: 0; padding: 0; border-width: 0; box-sizing: border-box; scrollbar-width: thin; }").unwrap(),
  ]);
  lui.set_html(
    r#"<html><body>
      <div style="padding: 24px">
        <div style="display:flex; width:300px">
          <div style="background:red; padding:0 20px">X</div>
        </div>
      </div>
    </body></html>"#,
  );
  let mut spy = spy;
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let red_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[0] > 0.9 && q.color[1] < 0.1 && q.color[2] < 0.1 && q.color[3] > 0.9)
    .collect();

  for (i, q) in red_quads.iter().enumerate() {
    eprintln!("[shell] red quad[{}]: x={:.1} y={:.1} w={:.1} h={:.1}", i, q.rect.x, q.rect.y, q.rect.w, q.rect.h);
  }

  assert_eq!(red_quads.len(), 1, "with shell-like styles: anonymous block should not paint its own background (got {} red quads)", red_quads.len());
}

#[test]
fn flex_item_anon_block_no_bleed_three_items() {
  let spy = support::RenderSpy::default();
  let mut lui = lui::Lui::new();
  lui.set_stylesheets(&[
    lui_parse::parse_stylesheet("* { margin: 0; padding: 0; border-width: 0; box-sizing: border-box; scrollbar-width: thin; }").unwrap(),
  ]);
  lui.set_html(
    r#"<html><body>
      <div style="padding: 24px">
        <div style="display:flex; width:400px; background:#16213e; border:1px solid #0f3460; padding:4px">
          <div style="background:#e94560; padding:8px 12px; color:white">A</div>
          <div style="background:#0f3460; padding:8px 12px; color:white">B</div>
          <div style="background:#533483; padding:8px 12px; color:white">C</div>
        </div>
      </div>
    </body></html>"#,
  );
  let mut spy = spy;
  lui.render_frame(&mut spy, TEST_WIDTH * 3, TEST_HEIGHT * 3, 1.0);
  let list = spy.take_last_list();

  for (i, q) in list.quads.iter().enumerate() {
    eprintln!("quad[{}]: x={:.1} y={:.1} w={:.1} h={:.1} color=[{:.3},{:.3},{:.3},{:.3}]", i, q.rect.x, q.rect.y, q.rect.w, q.rect.h, q.color[0], q.color[1], q.color[2], q.color[3]);
  }
  eprintln!("total quads: {}", list.quads.len());

  eprintln!("--- QUADS ---");
  for (i, q) in list.quads.iter().enumerate() {
    eprintln!("  quad[{}]: x={:.1} y={:.1} w={:.1} h={:.1} rgba=[{:.3},{:.3},{:.3},{:.3}]", i, q.rect.x, q.rect.y, q.rect.w, q.rect.h, q.color[0], q.color[1], q.color[2], q.color[3]);
  }
  eprintln!("--- GLYPHS ({}) ---", list.glyphs.len());
  for (i, g) in list.glyphs.iter().enumerate() {
    eprintln!("  glyph[{}]: x={:.1} y={:.1} w={:.1} h={:.1} uv=[{:.4},{:.4}]-[{:.4},{:.4}]", i, g.rect.x, g.rect.y, g.rect.w, g.rect.h, g.uv_min[0], g.uv_min[1], g.uv_max[0], g.uv_max[1]);
  }
  assert_eq!(list.quads.len(), 5, "expected 5 quads (container border + bg + 3 items), got {}", list.quads.len());
}
