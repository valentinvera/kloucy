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
            .bg(rgb(0x0d0f12))
            .size(px(520.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x27313d))
            .text_xl()
            .text_color(rgb(0xf5f7fa))
            .child(format!("{} Desktop", &self.product_name))
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(div().size_8().bg(rgb(0x7dd3fc)))
                    .child(div().size_8().bg(rgb(0xa7f3d0)))
                    .child(div().size_8().bg(rgb(0xfde68a)))
                    .child(div().size_8().bg(rgb(0xfca5a5))),
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
