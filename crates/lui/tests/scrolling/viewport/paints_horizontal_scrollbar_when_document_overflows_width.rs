use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn paints_horizontal_scrollbar_when_document_overflows_width() {
  let (mut lui, mut spy) = test_lui(
    r#"
    <html>
      <body style="scrollbar-width: thin; scrollbar-color: red blue;">
        <div style="margin-left: 260px; width: 40px; height: 40px; background: red;"></div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let horizontal_track = list
    .quads
    .iter()
    .find(|quad| {
      quad.rect.h > 0.0
        && quad.rect.h < 12.0
        && quad.rect.w > 150.0
        && (quad.color[2] - 1.0).abs() < 0.01
        && quad.color[0].abs() < 0.01
    })
    .expect("expected viewport horizontal scrollbar track");

  let horizontal_thumb = list
    .quads
    .iter()
    .find(|quad| {
      quad.rect.h > 0.0
        && quad.rect.h < 12.0
        && quad.rect.w < horizontal_track.rect.w
        && (quad.color[0] - 1.0).abs() < 0.01
        && quad.color[1].abs() < 0.01
        && quad.color[2].abs() < 0.01
    })
    .expect("expected viewport horizontal scrollbar thumb");

  assert!(
    (horizontal_track.rect.h - 8.0).abs() < 1.0,
    "thin viewport scrollbar should paint at 8px height, got {}",
    horizontal_track.rect.h
  );
  assert!(
    horizontal_thumb.rect.w < horizontal_track.rect.w,
    "viewport thumb should be smaller than its track: thumb={:?} track={:?}",
    horizontal_thumb.rect,
    horizontal_track.rect
  );
  assert!(
    horizontal_thumb.rect.h < horizontal_track.rect.h,
    "viewport thumb should be inset inside the track: thumb={:?} track={:?}",
    horizontal_thumb.rect,
    horizontal_track.rect
  );
  assert!(
    horizontal_thumb.rect.x > horizontal_track.rect.x
      || horizontal_thumb.rect.x + horizontal_thumb.rect.w < horizontal_track.rect.x + horizontal_track.rect.w,
    "viewport track should remain visible around the thumb: thumb={:?} track={:?}",
    horizontal_thumb.rect,
    horizontal_track.rect
  );
}
