use crate::support::{RenderSpy, TEST_HEIGHT, TEST_WIDTH};

fn shell_lui() -> (lui::Lui, RenderSpy) {
  let spy = RenderSpy::default();
  let mut lui = lui::Lui::new();
  let css = "* { margin: 0; padding: 0; border-width: 0; box-sizing: border-box; }";
  lui.set_stylesheets(&[lui_parse::parse_stylesheet(css).unwrap()]);
  lui.set_html(r#"
    <html style="width:100%;height:100%"><body style="width:100%;height:100%">
      <div style="display:flex; width:100%; height:100%; overflow:hidden">
        <div style="width:80px; height:100%; overflow-y:scroll; flex-shrink:0">
          <div style="height:800px; background:#ccc"></div>
        </div>
        <div style="flex:1; height:100%; overflow-y:scroll">
          <div style="height:1200px; background:#eee"></div>
        </div>
      </div>
    </body></html>
  "#);
  (lui, spy)
}

#[test]
fn main_panel_scrolls_independently_from_viewport() {
  let (mut lui, mut spy) = shell_lui();

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Place cursor in the main area (right half of 200px viewport)
  lui.set_cursor_position(150.0, 100.0);

  // Scroll down 50px
  let changed = lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 50.0);
  assert!(changed, "wheel should scroll something");

  // The viewport should NOT have scrolled — the main panel should have absorbed it
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let vp = lui.viewport_scroll();
  assert!(
    vp.1.abs() < 0.5,
    "viewport should not scroll when main panel has overflow-y:auto; viewport_scroll_y={}",
    vp.1
  );
}

#[test]
fn sidebar_scrolls_independently_from_main() {
  let (mut lui, mut spy) = shell_lui();

  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Place cursor in the sidebar (left side, x=40)
  lui.set_cursor_position(40.0, 100.0);

  // Scroll down 50px
  let changed = lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 50.0);
  assert!(changed, "wheel should scroll sidebar");

  // Move cursor to main area
  lui.set_cursor_position(150.0, 100.0);

  // Scroll down 30px
  let changed = lui.handle_wheel(TEST_WIDTH, TEST_HEIGHT, 1.0, 0.0, 30.0);
  assert!(changed, "wheel should scroll main");

  // Viewport should still not have scrolled
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let vp = lui.viewport_scroll();
  assert!(
    vp.1.abs() < 0.5,
    "viewport should not scroll; viewport_scroll_y={}",
    vp.1
  );
}
