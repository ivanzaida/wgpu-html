//! Field-by-field cascade merge for `Style`. Any `Some` field on `rhs`
//! overwrites the corresponding `lhs` field.

use wgpu_html_models::Style;

macro_rules! merge_field {
    ($lhs:ident, $rhs:ident, $field:ident) => {
        if $rhs.$field.is_some() {
            $lhs.$field = $rhs.$field.clone();
        }
    };
}

pub fn merge(lhs: &mut Style, rhs: &Style) {
    merge_field!(lhs, rhs, display);
    merge_field!(lhs, rhs, position);
    merge_field!(lhs, rhs, top);
    merge_field!(lhs, rhs, right);
    merge_field!(lhs, rhs, bottom);
    merge_field!(lhs, rhs, left);
    merge_field!(lhs, rhs, width);
    merge_field!(lhs, rhs, height);
    merge_field!(lhs, rhs, min_width);
    merge_field!(lhs, rhs, min_height);
    merge_field!(lhs, rhs, max_width);
    merge_field!(lhs, rhs, max_height);
    merge_field!(lhs, rhs, margin);
    merge_field!(lhs, rhs, margin_top);
    merge_field!(lhs, rhs, margin_right);
    merge_field!(lhs, rhs, margin_bottom);
    merge_field!(lhs, rhs, margin_left);
    merge_field!(lhs, rhs, padding);
    merge_field!(lhs, rhs, padding_top);
    merge_field!(lhs, rhs, padding_right);
    merge_field!(lhs, rhs, padding_bottom);
    merge_field!(lhs, rhs, padding_left);
    merge_field!(lhs, rhs, color);
    merge_field!(lhs, rhs, background);
    merge_field!(lhs, rhs, background_color);
    merge_field!(lhs, rhs, background_image);
    merge_field!(lhs, rhs, background_size);
    merge_field!(lhs, rhs, background_position);
    merge_field!(lhs, rhs, background_repeat);
    merge_field!(lhs, rhs, border);
    merge_field!(lhs, rhs, border_width);
    merge_field!(lhs, rhs, border_style);
    merge_field!(lhs, rhs, border_color);
    merge_field!(lhs, rhs, border_radius);
    merge_field!(lhs, rhs, font_family);
    merge_field!(lhs, rhs, font_size);
    merge_field!(lhs, rhs, font_weight);
    merge_field!(lhs, rhs, font_style);
    merge_field!(lhs, rhs, line_height);
    merge_field!(lhs, rhs, letter_spacing);
    merge_field!(lhs, rhs, text_align);
    merge_field!(lhs, rhs, text_decoration);
    merge_field!(lhs, rhs, text_transform);
    merge_field!(lhs, rhs, white_space);
    merge_field!(lhs, rhs, overflow);
    merge_field!(lhs, rhs, overflow_x);
    merge_field!(lhs, rhs, overflow_y);
    merge_field!(lhs, rhs, opacity);
    merge_field!(lhs, rhs, visibility);
    merge_field!(lhs, rhs, z_index);
    merge_field!(lhs, rhs, flex_direction);
    merge_field!(lhs, rhs, flex_wrap);
    merge_field!(lhs, rhs, justify_content);
    merge_field!(lhs, rhs, align_items);
    merge_field!(lhs, rhs, align_content);
    merge_field!(lhs, rhs, gap);
    merge_field!(lhs, rhs, row_gap);
    merge_field!(lhs, rhs, column_gap);
    merge_field!(lhs, rhs, flex);
    merge_field!(lhs, rhs, flex_grow);
    merge_field!(lhs, rhs, flex_shrink);
    merge_field!(lhs, rhs, flex_basis);
    merge_field!(lhs, rhs, grid_template_columns);
    merge_field!(lhs, rhs, grid_template_rows);
    merge_field!(lhs, rhs, grid_column);
    merge_field!(lhs, rhs, grid_row);
    merge_field!(lhs, rhs, transform);
    merge_field!(lhs, rhs, transform_origin);
    merge_field!(lhs, rhs, transition);
    merge_field!(lhs, rhs, animation);
    merge_field!(lhs, rhs, cursor);
    merge_field!(lhs, rhs, pointer_events);
    merge_field!(lhs, rhs, user_select);
    merge_field!(lhs, rhs, box_shadow);
    merge_field!(lhs, rhs, box_sizing);
}
