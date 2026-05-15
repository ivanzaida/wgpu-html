use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

fn has_green_quad(list: &lui::display_list::DisplayList) -> bool {
  list
    .quads
    .iter()
    .any(|q| q.color[1] > 0.1 && q.color[0] < 0.05 && q.color[2] < 0.05)
}

fn has_red_quad(list: &lui::display_list::DisplayList) -> bool {
  list
    .quads
    .iter()
    .any(|q| q.color[0] > 0.9 && q.color[1] < 0.2 && q.color[2] < 0.2)
}

#[test]
fn active_applies_style_on_mousedown() {
  let (mut lui, spy) = test_lui(r#"<html><body><div id="box"></div></body></html>"#);
  lui.set_stylesheets(&[lui_parse::parse_stylesheet(
    "* { margin: 0; padding: 0; border-width: 0; }
     #box { width: 100px; height: 100px; background: red; }
     #box:active { background: green; }",
  )
  .unwrap()]);

  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert!(has_red_quad(&spy.take_last_list()), "should be red before mousedown");

  // Mousedown sets active_path
  lui.handle_mouse_down(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  assert!(has_green_quad(&list), "should be green during active");

  // Mouseup clears active_path
  lui.handle_mouse_up(TEST_WIDTH, TEST_HEIGHT, 1.0, 0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  assert!(has_red_quad(&list), "should revert to red after mouseup");
  assert!(!has_green_quad(&list), "should not be green after mouseup");
}
