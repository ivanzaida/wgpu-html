use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

#[test]
fn focused_input_renders_value_as_glyphs() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="hello" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  // Focus the input
  lui.set_cursor_position(100.0, 15.0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  assert!(
    !list.glyphs.is_empty(),
    "focused input with value should render glyph quads"
  );
}

#[test]
fn focused_input_renders_caret_quad() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input id="field" type="text" value="" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  lui.set_cursor_position(100.0, 15.0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let has_thin_quad = list.quads.iter().any(|q| q.rect.w < 3.0 && q.rect.h > 10.0);
  assert!(
    has_thin_quad,
    "focused empty input should render a caret quad (thin vertical line)"
  );
}

#[test]
fn unfocused_input_with_value_renders_glyphs() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input type="text" value="world" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  assert!(
    !list.glyphs.is_empty(),
    "unfocused input with value should still render glyph quads"
  );
}

#[test]
fn placeholder_renders_when_value_empty() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <input type="text" value="" placeholder="Type here" style="width: 200px; height: 30px" />
    </body></html>"#,
  );

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  assert!(
    !list.glyphs.is_empty(),
    "input with placeholder and empty value should render placeholder glyphs"
  );
}
