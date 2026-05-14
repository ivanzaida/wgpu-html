use crate::support::{TEST_HEIGHT, TEST_WIDTH, red_quad_y, test_lui};

#[test]
fn wheel_scroll_falls_back_to_viewport_translation() {
  let (mut lui, spy) = test_lui(
    r#"
    <html>
      <body>
        <div style="height: 320px"></div>
        <div style="width: 80px; height: 40px; background: red"></div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let before = red_quad_y(&spy.take_last_list());

  lui.set_cursor_position(10.0, 10.0);
  assert!(lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 80.0));

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let after = red_quad_y(&spy.take_last_list());

  assert!(
    after < before - 70.0,
    "viewport scroll should translate painted content: before={before} after={after}"
  );
}
