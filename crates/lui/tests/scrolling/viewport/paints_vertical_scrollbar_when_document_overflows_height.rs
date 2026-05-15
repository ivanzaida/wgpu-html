use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn paints_vertical_scrollbar_when_document_overflows_height() {
  let (mut lui, mut spy) = test_lui(
    r#"
    <html>
      <body style="scrollbar-width: thin; scrollbar-color: red blue;">
        <div style="height: 320px"></div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let vertical_track = list
    .quads
    .iter()
    .find(|quad| {
      quad.rect.w > 0.0
        && quad.rect.w < 12.0
        && quad.rect.h > 150.0
        && (quad.color[2] - 1.0).abs() < 0.01
        && quad.color[0].abs() < 0.01
    })
    .expect("expected viewport vertical scrollbar track");

  let vertical_thumb = list
    .quads
    .iter()
    .find(|quad| {
      quad.rect.w > 0.0
        && quad.rect.w < 12.0
        && quad.rect.h < vertical_track.rect.h
        && (quad.color[0] - 1.0).abs() < 0.01
        && quad.color[1].abs() < 0.01
        && quad.color[2].abs() < 0.01
    })
    .expect("expected viewport vertical scrollbar thumb");

  assert!(
    (vertical_track.rect.w - 8.0).abs() < 1.0,
    "thin viewport scrollbar should paint at 8px width, got {}",
    vertical_track.rect.w
  );
  assert!(
    vertical_thumb.rect.h < vertical_track.rect.h,
    "viewport thumb should be smaller than its track: thumb={:?} track={:?}",
    vertical_thumb.rect,
    vertical_track.rect
  );
  assert!(
    vertical_thumb.rect.w < vertical_track.rect.w,
    "viewport thumb should be inset inside the track: thumb={:?} track={:?}",
    vertical_thumb.rect,
    vertical_track.rect
  );
  assert!(
    vertical_thumb.rect.y > vertical_track.rect.y
      || vertical_thumb.rect.y + vertical_thumb.rect.h < vertical_track.rect.y + vertical_track.rect.h,
    "viewport track should remain visible around the thumb: thumb={:?} track={:?}",
    vertical_thumb.rect,
    vertical_track.rect
  );
}
