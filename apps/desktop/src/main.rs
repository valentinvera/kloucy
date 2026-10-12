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

use crate::api::{
    Agent, AgentSession, AgentTask, ApiError, KloucyClient, Review, SessionStatus, Workspace,
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

fn load_tasks(client: &KloucyClient, workspace_id: &str) -> LoadState<Vec<AgentTask>> {
    let input = serde_json::json!({ "workspaceId": workspace_id });

    match client.query::<Vec<AgentTask>>("agentTasks.list", Some(input)) {
        Ok(tasks) => LoadState::Loaded(tasks),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load tasks.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

fn load_sessions(client: &KloucyClient, workspace_id: &str) -> LoadState<Vec<AgentSession>> {
    let input = serde_json::json!({ "workspaceId": workspace_id });

    match client.query::<Vec<AgentSession>>("sessions.list", Some(input)) {
        Ok(sessions) => LoadState::Loaded(sessions),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load sessions.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

enum SessionAction {
    Start(String),
    Stop(String),
    Retry(String),
}

fn load_reviews(client: &KloucyClient, workspace_id: &str) -> LoadState<Vec<Review>> {
    let input = serde_json::json!({ "workspaceId": workspace_id });

    match client.query::<Vec<Review>>("reviews.list", Some(input)) {
        Ok(reviews) => LoadState::Loaded(reviews),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load reviews.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

enum ReviewAction {
    Approve(String),
    Reject(String),
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
    selected_workspace: Option<String>,
    tasks: LoadState<Vec<AgentTask>>,
    sessions: LoadState<Vec<AgentSession>>,
    reviews: LoadState<Vec<Review>>,
    email: Entity<InputState>,
    password: Entity<InputState>,
    workspace_name: Entity<InputState>,
    workspace_path: Entity<InputState>,
    task_title: Entity<InputState>,
    task_prompt: Entity<InputState>,
    create_error: Option<String>,
    creating_task: bool,
    creating_workspace: bool,
    show_create_form: bool,
    show_task_form: bool,
    signing_in: bool,
    sign_in_error: Option<String>,
    task_error: Option<String>,
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
        let task_title = cx.new(|cx| InputState::new(window, cx).placeholder("Task title"));
        let task_prompt =
            cx.new(|cx| InputState::new(window, cx).placeholder("What should the agent do?"));

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
            creating_task: false,
            creating_workspace: false,
            email,
            password,
            reviews: LoadState::Loading,
            section: Section::Workspaces,
            selected_workspace: None,
            session: SessionStatus::Checking,
            sessions: LoadState::Loading,
            show_create_form: false,
            show_task_form: false,
            sign_in_error: None,
            signing_in: false,
            task_error: None,
            task_prompt,
            task_title,
            tasks: LoadState::Loading,
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
                this.reviews = LoadState::Loading;
                this.selected_workspace = None;
                this.sessions = LoadState::Loading;
                this.show_create_form = false;
                this.tasks = LoadState::Loading;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn select_workspace(&mut self, workspace_id: String, cx: &mut Context<Self>) {
        if self.selected_workspace.as_deref() == Some(workspace_id.as_str()) {
            return;
        }

        self.selected_workspace = Some(workspace_id.clone());
        self.reviews = LoadState::Loading;
        self.sessions = LoadState::Loading;
        self.tasks = LoadState::Loading;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let (tasks, sessions, reviews) = cx
                .background_executor()
                .spawn(async move {
                    let tasks = load_tasks(&client, &workspace_id);
                    let sessions = load_sessions(&client, &workspace_id);
                    let reviews = load_reviews(&client, &workspace_id);
                    (tasks, sessions, reviews)
                })
                .await;

            this.update(cx, |this, cx| {
                this.reviews = reviews;
                this.sessions = sessions;
                this.tasks = tasks;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn review_action(&mut self, action: ReviewAction, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let reviews = cx
                .background_executor()
                .spawn(async move {
                    match action {
                        ReviewAction::Approve(id) => {
                            let _ = client.approve_review(&id);
                        }
                        ReviewAction::Reject(id) => {
                            let _ = client.reject_review(&id);
                        }
                    }

                    load_reviews(&client, &workspace_id)
                })
                .await;

            this.update(cx, |this, cx| {
                this.reviews = reviews;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn session_action(&mut self, action: SessionAction, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let sessions = cx
                .background_executor()
                .spawn(async move {
                    match action {
                        SessionAction::Start(id) => {
                            let _ = client.start_session(&id);
                        }
                        SessionAction::Stop(id) => {
                            let _ = client.stop_session(&id);
                        }
                        SessionAction::Retry(id) => {
                            let _ = client.retry_session(&id);
                        }
                    }

                    load_sessions(&client, &workspace_id)
                })
                .await;

            this.update(cx, |this, cx| {
                this.sessions = sessions;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn start_session_from_task(&mut self, task_id: String, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let sessions = cx
                .background_executor()
                .spawn(async move {
                    let _ = client.create_session(&workspace_id, &task_id);
                    load_sessions(&client, &workspace_id)
                })
                .await;

            this.update(cx, |this, cx| {
                this.section = Section::Sessions;
                this.sessions = sessions;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit_create_task(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let title = self.task_title.read(cx).value().to_string();
        let prompt = self.task_prompt.read(cx).value().to_string();

        self.task_error = None;
        self.creating_task = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.create_task(&workspace_id, &title, &prompt)?;
                    Ok::<_, ApiError>(load_tasks(&client, &workspace_id))
                })
                .await;

            this.update(cx, |this, cx| {
                this.creating_task = false;

                match result {
                    Ok(LoadState::Loaded(tasks)) => {
                        this.tasks = LoadState::Loaded(tasks);
                        this.show_task_form = false;
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.task_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.task_error = Some(error_message(&error));
                    }
                }

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
                let mut rows = v_flex().gap_1();

                for (index, workspace) in workspaces.iter().enumerate() {
                    let workspace_id = workspace.id.clone();
                    let is_selected =
                        self.selected_workspace.as_deref() == Some(workspace_id.as_str());

                    rows = rows.child(
                        v_flex()
                            .id(("workspace", index))
                            .gap_1()
                            .p_2()
                            .rounded(px(6.0))
                            .when(is_selected, |element| element.bg(cx.theme().list_active))
                            .hover(|element| element.bg(cx.theme().list_hover))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select_workspace(workspace_id.clone(), cx);
                            }))
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

    fn render_create_task(&self, cx: &mut Context<Self>) -> AnyElement {
        let submit = Button::new("create-task")
            .primary()
            .label(if self.creating_task {
                "Creating…"
            } else {
                "Create task"
            })
            .disabled(self.creating_task)
            .on_click(cx.listener(|this, _, _, cx| this.submit_create_task(cx)));

        let cancel = Button::new("cancel-task")
            .label("Cancel")
            .disabled(self.creating_task)
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_task_form = false;
                this.task_error = None;
                cx.notify();
            }));

        let mut form = v_flex()
            .w(px(360.0))
            .gap_3()
            .child(Input::new(&self.task_title))
            .child(Input::new(&self.task_prompt))
            .child(h_flex().gap_2().child(submit).child(cancel));

        if let Some(error) = &self.task_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        form.into_any_element()
    }

    fn render_inbox(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_workspace.is_none() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to see its captured tasks.")
                .into_any_element();
        }

        if self.show_task_form {
            return self.render_create_task(cx);
        }

        let new_button = Button::new("new-task")
            .small()
            .label("New task")
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_task_form = true;
                this.task_error = None;
                cx.notify();
            }));

        let content: AnyElement = match &self.tasks {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading tasks…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(tasks) if tasks.is_empty() => v_flex()
                .gap_1()
                .child(div().text_sm().child("No captured tasks yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Capture a task to assign it to an agent."),
                )
                .into_any_element(),
            LoadState::Loaded(tasks) => {
                let mut list = v_flex().gap_3();

                for (index, task) in tasks.iter().enumerate() {
                    let task_id = task.id.clone();

                    list = list.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().child(task.title.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(task.status.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(task.source.clone()),
                            )
                            .child(
                                Button::new(("task-start", index))
                                    .small()
                                    .label("Start session")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.start_session_from_task(task_id.clone(), cx);
                                    })),
                            ),
                    );
                }

                list.into_any_element()
            }
        };

        v_flex()
            .gap_4()
            .child(new_button)
            .child(content)
            .into_any_element()
    }

    fn render_sessions(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_workspace.is_none() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to see its sessions.")
                .into_any_element();
        }

        match &self.sessions {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading sessions…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(sessions) if sessions.is_empty() => v_flex()
                .gap_1()
                .child(div().text_sm().child("No sessions yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Start a task to run an agent session."),
                )
                .into_any_element(),
            LoadState::Loaded(sessions) => {
                let mut list = v_flex().gap_3();

                for (index, session) in sessions.iter().enumerate() {
                    let start_id = session.id.clone();
                    let stop_id = session.id.clone();
                    let retry_id = session.id.clone();

                    list = list.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div().text_sm().child(
                                    session
                                        .prompt
                                        .clone()
                                        .unwrap_or_else(|| "Untitled session".to_string()),
                                ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(session.status.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Button::new(("session-start", index))
                                            .small()
                                            .label("Start")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.session_action(
                                                    SessionAction::Start(start_id.clone()),
                                                    cx,
                                                );
                                            })),
                                    )
                                    .child(
                                        Button::new(("session-stop", index))
                                            .small()
                                            .label("Stop")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.session_action(
                                                    SessionAction::Stop(stop_id.clone()),
                                                    cx,
                                                );
                                            })),
                                    )
                                    .child(
                                        Button::new(("session-retry", index))
                                            .small()
                                            .label("Retry")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.session_action(
                                                    SessionAction::Retry(retry_id.clone()),
                                                    cx,
                                                );
                                            })),
                                    ),
                            ),
                    );
                }

                list.into_any_element()
            }
        }
    }

    fn render_reviews(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_workspace.is_none() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to see its reviews.")
                .into_any_element();
        }

        match &self.reviews {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading reviews…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(reviews) if reviews.is_empty() => v_flex()
                .gap_1()
                .child(div().text_sm().child("Nothing to review yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Completed sessions with changes appear here."),
                )
                .into_any_element(),
            LoadState::Loaded(reviews) => {
                let mut list = v_flex().gap_3();

                for (index, review) in reviews.iter().enumerate() {
                    let approve_id = review.id.clone();
                    let reject_id = review.id.clone();

                    list = list.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div().text_sm().child(
                                    review
                                        .artifact_id
                                        .clone()
                                        .unwrap_or_else(|| "Changed files".to_string()),
                                ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(review.action.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Button::new(("review-approve", index))
                                            .small()
                                            .label("Approve")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.review_action(
                                                    ReviewAction::Approve(approve_id.clone()),
                                                    cx,
                                                );
                                            })),
                                    )
                                    .child(
                                        Button::new(("review-reject", index))
                                            .small()
                                            .label("Reject")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.review_action(
                                                    ReviewAction::Reject(reject_id.clone()),
                                                    cx,
                                                );
                                            })),
                                    ),
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
            Section::Inbox => self.render_inbox(cx),
            Section::Reviews => self.render_reviews(cx),
            Section::Sessions => self.render_sessions(cx),
            Section::Workspaces => self.render_workspaces(cx),
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
