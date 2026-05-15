mod support;

#[path = "dispatch/click/click_dispatches_to_target.rs"]
mod click_dispatches_to_target;

#[path = "dispatch/click/click_misses_outside_target.rs"]
mod click_misses_outside_target;

#[path = "dispatch/click/dblclick_fires.rs"]
mod dblclick_fires;

#[path = "dispatch/click/contextmenu_fires.rs"]
mod contextmenu_fires;

#[path = "dispatch/bubble/click_bubbles_to_parent.rs"]
mod click_bubbles_to_parent;

#[path = "dispatch/bubble/capture_fires_before_bubble.rs"]
mod capture_fires_before_bubble;

#[path = "dispatch/propagation/stop_propagation_halts_bubble.rs"]
mod stop_propagation_halts_bubble;

#[path = "dispatch/propagation/stop_immediate_halts_same_node.rs"]
mod stop_immediate_halts_same_node;

#[path = "dispatch/hover/hover_applies_style.rs"]
mod hover_applies_style;

#[path = "dispatch/hover/active_applies_style.rs"]
mod active_applies_style;

#[path = "dispatch/mouse/mousemove_fires.rs"]
mod mousemove_fires;

#[path = "dispatch/mouse/mouseenter_leave.rs"]
mod mouseenter_leave;

#[path = "dispatch/mouse/mouseover_bubbles.rs"]
mod mouseover_bubbles;

#[path = "dispatch/mouse/mouseout_fires.rs"]
mod mouseout_fires;

#[path = "dispatch/pointer_events/none_transparent_to_click.rs"]
mod pointer_events_none_transparent;

#[path = "dispatch/pointer_events/none_children_still_targetable.rs"]
mod pointer_events_none_children;

#[path = "dispatch/pointer_events/cursor_resolves.rs"]
mod cursor_resolves;

#[path = "dispatch/pointer/pointer_fires_before_mouse.rs"]
mod pointer_fires_before_mouse;

#[path = "dispatch/pointer/pointermove_fires_before_mousemove.rs"]
mod pointermove_fires_before_mousemove;

#[path = "dispatch/pointer/pointer_has_correct_fields.rs"]
mod pointer_has_correct_fields;

#[path = "dispatch/wheel/wheel_event_dispatches.rs"]
mod wheel_event_dispatches;

#[path = "dispatch/keyboard/keydown_dispatches.rs"]
mod keydown_dispatches;

#[path = "dispatch/focus/focus_style_applies.rs"]
mod focus_style_applies;
