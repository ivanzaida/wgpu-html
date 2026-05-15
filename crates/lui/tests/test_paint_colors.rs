mod support;

const SHELL_HTML: &str = include_str!("../../../demo/lui-demo/html/shell.html");

#[test]
fn no_viewport_scrollbar_when_shell_fits() {
    let spy = support::RenderSpy::default();
    let mut lui = lui::Lui::new();
    lui.set_html(SHELL_HTML);
    let mut spy = spy;
    lui.render_frame(&mut spy, 800, 600, 1.0);
    let list = spy.take_last_list();

    // Check for viewport scrollbar quads at the right/bottom edge of viewport
    let viewport_scrollbar_quads: Vec<_> = list
        .quads
        .iter()
        .filter(|q| {
            // Right-edge scrollbar: x > 770
            (q.rect.x > 770.0 && q.rect.h > 100.0)
            // Bottom-edge scrollbar: y > 570
            || (q.rect.y > 570.0 && q.rect.w > 100.0)
        })
        .collect();

    for q in &viewport_scrollbar_quads {
        println!(
            "viewport scrollbar quad: rect=({:.0},{:.0},{:.0},{:.0}) color=[{:.3},{:.3},{:.3},{:.3}]",
            q.rect.x, q.rect.y, q.rect.w, q.rect.h,
            q.color[0], q.color[1], q.color[2], q.color[3],
        );
    }

    assert!(
        viewport_scrollbar_quads.is_empty(),
        "expected no viewport scrollbars when shell has overflow:hidden and height:100%, found {} quads",
        viewport_scrollbar_quads.len(),
    );
}
