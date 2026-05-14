use lui_cascade::{cascade::InteractionState, media::MediaContext};
use lui_layout::engine::layout_tree;

use crate::helpers::{find_by_tag, flex_lt};

#[test]
fn scrollbar_gutter_stable_reserves_height_for_horizontal_auto_scrollbar() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-x:auto; scrollbar-gutter:stable">
            <div style="width:50px; height:50px">fits</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();

  assert!(
    (container.content.height - 85.0).abs() < 1.0,
    "stable gutter should reserve one 15px horizontal gutter, got {}",
    container.content.height
  );
}
