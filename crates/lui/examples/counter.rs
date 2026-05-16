use lui::{Lui, WgpuRenderer, WinitHarness};
use lui::live::builder::el::*;
use lui::live::{Ctx, Runtime};

fn counter_button(
  ctx: &Ctx,
  (label, color, on_click): (&str, &str, impl Fn(&lui::lui_core::events::MouseEventInit) + Send + Sync + 'static),
) -> El {
  ctx.on_mounted(|| {});

  button()
    .text(label)
    .style(&format!(
      "padding: 10px 20px; font-size: 15px; font-weight: 700; font-family: sans-serif; \
       color: #f8fafc; background: {}; border: 1px solid rgba(255,255,255,0.15); \
       border-radius: 6px; box-sizing: border-box",
      color
    ))
    .on_click(on_click)
}

fn counter(ctx: &Ctx) -> El {
  let count = ctx.signal(0i32);

  ctx.on_effect({
    let count = count.clone();
    move || {
      println!("Count changed: {}", count.get());
    }
  });

  let val = count.get();
  let count_color = if val > 0 { "#16a34a" } else if val < 0 { "#dc2626" } else { "#8ecae6" };

  div()
    .style(
      "min-height: 100%; display: flex; align-items: center; justify-content: center; \
       background: #12151f; font-family: sans-serif",
    )
    .child(
      div()
        .style(
          "padding: 32px 40px; background: #181d2b; border: 1px solid #27324a; \
           border-radius: 12px; display: flex; flex-direction: column; align-items: center; gap: 24px",
        )
        .children([
          div()
            .style("color: #8ecae6; font-size: 13px; font-weight: 700; text-transform: uppercase")
            .child(text("lui-live counter")),
          div()
            .style(&format!(
              "font-size: 48px; font-weight: 700; color: {}; font-family: sans-serif",
              count_color
            ))
            .child(text(&format!("{}", val))),
          div().style("display: flex; gap: 10px").children([
            ctx.component(counter_button, ("-1", "#7c3aed", {
              let count = count.clone();
              move |_| count.update(|n| *n -= 1)
            })),
            ctx.component(counter_button, ("Reset", "#475569", {
              let count = count.clone();
              move |_| count.set(0)
            })),
            ctx.component(counter_button, ("+1", "#2563eb", {
              let count = count.clone();
              move |_| count.update(|n| *n += 1)
            })),
          ]),
        ]),
    )
}

fn main() {
  let mut lui = Lui::new();
  lui.set_html(r#"<html><body style="margin:0;width:100%;height:100%"><div id="app" style="width:100%;height:100%"></div></body></html>"#);

  let mut rt = Runtime::new("#app", counter);
  rt.render(&mut lui);

  let harness = WinitHarness::new(400, 300, "lui-live counter");
  let renderer = WgpuRenderer::new();

  harness.run_with(lui, renderer, move |hctx| {
    rt.process(&mut hctx.lui);
  });
}
