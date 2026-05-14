use crate::support::{TEST_HEIGHT, TEST_WIDTH, ua_lui};

#[test]
fn applies_default_scrollbar_style_from_ua() {
  let (mut lui, spy) = ua_lui(
    r#"
    <html>
      <body>
        <div style="height: 320px"></div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let track = list
    .quads
    .iter()
    .find(|quad| quad.rect.w > 0.0 && quad.rect.w < 20.0 && quad.rect.h > 150.0)
    .expect("expected viewport scrollbar track from UA defaults");
  let thumb = list
    .quads
    .iter()
    .find(|quad| quad.rect.w < track.rect.w && quad.rect.h < track.rect.h)
    .expect("expected viewport scrollbar thumb from UA defaults");

  assert!(
    approx_eq(track.color, expected("#222")),
    "UA viewport scrollbar track color should come from UA stylesheet: got={:?}",
    track.color
  );
  assert!(
    approx_eq(thumb.color, expected("#888")),
    "UA viewport scrollbar thumb color should come from UA stylesheet: got={:?}",
    thumb.color
  );
}

fn expected(s: &str) -> [f32; 4] {
  let value = lui_parse::parse_value(s).expect("expected parseable css value");
  lui::paint::color::resolve_color(Some(&value)).expect("expected resolvable color")
}

fn approx_eq(a: [f32; 4], b: [f32; 4]) -> bool {
  a.iter().zip(b.iter()).all(|(lhs, rhs)| (*lhs - *rhs).abs() < 0.01)
}
