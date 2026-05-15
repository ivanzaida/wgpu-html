use crate::support::{TEST_HEIGHT, TEST_WIDTH, ua_lui};

fn resolve_color(s: &str) -> [f32; 4] {
  let v = lui_parse::parse_value(s).unwrap();
  lui::paint::color::resolve_color(Some(&v)).unwrap()
}

fn find_quads_approx(list: &lui::display_list::DisplayList, color: [f32; 4]) -> Vec<lui::display_list::Quad> {
  list
    .quads
    .iter()
    .copied()
    .filter(|q| q.color.iter().zip(color.iter()).all(|(a, b)| (a - b).abs() < 0.02))
    .collect()
}

#[test]
fn element_scrollbar_uses_pseudo_styles_from_ua() {
  let (mut lui, mut spy) = ua_lui(
    r#"
    <html>
      <body>
        <div style="width: 120px; height: 100px; overflow-y: scroll;">
          <div style="height: 400px"></div>
        </div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let ua_thumb_color = resolve_color("#888");
  let ua_track_color = resolve_color("#222");
  let fallback_track = [0.95_f32, 0.95, 0.95, 1.0];

  let thumb_quads = find_quads_approx(&list, ua_thumb_color);
  let track_quads = find_quads_approx(&list, ua_track_color);
  let fallback_quads = find_quads_approx(&list, fallback_track);

  assert!(
    fallback_quads.is_empty(),
    "element scrollbar should NOT use hardcoded fallback track color [0.95,0.95,0.95]; \
     UA pseudo-element styles should apply. got quads: {:?}",
    list.quads.iter().map(|q| (q.color, q.rect)).collect::<Vec<_>>()
  );
  assert!(
    !track_quads.is_empty(),
    "element scrollbar track should use UA ::lui-scrollbar-track color #222, \
     got quads: {:?}",
    list.quads.iter().map(|q| (q.color, q.rect)).collect::<Vec<_>>()
  );
  assert!(
    !thumb_quads.is_empty(),
    "element scrollbar thumb should use UA ::lui-scrollbar-thumb color #888, \
     got quads: {:?}",
    list.quads.iter().map(|q| (q.color, q.rect)).collect::<Vec<_>>()
  );
}
