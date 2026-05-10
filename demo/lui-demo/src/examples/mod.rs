use lui_tree::Tree;

mod bg_image;
mod click_demo;
mod custom_elements;
mod css_vars;
mod devtools_demo;
mod devtools_test;
mod events_test;
mod flex_browser_like;
mod flex_grow;
mod form;
mod forms_demo;
mod gif;
mod gradient_test;
mod grid;
mod hello_text;
mod icons_demo;
mod img_test;
mod map_editor;
mod media_queries;
mod overflow;
mod p0_demo;
mod position_overlay;
mod resize;
mod scroll_test;
mod scrollbar_test;
mod styled_inputs;
mod svg_test;
mod text_selection;
mod text_shrink;
mod text_wrapping;
mod styled_pickers;
mod table;

pub fn get_example_tree(name: &str) -> Option<Tree> {
  match name {
    "bg_image" => Some(bg_image::build()),
    "click_demo" => Some(click_demo::build()),
    "custom_elements" => Some(custom_elements::build()),
    "css_vars" => Some(css_vars::build()),
    "devtools_demo" => Some(devtools_demo::build()),
    "devtools_test" => Some(devtools_test::build()),
    "events_test" => Some(events_test::build()),
    "flex_browser_like" => Some(flex_browser_like::build()),
    "flex_grow" => Some(flex_grow::build()),
    "form" => Some(form::build()),
    "forms_demo" => Some(forms_demo::build()),
    "gif" => Some(gif::build()),
    "gradient_test" => Some(gradient_test::build()),
    "grid" => Some(grid::build()),
    "hello_text" => Some(hello_text::build()),
    "icons_demo" => Some(icons_demo::build()),
    "img_test" => Some(img_test::build()),
    "map_editor" => Some(map_editor::build()),
    "media_queries" => Some(media_queries::build()),
    "overflow" => Some(overflow::build()),
    "p0_demo" => Some(p0_demo::build()),
    "position_overlay" => Some(position_overlay::build()),
    "resize" => Some(resize::build()),
    "scroll_test" => Some(scroll_test::build()),
    "scrollbar_test" => Some(scrollbar_test::build()),
    "styled_inputs" => Some(styled_inputs::build()),
    "svg_test" => Some(svg_test::build()),
    "text_selection" => Some(text_selection::build()),
    "text_shrink" => Some(text_shrink::build()),
    "text_wrapping" => Some(text_wrapping::build()),
    "styled_pickers" => Some(styled_pickers::build()),
    "table" => Some(table::build()),
    _ => None,
  }
}

pub fn list_examples() -> Vec<&'static str> {
  vec![
    "bg_image",
    "click_demo",
    "custom_elements",
    "css_vars",
    "devtools_demo",
    "devtools_test",
    "events_test",
    "flex_browser_like",
    "flex_grow",
    "form",
    "forms_demo",
    "gif",
    "gradient_test",
    "grid",
    "hello_text",
    "icons_demo",
    "img_test",
    "map_editor",
    "media_queries",
    "overflow",
    "p0_demo",
    "position_overlay",
    "resize",
    "scroll_test",
    "scrollbar_test",
    "styled_inputs",
    "svg_test",
    "text_selection",
    "text_shrink",
    "text_wrapping",
    "table"
  ]
}
