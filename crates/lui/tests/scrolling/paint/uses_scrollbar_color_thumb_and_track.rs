use lui::display_list::DisplayList;

use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

fn find_quad_by_color(list: &DisplayList, color: [f32; 4]) -> Option<lui::display_list::Quad> {
  list.quads.iter().copied().find(|quad| {
    quad
      .color
      .iter()
      .zip(color.iter())
      .all(|(actual, expected)| (*actual - *expected).abs() < 0.01)
  })
}

#[test]
fn paints_scrollbar_thumb_and_track_using_scrollbar_color() {
  let (mut lui, spy) = test_lui(
    r#"
    <html>
      <body>
        <div style="width: 120px; height: 100px; overflow-y: scroll; scrollbar-width: thin; scrollbar-color: red blue;">
          <div style="height: 400px"></div>
        </div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let thumb = find_quad_by_color(&list, [1.0, 0.0, 0.0, 1.0]).expect("expected red scrollbar thumb quad");
  let track = find_quad_by_color(&list, [0.0, 0.0, 1.0, 1.0]).expect("expected blue scrollbar track quad");

  assert!(
    thumb.rect.h < track.rect.h,
    "thumb should be smaller than track: thumb={:?} track={:?}",
    thumb.rect,
    track.rect
  );
  assert!(
    (track.rect.w - 8.0).abs() < 1.0,
    "thin scrollbar track width should paint at 8px, got {}",
    track.rect.w
  );
}
