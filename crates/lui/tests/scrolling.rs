mod support;

#[path = "scrolling/element/wheel_scroll_persists_between_frames.rs"]
mod wheel_scroll_persists_between_frames;

#[path = "scrolling/paint/uses_scrollbar_color_thumb_and_track.rs"]
mod uses_scrollbar_color_thumb_and_track;

#[path = "scrolling/paint/uses_scrollbar_color_hex_pair.rs"]
mod uses_scrollbar_color_hex_pair;

#[path = "scrolling/paint/uses_scrollbar_color_function_pair.rs"]
mod uses_scrollbar_color_function_pair;

#[path = "scrolling/paint/uses_scrollbar_width_for_track_thickness.rs"]
mod uses_scrollbar_width_for_track_thickness;

#[path = "scrolling/viewport/wheel_scroll_falls_back_to_viewport_translation.rs"]
mod wheel_scroll_falls_back_to_viewport_translation;

#[path = "scrolling/viewport/paints_vertical_scrollbar_when_document_overflows_height.rs"]
mod paints_vertical_scrollbar_when_document_overflows_height;

#[path = "scrolling/viewport/paints_horizontal_scrollbar_when_document_overflows_width.rs"]
mod paints_horizontal_scrollbar_when_document_overflows_width;

#[cfg(feature = "winit")]
#[path = "scrolling/winit/wheel_delta_to_css_inverts_winit_y_sign.rs"]
mod wheel_delta_to_css_inverts_winit_y_sign;

#[cfg(feature = "winit")]
#[path = "scrolling/winit/shift_wheel_maps_vertical_delta_to_horizontal.rs"]
mod shift_wheel_maps_vertical_delta_to_horizontal;

#[cfg(feature = "ua_whatwg")]
#[path = "scrolling/ua/applies_default_scrollbar_style_from_ua.rs"]
mod applies_default_scrollbar_style_from_ua;
