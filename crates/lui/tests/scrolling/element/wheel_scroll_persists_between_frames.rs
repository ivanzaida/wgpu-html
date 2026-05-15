use crate::support::{TEST_HEIGHT, TEST_WIDTH, red_quad_y, test_lui};

#[test]
fn wheel_scroll_persists_element_scroll_between_frames() {
  let (mut lui, mut spy) = test_lui(
    r#"
    <html>
      <body>
        <div style="width: 120px; height: 100px; overflow-y: scroll">
          <div style="height: 400px; background: red"></div>
        </div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let before = red_quad_y(&spy.take_last_list());

  lui.set_cursor_position(10.0, 10.0);
  assert!(lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 60.0));

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let after_first_render = red_quad_y(&spy.take_last_list());

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let after_second_render = red_quad_y(&spy.take_last_list());

  assert!(
    after_first_render < before - 50.0,
    "element scroll should move painted content: before={before} after={after_first_render}"
  );
  assert!(
    (after_second_render - after_first_render).abs() < 0.01,
    "element scroll should persist across fresh layouts: first={after_first_render} second={after_second_render}"
  );
}
