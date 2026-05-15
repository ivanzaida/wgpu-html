use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

fn has_blue_quad(list: &lui::display_list::DisplayList) -> bool {
  list.quads.iter().any(|q| q.color[2] > 0.9 && q.color[0] < 0.2 && q.color[1] < 0.2)
}

fn has_red_quad(list: &lui::display_list::DisplayList) -> bool {
  list.quads.iter().any(|q| q.color[0] > 0.9 && q.color[1] < 0.2 && q.color[2] < 0.2)
}

#[test]
fn focus_pseudo_applies_style_on_click() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body><div id="box" tabindex="0"></div></body></html>"#,
  );
  lui.set_stylesheets(&[lui_parse::parse_stylesheet(
    "* { margin: 0; padding: 0; border-width: 0; }
     #box { width: 100px; height: 100px; background: red; }
     #box:focus { background: blue; }",
  ).unwrap()]);

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert!(has_red_quad(&spy.take_last_list()), "should be red before focus");

  // Click to focus
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  assert!(has_blue_quad(&list), "should be blue when focused");
  assert!(!has_red_quad(&list), "should not be red when focused");
}

#[test]
fn focus_clears_on_click_elsewhere() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div id="box" tabindex="0"></div>
      <div id="other" style="width: 100px; height: 100px; background: green"></div>
    </body></html>"#,
  );
  lui.set_stylesheets(&[lui_parse::parse_stylesheet(
    "* { margin: 0; padding: 0; border-width: 0; }
     #box { width: 100px; height: 100px; background: red; }
     #box:focus { background: blue; }",
  ).unwrap()]);

  // Focus the box
  lui.set_cursor_position(50.0, 50.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert!(has_blue_quad(&spy.take_last_list()), "should be blue when focused");

  // Click elsewhere to blur
  lui.set_cursor_position(50.0, 150.0);
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.handle_mouse_release(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  assert!(has_red_quad(&list), "should revert to red after blur");
  assert!(!has_blue_quad(&list), "should not be blue after blur");
}
