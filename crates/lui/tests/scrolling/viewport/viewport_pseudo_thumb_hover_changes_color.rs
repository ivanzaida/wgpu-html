use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

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

#[test]
fn viewport_thumb_uses_nested_hover_and_later_base_rule() {
  let (mut lui, spy) = test_lui(
    r#"
    <html>
      <style>
        *::lui-scrollbar-thumb {
          &:hover {
            background-color: red;
          }
        }

        *::lui-scrollbar-thumb {
          background-color: blue;
        }
      </style>
      <body>
        <div style="height: 420px"></div>
      </body>
    </html>
    "#,
  );

  let blue = resolve_color("blue");
  let red = resolve_color("red");

  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();
  let blue_quads = find_quads_approx(&list, blue);
  assert!(
    !blue_quads.is_empty(),
    "expected blue viewport scrollbar thumb before hover, quads: {:?}",
    list.quads.iter().map(|q| q.color).collect::<Vec<_>>()
  );

  let thumb = blue_quads[0];
  lui.set_cursor_position(thumb.rect.x + thumb.rect.w * 0.5, thumb.rect.y + thumb.rect.h * 0.5);
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);
  assert!(
    lui.take_needs_redraw(),
    "hovering the viewport scrollbar thumb should schedule a follow-up redraw for the recascade"
  );
  lui.render_frame(TEST_WIDTH, TEST_HEIGHT, 1.0);

  let list = spy.take_last_list();
  let red_quads = find_quads_approx(&list, red);
  assert!(
    !red_quads.is_empty(),
    "expected red viewport scrollbar thumb on hover, quads: {:?}",
    list.quads.iter().map(|q| q.color).collect::<Vec<_>>()
  );
}
