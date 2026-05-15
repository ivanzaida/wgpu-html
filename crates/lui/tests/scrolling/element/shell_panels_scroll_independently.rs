use crate::support::{RenderSpy, TEST_HEIGHT, TEST_WIDTH};

fn shell_lui() -> (lui::Lui, RenderSpy) {
  let spy = RenderSpy::default();
  let mut lui = lui::Lui::new();
  // The demo uses UA stylesheet — test with it
  #[cfg(feature = "ua_whatwg")]
  {
    // UA is auto-loaded in Lui::new()
  }
  lui.set_html(r#"
    <html><body>
      <style>
        * { margin: 0; padding: 0; border-width: 0; box-sizing: border-box; }
        html, body { width: 100%; height: 100%; }
        .shell { display: flex; width: 100%; height: 100%; overflow: hidden; }
        .sidebar { width: 80px; height: 100%; overflow-y: auto; flex-shrink: 0; }
        .main { flex: 1; height: 100%; overflow-y: auto; }
      </style>
      <div class="shell">
        <div class="sidebar">
          <div style="height: 800px; background: #ccc"></div>
        </div>
        <div class="main">
          <div style="height: 1200px; background: #eee"></div>
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
