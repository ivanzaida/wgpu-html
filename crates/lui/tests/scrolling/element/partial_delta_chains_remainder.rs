use crate::support::{TEST_HEIGHT, TEST_WIDTH, red_quad_y, test_lui};

#[test]
fn partial_delta_consumed_by_inner_remainder_chains_to_outer() {
  let (mut lui, mut spy) = test_lui(
    r#"
    <html>
      <body>
        <div style="width: 180px; height: 150px; overflow-y: scroll; scrollbar-width: none">
          <div style="width: 180px; height: 80px; overflow-y: scroll; scrollbar-width: none">
            <div style="height: 200px; background: red"></div>
          </div>
          <div style="height: 300px; background: blue"></div>
        </div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(10.0, 10.0);

  // Scroll 100 of inner's 120 max first
  lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 100.0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let before = red_quad_y(&spy.take_last_list());

  // Scroll 60 more: inner consumes 20 (to hit 120 limit), outer gets remaining 40
  assert!(lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 60.0));
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let after = red_quad_y(&spy.take_last_list());

  // Red quad should have moved by the full 60 (20 inner + 40 outer)
  let moved = before - after;
  assert!(
    moved > 50.0,
    "both inner and outer should consume the delta: moved={moved}"
  );
}
