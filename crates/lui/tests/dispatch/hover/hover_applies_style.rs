use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

fn has_blue_quad(list: &lui::display_list::DisplayList) -> bool {
  list.quads.iter().any(|q| q.color[2] > 0.9 && q.color[0] < 0.2 && q.color[1] < 0.2)
}

fn has_red_quad(list: &lui::display_list::DisplayList) -> bool {
  list.quads.iter().any(|q| q.color[0] > 0.9 && q.color[1] < 0.2 && q.color[2] < 0.2)
}

#[test]
fn hover_applies_style_when_cursor_over_element() {
  let (mut lui, spy) = test_lui(
    r#"<html><body><div id="box"></div></body></html>"#,
  );
  lui.set_stylesheets(&[lui_parse::parse_stylesheet(
    "* { margin: 0; padding: 0; border-width: 0; }
     #box { width: 100px; height: 100px; background: red; }
     #box:hover { background: blue; }",
  ).unwrap()]);

  // Frame 1: no cursor → red
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  assert!(has_red_quad(&list), "should be red without hover");
  assert!(!has_blue_quad(&list), "should not be blue without hover");

  // Move cursor over element
  lui.set_cursor_position(50.0, 50.0);

  // Frame 2: hover_path is computed from hit-test (one-frame lag)
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Frame 3: cascade now uses the hover_path set in frame 2
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  assert!(has_blue_quad(&list), "should be blue with hover");
  assert!(!has_red_quad(&list), "should not be red with hover");
}

#[test]
fn hover_clears_when_cursor_leaves() {
  let (mut lui, spy) = test_lui(
    r#"<html><body><div id="box"></div></body></html>"#,
  );
  lui.set_stylesheets(&[lui_parse::parse_stylesheet(
    "* { margin: 0; padding: 0; border-width: 0; }
     #box { width: 100px; height: 100px; background: red; }
     #box:hover { background: blue; }",
  ).unwrap()]);

  // Establish hover
  lui.set_cursor_position(50.0, 50.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Cursor leaves
  lui.clear_cursor_position();
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  assert!(has_red_quad(&list), "should revert to red after cursor leaves");
  assert!(!has_blue_quad(&list), "should not be blue after cursor leaves");
}
