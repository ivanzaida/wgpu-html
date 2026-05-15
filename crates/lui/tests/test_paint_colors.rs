mod support;

use support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

/// Flex items with padding+background must not paint extra quads beyond
/// their border rect. Anonymous block wrappers inherit the parent's style
/// (including background); if painted, they bleed outside the item.
#[test]
fn flex_item_anon_block_no_background_bleed() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="display:flex; width:300px">
        <div id="item" style="background:red; padding:0 20px">X</div>
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

  for (i, q) in red_quads.iter().enumerate() {
    eprintln!("red quad[{}]: x={:.1} y={:.1} w={:.1} h={:.1}", i, q.rect.x, q.rect.y, q.rect.w, q.rect.h);
  }
  eprintln!("total red quads: {}", red_quads.len());

  assert!(!red_quads.is_empty(), "expected at least one red quad for the flex item");

  // There should be exactly 1 red quad (the flex item).
  // If anonymous blocks paint their inherited background, there will be 2+.
  assert_eq!(
    red_quads.len(),
    1,
    "only the flex item should paint a red background, not its anonymous block wrapper (got {} red quads)",
    red_quads.len(),
  );
}
