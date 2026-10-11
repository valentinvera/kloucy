use gpui::{
    App, Application, Bounds, Context, FontWeight, IntoElement, Render, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Selectable as _, Theme, ThemeMode,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Workspaces,
    Agents,
    Inbox,
    Sessions,
    Reviews,
}

const SECTIONS: [Section; 5] = [
    Section::Workspaces,
    Section::Agents,
    Section::Inbox,
    Section::Sessions,
    Section::Reviews,
];

impl Section {
    fn id(self) -> &'static str {
        match self {
            Section::Workspaces => "nav-workspaces",
            Section::Agents => "nav-agents",
            Section::Inbox => "nav-inbox",
            Section::Sessions => "nav-sessions",
            Section::Reviews => "nav-reviews",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Section::Workspaces => "Workspaces",
            Section::Agents => "Agents",
            Section::Inbox => "Inbox",
            Section::Sessions => "Sessions",
            Section::Reviews => "Reviews",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Section::Workspaces => "Local projects Kloucy can supervise.",
            Section::Agents => "Local agents available to run tasks.",
            Section::Inbox => "Tasks captured before they run.",
            Section::Sessions => "Running, waiting, and finished agent work.",
            Section::Reviews => "Changed files and artifacts awaiting a decision.",
        }
    }

    fn empty(self) -> &'static str {
        match self {
            Section::Workspaces => "Open your first workspace.",
            Section::Agents => "No agents configured yet.",
            Section::Inbox => "No captured tasks yet.",
            Section::Sessions => "No sessions yet.",
            Section::Reviews => "Nothing to review yet.",
        }
    }
}

struct DesktopShell {
    section: Section,
}

impl DesktopShell {
    fn new() -> Self {
        Self {
            section: Section::Workspaces,
        }
    }

    fn render_nav(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut nav = v_flex()
            .w(px(240.0))
            .h_full()
            .flex_shrink_0()
            .gap_1()
            .p_3()
            .border_r_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground);

        for section in SECTIONS {
            nav = nav.child(
                Button::new(section.id())
                    .ghost()
                    .w_full()
                    .justify_start()
                    .label(section.label())
                    .selected(self.section == section)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.section = section;
                        cx.notify();
                    })),
            );
        }

        nav
    }

    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .p_8()
            .gap_2()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.section.label()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.section.description()),
            )
            .child(
                div()
                    .mt_6()
                    .pt_6()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .text_sm()
                    .child(self.section.empty()),
            )
    }
}

impl Render for DesktopShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .size_full()
            .child(self.render_nav(cx))
            .child(self.render_content(cx))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Dark, None, cx);

        let bounds = Bounds::centered(None, size(px(1120.0), px(720.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|_| DesktopShell::new());
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open Kloucy desktop window");
    });
}
