use lui::wheel_delta_to_css;
use winit::{dpi::PhysicalPosition, event::MouseScrollDelta, keyboard::ModifiersState};

#[test]
fn wheel_delta_to_css_inverts_winit_y_sign() {
  let (_, dy) = wheel_delta_to_css(&MouseScrollDelta::LineDelta(0.0, 1.0), 1.0, ModifiersState::default());
  assert_eq!(dy, -40.0);

  let (_, dy_px) = wheel_delta_to_css(
    &MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 24.0)),
    2.0,
    ModifiersState::default(),
  );
  assert_eq!(dy_px, -12.0);
}
