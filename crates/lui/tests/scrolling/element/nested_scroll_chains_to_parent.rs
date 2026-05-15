use crate::support::{TEST_HEIGHT, TEST_WIDTH, red_quad_y, test_lui};

#[test]
fn nested_scroll_chains_to_parent_when_inner_hits_limit() {
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

  // Scroll inner to its limit (max_scroll = 200 - 80 = 120)
  lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 120.0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let after_inner_maxed = red_quad_y(&spy.take_last_list());

  // Scroll more — should chain to outer container
  assert!(
    lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 50.0),
    "scroll should chain to outer container when inner is at limit"
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let after_chain = red_quad_y(&spy.take_last_list());

  assert!(
    after_chain < after_inner_maxed - 40.0,
    "outer container should have scrolled: after_inner_maxed={after_inner_maxed} after_chain={after_chain}"
  );
}
