use wgpu_html_models::common::css_enums::*;
use wgpu_html_ui::style::{self, px,};
use wgpu_html_ui::{el, App, Component, Ctx, El, ShouldRender};

// ── Counter Component ───────────────────────────────────────────────────────

struct Counter {
    count: i32,
}

#[derive(Clone)]
struct CounterProps {
    label: String,
}

#[derive(Clone)]
enum CounterMsg {
    Inc,
    Dec,
    Reset,
}

impl Component for Counter {
    type Props = CounterProps;
    type Msg = CounterMsg;
    type Env = ();

    fn scope() -> &'static str { "ctr" }

    fn styles() -> style::Stylesheet {
        style::sheet([
            style::rule(".root")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(AlignItems::Center)
                .gap(px(12))
                .padding(px(24))
                .background_color("#292A2D")
                .border_radius(px(12)),

            style::rule(".label")
                .font_size(px(12))
                .font_weight(FontWeight::Weight(500))
                .color("#9AA0A6")
                .white_space(WhiteSpace::Nowrap),

            style::rule(".value")
                .font_size(px(36))
                .font_weight(FontWeight::Bold)
                .color("#E8EAED")
                .white_space(WhiteSpace::Nowrap)
                .text_align(TextAlign::Center),

            style::rule(".controls")
                .display(Display::Flex)
                .gap(px(8))
                .align_items(AlignItems::Center),

            style::rule(".btn")
                .display(Display::Flex)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .width(px(36))
                .height(px(36))
                .border_radius(px(8))
                .background_color("#35363A")
                .color("#E8EAED")
                .font_size(px(18))
                .white_space(WhiteSpace::Nowrap)
                .cursor(Cursor::Pointer),

            style::rule(".btn:hover")
                .background_color("#4A4B4F"),

            style::rule(".btn-reset")
                .width(px(64))
                .font_size(px(12))
                .font_weight(FontWeight::Weight(500)),
        ])
    }

    fn create(_props: &CounterProps) -> Self {
        Counter { count: 0 }
    }

    fn update(&mut self, msg: CounterMsg, _props: &CounterProps) -> ShouldRender {
        match msg {
            CounterMsg::Inc => self.count += 1,
            CounterMsg::Dec => self.count -= 1,
            CounterMsg::Reset => self.count = 0,
        }
        ShouldRender::Yes
    }

    fn view(&self, props: &CounterProps, ctx: &Ctx<CounterMsg>, _env: &()) -> El {
        el::div().class(ctx.scoped("root")).children([
            el::div().class(ctx.scoped("label")).text(&props.label),
            el::div()
                .class(ctx.scoped("value"))
                .text(self.count.to_string()),
            el::div().class(ctx.scoped("controls")).children([
                el::div()
                    .class(ctx.scoped("btn"))
                    .text("\u{2212}")
                    .on_click_cb(ctx.msg(CounterMsg::Dec)),
                el::div()
                    .class(format!("{} {}", ctx.scoped("btn"), ctx.scoped("btn-reset")))
                    .text("Reset")
                    .on_click_cb(ctx.msg(CounterMsg::Reset)),
                el::div()
                    .class(ctx.scoped("btn"))
                    .text("+")
                    .on_click_cb(ctx.msg(CounterMsg::Inc)),
            ]),
        ])
    }
}

// ── Root App Component ──────────────────────────────────────────────────────

struct DemoApp;

#[derive(Clone)]
struct DemoProps;

#[derive(Clone)]
enum DemoMsg {}

impl Component for DemoApp {
    type Props = DemoProps;
    type Msg = DemoMsg;
    type Env = ();

    fn scope() -> &'static str { "app" }

    fn styles() -> style::Stylesheet {
        style::sheet([
            style::rule(".root")
                .display(Display::Flex)
                .flex_direction(FlexDirection::Column)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center)
                .flex_grow(1.0)
                .background_color("#1a1a1a")
                .font_family("sans-serif"),

            style::rule(".title")
                .font_size(px(20))
                .font_weight(FontWeight::Weight(600))
                .color("#8AB4F8")
                .margin_bottom(px(40))
                .white_space(WhiteSpace::Nowrap),

            style::rule(".counters")
                .display(Display::Flex)
                .gap(px(24)),
        ])
    }

    fn create(_props: &DemoProps) -> Self {
        DemoApp
    }

    fn update(&mut self, _msg: DemoMsg, _props: &DemoProps) -> ShouldRender {
        ShouldRender::No
    }

    fn view(&self, _props: &DemoProps, ctx: &Ctx<DemoMsg>, _env: &()) -> El {
        el::div().class(ctx.scoped("root")).children([
            el::div()
                .class(ctx.scoped("title"))
                .text("wgpu-html-ui Demo"),
            el::div().class(ctx.scoped("counters")).children([
                ctx.child::<Counter>(CounterProps {
                    label: "Clicks".into(),
                }),
                ctx.child::<Counter>(CounterProps {
                    label: "Score".into(),
                }),
            ]),
        ])
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
    App::new::<DemoApp>(DemoProps)
        .title("wgpu-html-ui demo")
        .size(800, 500)
        .stylesheet("html, body { height: 100%; margin: 0; background: #1a1a1a; } body { display: flex; flex-direction: column; }")
        .with_secondary(|tree| {
            Box::new(wgpu_html_devtools::Devtools::attach(tree, false))
        })
        .run()
        .unwrap();
}
