use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn nested_scroll_chains_through_element_to_viewport() {
  let (mut lui, _spy) = test_lui(
    r#"
    <html>
      <body>
        <div style="width: 180px; height: 100px; overflow-y: scroll; scrollbar-width: none">
          <div style="height: 200px; background: red"></div>
        </div>
        <div style="height: 500px; background: blue"></div>
      </body>
    </html>
    "#,
  );

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.set_cursor_position(10.0, 10.0);

  // Scroll container to its limit (max = 200 - 100 = 100)
  lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 100.0);

  // Scroll more — remaining delta should chain to viewport
  assert!(
    lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 50.0),
    "remaining delta should chain to viewport when element is at limit"
  );
}
