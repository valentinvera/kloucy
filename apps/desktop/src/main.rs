mod api;

use gpui::{
    AnyElement, App, Application, Bounds, Context, Entity, FontWeight, Hsla, IntoElement, Render,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::{
    ActiveTheme as _, Disableable as _, Root, Sizable as _, Theme, ThemeMode,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    sidebar::{Sidebar, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use crate::api::{Agent, ApiError, KloucyClient, SessionStatus, Workspace};

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

fn load_agents(client: &KloucyClient, session: &SessionStatus) -> LoadState<Vec<Agent>> {
    if !matches!(session, SessionStatus::SignedIn(_)) {
        return LoadState::Failed(match session {
            SessionStatus::Unreachable => "API offline.".to_string(),
            _ => "Sign in to load agents.".to_string(),
        });
    }

    match client.query::<Vec<Agent>>("agents.list", None) {
        Ok(agents) => LoadState::Loaded(agents),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load agents.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

fn error_message(error: &ApiError) -> String {
    match error {
        ApiError::Unauthorized => "Invalid email or password.".to_string(),
        ApiError::Unreachable => "API offline.".to_string(),
        ApiError::Message(message) => message.clone(),
    }
}

struct DesktopShell {
    section: Section,
    client: KloucyClient,
    session: SessionStatus,
    workspaces: LoadState<Vec<Workspace>>,
    agents: LoadState<Vec<Agent>>,
    email: Entity<InputState>,
    password: Entity<InputState>,
    workspace_name: Entity<InputState>,
    workspace_path: Entity<InputState>,
    create_error: Option<String>,
    creating_workspace: bool,
    show_create_form: bool,
    signing_in: bool,
    sign_in_error: Option<String>,
}

impl DesktopShell {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let client = KloucyClient::new();
        let email = cx.new(|cx| InputState::new(window, cx).placeholder("you@example.com"));
        let password = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("Password")
        });
        let workspace_name = cx.new(|cx| InputState::new(window, cx).placeholder("Project name"));
        let workspace_path =
            cx.new(|cx| InputState::new(window, cx).placeholder("/Users/you/project"));

        let task_client = client.clone();
        cx.spawn(async move |this, cx| {
            let (session, workspaces, agents) = cx
                .background_executor()
                .spawn(async move {
                    let session = task_client.session();
                    let workspaces = load_workspaces(&task_client, &session);
                    let agents = load_agents(&task_client, &session);
                    (session, workspaces, agents)
                })
                .await;

            this.update(cx, |this, cx| {
                this.session = session;
                this.workspaces = workspaces;
                this.agents = agents;
                cx.notify();
            })
            .ok();
        })
        .detach();

        Self {
            agents: LoadState::Loading,
            client,
            create_error: None,
            creating_workspace: false,
            email,
            password,
            section: Section::Workspaces,
            session: SessionStatus::Checking,
            show_create_form: false,
            sign_in_error: None,
            signing_in: false,
            workspace_name,
            workspace_path,
            workspaces: LoadState::Loading,
        }
    }

    fn submit_sign_in(&mut self, cx: &mut Context<Self>) {
        let email = self.email.read(cx).value().to_string();
        let password = self.password.read(cx).unmask_value().to_string();

        self.sign_in_error = None;
        self.signing_in = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.sign_in(&email, &password)?;
                    let session = client.session();
                    let workspaces = load_workspaces(&client, &session);
                    let agents = load_agents(&client, &session);
                    Ok::<_, ApiError>((session, workspaces, agents))
                })
                .await;

            this.update(cx, |this, cx| {
                this.signing_in = false;

                match result {
                    Ok((session, workspaces, agents)) => {
                        this.session = session;
                        this.workspaces = workspaces;
                        this.agents = agents;
                    }
                    Err(error) => {
                        this.sign_in_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit_create_workspace(&mut self, cx: &mut Context<Self>) {
        let name = self.workspace_name.read(cx).value().to_string();
        let local_path = self.workspace_path.read(cx).value().to_string();

        self.create_error = None;
        self.creating_workspace = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.create_workspace(&name, &local_path)?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                this.creating_workspace = false;

                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                        this.show_create_form = false;
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.create_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.create_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit_sign_out(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .spawn(async move {
                    let _ = client.sign_out();
                })
                .await;

            this.update(cx, |this, cx| {
                this.session = SessionStatus::SignedOut;
                this.create_error = None;
                this.show_create_form = false;
                cx.notify();
            })
            .ok();
        })
        .detach();
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

        let mut footer = h_flex()
            .gap_3()
            .items_center()
            .child(self.render_status(cx));

        if matches!(self.session, SessionStatus::SignedIn(_)) {
            footer = footer.child(
                Button::new("sign-out")
                    .ghost()
                    .small()
                    .label("Sign out")
                    .on_click(cx.listener(|this, _, _, cx| this.submit_sign_out(cx))),
            );
        }

        Sidebar::left()
            .header(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Kloucy"),
            )
            .footer(footer)
            .child(menu)
    }

    fn render_workspaces(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.show_create_form {
            return self.render_create_workspace(cx);
        }

        let new_button = Button::new("new-workspace")
            .small()
            .label("New workspace")
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_create_form = true;
                this.create_error = None;
                cx.notify();
            }));

        let content: AnyElement = match &self.workspaces {
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
                let mut rows = v_flex().gap_3();

                for workspace in workspaces {
                    rows = rows.child(
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

                rows.into_any_element()
            }
        };

        v_flex()
            .gap_4()
            .child(new_button)
            .child(content)
            .into_any_element()
    }

    fn render_create_workspace(&self, cx: &mut Context<Self>) -> AnyElement {
        let submit = Button::new("create-workspace")
            .primary()
            .label(if self.creating_workspace {
                "Creating…"
            } else {
                "Create workspace"
            })
            .disabled(self.creating_workspace)
            .on_click(cx.listener(|this, _, _, cx| this.submit_create_workspace(cx)));

        let cancel = Button::new("cancel-create")
            .label("Cancel")
            .disabled(self.creating_workspace)
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_create_form = false;
                this.create_error = None;
                cx.notify();
            }));

        let mut form = v_flex()
            .w(px(360.0))
            .gap_3()
            .child(Input::new(&self.workspace_name))
            .child(Input::new(&self.workspace_path))
            .child(h_flex().gap_2().child(submit).child(cancel));

        if let Some(error) = &self.create_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        form.into_any_element()
    }

    fn render_agents(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.agents {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading agents…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(agents) if agents.is_empty() => v_flex()
                .gap_1()
                .child(div().text_sm().child("No agents configured yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Register an agent to run tasks in a workspace."),
                )
                .into_any_element(),
            LoadState::Loaded(agents) => {
                let mut list = v_flex().gap_3();

                for agent in agents {
                    list = list.child(
                        h_flex()
                            .gap_2()
                            .child(div().text_sm().child(agent.name.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(agent.provider.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(agent.status.clone()),
                            ),
                    );
                }

                list.into_any_element()
            }
        }
    }

    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let body: AnyElement = match self.section {
            Section::Agents => self.render_agents(cx).into_any_element(),
            Section::Workspaces => self.render_workspaces(cx),
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

    fn render_shell(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .size_full()
            .child(self.render_nav(cx))
            .child(self.render_content(cx))
            .into_any_element()
    }

    fn render_sign_in(&self, cx: &mut Context<Self>) -> AnyElement {
        let submit = Button::new("sign-in")
            .primary()
            .label(if self.signing_in {
                "Signing in…"
            } else {
                "Sign in"
            })
            .disabled(self.signing_in)
            .on_click(cx.listener(|this, _, _, cx| this.submit_sign_in(cx)));

        let mut form = v_flex()
            .w(px(320.0))
            .gap_4()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Sign in to Kloucy"),
            )
            .child(Input::new(&self.email))
            .child(Input::new(&self.password));

        if let Some(error) = &self.sign_in_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(form.child(submit))
            .into_any_element()
    }
}

impl Render for DesktopShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.session {
            SessionStatus::Checking => centered("Checking API…", cx),
            SessionStatus::Unreachable => centered("API offline.", cx),
            SessionStatus::SignedOut => self.render_sign_in(cx),
            SessionStatus::SignedIn(_) => self.render_shell(cx),
        }
    }
}

fn centered(label: &str, cx: &App) -> AnyElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_string()),
        )
        .into_any_element()
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
                let view = cx.new(|cx| DesktopShell::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open Kloucy desktop window");
    });
}
