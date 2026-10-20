mod api;
mod assets;
mod detect;
mod git;
mod runner;
mod safety;
mod settings;
mod typography;

use gpui::{
    AnyElement, App, Application, Bounds, ClipboardItem, Context, Corner, Entity, FocusHandle,
    FontWeight, IntoElement, KeyBinding, Render, SharedString, Subscription, Window, WindowBounds,
    WindowOptions, actions, div, hsla, prelude::*, px, rgb, size,
};
use gpui_component::{
    ActiveTheme as _, Disableable as _, Icon, IconName, Root, Selectable, Sizable as _, Theme,
    ThemeMode,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    popover::Popover,
    sidebar::{Sidebar, SidebarGroup, SidebarMenu, SidebarMenuItem},
    v_flex,
};

use crate::api::{
    Agent, AgentSession, AgentTask, ApiError, Decision, KloucyClient, PairingSession,
    ReadinessSnapshot, Review, SessionLog, SessionStatus, SessionStep, WeeklyReview, Workspace,
};

use crate::detect::DetectedAgent;

use std::rc::Rc;
use std::sync::Arc;

actions!(kloucy, [OpenCommandPalette]);

/// Key context of the signed-in shell, so the palette shortcut works anywhere in it.
const SHELL_CONTEXT: &str = "KloucyShell";

/// A command offered by the palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PaletteCommand {
    DetectAgents,
    NewPairingCode,
    NewWorkspace,
    RegisterAgent,
    Section(Section),
}

struct PaletteEntry {
    command: PaletteCommand,
    group: &'static str,
    keywords: &'static str,
    label: &'static str,
}

const PALETTE_ENTRIES: [PaletteEntry; 15] = [
    PaletteEntry {
        command: PaletteCommand::Section(Section::Home),
        group: "Go to",
        keywords: "home overview activity",
        label: "Open home",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Workspaces),
        group: "Go to",
        keywords: "projects",
        label: "Open workspaces",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Agents),
        group: "Go to",
        keywords: "registry providers",
        label: "Open agents",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Inbox),
        group: "Go to",
        keywords: "tasks capture",
        label: "Open inbox",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Sessions),
        group: "Go to",
        keywords: "runs logs",
        label: "Open sessions",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Reviews),
        group: "Go to",
        keywords: "changes approve reject",
        label: "Open reviews",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Decisions),
        group: "Go to",
        keywords: "log history",
        label: "Open decisions",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Readiness),
        group: "Go to",
        keywords: "blockers safety",
        label: "Open readiness",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::WeeklyReview),
        group: "Go to",
        keywords: "summary week",
        label: "Open weekly review",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Pairing),
        group: "Go to",
        keywords: "mobile qr",
        label: "Open pairing",
    },
    PaletteEntry {
        command: PaletteCommand::Section(Section::Settings),
        group: "Go to",
        keywords: "workspace config archive",
        label: "Open settings",
    },
    PaletteEntry {
        command: PaletteCommand::NewWorkspace,
        group: "Action",
        keywords: "project create",
        label: "New workspace",
    },
    PaletteEntry {
        command: PaletteCommand::RegisterAgent,
        group: "Action",
        keywords: "agent add command",
        label: "Register an agent",
    },
    PaletteEntry {
        command: PaletteCommand::DetectAgents,
        group: "Action",
        keywords: "path detect installed",
        label: "Detect installed agents",
    },
    PaletteEntry {
        command: PaletteCommand::NewPairingCode,
        group: "Action",
        keywords: "mobile qr code",
        label: "New pairing code",
    },
];

/// Commands matching a query, by label or keywords.
fn palette_matches(query: &str) -> Vec<&'static PaletteEntry> {
    let needle = query.trim().to_lowercase();

    PALETTE_ENTRIES
        .iter()
        .filter(|entry| {
            needle.is_empty()
                || entry.label.to_lowercase().contains(&needle)
                || entry.keywords.contains(&needle)
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Home,
    Workspaces,
    Agents,
    Inbox,
    Sessions,
    Reviews,
    Decisions,
    Readiness,
    WeeklyReview,
    Pairing,
    Settings,
}

/// Sidebar groups in reading order: the daily drivers first, then the work they
/// produce, then what was decided, then the registry itself.
const NAV_GROUPS: [(&str, &[Section]); 4] = [
    ("🧭 Now", &[Section::Home, Section::Inbox]),
    (
        "⚡ Run",
        &[Section::Sessions, Section::Reviews, Section::Readiness],
    ),
    ("🗂 History", &[Section::Decisions, Section::WeeklyReview]),
    (
        "⚙️ Setup",
        &[
            Section::Workspaces,
            Section::Agents,
            Section::Pairing,
            Section::Settings,
        ],
    ),
];

impl Section {
    fn label(self) -> &'static str {
        match self {
            Section::Home => "Home",
            Section::Workspaces => "Workspaces",
            Section::Agents => "Agents",
            Section::Inbox => "Inbox",
            Section::Sessions => "Sessions",
            Section::Reviews => "Reviews",
            Section::Decisions => "Decisions",
            Section::Readiness => "Readiness",
            Section::WeeklyReview => "Weekly review",
            Section::Pairing => "Pairing",
            Section::Settings => "Settings",
        }
    }

    /// The icon shown in the sidebar for this section.
    fn icon(self) -> IconName {
        match self {
            Section::Agents => IconName::Bot,
            Section::Decisions => IconName::BookOpen,
            Section::Home => IconName::LayoutDashboard,
            Section::Inbox => IconName::Inbox,
            Section::Pairing => IconName::Globe,
            Section::Readiness => IconName::ChartPie,
            Section::Reviews => IconName::CircleCheck,
            Section::Sessions => IconName::SquareTerminal,
            Section::Settings => IconName::Settings,
            Section::WeeklyReview => IconName::Calendar,
            Section::Workspaces => IconName::Folder,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Section::Home => "What your agents are doing right now.",
            Section::Workspaces => "Local projects Kloucy can supervise.",
            Section::Agents => "Local agents available to run tasks.",
            Section::Inbox => "Tasks captured before they run.",
            Section::Sessions => "Running, waiting, and finished agent work.",
            Section::Reviews => "Changed files and artifacts awaiting a decision.",
            Section::Decisions => "Control decisions and their reasons.",
            Section::Readiness => "Whether this workspace is ready to run agents.",
            Section::WeeklyReview => "What shipped, what was blocked, and what comes next.",
            Section::Pairing => "Pair a phone to keep control away from the desk.",
            Section::Settings => "Correct the setup data of the selected workspace.",
        }
    }
}

enum LoadState<T> {
    Loading,
    Loaded(T),
    Failed(String),
}

/// The chip menus the home composer opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HomeMenu {
    Workspace,
    Agent,
    Account,
    Checkout,
    Branch,
    Attach,
}

/// What the composer last did, shown under the checkout row.
#[derive(Clone, Debug)]
enum ComposerStatus {
    Captured(String),
    Notice(String),
    Failed(String),
}

/// How much of a submission becomes the task title.
const COMPOSER_TITLE_LIMIT: usize = 80;

/// The title a composer submission is captured under: its first line, short
/// enough to read in a list, with the full text kept as the prompt.
fn composer_title(text: &str) -> String {
    let first = text.lines().next().unwrap_or_default().trim();

    if first.chars().count() <= COMPOSER_TITLE_LIMIT {
        return first.to_string();
    }

    let mut title: String = first.chars().take(COMPOSER_TITLE_LIMIT).collect();
    title.push('…');

    title
}

/// How a workspace's safety mode reads in the composer pill.
fn safety_label(mode: &str) -> &'static str {
    match mode {
        "guarded" => "guarded",
        _ => "open",
    }
}

/// The last segment of a path, for chips that cannot show all of it.
fn folder_name(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');

    match trimmed.rsplit('/').next() {
        Some(name) if !name.is_empty() => name.to_string(),
        _ => path.to_string(),
    }
}

/// One row of a home menu: an icon, some content, and what happens on click.
fn menu_row(
    id: (&'static str, usize),
    cx: &App,
    selected: bool,
    children: Vec<AnyElement>,
    on_click: Option<Box<dyn Fn(&mut Window, &mut App)>>,
) -> AnyElement {
    let mut row = h_flex()
        .id(id)
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded(px(4.0))
        .text_sm()
        .when(selected, |element| element.bg(cx.theme().list_hover))
        .hover(|element| element.bg(cx.theme().list_hover));

    if let Some(on_click) = on_click {
        row = row.on_click(move |_, window, cx| on_click(window, cx));
    }

    for child in children {
        row = row.child(child);
    }

    row.into_any_element()
}

/// A read-only line inside a home menu: a fact, not an action.
fn menu_note(text: impl Into<SharedString>, cx: &App) -> AnyElement {
    let text: SharedString = text.into();

    div()
        .px_2()
        .py_1()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(text)
        .into_any_element()
}

/// A popover trigger that keeps Kloucy's chip styling.
///
/// `Popover` needs its trigger to be selectable, but it only uses that to style
/// triggers it owns; the chip is already built, so the flag is recorded and
/// ignored.
struct Chip {
    selected: bool,
    element: AnyElement,
}

impl Selectable for Chip {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl IntoElement for Chip {
    type Element = AnyElement;

    fn into_element(self) -> Self::Element {
        self.element
    }
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

fn load_agents_list(client: &KloucyClient) -> LoadState<Vec<Agent>> {
    match client.query::<Vec<Agent>>("agents.list", None) {
        Ok(agents) => LoadState::Loaded(agents),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load agents.".to_string()),
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

    load_agents_list(client)
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

fn load_session_logs(client: &KloucyClient, session_id: &str) -> LoadState<Vec<SessionLog>> {
    let input = serde_json::json!({ "sessionId": session_id });

    match client.query::<Vec<SessionLog>>("sessions.logs", Some(input)) {
        Ok(logs) => LoadState::Loaded(logs),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load logs.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

fn load_session_steps(client: &KloucyClient, session_id: &str) -> LoadState<Vec<SessionStep>> {
    let input = serde_json::json!({ "sessionId": session_id });

    match client.query::<Vec<SessionStep>>("sessionSteps.list", Some(input)) {
        Ok(steps) => LoadState::Loaded(steps),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load steps.".to_string()),
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

fn load_decisions(client: &KloucyClient, workspace_id: &str) -> LoadState<Vec<Decision>> {
    let input = serde_json::json!({ "workspaceId": workspace_id });

    match client.query::<Vec<Decision>>("decisions.list", Some(input)) {
        Ok(decisions) => LoadState::Loaded(decisions),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load decisions.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

fn load_readiness(
    client: &KloucyClient,
    workspace_id: &str,
) -> LoadState<Option<ReadinessSnapshot>> {
    let input = serde_json::json!({ "workspaceId": workspace_id });

    match client.query::<Option<ReadinessSnapshot>>("readiness.latest", Some(input)) {
        Ok(snapshot) => LoadState::Loaded(snapshot),
        Err(ApiError::Unauthorized) => LoadState::Failed("Sign in to load readiness.".to_string()),
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

fn load_weekly_review(
    client: &KloucyClient,
    workspace_id: &str,
) -> LoadState<Option<WeeklyReview>> {
    let input = serde_json::json!({ "workspaceId": workspace_id });

    match client.query::<Option<WeeklyReview>>("weeklyReviews.latest", Some(input)) {
        Ok(review) => LoadState::Loaded(review),
        Err(ApiError::Unauthorized) => {
            LoadState::Failed("Sign in to load weekly reviews.".to_string())
        }
        Err(ApiError::Message(message)) => LoadState::Failed(message),
        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
    }
}

/// Why a review decision needs a written reason.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ReviewReason {
    FollowUp,
    Reject,
}

impl ReviewReason {
    fn title(self) -> &'static str {
        match self {
            ReviewReason::FollowUp => "Follow-up condition",
            ReviewReason::Reject => "Why reject this change?",
        }
    }

    fn submit_label(self) -> &'static str {
        match self {
            ReviewReason::FollowUp => "Save condition",
            ReviewReason::Reject => "Reject change",
        }
    }

    fn missing_reason(self) -> &'static str {
        match self {
            ReviewReason::FollowUp => "Describe when this should run again.",
            ReviewReason::Reject => "Describe why this change is rejected.",
        }
    }

    fn placeholder(self) -> &'static str {
        match self {
            ReviewReason::FollowUp => "When should this run again?",
            ReviewReason::Reject => "What is wrong with this change?",
        }
    }
}

fn error_message(error: &ApiError) -> String {
    match error {
        ApiError::Unauthorized => "Invalid email or password.".to_string(),
        ApiError::Unreachable => "API offline.".to_string(),
        ApiError::Message(message) => message.clone(),
    }
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

/// The stored name of a theme mode.
fn mode_name(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        _ => "dark",
    }
}

/// A theme mode from its stored name, defaulting to Kloucy's dark.
fn theme_mode(name: &str) -> ThemeMode {
    match name {
        "light" => ThemeMode::Light,
        _ => ThemeMode::Dark,
    }
}

/// Kloucy's app font and tinted sidebar, for the given mode.
fn apply_theme(mode: ThemeMode, window: Option<&mut Window>, cx: &mut App) {
    Theme::change(mode, window, cx);

    let theme = Theme::global_mut(cx);
    theme.font_family = assets::FONT_FAMILY.into();

    // The sidebar is the one surface that carries a tint, so it reads as its
    // own area: a faint olive over black, a light sage over white.
    let (sidebar, sidebar_accent, sidebar_border) = match mode {
        ThemeMode::Light => (
            hsla(80.0 / 360.0, 0.24, 0.94, 1.0),
            hsla(80.0 / 360.0, 0.20, 0.88, 1.0),
            hsla(80.0 / 360.0, 0.12, 0.85, 1.0),
        ),
        _ => (
            hsla(80.0 / 360.0, 0.09, 0.09, 1.0),
            hsla(80.0 / 360.0, 0.07, 0.16, 1.0),
            hsla(80.0 / 360.0, 0.06, 0.18, 1.0),
        ),
    };

    theme.sidebar = sidebar;
    theme.sidebar_accent = sidebar_accent;
    theme.sidebar_accent_foreground = theme.foreground;
    theme.sidebar_border = sidebar_border;
}

/// A one-line answer to "what are my agents doing?", when there is anything.
fn session_activity(sessions: &LoadState<Vec<AgentSession>>) -> Option<String> {
    let waiting = count_where(sessions, |session| session.status == "waiting");
    let running = count_where(sessions, |session| session.status == "running");
    let mut parts = Vec::new();

    if waiting > 0 {
        parts.push(format!("{waiting} waiting"));
    }

    if running > 0 {
        parts.push(format!("{running} running"));
    }

    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// How many loaded items match, and zero while loading or after a failure.
fn count_where<T>(items: &LoadState<Vec<T>>, predicate: impl Fn(&T) -> bool) -> usize {
    match items {
        LoadState::Loaded(items) => items.iter().filter(|item| predicate(item)).count(),
        _ => 0,
    }
}

/// Up to two initials for the account row.
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

struct DesktopShell {
    section: Section,
    client: KloucyClient,
    registry: Arc<runner::RunRegistry>,
    focus_handle: FocusHandle,
    mono_font: SharedString,
    theme_mode: ThemeMode,
    composer: Entity<InputState>,
    _composer_subscription: Subscription,
    composer_status: Option<ComposerStatus>,
    sending_composer: bool,
    home_agent: Option<String>,
    home_checkout: git::Checkout,
    home_menu: Option<HomeMenu>,
    palette_open: bool,
    palette_query: Entity<InputState>,
    session: SessionStatus,
    workspaces: LoadState<Vec<Workspace>>,
    agents: LoadState<Vec<Agent>>,
    detected_agents: Option<Vec<DetectedAgent>>,
    selected_workspace: Option<String>,
    selected_session: Option<String>,
    session_logs: LoadState<Vec<SessionLog>>,
    session_steps: LoadState<Vec<SessionStep>>,
    new_step_reason: Entity<InputState>,
    new_step_lane: String,
    creating_step: bool,
    step_error: Option<String>,
    tasks: LoadState<Vec<AgentTask>>,
    sessions: LoadState<Vec<AgentSession>>,
    reviews: LoadState<Vec<Review>>,
    decisions: LoadState<Vec<Decision>>,
    readiness: LoadState<Option<ReadinessSnapshot>>,
    weekly_review: LoadState<Option<WeeklyReview>>,
    pairing: LoadState<Option<PairingSession>>,
    email: Entity<InputState>,
    password: Entity<InputState>,
    workspace_name: Entity<InputState>,
    workspace_path: Entity<InputState>,
    task_title: Entity<InputState>,
    task_prompt: Entity<InputState>,
    decision_title: Entity<InputState>,
    decision_summary: Entity<InputState>,
    agent_name: Entity<InputState>,
    agent_provider: Entity<InputState>,
    agent_command: Entity<InputState>,
    intake_project_type: Entity<InputState>,
    intake_package_manager: Entity<InputState>,
    intake_runtime: Entity<InputState>,
    intake_safe_commands: Entity<InputState>,
    intake_ignored_paths: Entity<InputState>,
    intake_mobile_pairing: bool,
    intake_safety_mode: String,
    settings_name: Entity<InputState>,
    settings_path: Entity<InputState>,
    settings_safe_commands: Entity<InputState>,
    settings_ignored_paths: Entity<InputState>,
    saving_settings: bool,
    archiving_workspace: bool,
    settings_error: Option<String>,
    create_error: Option<String>,
    agent_error: Option<String>,
    completing_intake: bool,
    creating_agent: bool,
    intake_error: Option<String>,
    creating_decision: bool,
    creating_task: bool,
    creating_workspace: bool,
    decision_error: Option<String>,
    show_agent_form: bool,
    show_create_form: bool,
    show_decision_form: bool,
    show_task_form: bool,
    signing_in: bool,
    sign_in_error: Option<String>,
    session_error: Option<String>,
    blocked_command: Option<String>,
    session_notice: Option<String>,
    review_reason_for: Option<(String, ReviewReason)>,
    review_reason: Entity<InputState>,
    saving_review_reason: bool,
    review_error: Option<String>,
    instruction: Entity<InputState>,
    sending_instruction: bool,
    instruction_error: Option<String>,
    task_error: Option<String>,
}

impl DesktopShell {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let client = KloucyClient::new();
        let registry = runner::RunRegistry::new();
        let focus_handle = cx.focus_handle();
        let theme_mode = theme_mode(
            &settings::load_theme_mode()
                .unwrap_or_else(|| settings::DEFAULT_THEME_MODE.to_string()),
        );
        let mono_font = typography::pick_mono_family(&cx.text_system().all_font_names())
            .map(SharedString::from)
            .unwrap_or_else(|| cx.theme().font_family.clone());
        let palette_query = cx.new(|cx| InputState::new(window, cx).placeholder("Search commands"));
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
        let decision_title = cx.new(|cx| InputState::new(window, cx).placeholder("Decision"));
        let decision_summary =
            cx.new(|cx| InputState::new(window, cx).placeholder("What changed and why"));
        let agent_name = cx.new(|cx| InputState::new(window, cx).placeholder("Agent name"));
        let agent_provider = cx.new(|cx| InputState::new(window, cx).placeholder("claude-code"));
        let agent_command =
            cx.new(|cx| InputState::new(window, cx).placeholder("codex exec --full-auto"));
        let intake_project_type =
            cx.new(|cx| InputState::new(window, cx).placeholder("Project type (rust, node, …)"));
        let intake_package_manager =
            cx.new(|cx| InputState::new(window, cx).placeholder("Package manager (bun, cargo, …)"));
        let intake_runtime =
            cx.new(|cx| InputState::new(window, cx).placeholder("Runtime (bun, node, rust, …)"));
        let intake_safe_commands =
            cx.new(|cx| InputState::new(window, cx).placeholder("Safe commands, comma separated"));
        let intake_ignored_paths =
            cx.new(|cx| InputState::new(window, cx).placeholder("Ignored paths, comma separated"));
        let settings_name =
            cx.new(|cx| InputState::new(window, cx).placeholder("New workspace name"));
        let settings_path = cx.new(|cx| InputState::new(window, cx).placeholder("New local path"));
        let settings_safe_commands =
            cx.new(|cx| InputState::new(window, cx).placeholder("Safe commands, comma separated"));
        let settings_ignored_paths =
            cx.new(|cx| InputState::new(window, cx).placeholder("Ignored paths, comma separated"));
        let new_step_reason = cx.new(|cx| InputState::new(window, cx).placeholder("Step"));
        let review_reason = cx.new(|cx| {
            InputState::new(window, cx).placeholder(ReviewReason::FollowUp.placeholder())
        });
        let instruction =
            cx.new(|cx| InputState::new(window, cx).placeholder("Send a follow-up instruction"));
        let composer = cx.new(|cx| InputState::new(window, cx).placeholder("Do anything…"));
        let composer_subscription = cx.subscribe_in(
            &composer,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { secondary: false } = event {
                    this.submit_composer(window, cx);
                }
            },
        );

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

                let first = match &this.workspaces {
                    LoadState::Loaded(list) => list.first().map(|workspace| workspace.id.clone()),
                    _ => None,
                };

                if this.selected_workspace.is_none() {
                    if let Some(id) = first {
                        this.select_workspace(id, cx);
                    }
                }
            })
            .ok();
        })
        .detach();

        Self {
            agents: LoadState::Loading,
            detected_agents: None,
            client,
            registry,
            focus_handle,
            mono_font,
            theme_mode,
            composer,
            _composer_subscription: composer_subscription,
            composer_status: None,
            sending_composer: false,
            home_agent: None,
            home_checkout: git::Checkout::None,
            home_menu: None,
            palette_open: false,
            palette_query,
            agent_command,
            agent_error: None,
            agent_name,
            agent_provider,
            completing_intake: false,
            create_error: None,
            creating_agent: false,
            creating_step: false,
            intake_error: None,
            intake_ignored_paths,
            intake_project_type,
            intake_package_manager,
            intake_runtime,
            intake_mobile_pairing: false,
            intake_safety_mode: "open".to_string(),
            intake_safe_commands,
            archiving_workspace: false,
            saving_settings: false,
            settings_error: None,
            settings_ignored_paths,
            settings_name,
            settings_path,
            settings_safe_commands,
            new_step_lane: "now".to_string(),
            new_step_reason,
            session_steps: LoadState::Loading,
            step_error: None,
            creating_decision: false,
            creating_task: false,
            creating_workspace: false,
            decision_error: None,
            decision_summary,
            decision_title,
            decisions: LoadState::Loading,
            email,
            password,
            reviews: LoadState::Loading,
            readiness: LoadState::Loading,
            pairing: LoadState::Loaded(None),
            section: Section::Home,
            selected_session: None,
            selected_workspace: None,
            session: SessionStatus::Checking,
            session_logs: LoadState::Loading,
            sessions: LoadState::Loading,
            weekly_review: LoadState::Loading,
            show_agent_form: false,
            show_create_form: false,
            show_decision_form: false,
            show_task_form: false,
            sign_in_error: None,
            session_error: None,
            blocked_command: None,
            session_notice: None,
            review_reason_for: None,
            review_reason,
            saving_review_reason: false,
            review_error: None,
            instruction,
            sending_instruction: false,
            instruction_error: None,
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
                this.decisions = LoadState::Loading;
                this.pairing = LoadState::Loaded(None);
                this.selected_session = None;
                this.session_steps = LoadState::Loading;
                this.readiness = LoadState::Loading;
                this.reviews = LoadState::Loading;
                this.weekly_review = LoadState::Loading;
                this.selected_workspace = None;
                this.sessions = LoadState::Loading;
                this.show_agent_form = false;
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
        self.selected_session = None;
        self.session_logs = LoadState::Loading;
        self.session_steps = LoadState::Loading;
        self.decisions = LoadState::Loading;
        self.readiness = LoadState::Loading;
        self.reviews = LoadState::Loading;
        self.sessions = LoadState::Loading;
        self.tasks = LoadState::Loading;
        self.weekly_review = LoadState::Loading;
        self.home_agent = None;
        self.home_checkout = git::Checkout::None;

        let checkout = self
            .selected_workspace_data()
            .map(|workspace| workspace.local_path.clone());

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let (tasks, sessions, reviews, decisions, readiness, weekly_review, checkout) = cx
                .background_executor()
                .spawn(async move {
                    let tasks = load_tasks(&client, &workspace_id);
                    let sessions = load_sessions(&client, &workspace_id);
                    let reviews = load_reviews(&client, &workspace_id);
                    let decisions = load_decisions(&client, &workspace_id);
                    let readiness = load_readiness(&client, &workspace_id);
                    let weekly_review = load_weekly_review(&client, &workspace_id);
                    let checkout = checkout
                        .as_deref()
                        .map(|path| git::describe(std::path::Path::new(path)))
                        .unwrap_or(git::Checkout::None);
                    (
                        tasks,
                        sessions,
                        reviews,
                        decisions,
                        readiness,
                        weekly_review,
                        checkout,
                    )
                })
                .await;

            this.update(cx, |this, cx| {
                this.decisions = decisions;
                this.readiness = readiness;
                this.reviews = reviews;
                this.sessions = sessions;
                this.tasks = tasks;
                this.weekly_review = weekly_review;
                this.home_checkout = checkout;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn approve_review(&mut self, review_id: String, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let reviews = cx
                .background_executor()
                .spawn(async move {
                    let _ = client.approve_review(&review_id);
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

    fn open_review_reason(
        &mut self,
        review_id: String,
        kind: ReviewReason,
        reason: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.review_error = None;
        self.review_reason_for = Some((review_id, kind));

        let value = reason.unwrap_or_default();
        self.review_reason.update(cx, |state, cx| {
            state.set_placeholder(kind.placeholder(), window, cx);
            state.set_value(value, window, cx);
        });

        cx.notify();
    }

    fn cancel_review_reason(&mut self, cx: &mut Context<Self>) {
        self.review_reason_for = None;
        self.review_error = None;
        cx.notify();
    }

    fn submit_review_reason(&mut self, cx: &mut Context<Self>) {
        let Some((review_id, kind)) = self.review_reason_for.clone() else {
            return;
        };

        let reason = self.review_reason.read(cx).value().to_string();

        if reason.trim().is_empty() {
            self.review_error = Some(kind.missing_reason().to_string());
            cx.notify();
            return;
        }

        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        self.review_error = None;
        self.saving_review_reason = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    match kind {
                        ReviewReason::FollowUp => {
                            client.follow_up_review(&review_id, &reason)?;
                        }
                        ReviewReason::Reject => {
                            client.reject_review(&review_id, Some(&reason))?;
                        }
                    }

                    Ok::<_, ApiError>(load_reviews(&client, &workspace_id))
                })
                .await;

            this.update(cx, |this, cx| {
                this.saving_review_reason = false;

                match result {
                    Ok(LoadState::Loaded(reviews)) => {
                        this.reviews = LoadState::Loaded(reviews);
                        this.review_reason_for = None;
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.review_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.review_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit_instruction(&mut self, cx: &mut Context<Self>) {
        let Some(session_id) = self.selected_session.clone() else {
            return;
        };

        let instruction = self.instruction.read(cx).value().to_string();

        if instruction.trim().is_empty() {
            self.instruction_error = Some("Write an instruction to send.".to_string());
            cx.notify();
            return;
        }

        self.instruction_error = None;
        self.sending_instruction = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.send_session_instruction(&session_id, &instruction)?;
                    Ok::<_, ApiError>(load_session_logs(&client, &session_id))
                })
                .await;

            this.update(cx, |this, cx| {
                this.sending_instruction = false;

                match result {
                    Ok(logs) => {
                        this.session_logs = logs;
                    }
                    Err(error) => {
                        this.instruction_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn agent_command(&self, agent_id: &str) -> Option<String> {
        let LoadState::Loaded(agents) = &self.agents else {
            return None;
        };

        let agent = agents.iter().find(|agent| agent.id == agent_id)?;

        agent
            .command_template
            .clone()
            .or_else(|| agent.executable_path.clone())
    }

    fn session_agent_id(&self, session_id: &str) -> Option<String> {
        let from_session = match &self.sessions {
            LoadState::Loaded(sessions) => sessions
                .iter()
                .find(|session| session.id == session_id)
                .and_then(|session| session.agent_id.clone()),
            _ => None,
        };

        from_session.or_else(|| {
            self.selected_workspace_data()
                .and_then(|workspace| workspace.default_agent_id.clone())
        })
    }

    fn session_action(&mut self, action: SessionAction, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let (command, cwd) = match &action {
            SessionAction::Start(id) => {
                let command = self
                    .session_agent_id(id)
                    .and_then(|agent_id| self.agent_command(&agent_id));
                let cwd = self
                    .selected_workspace_data()
                    .map(|workspace| workspace.local_path.clone());

                (command, cwd)
            }
            _ => (None, None),
        };

        if matches!(&action, SessionAction::Start(_)) {
            if command.is_none() || cwd.is_none() {
                self.session_error = Some(
                    "Set a default agent for this workspace before starting a session.".to_string(),
                );
                cx.notify();
                return;
            }

            let workspace = self.selected_workspace_data();
            let mode = workspace
                .map(|workspace| workspace.safety_mode.clone())
                .unwrap_or_default();
            let safe_commands = workspace
                .and_then(|workspace| workspace.safe_commands.clone())
                .unwrap_or_default();

            if let Some(command) = command.as_deref() {
                if !safety::command_allowed(&mode, command, &safe_commands) {
                    self.blocked_command = Some(command.to_string());
                    self.session_error = None;
                    self.session_notice = None;
                    cx.notify();
                    return;
                }
            }
        }

        self.session_error = None;
        self.blocked_command = None;
        self.session_notice = None;

        let client = self.client.clone();
        let registry = self.registry.clone();
        cx.spawn(async move |this, cx| {
            let sessions = cx
                .background_executor()
                .spawn(async move {
                    match &action {
                        SessionAction::Start(id) => {
                            let _ = client.start_session(id);

                            if let (Some(command), Some(cwd)) = (command, cwd) {
                                let runner_client = client.clone();
                                let runner_registry = registry.clone();
                                let runner_workspace_id = workspace_id.clone();
                                let session_id = id.clone();
                                std::thread::spawn(move || {
                                    runner::run_agent(
                                        runner_registry,
                                        runner_client,
                                        runner_workspace_id,
                                        session_id,
                                        command,
                                        cwd,
                                    );
                                });
                            }
                        }
                        SessionAction::Stop(id) => {
                            let _ = client.stop_session(id);
                            registry.stop(id);
                        }
                        SessionAction::Retry(id) => {
                            let _ = client.retry_session(id);
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
            let (sessions, tasks) = cx
                .background_executor()
                .spawn(async move {
                    let _ = client.start_session_from_task(&task_id);
                    let sessions = load_sessions(&client, &workspace_id);
                    let tasks = load_tasks(&client, &workspace_id);
                    (sessions, tasks)
                })
                .await;

            this.update(cx, |this, cx| {
                this.section = Section::Sessions;
                this.sessions = sessions;
                this.tasks = tasks;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// The agent the composer assigns: the one picked in the pill, else the
    /// workspace default.
    fn composer_agent(&self) -> Option<&Agent> {
        let LoadState::Loaded(agents) = &self.agents else {
            return None;
        };

        let selected = self.home_agent.clone().or_else(|| {
            self.selected_workspace_data()
                .and_then(|workspace| workspace.default_agent_id.clone())
        });

        selected.and_then(|id| agents.iter().find(|agent| agent.id == id))
    }

    /// Capture whatever the composer holds as a task, assigned to the chosen
    /// agent. Tasks do not run on their own; this only queues the work.
    fn submit_composer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.sending_composer {
            return;
        }

        let text = self.composer.read(cx).value().trim().to_string();

        if text.is_empty() {
            self.composer_status = Some(ComposerStatus::Failed(
                "Write what you want the agent to do.".to_string(),
            ));
            cx.notify();
            return;
        }

        let Some(workspace_id) = self.selected_workspace.clone() else {
            self.composer_status = Some(ComposerStatus::Failed(
                "Create a workspace before capturing work.".to_string(),
            ));
            cx.notify();
            return;
        };

        let agent_id = self.composer_agent().map(|agent| agent.id.clone());
        let title = composer_title(&text);
        let prompt = text.clone();
        let captured = title.clone();

        self.sending_composer = true;
        self.composer_status = None;

        let client = self.client.clone();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.create_task(&workspace_id, &title, &prompt, agent_id.as_deref())?;
                    Ok::<_, ApiError>(load_tasks(&client, &workspace_id))
                })
                .await;

            this.update_in(cx, |this, window, cx| {
                this.sending_composer = false;

                match result {
                    Ok(LoadState::Loaded(tasks)) => {
                        this.tasks = LoadState::Loaded(tasks);
                        this.composer
                            .update(cx, |state, cx| state.set_value("", window, cx));
                        this.composer_status = Some(ComposerStatus::Captured(captured));
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.composer_status = Some(ComposerStatus::Failed(message));
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.composer_status = Some(ComposerStatus::Failed(error_message(&error)));
                    }
                }

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
                    client.create_task(&workspace_id, &title, &prompt, None)?;
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

    fn submit_create_decision(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let title = self.decision_title.read(cx).value().to_string();
        let summary = self.decision_summary.read(cx).value().to_string();

        self.decision_error = None;
        self.creating_decision = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.create_decision(&workspace_id, &title, &summary)?;
                    Ok::<_, ApiError>(load_decisions(&client, &workspace_id))
                })
                .await;

            this.update(cx, |this, cx| {
                this.creating_decision = false;

                match result {
                    Ok(LoadState::Loaded(decisions)) => {
                        this.decisions = LoadState::Loaded(decisions);
                        this.show_decision_form = false;
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.decision_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.decision_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn calculate_readiness(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        self.readiness = LoadState::Loading;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let readiness = cx
                .background_executor()
                .spawn(async move {
                    let _ = client.calculate_readiness(&workspace_id);
                    load_readiness(&client, &workspace_id)
                })
                .await;

            this.update(cx, |this, cx| {
                this.readiness = readiness;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn generate_review(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        self.weekly_review = LoadState::Loading;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let review = cx
                .background_executor()
                .spawn(async move {
                    if let Ok(draft) = client.generate_weekly_review(&workspace_id) {
                        let _ = client.create_weekly_review(
                            &workspace_id,
                            &draft.summary,
                            &draft.shipped,
                            &draft.blockers,
                            &draft.rejected_changes,
                            &draft.next_actions,
                            &draft.risky_patterns,
                        );
                    }

                    load_weekly_review(&client, &workspace_id)
                })
                .await;

            this.update(cx, |this, cx| {
                this.weekly_review = review;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn review_list(
        &self,
        label: &str,
        items: &Option<Vec<String>>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let entries = items.clone().unwrap_or_default();

        if entries.is_empty() {
            return div().into_any_element();
        }

        let mut list = v_flex().gap_1().child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(label.to_string()),
        );

        for entry in entries {
            list = list.child(div().text_sm().child(entry));
        }

        list.into_any_element()
    }

    fn create_pairing(&mut self, cx: &mut Context<Self>) {
        self.pairing = LoadState::Loading;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let pairing = cx
                .background_executor()
                .spawn(async move {
                    match client.create_qr_session() {
                        Ok(session) => LoadState::Loaded(Some(session)),
                        Err(ApiError::Unauthorized) => {
                            LoadState::Failed("Sign in to pair a device.".to_string())
                        }
                        Err(ApiError::Message(message)) => LoadState::Failed(message),
                        Err(ApiError::Unreachable) => LoadState::Failed("API offline.".to_string()),
                    }
                })
                .await;

            this.update(cx, |this, cx| {
                this.pairing = pairing;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_qr(&self, value: &str) -> AnyElement {
        let Ok(code) = qrcode::QrCode::new(value.as_bytes()) else {
            return div()
                .text_sm()
                .child("Could not render pairing code")
                .into_any_element();
        };

        let width = code.width();
        let colors = code.to_colors();
        let mut grid = v_flex();

        for row in 0..width {
            let mut line = h_flex();
            for col in 0..width {
                let dark = colors[row * width + col] == qrcode::Color::Dark;
                let color = if dark { rgb(0x000000) } else { rgb(0xffffff) };
                line = line.child(div().size(px(6.0)).bg(color));
            }
            grid = grid.child(line);
        }

        grid.into_any_element()
    }

    fn select_session(&mut self, session_id: String, cx: &mut Context<Self>) {
        if self.selected_session.as_deref() == Some(session_id.as_str()) {
            return;
        }

        self.selected_session = Some(session_id.clone());
        self.session_logs = LoadState::Loading;
        self.session_steps = LoadState::Loading;
        self.step_error = None;

        let steps_client = self.client.clone();
        let steps_session = session_id.clone();
        cx.spawn(async move |this, cx| {
            let steps = cx
                .background_executor()
                .spawn(async move { load_session_steps(&steps_client, &steps_session) })
                .await;

            this.update(cx, |this, cx| {
                this.session_steps = steps;
                cx.notify();
            })
            .ok();
        })
        .detach();

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let mut known = 0usize;

            loop {
                let is_active = this
                    .update(cx, |this, _| {
                        this.selected_session.as_deref() == Some(session_id.as_str())
                    })
                    .unwrap_or(false);

                if !is_active {
                    break;
                }

                let from = known;

                let stream = cx
                    .background_executor()
                    .spawn({
                        let client = client.clone();
                        let session_id = session_id.clone();
                        async move { client.stream_session(&session_id, from) }
                    })
                    .await;

                let appended = this
                    .update(cx, |this, cx| {
                        if this.selected_session.as_deref() != Some(session_id.as_str()) {
                            return None;
                        }

                        match stream {
                            Ok(stream) => {
                                let appended = stream.logs.len();

                                if let LoadState::Loaded(logs) = &mut this.session_logs {
                                    logs.extend(stream.logs);
                                } else {
                                    this.session_logs = LoadState::Loaded(stream.logs);
                                }

                                if let LoadState::Loaded(sessions) = &mut this.sessions {
                                    if let Some(session) = sessions
                                        .iter_mut()
                                        .find(|session| session.id == stream.session.id)
                                    {
                                        session.status = stream.session.status;
                                    }
                                }

                                cx.notify();
                                Some(appended)
                            }
                            Err(error) => {
                                if matches!(this.session_logs, LoadState::Loading) {
                                    this.session_logs = LoadState::Failed(error_message(&error));
                                    cx.notify();
                                }

                                Some(0)
                            }
                        }
                    })
                    .unwrap_or(None);

                let Some(appended) = appended else {
                    break;
                };

                known += appended;

                cx.background_executor()
                    .timer(std::time::Duration::from_secs(2))
                    .await;
            }
        })
        .detach();
    }

    fn submit_create_agent(&mut self, cx: &mut Context<Self>) {
        let name = self.agent_name.read(cx).value().to_string();
        let provider = self.agent_provider.read(cx).value().to_string();
        let command = self.agent_command.read(cx).value().to_string();

        self.agent_error = None;
        self.creating_agent = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.create_agent(&name, &provider, &command)?;
                    Ok::<_, ApiError>(load_agents_list(&client))
                })
                .await;

            this.update(cx, |this, cx| {
                this.creating_agent = false;

                match result {
                    Ok(LoadState::Loaded(agents)) => {
                        this.agents = LoadState::Loaded(agents);
                        this.show_agent_form = false;
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.agent_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.agent_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit_complete_intake(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let project_type = self.intake_project_type.read(cx).value().to_string();
        let package_manager = self.intake_package_manager.read(cx).value().to_string();
        let runtime = self.intake_runtime.read(cx).value().to_string();
        let safe = self.intake_safe_commands.read(cx).value().to_string();
        let ignored = self.intake_ignored_paths.read(cx).value().to_string();
        let mobile_pairing = self.intake_mobile_pairing;
        let safety_mode = self.intake_safety_mode.clone();

        let safe_commands = split_list(&safe);
        let ignored_paths = split_list(&ignored);

        self.intake_error = None;
        self.completing_intake = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.complete_workspace_intake(
                        &workspace_id,
                        &project_type,
                        &package_manager,
                        &runtime,
                        &safe_commands,
                        &ignored_paths,
                        mobile_pairing,
                        &safety_mode,
                    )?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                this.completing_intake = false;

                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.intake_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.intake_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn load_settings_values(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.selected_workspace_data() else {
            return;
        };

        let name = workspace.name.clone();
        let local_path = workspace.local_path.clone();
        let safe_commands = workspace
            .safe_commands
            .clone()
            .unwrap_or_default()
            .join(", ");
        let ignored_paths = workspace
            .ignored_paths
            .clone()
            .unwrap_or_default()
            .join(", ");

        self.settings_name
            .update(cx, |state, cx| state.set_value(name, window, cx));
        self.settings_path
            .update(cx, |state, cx| state.set_value(local_path, window, cx));
        self.settings_safe_commands
            .update(cx, |state, cx| state.set_value(safe_commands, window, cx));
        self.settings_ignored_paths
            .update(cx, |state, cx| state.set_value(ignored_paths, window, cx));

        self.settings_error = None;
        cx.notify();
    }

    fn submit_save_settings(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        let name = self.settings_name.read(cx).value().to_string();
        let local_path = self.settings_path.read(cx).value().to_string();
        let safe = self.settings_safe_commands.read(cx).value().to_string();
        let ignored = self.settings_ignored_paths.read(cx).value().to_string();

        let safe_commands = split_list(&safe);
        let ignored_paths = split_list(&ignored);

        if name.is_empty()
            && local_path.is_empty()
            && safe_commands.is_empty()
            && ignored_paths.is_empty()
        {
            self.settings_error = Some("Enter at least one value to save.".to_string());
            cx.notify();
            return;
        }

        self.settings_error = None;
        self.saving_settings = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.update_workspace(
                        &workspace_id,
                        &name,
                        &local_path,
                        &safe_commands,
                        &ignored_paths,
                    )?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                this.saving_settings = false;

                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.settings_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.settings_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit_archive_workspace(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        self.settings_error = None;
        self.archiving_workspace = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.archive_workspace(&workspace_id)?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                this.archiving_workspace = false;

                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                        this.selected_workspace = None;
                        this.section = Section::Workspaces;
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.settings_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.settings_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn set_default_agent(&mut self, agent_id: Option<String>, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        self.settings_error = None;
        self.saving_settings = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.set_default_agent(&workspace_id, agent_id.as_deref())?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                this.saving_settings = false;

                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.settings_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.settings_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn allow_blocked_command(&mut self, cx: &mut Context<Self>) {
        let (Some(workspace_id), Some(command)) = (
            self.selected_workspace.clone(),
            self.blocked_command.clone(),
        ) else {
            return;
        };

        let safe_commands = self
            .selected_workspace_data()
            .and_then(|workspace| workspace.safe_commands.clone())
            .unwrap_or_default();
        let next = safety::with_safe_command(&safe_commands, &command);

        self.blocked_command = None;
        self.session_error = None;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.set_safe_commands(&workspace_id, &next)?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                        this.session_notice = Some(format!(
                            "Added \"{command}\" to this workspace's safe commands. Press Start again."
                        ));
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.session_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.session_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn toggle_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next = match self.theme_mode {
            ThemeMode::Light => ThemeMode::Dark,
            _ => ThemeMode::Light,
        };

        self.theme_mode = next;
        apply_theme(next, Some(window), cx);
        settings::save_theme_mode(mode_name(next));
        cx.notify();
    }

    fn set_safety_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        self.settings_error = None;
        self.saving_settings = true;

        let mode = mode.to_string();
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.set_safety_mode(&workspace_id, &mode)?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                this.saving_settings = false;

                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.settings_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.settings_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn set_mobile_pairing(&mut self, enabled: bool, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };

        self.settings_error = None;
        self.saving_settings = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.set_mobile_pairing(&workspace_id, enabled)?;
                    let session = client.session();
                    Ok::<_, ApiError>(load_workspaces(&client, &session))
                })
                .await;

            this.update(cx, |this, cx| {
                this.saving_settings = false;

                match result {
                    Ok(LoadState::Loaded(workspaces)) => {
                        this.workspaces = LoadState::Loaded(workspaces);
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.settings_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.settings_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn submit_create_step(&mut self, cx: &mut Context<Self>) {
        let Some(workspace_id) = self.selected_workspace.clone() else {
            return;
        };
        let Some(session_id) = self.selected_session.clone() else {
            return;
        };

        let reason = self.new_step_reason.read(cx).value().to_string();
        let lane = self.new_step_lane.clone();
        let sort_order = match &self.session_steps {
            LoadState::Loaded(steps) => {
                steps.iter().filter(|step| step.lane == lane).count() as i64
            }
            _ => 0,
        };

        self.step_error = None;
        self.creating_step = true;

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    client.create_session_step(
                        &workspace_id,
                        &session_id,
                        &lane,
                        sort_order,
                        &reason,
                    )?;
                    Ok::<_, ApiError>(load_session_steps(&client, &session_id))
                })
                .await;

            this.update(cx, |this, cx| {
                this.creating_step = false;

                match result {
                    Ok(LoadState::Loaded(steps)) => {
                        this.session_steps = LoadState::Loaded(steps);
                    }
                    Ok(LoadState::Failed(message)) => {
                        this.step_error = Some(message);
                    }
                    Ok(LoadState::Loading) => {}
                    Err(error) => {
                        this.step_error = Some(error_message(&error));
                    }
                }

                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_session_timeline(&self, cx: &mut Context<Self>) -> AnyElement {
        let lanes = [
            ("now", "Now"),
            ("next", "Next"),
            ("later", "Later"),
            ("not_doing", "Not doing"),
        ];

        let mut groups = v_flex().gap_3();

        for (lane, label) in lanes {
            let mut group = v_flex().gap_1().child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            );

            let mut count = 0usize;

            if let LoadState::Loaded(all) = &self.session_steps {
                for step in all {
                    if step.lane == lane {
                        count += 1;
                        group = group.child(
                            div().text_sm().child(
                                step.reason
                                    .clone()
                                    .unwrap_or_else(|| "Untitled step".to_string()),
                            ),
                        );
                    }
                }
            }

            if count == 0 {
                group = group.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("—"),
                );
            }

            groups = groups.child(group);
        }

        let mut lane_buttons = h_flex().gap_1();

        for (lane, label) in lanes {
            lane_buttons = lane_buttons.child(
                Button::new(lane)
                    .small()
                    .label(label)
                    .selected(self.new_step_lane == lane)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.new_step_lane = lane.to_string();
                        cx.notify();
                    })),
            );
        }

        let add = Button::new("add-step")
            .small()
            .label(if self.creating_step {
                "Adding…"
            } else {
                "Add step"
            })
            .disabled(self.creating_step)
            .on_click(cx.listener(|this, _, _, cx| this.submit_create_step(cx)));

        let mut form = v_flex().gap_2().child(lane_buttons).child(
            h_flex()
                .gap_2()
                .child(Input::new(&self.new_step_reason))
                .child(add),
        );

        if let Some(error) = &self.step_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        v_flex()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Timeline"),
            )
            .child(groups)
            .child(form)
            .into_any_element()
    }

    /// A sidebar row for a section: icon, active state, and any badge.
    fn nav_item(&self, section: Section, cx: &mut Context<Self>) -> SidebarMenuItem {
        let item = SidebarMenuItem::new(section.label())
            .icon(Icon::new(section.icon()))
            .active(self.section == section);

        let item = match self.nav_badge(section, cx) {
            Some(badge) => item.suffix(badge),
            None => item,
        };

        item.on_click(cx.listener(move |this, _, _, cx| {
            this.section = section;
            cx.notify();
        }))
    }

    /// A badge for the sections that carry work a developer has to act on.
    fn nav_badge(&self, section: Section, cx: &Context<Self>) -> Option<AnyElement> {
        let (count, background, foreground) = match section {
            Section::Inbox => (
                count_where(&self.tasks, |task| task.status == "new"),
                cx.theme().accent,
                cx.theme().accent_foreground,
            ),
            Section::Reviews => (
                count_where(&self.reviews, |review| review.action == "pending"),
                cx.theme().warning,
                cx.theme().warning_foreground,
            ),
            Section::Sessions => (
                count_where(&self.sessions, |session| session.status == "waiting"),
                cx.theme().warning,
                cx.theme().warning_foreground,
            ),
            _ => return None,
        };

        if count == 0 {
            return None;
        }

        Some(
            div()
                .min_w(px(18.0))
                .px_1()
                .rounded(px(9.0))
                .bg(background)
                .text_color(foreground)
                .flex()
                .justify_center()
                .child(div().text_xs().child(count.to_string()))
                .into_any_element(),
        )
    }

    /// Wordmark plus the palette trigger, which is the sidebar's search row.
    fn render_nav_header(&self, cx: &mut Context<Self>) -> AnyElement {
        let shortcut = if cfg!(target_os = "macos") {
            "⌘K"
        } else {
            "Ctrl K"
        };

        let search = h_flex()
            .id("sidebar-search")
            .w_full()
            .gap_2()
            .items_center()
            .px_2()
            .py_1()
            .rounded(px(6.0))
            .bg(cx.theme().sidebar_accent)
            .text_color(cx.theme().muted_foreground)
            .hover(|element| element.text_color(cx.theme().sidebar_accent_foreground))
            .on_click(cx.listener(|this, _, window, cx| this.open_palette(window, cx)))
            .child(Icon::new(IconName::Search).size_4())
            .child(div().flex_1().text_sm().child("Search everything"))
            .child(div().text_xs().child(shortcut));

        v_flex()
            .w_full()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Kloucy"),
            )
            .child(search)
            .into_any_element()
    }

    /// Who is signed in, how the API looks, and the way out.
    fn render_nav_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        let (email, name, status, status_color) = match &self.session {
            SessionStatus::Checking => (
                None,
                "Kloucy".to_string(),
                "Checking API…".to_string(),
                cx.theme().muted_foreground,
            ),
            SessionStatus::SignedIn(user) => (
                Some(user.email.clone()),
                user.name.clone(),
                "API ok".to_string(),
                cx.theme().success_foreground,
            ),
            SessionStatus::SignedOut => (
                None,
                "Kloucy".to_string(),
                "Not signed in".to_string(),
                cx.theme().warning_foreground,
            ),
            SessionStatus::Unreachable => (
                None,
                "Kloucy".to_string(),
                "API offline".to_string(),
                cx.theme().danger_foreground,
            ),
        };

        let avatar = div()
            .size_7()
            .flex_shrink_0()
            .rounded(px(999.0))
            .bg(cx.theme().sidebar_accent)
            .flex()
            .items_center()
            .justify_center()
            .child(div().text_xs().child(match &email {
                Some(_) => initials(&name),
                None => "K".to_string(),
            }));

        let mut row = h_flex()
            .w_full()
            .gap_2()
            .items_center()
            .child(avatar)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(div().text_sm().truncate().child(email.unwrap_or(name)))
                    .child(
                        h_flex()
                            .gap_1()
                            .items_center()
                            .child(div().size(px(6.0)).rounded(px(3.0)).bg(status_color))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(status),
                            ),
                    ),
            );

        let theme_icon = match self.theme_mode {
            ThemeMode::Light => IconName::Moon,
            _ => IconName::Sun,
        };

        row = row.child(
            Button::new("toggle-theme")
                .ghost()
                .small()
                .icon(Icon::new(theme_icon))
                .on_click(cx.listener(|this, _, window, cx| this.toggle_theme(window, cx))),
        );

        if matches!(self.session, SessionStatus::SignedIn(_)) {
            row = row.child(
                Button::new("sign-out")
                    .ghost()
                    .small()
                    .label("Sign out")
                    .on_click(cx.listener(|this, _, _, cx| this.submit_sign_out(cx))),
            );
        }

        row.into_any_element()
    }

    fn render_nav(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut groups = Vec::new();

        for (label, sections) in NAV_GROUPS {
            let mut menu = SidebarMenu::new();

            for section in sections {
                menu = menu.child(self.nav_item(*section, cx));
            }

            groups.push(SidebarGroup::new(label).child(menu));
        }

        let mut sidebar = Sidebar::left().header(self.render_nav_header(cx));

        for group in groups {
            sidebar = sidebar.child(group);
        }

        sidebar.footer(self.render_nav_footer(cx))
    }

    fn render_intake(&self, cx: &mut Context<Self>) -> AnyElement {
        let submit = Button::new("complete-intake")
            .primary()
            .small()
            .label(if self.completing_intake {
                "Saving…"
            } else {
                "Complete intake"
            })
            .disabled(self.completing_intake)
            .on_click(cx.listener(|this, _, _, cx| this.submit_complete_intake(cx)));

        let detect = Button::new("detect-intake-project")
            .small()
            .label("Detect project")
            .on_click(cx.listener(|this, _, window, cx| this.detect_intake_project(window, cx)));

        let mobile_pairing = Button::new("intake-mobile-pairing")
            .small()
            .label(if self.intake_mobile_pairing {
                "Mobile pairing allowed"
            } else {
                "Desktop only"
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.intake_mobile_pairing = !this.intake_mobile_pairing;
                cx.notify();
            }));

        let safety_mode = Button::new("intake-safety-mode")
            .small()
            .label(if self.intake_safety_mode == "guarded" {
                "Guarded: safe commands only"
            } else {
                "Open: any command"
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.intake_safety_mode = if this.intake_safety_mode == "guarded" {
                    "open".to_string()
                } else {
                    "guarded".to_string()
                };
                cx.notify();
            }));

        let mut form = v_flex()
            .w(px(360.0))
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Workspace intake"),
            )
            .child(Input::new(&self.intake_project_type))
            .child(Input::new(&self.intake_package_manager))
            .child(Input::new(&self.intake_runtime))
            .child(Input::new(&self.intake_safe_commands))
            .child(Input::new(&self.intake_ignored_paths))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(detect)
                    .child(mobile_pairing),
            )
            .child(safety_mode)
            .child(submit);

        if let Some(error) = &self.intake_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        form.into_any_element()
    }

    fn detect_intake_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(workspace) = self.selected_workspace_data() else {
            return;
        };

        let files = detect::read_project_files(&workspace.local_path);
        let Some(profile) = detect::detect_project(&files) else {
            self.intake_error = Some("Could not recognise the project from its files.".to_string());
            cx.notify();
            return;
        };

        self.intake_error = None;

        if let Some(project_type) = profile.project_type {
            self.intake_project_type
                .update(cx, |state, cx| state.set_value(project_type, window, cx));
        }
        if let Some(package_manager) = profile.package_manager {
            self.intake_package_manager
                .update(cx, |state, cx| state.set_value(package_manager, window, cx));
        }
        if let Some(runtime) = profile.runtime {
            self.intake_runtime
                .update(cx, |state, cx| state.set_value(runtime, window, cx));
        }
        if !profile.safe_commands.is_empty() {
            let commands = profile.safe_commands.join(", ");
            self.intake_safe_commands
                .update(cx, |state, cx| state.set_value(commands, window, cx));
        }
        if !profile.ignored_paths.is_empty() {
            let paths = profile.ignored_paths.join(", ");
            self.intake_ignored_paths
                .update(cx, |state, cx| state.set_value(paths, window, cx));
        }

        cx.notify();
    }

    fn selected_workspace_data(&self) -> Option<&Workspace> {
        let id = self.selected_workspace.as_deref()?;

        match &self.workspaces {
            LoadState::Loaded(workspaces) => workspaces.iter().find(|workspace| workspace.id == id),
            _ => None,
        }
    }

    fn settings_value(
        &self,
        label: &str,
        value: String,
        mono: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut row = div().text_sm();

        if mono {
            row = row.font_family(self.mono_font.clone());
        }

        h_flex()
            .gap_2()
            .child(
                div()
                    .w(px(120.0))
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(row.child(if value.is_empty() {
                "None".to_string()
            } else {
                value
            }))
            .into_any_element()
    }

    fn render_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(workspace) = self.selected_workspace_data() else {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to correct its setup data.")
                .into_any_element();
        };

        let default_agent = workspace
            .default_agent_id
            .as_deref()
            .and_then(|agent_id| match &self.agents {
                LoadState::Loaded(agents) => agents
                    .iter()
                    .find(|agent| agent.id == agent_id)
                    .map(|agent| agent.name.clone()),
                _ => None,
            })
            .unwrap_or_default();

        let name = workspace.name.clone();
        let local_path = workspace.local_path.clone();
        let safe_commands = workspace
            .safe_commands
            .clone()
            .unwrap_or_default()
            .join(", ");
        let ignored_paths = workspace
            .ignored_paths
            .clone()
            .unwrap_or_default()
            .join(", ");
        let mobile_pairing = if workspace.mobile_pairing_enabled {
            "Shared with mobile"
        } else {
            "Desktop only"
        };
        let mobile_pairing_enabled = workspace.mobile_pairing_enabled;
        let safety_mode = if workspace.safety_mode == "guarded" {
            "Guarded"
        } else {
            "Open"
        };
        let safety_mode_value = workspace.safety_mode.clone();

        let mut agent_picker = v_flex().gap_1().child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Set default agent"),
        );

        match &self.agents {
            LoadState::Loaded(agents) if agents.is_empty() => {
                agent_picker = agent_picker.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Register or detect an agent first."),
                );
            }
            LoadState::Loaded(agents) => {
                for (index, agent) in agents.iter().enumerate() {
                    let agent_id = agent.id.clone();
                    let is_default =
                        workspace.default_agent_id.as_deref() == Some(agent.id.as_str());

                    agent_picker = agent_picker.child(
                        h_flex()
                            .gap_2()
                            .child(div().text_sm().child(agent.name.clone()))
                            .child(
                                Button::new(("default-agent", index))
                                    .small()
                                    .label(if is_default {
                                        "Default"
                                    } else {
                                        "Set as default"
                                    })
                                    .disabled(is_default)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.set_default_agent(Some(agent_id.clone()), cx);
                                    })),
                            ),
                    );
                }

                if workspace.default_agent_id.is_some() {
                    agent_picker = agent_picker.child(
                        Button::new("clear-default-agent")
                            .small()
                            .label("Clear default")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.set_default_agent(None, cx);
                            })),
                    );
                }
            }
            _ => {}
        }

        let load = Button::new("load-settings")
            .small()
            .label("Load current values")
            .on_click(cx.listener(|this, _, window, cx| this.load_settings_values(window, cx)));

        let safety_mode_toggle = Button::new("toggle-safety-mode")
            .small()
            .label(if safety_mode_value == "guarded" {
                "Allow any command"
            } else {
                "Guard commands"
            })
            .disabled(self.saving_settings)
            .on_click(cx.listener(move |this, _, _, cx| {
                let next = if safety_mode_value == "guarded" {
                    "open"
                } else {
                    "guarded"
                };

                this.set_safety_mode(next, cx);
            }));

        let mobile_pairing_toggle = Button::new("toggle-mobile-pairing")
            .small()
            .label(if mobile_pairing_enabled {
                "Stop sharing"
            } else {
                "Share with mobile"
            })
            .disabled(self.saving_settings)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_mobile_pairing(!mobile_pairing_enabled, cx);
            }));

        let save = Button::new("save-settings")
            .primary()
            .small()
            .label(if self.saving_settings {
                "Saving…"
            } else {
                "Save changes"
            })
            .disabled(self.saving_settings)
            .on_click(cx.listener(|this, _, _, cx| this.submit_save_settings(cx)));

        let archive = Button::new("archive-workspace")
            .small()
            .label(if self.archiving_workspace {
                "Archiving…"
            } else {
                "Archive workspace"
            })
            .disabled(self.archiving_workspace)
            .on_click(cx.listener(|this, _, _, cx| this.submit_archive_workspace(cx)));

        let mut form = v_flex()
            .w(px(360.0))
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Workspace settings"),
            )
            .child(self.settings_value("Name", name, false, cx))
            .child(self.settings_value("Local path", local_path, true, cx))
            .child(self.settings_value("Default agent", default_agent, false, cx))
            .child(agent_picker)
            .child(self.settings_value("Safe commands", safe_commands, true, cx))
            .child(self.settings_value("Ignored paths", ignored_paths, true, cx))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(self.settings_value(
                        "Mobile pairing",
                        mobile_pairing.to_string(),
                        false,
                        cx,
                    ))
                    .child(mobile_pairing_toggle),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(self.settings_value("Safety mode", safety_mode.to_string(), false, cx))
                    .child(safety_mode_toggle),
            )
            .child(load)
            .child(Input::new(&self.settings_name))
            .child(Input::new(&self.settings_path))
            .child(Input::new(&self.settings_safe_commands))
            .child(Input::new(&self.settings_ignored_paths))
            .child(h_flex().gap_2().child(save).child(archive));

        if let Some(error) = &self.settings_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        form.into_any_element()
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
                                    .font_family(self.mono_font.clone())
                                    .child(workspace.local_path.clone()),
                            ),
                    );
                }

                rows.into_any_element()
            }
        };

        let mut section = v_flex().gap_4().child(new_button).child(content);

        if self.selected_workspace.is_some() {
            section = section.child(self.render_intake(cx));
        }

        section.into_any_element()
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

    fn detect_agents(&mut self, cx: &mut Context<Self>) {
        self.detected_agents = Some(detect::detect_installed_agents());
        self.show_agent_form = false;
        self.agent_error = None;
        cx.notify();
    }

    fn register_detected_agent(
        &mut self,
        agent: DetectedAgent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_agent_form = true;
        self.agent_error = None;

        self.agent_name
            .update(cx, |state, cx| state.set_value(agent.name, window, cx));
        self.agent_provider
            .update(cx, |state, cx| state.set_value(agent.provider, window, cx));
        self.agent_command.update(cx, |state, cx| {
            state.set_value(agent.executable_path, window, cx)
        });

        cx.notify();
    }

    fn render_create_agent(&self, cx: &mut Context<Self>) -> AnyElement {
        let submit = Button::new("create-agent")
            .primary()
            .label(if self.creating_agent {
                "Creating…"
            } else {
                "Create agent"
            })
            .disabled(self.creating_agent)
            .on_click(cx.listener(|this, _, _, cx| this.submit_create_agent(cx)));

        let cancel = Button::new("cancel-agent")
            .label("Cancel")
            .disabled(self.creating_agent)
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_agent_form = false;
                this.agent_error = None;
                cx.notify();
            }));

        let mut form = v_flex()
            .w(px(360.0))
            .gap_3()
            .child(Input::new(&self.agent_name))
            .child(Input::new(&self.agent_provider))
            .child(Input::new(&self.agent_command))
            .child(h_flex().gap_2().child(submit).child(cancel));

        if let Some(error) = &self.agent_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        form.into_any_element()
    }

    fn render_agents(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.show_agent_form {
            return self.render_create_agent(cx);
        }

        let new_button = Button::new("new-agent")
            .small()
            .label("New agent")
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_agent_form = true;
                this.agent_error = None;
                cx.notify();
            }));

        let detect_button = Button::new("detect-agents")
            .small()
            .label("Detect installed agents")
            .on_click(cx.listener(|this, _, _, cx| this.detect_agents(cx)));

        let content: AnyElement = match &self.agents {
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
        };

        let mut section = v_flex()
            .gap_4()
            .child(h_flex().gap_2().child(new_button).child(detect_button))
            .child(content);

        if let Some(detected) = &self.detected_agents {
            section = section.child(self.render_detected_agents(detected, cx));
        }

        section.into_any_element()
    }

    fn render_detected_agents(
        &self,
        detected: &[DetectedAgent],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut list = v_flex().gap_1().child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Detected on your PATH"),
        );

        if detected.is_empty() {
            return list
                .child(
                    div()
                        .text_sm()
                        .child("No known agent CLI was found on your PATH."),
                )
                .into_any_element();
        }

        for (index, agent) in detected.iter().enumerate() {
            let detected_agent = agent.clone();

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
                            .font_family(self.mono_font.clone())
                            .child(agent.executable_path.clone()),
                    )
                    .child(
                        Button::new(("add-detected-agent", index))
                            .small()
                            .label("Add")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.register_detected_agent(detected_agent.clone(), window, cx);
                            })),
                    ),
            );
        }

        list.into_any_element()
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

        let content: AnyElement = match &self.sessions {
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
                    let logs_id = session.id.clone();

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
                                    )
                                    .child(
                                        Button::new(("session-logs", index))
                                            .small()
                                            .label("Logs")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.select_session(logs_id.clone(), cx);
                                            })),
                                    ),
                            ),
                    );
                }

                list.into_any_element()
            }
        };

        let mut section = v_flex().gap_6().child(content);

        if let Some(command) = &self.blocked_command {
            let allow = Button::new("allow-blocked-command")
                .small()
                .label("Add to safe commands")
                .on_click(cx.listener(|this, _, _, cx| this.allow_blocked_command(cx)));

            section = section.child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().danger_foreground)
                            .child(format!(
                                "This workspace is guarded, so \"{command}\" is not allowed to start. Add it to the safe commands, or allow any command in Settings."
                            )),
                    )
                    .child(allow),
            );
        }

        if let Some(notice) = &self.session_notice {
            section = section.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().success_foreground)
                    .child(notice.clone()),
            );
        }

        if let Some(error) = &self.session_error {
            section = section.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        section
            .child(self.render_session_logs(cx))
            .into_any_element()
    }

    fn render_session_logs(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_session.is_none() {
            return div().into_any_element();
        }

        let title = div()
            .text_sm()
            .font_weight(FontWeight::SEMIBOLD)
            .child("Session logs");

        let content: AnyElement = match &self.session_logs {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading logs…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(logs) if logs.is_empty() => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("No output yet.")
                .into_any_element(),
            LoadState::Loaded(logs) => {
                let mut list = v_flex().gap_1();

                for (index, log) in logs.iter().enumerate() {
                    let color = if log.stream == "stderr" {
                        cx.theme().danger_foreground
                    } else {
                        cx.theme().muted_foreground
                    };

                    list = list.child(
                        div()
                            .text_xs()
                            .text_color(color)
                            .font_family(self.mono_font.clone())
                            .child(format!("{} {}", index + 1, log.message)),
                    );
                }

                list.into_any_element()
            }
        };

        let send = Button::new("send-instruction")
            .small()
            .label(if self.sending_instruction {
                "Sending…"
            } else {
                "Send"
            })
            .disabled(self.sending_instruction)
            .on_click(cx.listener(|this, _, _, cx| this.submit_instruction(cx)));

        let mut panel = v_flex()
            .gap_3()
            .child(title)
            .child(content)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Input::new(&self.instruction))
                    .child(send),
            )
            .child(self.render_session_timeline(cx));

        if let Some(error) = &self.instruction_error {
            panel = panel.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        panel.into_any_element()
    }

    fn render_reviews(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_workspace.is_none() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to see its reviews.")
                .into_any_element();
        }

        let content: AnyElement = match &self.reviews {
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
                    let follow_up_id = review.id.clone();
                    let follow_up_reason = review.reason.clone();
                    let reject_reason = review.reason.clone();

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
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(review.reason.clone().unwrap_or_default()),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Button::new(("review-approve", index))
                                            .small()
                                            .label("Approve")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.approve_review(approve_id.clone(), cx);
                                            })),
                                    )
                                    .child(
                                        Button::new(("review-reject", index))
                                            .small()
                                            .label("Reject")
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.open_review_reason(
                                                    reject_id.clone(),
                                                    ReviewReason::Reject,
                                                    reject_reason.clone(),
                                                    window,
                                                    cx,
                                                );
                                            })),
                                    )
                                    .child(
                                        Button::new(("review-follow-up", index))
                                            .small()
                                            .label("Follow up")
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.open_review_reason(
                                                    follow_up_id.clone(),
                                                    ReviewReason::FollowUp,
                                                    follow_up_reason.clone(),
                                                    window,
                                                    cx,
                                                );
                                            })),
                                    ),
                            ),
                    );
                }

                list.into_any_element()
            }
        };

        let mut section = v_flex().gap_6().child(content);

        if let Some(error) = &self.review_error {
            section = section.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        if let Some((_, kind)) = &self.review_reason_for {
            section = section.child(self.render_review_reason(*kind, cx));
        }

        section.into_any_element()
    }

    fn render_review_reason(&self, kind: ReviewReason, cx: &mut Context<Self>) -> AnyElement {
        let submit = Button::new("save-review-reason")
            .primary()
            .small()
            .label(if self.saving_review_reason {
                "Saving…"
            } else {
                kind.submit_label()
            })
            .disabled(self.saving_review_reason)
            .on_click(cx.listener(|this, _, _, cx| this.submit_review_reason(cx)));

        let cancel = Button::new("cancel-review-reason")
            .small()
            .label("Cancel")
            .disabled(self.saving_review_reason)
            .on_click(cx.listener(|this, _, _, cx| this.cancel_review_reason(cx)));

        v_flex()
            .w(px(360.0))
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(kind.title()),
            )
            .child(Input::new(&self.review_reason))
            .child(h_flex().gap_2().child(submit).child(cancel))
            .into_any_element()
    }

    fn render_create_decision(&self, cx: &mut Context<Self>) -> AnyElement {
        let submit = Button::new("create-decision")
            .primary()
            .label(if self.creating_decision {
                "Saving…"
            } else {
                "Add decision"
            })
            .disabled(self.creating_decision)
            .on_click(cx.listener(|this, _, _, cx| this.submit_create_decision(cx)));

        let cancel = Button::new("cancel-decision")
            .label("Cancel")
            .disabled(self.creating_decision)
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_decision_form = false;
                this.decision_error = None;
                cx.notify();
            }));

        let mut form = v_flex()
            .w(px(360.0))
            .gap_3()
            .child(Input::new(&self.decision_title))
            .child(Input::new(&self.decision_summary))
            .child(h_flex().gap_2().child(submit).child(cancel));

        if let Some(error) = &self.decision_error {
            form = form.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger_foreground)
                    .child(error.clone()),
            );
        }

        form.into_any_element()
    }

    fn render_decisions(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_workspace.is_none() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to see its decisions.")
                .into_any_element();
        }

        if self.show_decision_form {
            return self.render_create_decision(cx);
        }

        let add_button = Button::new("new-decision")
            .small()
            .label("Add decision")
            .on_click(cx.listener(|this, _, _, cx| {
                this.show_decision_form = true;
                this.decision_error = None;
                cx.notify();
            }));

        let content: AnyElement = match &self.decisions {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading decisions…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(decisions) if decisions.is_empty() => v_flex()
                .gap_1()
                .child(div().text_sm().child("No decisions yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Record scope, permission, run, or review decisions here."),
                )
                .into_any_element(),
            LoadState::Loaded(decisions) => {
                let mut list = v_flex().gap_3();

                for decision in decisions {
                    list = list.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().child(decision.title.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(decision.summary.clone().unwrap_or_default()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(decision.decision_type.clone()),
                            ),
                    );
                }

                list.into_any_element()
            }
        };

        v_flex()
            .gap_4()
            .child(add_button)
            .child(content)
            .into_any_element()
    }

    fn render_readiness(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_workspace.is_none() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to check readiness.")
                .into_any_element();
        }

        let calculate = Button::new("calculate-readiness")
            .small()
            .label("Calculate")
            .on_click(cx.listener(|this, _, _, cx| this.calculate_readiness(cx)));

        let content: AnyElement = match &self.readiness {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading readiness…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(None) => v_flex()
                .gap_1()
                .child(div().text_sm().child("No readiness check yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Calculate readiness to see blockers before running agents."),
                )
                .into_any_element(),
            LoadState::Loaded(Some(snapshot)) => {
                let mut body = v_flex().gap_3().child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_2xl()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(snapshot.score.to_string()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(snapshot.status.clone()),
                        ),
                );

                let blockers = snapshot.blockers.clone().unwrap_or_default();
                if !blockers.is_empty() {
                    let mut list = v_flex().gap_1().child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Blockers"),
                    );
                    for blocker in blockers {
                        list = list.child(div().text_sm().child(blocker));
                    }
                    body = body.child(list);
                }

                let not_required = snapshot.not_required_yet.clone().unwrap_or_default();
                if !not_required.is_empty() {
                    let mut list = v_flex().gap_1().child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Not required yet"),
                    );
                    for item in not_required {
                        list = list.child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(item),
                        );
                    }
                    body = body.child(list);
                }

                body.into_any_element()
            }
        };

        v_flex()
            .gap_4()
            .child(calculate)
            .child(content)
            .into_any_element()
    }

    fn render_weekly_review(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.selected_workspace.is_none() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Select a workspace to review its week.")
                .into_any_element();
        }

        let generate = Button::new("generate-review")
            .small()
            .label("Generate review")
            .on_click(cx.listener(|this, _, _, cx| this.generate_review(cx)));

        let content: AnyElement = match &self.weekly_review {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Loading weekly review…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(None) => v_flex()
                .gap_1()
                .child(div().text_sm().child("No weekly review yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Generate a review from this week's sessions and reviews."),
                )
                .into_any_element(),
            LoadState::Loaded(Some(review)) => {
                let mut body = v_flex().gap_3();

                if let Some(summary) = &review.summary {
                    body = body.child(div().text_sm().child(summary.clone()));
                }

                body = body
                    .child(self.review_list("Shipped", &review.shipped, cx))
                    .child(self.review_list("Blockers", &review.blockers, cx))
                    .child(self.review_list("Rejected changes", &review.rejected_changes, cx))
                    .child(self.review_list("Risky patterns", &review.risky_patterns, cx))
                    .child(self.review_list("Next actions", &review.next_actions, cx));

                body.into_any_element()
            }
        };

        v_flex()
            .gap_4()
            .child(generate)
            .child(content)
            .into_any_element()
    }

    fn render_pairing(&self, cx: &mut Context<Self>) -> AnyElement {
        let new_code = Button::new("new-pairing-code")
            .small()
            .label("New pairing code")
            .on_click(cx.listener(|this, _, _, cx| this.create_pairing(cx)));

        let content: AnyElement = match &self.pairing {
            LoadState::Loading => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Creating pairing code…")
                .into_any_element(),
            LoadState::Failed(message) => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(message.clone())
                .into_any_element(),
            LoadState::Loaded(None) => v_flex()
                .gap_1()
                .child(div().text_sm().child("No pairing code yet."))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("Scan the code from the Kloucy mobile app to pair a phone."),
                )
                .into_any_element(),
            LoadState::Loaded(Some(session)) => v_flex()
                .gap_3()
                .child(self.render_qr(&format!("kloucy://pair?token={}", session.token)))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Status: {}", session.status)),
                )
                .into_any_element(),
        };

        v_flex()
            .gap_4()
            .child(new_code)
            .child(content)
            .into_any_element()
    }

    /// Home: capture work in one line, with the checkout and agent it belongs to
    /// visible before it is captured.
    fn render_home(&self, cx: &mut Context<Self>) -> AnyElement {
        let workspace = self.selected_workspace_data();
        let workspace_name = workspace
            .map(|workspace| workspace.name.clone())
            .unwrap_or_else(|| "No workspace".to_string());
        let safety = workspace.map(|workspace| workspace.safety_mode.clone());
        let has_workspace = workspace.is_some();
        let checkout = self.home_checkout.clone();
        let account = match &self.session {
            SessionStatus::SignedIn(user) => user.name.clone(),
            _ => "Kloucy".to_string(),
        };
        let agent_name = self
            .composer_agent()
            .map(|agent| agent.name.clone())
            .unwrap_or_else(|| "No agent".to_string());

        let chips = h_flex()
            .w_full()
            .items_center()
            .justify_end()
            .gap_1()
            .child(
                self.home_popover(
                    "home-workspace",
                    Self::chip(
                        Icon::new(IconName::Folder)
                            .size(px(14.0))
                            .into_any_element(),
                        workspace_name,
                        cx,
                    ),
                    HomeMenu::Workspace,
                    cx,
                ),
            )
            .child(
                self.home_popover(
                    "home-account",
                    Self::chip(
                        Icon::new(IconName::CircleUser)
                            .size(px(14.0))
                            .into_any_element(),
                        account,
                        cx,
                    ),
                    HomeMenu::Account,
                    cx,
                ),
            );

        let attach = h_flex()
            .id("composer-attach")
            .size_7()
            .items_center()
            .justify_center()
            .rounded(px(6.0))
            .text_color(cx.theme().muted_foreground)
            .hover(|element| {
                element
                    .bg(cx.theme().list_hover)
                    .text_color(cx.theme().foreground)
            })
            .child(Icon::empty().path("icons/paperclip.svg").size(px(16.0)))
            .into_any_element();

        let agent_trigger = h_flex()
            .id("composer-agent")
            .items_center()
            .gap(px(6.0))
            .rounded(px(6.0))
            .px(px(6.0))
            .py(px(3.0))
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .hover(|element| {
                element
                    .bg(cx.theme().list_hover)
                    .text_color(cx.theme().foreground)
            })
            .child(Icon::new(IconName::Bot).size(px(14.0)))
            .child(div().child(agent_name))
            .when_some(safety, |element, mode| {
                element.child(div().opacity(0.7).child(safety_label(&mode)))
            })
            .into_any_element();

        let can_send = !self.composer.read(cx).value().trim().is_empty();
        let send = div()
            .id("composer-send")
            .size(px(30.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(15.0))
            .bg(cx.theme().foreground)
            .text_color(cx.theme().background)
            .when(!can_send, |element| element.opacity(0.4))
            .when(can_send, |element| {
                element.hover(|element| element.opacity(0.85))
            })
            .on_click(cx.listener(|this, _, window, cx| this.submit_composer(window, cx)))
            .child(Icon::new(IconName::ArrowUp).size(px(16.0)));

        let card = v_flex()
            .w_full()
            .rounded(px(16.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .p(px(14.0))
            .gap_4()
            .child(
                Input::new(&self.composer)
                    .appearance(false)
                    .bordered(false)
                    .focus_bordered(false),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(self.home_popover("composer-attach", attach, HomeMenu::Attach, cx))
                    .child(
                        h_flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(self.home_popover(
                                "composer-agent",
                                agent_trigger,
                                HomeMenu::Agent,
                                cx,
                            ))
                            .child(send),
                    ),
            );

        let mut checkout_row = h_flex().items_center().gap_1().child(
            self.home_popover(
                "home-checkout",
                Self::chip(
                    Icon::new(IconName::Folder)
                        .size(px(14.0))
                        .into_any_element(),
                    "Current checkout".to_string(),
                    cx,
                ),
                HomeMenu::Checkout,
                cx,
            ),
        );

        if has_workspace {
            checkout_row = checkout_row.child(
                self.home_popover(
                    "home-branch",
                    Self::chip(
                        Icon::empty()
                            .path("icons/git-branch.svg")
                            .size(px(14.0))
                            .into_any_element(),
                        checkout.label().to_string(),
                        cx,
                    ),
                    HomeMenu::Branch,
                    cx,
                ),
            );
        }

        let mut details = v_flex().gap_1().px(px(6.0)).child(checkout_row);

        match &self.composer_status {
            Some(ComposerStatus::Captured(title)) => {
                details = details.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().success_foreground)
                        .child(format!("Captured in Inbox: {title}")),
                );
            }
            Some(ComposerStatus::Notice(message)) => {
                details = details.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(message.clone()),
                );
            }
            Some(ComposerStatus::Failed(message)) => {
                details = details.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().danger_foreground)
                        .child(message.clone()),
                );
            }
            None => {}
        }

        if !has_workspace {
            details = details.child(
                h_flex()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("No checkout yet."),
                    )
                    .child(
                        Button::new("home-new-workspace")
                            .ghost()
                            .small()
                            .label("New workspace")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.section = Section::Workspaces;
                                this.show_create_form = true;
                                this.create_error = None;
                                cx.notify();
                            })),
                    ),
            );
        }

        if let Some(activity) = session_activity(&self.sessions) {
            details = details.child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(activity),
                    )
                    .child(
                        Button::new("home-open-sessions")
                            .ghost()
                            .small()
                            .label("Open sessions")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.section = Section::Sessions;
                                cx.notify();
                            })),
                    ),
            );
        }

        // The composer sits in the middle of the surface; the chips keep the top
        // right corner, so the work has one clear focus.
        div()
            .size_full()
            .relative()
            .child(div().absolute().top(px(20.0)).right(px(24.0)).child(chips))
            .child(
                v_flex()
                    .size_full()
                    .items_center()
                    .justify_center()
                    .px(px(24.0))
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(720.0))
                            .gap_3()
                            .child(card)
                            .child(details),
                    ),
            )
            .into_any_element()
    }

    /// The chip shape the home selectors share: icon, label, and the chevron
    /// that says a menu opens.
    fn chip(icon: AnyElement, label: String, cx: &App) -> AnyElement {
        h_flex()
            .items_center()
            .gap(px(6.0))
            .rounded(px(6.0))
            .px(px(6.0))
            .py(px(4.0))
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .hover(|element| {
                element
                    .bg(cx.theme().list_hover)
                    .text_color(cx.theme().foreground)
            })
            .child(icon)
            .child(div().min_w_0().max_w(px(220.0)).truncate().child(label))
            .child(Icon::new(IconName::ChevronDown).size(px(12.0)))
            .into_any_element()
    }

    /// A popover whose trigger is a home chip. The open state lives here so that
    /// picking an item closes the menu.
    fn home_popover(
        &self,
        id: &'static str,
        trigger: AnyElement,
        menu: HomeMenu,
        cx: &mut Context<Self>,
    ) -> Popover {
        let open = self.home_menu == Some(menu);
        let entity = cx.entity().downgrade();
        let panel = self.home_menu_panel(menu, cx);

        Popover::new(id)
            .anchor(Corner::TopRight)
            .open(open)
            .on_open_change(move |is_open, _window, cx| {
                let _ = entity.update(cx, |this, cx| {
                    this.home_menu = if *is_open {
                        Some(menu)
                    } else if this.home_menu == Some(menu) {
                        None
                    } else {
                        this.home_menu
                    };
                    cx.notify();
                });
            })
            .trigger(Chip {
                selected: open,
                element: trigger,
            })
            .content(move |_, _, cx| panel.clone()(cx))
    }

    /// A menu panel is rebuilt on every frame the popover is open, so each one is
    /// handed over as a factory of the data it needs.
    fn home_menu_panel(
        &self,
        menu: HomeMenu,
        cx: &mut Context<Self>,
    ) -> Rc<dyn Fn(&App) -> AnyElement> {
        let entity = cx.entity().downgrade();

        match menu {
            HomeMenu::Workspace | HomeMenu::Checkout => {
                let workspaces = match &self.workspaces {
                    LoadState::Loaded(workspaces) => workspaces.clone(),
                    _ => Vec::new(),
                };
                let selected = self.selected_workspace.clone();

                Rc::new(move |cx| {
                    if workspaces.is_empty() {
                        return menu_note("No workspaces yet.", cx);
                    }

                    let mut list = v_flex().w(px(260.0)).gap(px(2.0));

                    for (index, workspace) in workspaces.iter().enumerate() {
                        let id = workspace.id.clone();
                        let is_selected = selected.as_deref() == Some(workspace.id.as_str());
                        let folder = folder_name(&workspace.local_path);
                        let mut children = vec![
                            Icon::new(IconName::Folder)
                                .size(px(14.0))
                                .into_any_element(),
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .child(workspace.name.clone())
                                .into_any_element(),
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(folder)
                                .into_any_element(),
                        ];

                        if is_selected {
                            children
                                .push(Icon::new(IconName::Check).size(px(14.0)).into_any_element());
                        }

                        let entity = entity.clone();

                        list = list.child(menu_row(
                            ("home-workspace-item", index),
                            cx,
                            is_selected,
                            children,
                            Some(Box::new(move |_, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.select_workspace(id.clone(), cx);
                                    this.home_menu = None;
                                    cx.notify();
                                });
                            })),
                        ));
                    }

                    list.into_any_element()
                })
            }
            HomeMenu::Agent => {
                let agents = match &self.agents {
                    LoadState::Loaded(agents) => agents.clone(),
                    _ => Vec::new(),
                };
                let selected = self.composer_agent().map(|agent| agent.id.clone());

                Rc::new(move |cx| {
                    let mut list = v_flex().w(px(260.0)).gap(px(2.0));

                    if agents.is_empty() {
                        list = list.child(menu_note("No agents registered yet.", cx));

                        let entity = entity.clone();

                        list = list.child(menu_row(
                            ("home-agent-register", 0),
                            cx,
                            false,
                            vec![
                                Icon::new(IconName::Plus).size(px(14.0)).into_any_element(),
                                div().child("Register an agent").into_any_element(),
                            ],
                            Some(Box::new(move |_, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.section = Section::Agents;
                                    this.show_agent_form = true;
                                    this.agent_error = None;
                                    this.home_menu = None;
                                    cx.notify();
                                });
                            })),
                        ));
                    }

                    for (index, agent) in agents.iter().enumerate() {
                        let id = agent.id.clone();
                        let is_selected = selected.as_deref() == Some(agent.id.as_str());
                        let mut children = vec![
                            Icon::new(IconName::Bot).size(px(14.0)).into_any_element(),
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .child(agent.name.clone())
                                .into_any_element(),
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(agent.provider.clone())
                                .into_any_element(),
                        ];

                        if is_selected {
                            children
                                .push(Icon::new(IconName::Check).size(px(14.0)).into_any_element());
                        }

                        let entity = entity.clone();

                        list = list.child(menu_row(
                            ("home-agent-item", index),
                            cx,
                            is_selected,
                            children,
                            Some(Box::new(move |_, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.home_agent = Some(id.clone());
                                    this.home_menu = None;
                                    this.composer_status = None;
                                    cx.notify();
                                });
                            })),
                        ));
                    }

                    list.into_any_element()
                })
            }
            HomeMenu::Account => {
                let name = match &self.session {
                    SessionStatus::SignedIn(user) => user.name.clone(),
                    _ => "Kloucy".to_string(),
                };
                let email = match &self.session {
                    SessionStatus::SignedIn(user) => user.email.clone(),
                    _ => "Not signed in".to_string(),
                };
                let next_theme = match self.theme_mode {
                    ThemeMode::Light => "Switch to dark",
                    _ => "Switch to light",
                };

                Rc::new(move |cx| {
                    let light = next_theme == "Switch to light";

                    v_flex()
                        .w(px(240.0))
                        .gap(px(2.0))
                        .child(
                            v_flex()
                                .px_2()
                                .py_1()
                                .child(div().text_sm().child(name.clone()))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(email.clone()),
                                ),
                        )
                        .child(menu_row(
                            ("home-account-theme", 0),
                            cx,
                            false,
                            vec![
                                Icon::new(if light { IconName::Sun } else { IconName::Moon })
                                    .size(px(14.0))
                                    .into_any_element(),
                                div().child(next_theme).into_any_element(),
                            ],
                            Some(Box::new({
                                let entity = entity.clone();
                                move |window, cx| {
                                    let _ =
                                        entity.update(cx, |this, cx| this.toggle_theme(window, cx));
                                }
                            })),
                        ))
                        .child(menu_row(
                            ("home-account-sign-out", 1),
                            cx,
                            false,
                            vec![
                                Icon::new(IconName::SquareTerminal)
                                    .size(px(14.0))
                                    .into_any_element(),
                                div().child("Sign out").into_any_element(),
                            ],
                            Some(Box::new({
                                let entity = entity.clone();
                                move |_, cx| {
                                    let _ = entity.update(cx, |this, cx| {
                                        this.home_menu = None;
                                        this.submit_sign_out(cx);
                                    });
                                }
                            })),
                        ))
                        .into_any_element()
                })
            }
            HomeMenu::Branch => {
                let checkout = self.home_checkout.clone();
                let path = self
                    .selected_workspace_data()
                    .map(|workspace| workspace.local_path.clone());

                Rc::new(move |cx| {
                    let mut list = v_flex().w(px(300.0)).gap(px(2.0));

                    list = list.child(menu_note(checkout.detail(), cx));

                    if let Some(path) = &path {
                        list = list.child(menu_note(path.clone(), cx));
                    }

                    let Some(branch) = checkout.branch() else {
                        return list.into_any_element();
                    };

                    let copy = branch.to_string();
                    let entity = entity.clone();

                    list.child(menu_row(
                        ("home-branch-copy", 0),
                        cx,
                        false,
                        vec![
                            Icon::new(IconName::Copy).size(px(14.0)).into_any_element(),
                            div().child("Copy branch name").into_any_element(),
                        ],
                        Some(Box::new(move |_, cx| {
                            let _ = entity.update(cx, |this, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(copy.clone()));
                                this.home_menu = None;
                                this.composer_status =
                                    Some(ComposerStatus::Notice(format!("Copied {copy}")));
                                cx.notify();
                            });
                        })),
                    ))
                    .into_any_element()
                })
            }
            HomeMenu::Attach => {
                let checkout = self
                    .selected_workspace_data()
                    .map(|workspace| workspace.local_path.clone());
                let branch = self.home_checkout.branch().map(str::to_string);

                Rc::new(move |cx| {
                    let mut list = v_flex().w(px(260.0)).gap(px(2.0));

                    let checkout_row = checkout.clone().map(|checkout| {
                        let value = checkout.clone();
                        let entity = entity.clone();

                        menu_row(
                            ("home-attach-checkout", 0),
                            cx,
                            false,
                            vec![
                                Icon::new(IconName::Folder)
                                    .size(px(14.0))
                                    .into_any_element(),
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .child("Attach checkout path")
                                    .into_any_element(),
                            ],
                            Some(Box::new(move |window, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.insert_composer(value.clone(), window, cx);
                                });
                            })),
                        )
                    });

                    list = match checkout_row {
                        Some(row) => list.child(row),
                        None => list.child(menu_note("Select a workspace first.", cx)),
                    };

                    if let Some(branch) = branch.clone() {
                        let entity = entity.clone();

                        list = list.child(menu_row(
                            ("home-attach-branch", 1),
                            cx,
                            false,
                            vec![
                                Icon::empty()
                                    .path("icons/git-branch.svg")
                                    .size(px(14.0))
                                    .into_any_element(),
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .child("Attach branch name")
                                    .into_any_element(),
                            ],
                            Some(Box::new(move |window, cx| {
                                let _ = entity.update(cx, |this, cx| {
                                    this.insert_composer(branch.clone(), window, cx);
                                });
                            })),
                        ));
                    }

                    list.into_any_element()
                })
            }
        }
    }

    /// Put text in the composer without losing what is already there.
    fn insert_composer(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        let existing = self.composer.read(cx).value().to_string();
        let separator = if existing.is_empty() || existing.ends_with(' ') {
            ""
        } else {
            " "
        };
        let value = format!("{existing}{separator}{text} ");

        self.composer
            .update(cx, |state, cx| state.set_value(value, window, cx));
        self.home_menu = None;
        self.composer_status = None;
        cx.notify();
    }

    fn render_content(&self, cx: &mut Context<Self>) -> AnyElement {
        // Home is the composer, so it owns its whole surface instead of sitting
        // under a section heading.
        if self.section == Section::Home {
            return self.render_home(cx);
        }

        let body: AnyElement = match self.section {
            Section::Agents => self.render_agents(cx).into_any_element(),
            Section::Decisions => self.render_decisions(cx),
            Section::Home => self.render_home(cx),
            Section::Inbox => self.render_inbox(cx),
            Section::Pairing => self.render_pairing(cx),
            Section::Readiness => self.render_readiness(cx),
            Section::Reviews => self.render_reviews(cx),
            Section::Sessions => self.render_sessions(cx),
            Section::Settings => self.render_settings(cx),
            Section::WeeklyReview => self.render_weekly_review(cx),
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
            .into_any_element()
    }

    fn render_shell(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut shell = h_flex()
            .id("kloucy-shell")
            .relative()
            .key_context(SHELL_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_action_open_palette))
            .on_action(cx.listener(Self::on_action_close_palette))
            .size_full()
            .child(self.render_nav(cx))
            .child(self.render_content(cx));

        if self.palette_open {
            shell = shell.child(self.render_command_palette(cx));
        }

        shell.into_any_element()
    }

    fn on_action_open_palette(
        &mut self,
        _: &OpenCommandPalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.palette_open {
            self.close_palette(window, cx);
            return;
        }

        self.open_palette(window, cx);
    }

    /// Escape reaches the shell as the focused input's escape action.
    fn on_action_close_palette(
        &mut self,
        _: &gpui_component::input::Escape,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.palette_open {
            self.close_palette(window, cx);
        }
    }

    fn open_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette_open = true;

        self.palette_query
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.palette_query
            .update(cx, |state, cx| state.focus(window, cx));

        cx.notify();
    }

    fn close_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette_open = false;

        self.palette_query
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.focus_handle.focus(window);

        cx.notify();
    }

    fn run_palette_command(
        &mut self,
        command: PaletteCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_palette(window, cx);

        match command {
            PaletteCommand::Section(section) => {
                self.section = section;
            }
            PaletteCommand::NewWorkspace => {
                self.section = Section::Workspaces;
                self.show_create_form = true;
                self.create_error = None;
            }
            PaletteCommand::RegisterAgent => {
                self.section = Section::Agents;
                self.show_agent_form = true;
                self.agent_error = None;
            }
            PaletteCommand::DetectAgents => {
                self.section = Section::Agents;
                self.detect_agents(cx);
            }
            PaletteCommand::NewPairingCode => {
                self.section = Section::Pairing;
                self.create_pairing(cx);
            }
        }

        cx.notify();
    }

    fn render_command_palette(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.palette_query.read(cx).value().to_string();
        let matches = palette_matches(&query);

        let mut list = v_flex().gap_1();

        if matches.is_empty() {
            list = list.child(
                div()
                    .px_2()
                    .py_1()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("No matching commands."),
            );
        }

        for (index, entry) in matches.into_iter().enumerate() {
            let command = entry.command;

            list = list.child(
                div()
                    .id(("palette-command", index))
                    .px_2()
                    .py_1()
                    .rounded(px(4.0))
                    .hover(|element| element.bg(cx.theme().list_hover))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.run_palette_command(command, window, cx);
                    }))
                    .child(div().text_sm().child(entry.label))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(entry.group),
                    ),
            );
        }

        let panel = v_flex()
            .id("command-palette")
            .w(px(560.0))
            .max_h(px(420.0))
            .gap_3()
            .p_3()
            .rounded(px(8.0))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .child(Input::new(&self.palette_query))
            .child(list)
            .on_click(|_, _, cx| cx.stop_propagation());

        div()
            .id("command-palette-backdrop")
            .absolute()
            .inset_0()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(120.0))
            .bg(cx.theme().overlay)
            .on_click(cx.listener(|this, _, window, cx| this.close_palette(window, cx)))
            .child(panel)
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
    Application::new()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            cx.text_system()
                .add_fonts(assets::geist_fonts())
                .expect("Geist should load");

            let stored = settings::load_theme_mode()
                .unwrap_or_else(|| settings::DEFAULT_THEME_MODE.to_string());
            apply_theme(theme_mode(&stored), None, cx);

            cx.bind_keys([
                KeyBinding::new("cmd-k", OpenCommandPalette, Some(SHELL_CONTEXT)),
                KeyBinding::new("ctrl-k", OpenCommandPalette, Some(SHELL_CONTEXT)),
            ]);

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

#[cfg(test)]
mod nav_tests {
    use super::*;

    #[test]
    fn counts_only_matching_items() {
        let loaded: LoadState<Vec<AgentSession>> = LoadState::Loaded(vec![
            AgentSession {
                agent_id: None,
                id: "1".to_string(),
                prompt: None,
                status: "waiting".to_string(),
            },
            AgentSession {
                agent_id: None,
                id: "2".to_string(),
                prompt: None,
                status: "running".to_string(),
            },
        ]);

        assert_eq!(
            count_where(&loaded, |session| session.status == "waiting"),
            1
        );
        assert_eq!(count_where(&loaded, |session| session.status == "done"), 0);
    }

    #[test]
    fn counts_nothing_while_loading_or_after_a_failure() {
        let loading: LoadState<Vec<AgentSession>> = LoadState::Loading;
        let failed: LoadState<Vec<AgentSession>> = LoadState::Failed("nope".to_string());

        assert_eq!(count_where(&loading, |_| true), 0);
        assert_eq!(count_where(&failed, |_| true), 0);
    }

    #[test]
    fn takes_two_initials() {
        assert_eq!(initials("Alex Rivera"), "AR");
        assert_eq!(initials("Valen"), "V");
        assert_eq!(initials("  "), "");
    }
}

#[cfg(test)]
mod home_tests {
    use super::*;

    #[test]
    fn takes_the_first_line_as_the_title() {
        assert_eq!(
            composer_title("Fix the login redirect\n\nAnd the tests"),
            "Fix the login redirect"
        );
        assert_eq!(composer_title("   Capture me  "), "Capture me");
        assert_eq!(composer_title(""), "");
    }

    #[test]
    fn shortens_long_titles() {
        let long = "a".repeat(COMPOSER_TITLE_LIMIT * 2);
        let title = composer_title(&long);

        assert_eq!(title.chars().count(), COMPOSER_TITLE_LIMIT + 1);
        assert!(title.ends_with('…'));
    }

    #[test]
    fn reads_the_folder_out_of_a_path() {
        assert_eq!(folder_name("/home/valen/projects/plata"), "plata");
        assert_eq!(folder_name("/home/valen/projects/plata/"), "plata");
        assert_eq!(folder_name("plata"), "plata");
        assert_eq!(folder_name("/"), "/");
    }

    #[test]
    fn labels_the_known_safety_modes() {
        assert_eq!(safety_label("guarded"), "guarded");
        assert_eq!(safety_label("open"), "open");
        assert_eq!(safety_label("something-new"), "open");
    }

    fn session(id: &str, status: &str) -> AgentSession {
        AgentSession {
            agent_id: None,
            id: id.to_string(),
            prompt: None,
            status: status.to_string(),
        }
    }

    #[test]
    fn summarises_what_agents_are_doing() {
        assert_eq!(session_activity(&LoadState::Loading), None);
        assert_eq!(
            session_activity(&LoadState::Loaded(vec![session("1", "done")])),
            None
        );
        assert_eq!(
            session_activity(&LoadState::Loaded(vec![
                session("1", "waiting"),
                session("2", "waiting"),
                session("3", "running"),
            ])),
            Some("2 waiting · 1 running".to_string())
        );
    }
}

#[cfg(test)]
mod palette_tests {
    use super::*;

    #[test]
    fn an_empty_query_lists_every_command() {
        assert_eq!(palette_matches("").len(), PALETTE_ENTRIES.len());
        assert_eq!(palette_matches("   ").len(), PALETTE_ENTRIES.len());
    }

    #[test]
    fn matches_labels_and_keywords_ignoring_case() {
        assert!(
            palette_matches("PAIR")
                .iter()
                .any(|entry| entry.command == PaletteCommand::NewPairingCode)
        );
        assert!(
            palette_matches("blockers")
                .iter()
                .any(|entry| entry.command == PaletteCommand::Section(Section::Readiness))
        );
    }

    #[test]
    fn an_unknown_query_matches_nothing() {
        assert!(palette_matches("zzzz").is_empty());
    }
}
