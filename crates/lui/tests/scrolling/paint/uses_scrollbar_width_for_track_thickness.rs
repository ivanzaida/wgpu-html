use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn paints_scrollbar_track_using_layout_scrollbar_width() {
  let (mut lui, mut spy) = test_lui(
    r#"
    <html>
      <body>
        <div style="width: 120px; height: 100px; overflow-y: scroll; scrollbar-width: thin;">
          <div style="height: 400px"></div>
        </div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let scrollbar_track = list
    .quads
    .iter()
    .find(|quad| quad.rect.w > 0.0 && quad.rect.w < 12.0 && quad.rect.h > 90.0)
    .expect("expected vertical scrollbar track");

  assert!(
    (scrollbar_track.rect.w - 8.0).abs() < 1.0,
    "thin scrollbar should paint with 8px track width, got {}",
    scrollbar_track.rect.w
  );
}
