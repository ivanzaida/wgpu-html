use crate::support::{RenderSpy, TEST_HEIGHT, TEST_WIDTH};

fn resolve_color(s: &str) -> [f32; 4] {
  let v = lui_parse::parse_value(s).unwrap();
  lui::paint::color::resolve_color(Some(&v)).unwrap()
}

fn find_quads_approx(list: &lui::display_list::DisplayList, color: [f32; 4]) -> Vec<lui::display_list::Quad> {
  list
    .quads
    .iter()
    .copied()
    .filter(|q| q.color.iter().zip(color.iter()).all(|(a, b)| (a - b).abs() < 0.02))
    .collect()
}

fn scroller_lui(css: &str) -> (lui::Lui, RenderSpy) {
  let spy = RenderSpy::default();
  let mut lui = lui::Lui::new();
  let reset = "* { margin: 0; padding: 0; border-width: 0; }";
  let combined = format!("{reset}\n{css}");
  lui.set_stylesheets(&[lui_parse::parse_stylesheet(&combined).unwrap()]);
  lui.set_html(
    r#"<html><body>
      <div id="scroller" style="width: 120px; height: 100px; overflow-y: scroll; scrollbar-width: auto;">
        <div style="height: 400px"></div>
      </div>
    </body></html>"#,
  );
  (lui, spy)
}

#[test]
fn thumb_hover_pseudo_changes_background_color() {
  let (mut lui, mut spy) = scroller_lui(
    r#"
      #scroller::lui-scrollbar-thumb {
        background-color: green;
        &:hover { background-color: red; }
      }
      #scroller::lui-scrollbar-track { background-color: blue; }
    "#,
  );

  let green = resolve_color("green");
  let red = resolve_color("red");

  // Frame 1: no hover — thumb should be green
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let green_quads = find_quads_approx(&list, green);
  assert!(
    !green_quads.is_empty(),
    "expected green thumb quad without hover, quads: {:?}",
    list.quads.iter().map(|q| q.color).collect::<Vec<_>>()
  );

  // Identify thumb position from the green quad
  let thumb = green_quads[0];
  let thumb_cx = thumb.rect.x + thumb.rect.w * 0.5;
  let thumb_cy = thumb.rect.y + thumb.rect.h * 0.5;

  // Frame 2: move cursor over the thumb — computes scrollbar_hover
  lui.set_cursor_position(thumb_cx, thumb_cy);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Frame 3: scrollbar_hover feeds into cascade — thumb should now be red
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let red_quads = find_quads_approx(&list, red);
  assert!(
    !red_quads.is_empty(),
    "expected red thumb quad after hover, quads: {:?}",
    list.quads.iter().map(|q| q.color).collect::<Vec<_>>()
  );
}

#[test]
fn track_hover_does_not_affect_thumb_color() {
  let (mut lui, mut spy) = scroller_lui(
    r#"
      #scroller::lui-scrollbar-thumb {
        background-color: green;
        &:hover { background-color: red; }
      }
      #scroller::lui-scrollbar-track { background-color: blue; }
    "#,
  );

  let green = resolve_color("green");
  let red = resolve_color("red");
  let blue = resolve_color("blue");

  // Frame 1: render to get layout
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let track_quads = find_quads_approx(&list, blue);
  assert!(!track_quads.is_empty(), "expected blue track quad");
  let track = track_quads[0];

  // Move cursor to the bottom of the track (well below the thumb)
  let track_cx = track.rect.x + track.rect.w * 0.5;
  let track_cy = track.rect.y + track.rect.h - 2.0;
  lui.set_cursor_position(track_cx, track_cy);
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);

  // Frame 3: hover state feeds into cascade — thumb should still be green
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let green_quads = find_quads_approx(&list, green);
  let red_quads = find_quads_approx(&list, red);

  assert!(
    !green_quads.is_empty(),
    "thumb should remain green when hovering track, not thumb"
  );
  assert!(red_quads.is_empty(), "thumb should NOT be red when hovering track");
}
