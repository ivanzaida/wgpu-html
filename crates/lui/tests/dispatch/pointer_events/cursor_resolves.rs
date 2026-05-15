use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn cursor_resolves_from_hovered_element() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div id="link" style="width: 100px; height: 100px; cursor: pointer; background: red"></div>
    </body></html>"#,
  );

  assert_eq!(lui.current_cursor(), "auto", "default cursor before hover");

  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(lui.current_cursor(), "pointer", "cursor should be pointer over element");

  lui.set_cursor_position(150.0, 150.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(
    lui.current_cursor(),
    "auto",
    "cursor should revert to auto outside element"
  );
}

#[test]
fn cursor_inherits_from_parent() {
  let (mut lui, _spy) = test_lui(
    r#"<html><body>
      <div style="width: 100px; height: 100px; cursor: pointer">
        <div id="child" style="width: 100px; height: 100px; background: red"></div>
      </div>
    </body></html>"#,
  );

  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert_eq!(lui.current_cursor(), "pointer", "cursor should inherit from parent");
}
