use gpui::{
    App, Application, Bounds, Context, IntoElement, Render, SharedString, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};

struct DesktopShell {
    product_name: SharedString,
}

impl Render for DesktopShell {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x000000))
            .size(px(520.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x2d2e32))
            .font_family("Geist")
            .text_xl()
            .text_color(rgb(0xfdfdfd))
            .child(format!("{} Desktop", &self.product_name))
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(div().size_8().bg(rgb(0x70b8ff)))
                    .child(div().size_8().bg(rgb(0x46fea5)))
                    .child(div().size_8().bg(rgb(0xffca16)))
                    .child(div().size_8().bg(rgb(0xff9592))),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| DesktopShell {
                    product_name: "Kloucy".into(),
                })
            },
        )
        .expect("failed to open Kloucy desktop window");
    });
}
