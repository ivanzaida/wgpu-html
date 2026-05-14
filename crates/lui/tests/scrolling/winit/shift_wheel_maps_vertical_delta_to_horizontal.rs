use lui::wheel_delta_to_css;
use winit::{event::MouseScrollDelta, keyboard::ModifiersState};

#[test]
fn shift_wheel_maps_vertical_delta_to_horizontal() {
  let (dx, dy) = wheel_delta_to_css(&MouseScrollDelta::LineDelta(0.0, 1.0), 1.0, ModifiersState::SHIFT);
  assert_eq!(dx, -40.0);
  assert_eq!(dy, 0.0);
}
