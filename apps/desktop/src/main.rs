mod api;

use gpui::{
    App, Application, Bounds, Context, FontWeight, Hsla, IntoElement, Render, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Theme, ThemeMode, h_flex,
    sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use crate::api::SessionStatus;

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
    session: SessionStatus,
}

impl DesktopShell {
    fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            let base_url = api::base_url();
            let status = cx
                .background_executor()
                .spawn(async move { api::fetch_session(&base_url) })
                .await;

            this.update(cx, |this, cx| {
                this.session = status;
                cx.notify();
            })
            .ok();
        })
        .detach();

        Self {
            section: Section::Workspaces,
            session: SessionStatus::Checking,
        }
    }

    fn render_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (label, color): (String, Hsla) = match &self.session {
            SessionStatus::Checking => ("Checking API…".to_string(), cx.theme().muted_foreground),
            SessionStatus::SignedIn(user) => (
                format!("{} ({})", user.name, user.email),
                cx.theme().success_foreground,
            ),
            SessionStatus::SignedOut => {
                ("Not signed in".to_string(), cx.theme().warning_foreground)
            }
            SessionStatus::Unreachable => ("API offline".to_string(), cx.theme().danger_foreground),
        };

        h_flex()
            .gap_2()
            .child(div().size(px(8.0)).rounded(px(4.0)).bg(color))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
    }

    fn render_nav(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut menu = SidebarMenu::new();

        for section in SECTIONS {
            menu = menu.child(
                SidebarMenuItem::new(section.label())
                    .active(self.section == section)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.section = section;
                        cx.notify();
                    })),
            );
        }

        let status = self.render_status(cx);

        Sidebar::left()
            .header(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Kloucy"),
            )
            .footer(status)
            .child(menu)
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
                let view = cx.new(DesktopShell::new);
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open Kloucy desktop window");
    });
}
