mod api;

use gpui::{
    AnyElement, App, Application, Bounds, Context, FontWeight, Hsla, IntoElement, Render, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Theme, ThemeMode, h_flex,
    sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use crate::api::{ApiError, KloucyClient, SessionStatus, Workspace};

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

enum LoadState<T> {
    Loading,
    Loaded(T),
    Failed(String),
}

fn load_workspaces(client: &KloucyClient, session: &SessionStatus) -> LoadState<Vec<Workspace>> {
    if !matches!(session, SessionStatus::SignedIn(_)) {
        return LoadState::Failed(match session {
            SessionStatus::Unreachable => "API offline.".to_string(),
            _ => "Sign in to load workspaces.".to_string(),
        });
    }

    match client.query::<Vec<Workspace>>("workspaces.list", None) {
        Ok(workspaces) => LoadState::Loaded(workspaces),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load workspaces.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

struct DesktopShell {
    section: Section,
    session: SessionStatus,
    workspaces: LoadState<Vec<Workspace>>,
}

impl DesktopShell {
    fn new(cx: &mut Context<Self>) -> Self {
        let client = KloucyClient::new();

        cx.spawn(async move |this, cx| {
            let (session, workspaces) = cx
                .background_executor()
                .spawn(async move {
                    let session = client.session();
                    let workspaces = load_workspaces(&client, &session);
                    (session, workspaces)
                })
                .await;

            this.update(cx, |this, cx| {
                this.session = session;
                this.workspaces = workspaces;
                cx.notify();
            })
            .ok();
        })
        .detach();

        Self {
            section: Section::Workspaces,
            session: SessionStatus::Checking,
            workspaces: LoadState::Loading,
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

    fn render_workspaces(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.workspaces {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading workspaces…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(workspaces) if workspaces.is_empty() => v_flex()
                .gap_1()
                .child(div().text_sm().child("Open your first workspace."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Create a workspace to run and supervise local agents."),
                )
                .into_any_element(),
            LoadState::Loaded(workspaces) => {
                let mut list = v_flex().gap_3();

                for workspace in workspaces {
                    list = list.child(
                        v_flex()
                            .gap_1()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(div().text_sm().child(workspace.name.clone()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(workspace.stage.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(workspace.local_path.clone()),
                            ),
                    );
                }

                list.into_any_element()
            }
        }
    }

    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let body: AnyElement = match self.section {
            Section::Workspaces => self.render_workspaces(cx).into_any_element(),
            _ => div()
                .text_sm()
                .child(self.section.empty())
                .into_any_element(),
        };

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
                    .child(body),
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
