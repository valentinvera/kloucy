use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

/// The authenticated user as returned by `GET /api/users/me`.
#[derive(Clone, Debug, Deserialize)]
pub struct CurrentUser {
    pub email: String,
    pub name: String,
}

/// A workspace as returned by the `workspaces.list` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub local_path: String,
    pub name: String,
    pub stage: String,
}

/// An agent task as returned by the `agentTasks.list` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub source: String,
    pub status: String,
    pub title: String,
}

/// An agent as returned by the `agents.list` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
pub struct Agent {
    pub command_template: Option<String>,
    pub executable_path: Option<String>,
    pub id: String,
    pub name: String,
    pub provider: String,
    pub status: String,
}

/// An agent session as returned by the `sessions.list` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSession {
    pub agent_id: Option<String>,
    pub id: String,
    pub prompt: Option<String>,
    pub status: String,
}

/// A review as returned by the `reviews.list` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub action: String,
    pub artifact_id: Option<String>,
    pub id: String,
}

/// A decision as returned by the `decisions.list` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
pub struct Decision {
    pub summary: Option<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub decision_type: String,
}

/// A readiness snapshot as returned by the `readiness.latest` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessSnapshot {
    pub blockers: Option<Vec<String>>,
    pub not_required_yet: Option<Vec<String>>,
    pub score: i64,
    pub status: String,
}

/// A weekly review as returned by the `weeklyReviews.latest` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyReview {
    pub blockers: Option<Vec<String>>,
    pub next_actions: Option<Vec<String>>,
    pub rejected_changes: Option<Vec<String>>,
    pub risky_patterns: Option<Vec<String>>,
    pub shipped: Option<Vec<String>>,
    pub summary: Option<String>,
}

/// A pairing session as returned by `pairing.createQrSession`.
#[derive(Clone, Debug, Deserialize)]
pub struct PairingSession {
    pub status: String,
    pub token: String,
}

/// A session log line as returned by the `sessions.logs` tRPC procedure.
#[derive(Clone, Debug, Deserialize)]
pub struct SessionLog {
    pub message: String,
    pub stream: String,
}

/// The connection and authentication state against the Kloucy API.
#[derive(Clone, Debug)]
pub enum SessionStatus {
    Checking,
    SignedIn(CurrentUser),
    SignedOut,
    Unreachable,
}

/// A failed API call.
#[derive(Clone, Debug)]
pub enum ApiError {
    Unauthorized,
    Unreachable,
    Message(String),
}

/// Resolve the API base URL from the environment, defaulting to local.
pub fn base_url() -> String {
    std::env::var("KLOUCY_API_URL").unwrap_or_else(|_| "http://localhost:3001".to_string())
}

/// A blocking client for the Kloucy API that keeps the session cookie.
#[derive(Clone)]
pub struct KloucyClient {
    base_url: String,
    http: reqwest::blocking::Client,
}

impl KloucyClient {
    pub fn new() -> Self {
        let http = reqwest::blocking::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self {
            base_url: base_url(),
            http,
        }
    }

    /// Fetch the current session user.
    pub fn session(&self) -> SessionStatus {
        match self.get_json::<CurrentUser>("/api/users/me") {
            Ok(user) => SessionStatus::SignedIn(user),
            Err(ApiError::Unauthorized) => SessionStatus::SignedOut,
            Err(_) => SessionStatus::Unreachable,
        }
    }

    /// Sign in with email and password, storing the session cookie.
    pub fn sign_in(&self, email: &str, password: &str) -> Result<(), ApiError> {
        let url = format!("{}/api/auth/sign-in/email", self.base_url);
        let body = serde_json::json!({ "email": email, "password": password }).to_string();

        let response = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        if response.status().is_success() {
            return Ok(());
        }

        let text = response.text().unwrap_or_default();
        let message = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|value| {
                value
                    .get("message")
                    .and_then(Value::as_str)
                    .map(String::from)
            })
            .unwrap_or_else(|| "Sign in failed".to_string());

        Err(ApiError::Message(message))
    }

    /// Sign out the current session.
    pub fn sign_out(&self) -> Result<(), ApiError> {
        let url = format!("{}/api/auth/sign-out", self.base_url);

        let response = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .body("{}")
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApiError::Message("Sign out failed".to_string()))
        }
    }

    /// Run a tRPC mutation procedure and deserialize its `result.data`.
    pub fn mutation<T: DeserializeOwned>(&self, path: &str, input: Value) -> Result<T, ApiError> {
        let url = format!("{}/api/trpc/{}", self.base_url, path);
        let body = input.to_string();

        let response = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        let value = read_value(response)?;
        let data = value
            .get("result")
            .and_then(|result| result.get("data"))
            .cloned()
            .unwrap_or(Value::Null);

        serde_json::from_value(data).map_err(|error| ApiError::Message(error.to_string()))
    }

    /// Capture a task in a workspace.
    pub fn create_task(
        &self,
        workspace_id: &str,
        title: &str,
        prompt: &str,
    ) -> Result<(), ApiError> {
        let mut input = serde_json::json!({ "workspaceId": workspace_id, "title": title });

        if !prompt.is_empty() {
            input["prompt"] = Value::String(prompt.to_string());
        }

        self.mutation::<Value>("agentTasks.create", input)
            .map(|_| ())
    }

    /// Approve a review.
    pub fn approve_review(&self, id: &str) -> Result<(), ApiError> {
        self.mutation::<Value>("reviews.approve", serde_json::json!({ "id": id }))
            .map(|_| ())
    }

    /// Reject a review.
    pub fn reject_review(&self, id: &str) -> Result<(), ApiError> {
        self.mutation::<Value>("reviews.reject", serde_json::json!({ "id": id }))
            .map(|_| ())
    }

    /// Register an agent for the signed-in owner.
    pub fn create_agent(
        &self,
        name: &str,
        provider: &str,
        command_template: &str,
    ) -> Result<(), ApiError> {
        let mut input = serde_json::json!({ "name": name, "provider": provider });

        if !command_template.is_empty() {
            input["commandTemplate"] = Value::String(command_template.to_string());
        }

        self.mutation::<Value>("agents.create", input).map(|_| ())
    }

    /// Append a log line to a session.
    pub fn append_log(
        &self,
        session_id: &str,
        stream: &str,
        message: &str,
    ) -> Result<(), ApiError> {
        self.mutation::<Value>(
            "sessions.appendLog",
            serde_json::json!({
                "sessionId": session_id,
                "stream": stream,
                "message": message,
            }),
        )
        .map(|_| ())
    }

    /// Mark a session finished.
    pub fn finish_session(&self, session_id: &str, status: &str) -> Result<(), ApiError> {
        self.mutation::<Value>(
            "sessions.finish",
            serde_json::json!({ "id": session_id, "status": status }),
        )
        .map(|_| ())
    }

    /// Create a pairing session for the signed-in owner.
    pub fn create_qr_session(&self) -> Result<PairingSession, ApiError> {
        let url = format!("{}/api/trpc/pairing.createQrSession", self.base_url);

        let response = self
            .http
            .post(&url)
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        let value = read_value(response)?;
        let data = value
            .get("result")
            .and_then(|result| result.get("data"))
            .cloned()
            .unwrap_or(Value::Null);

        serde_json::from_value(data).map_err(|error| ApiError::Message(error.to_string()))
    }

    /// Save a weekly review for a workspace.
    pub fn create_weekly_review(
        &self,
        workspace_id: &str,
        summary: &str,
        shipped: &[String],
        blockers: &[String],
        rejected_changes: &[String],
        next_actions: &[String],
    ) -> Result<(), ApiError> {
        self.mutation::<Value>(
            "weeklyReviews.create",
            serde_json::json!({
                "workspaceId": workspace_id,
                "summary": summary,
                "shipped": shipped,
                "blockers": blockers,
                "rejectedChanges": rejected_changes,
                "nextActions": next_actions,
            }),
        )
        .map(|_| ())
    }

    /// Recalculate readiness for a workspace.
    pub fn calculate_readiness(&self, workspace_id: &str) -> Result<(), ApiError> {
        self.mutation::<Value>(
            "readiness.calculate",
            serde_json::json!({ "workspaceId": workspace_id }),
        )
        .map(|_| ())
    }

    /// Record a decision in a workspace.
    pub fn create_decision(
        &self,
        workspace_id: &str,
        title: &str,
        summary: &str,
    ) -> Result<(), ApiError> {
        let mut input = serde_json::json!({ "workspaceId": workspace_id, "title": title });

        if !summary.is_empty() {
            input["summary"] = Value::String(summary.to_string());
        }

        self.mutation::<Value>("decisions.create", input)
            .map(|_| ())
    }

    /// Create a session for a task.
    pub fn create_session(&self, workspace_id: &str, task_id: &str) -> Result<(), ApiError> {
        self.mutation::<Value>(
            "sessions.create",
            serde_json::json!({ "workspaceId": workspace_id, "taskId": task_id }),
        )
        .map(|_| ())
    }

    /// Start a session.
    pub fn start_session(&self, id: &str) -> Result<(), ApiError> {
        self.mutation::<Value>("sessions.start", serde_json::json!({ "id": id }))
            .map(|_| ())
    }

    /// Stop a session.
    pub fn stop_session(&self, id: &str) -> Result<(), ApiError> {
        self.mutation::<Value>("sessions.stop", serde_json::json!({ "id": id }))
            .map(|_| ())
    }

    /// Retry a session.
    pub fn retry_session(&self, id: &str) -> Result<(), ApiError> {
        self.mutation::<Value>("sessions.retry", serde_json::json!({ "id": id }))
            .map(|_| ())
    }

    /// Create a workspace owned by the signed-in user.
    pub fn create_workspace(&self, name: &str, local_path: &str) -> Result<(), ApiError> {
        self.mutation::<Value>(
            "workspaces.create",
            serde_json::json!({ "name": name, "localPath": local_path }),
        )
        .map(|_| ())
    }

    /// Run a tRPC query procedure and deserialize its `result.data`.
    pub fn query<T: DeserializeOwned>(
        &self,
        path: &str,
        input: Option<Value>,
    ) -> Result<T, ApiError> {
        let mut url = format!("{}/api/trpc/{}", self.base_url, path);

        if let Some(input) = input {
            let serialized = input.to_string();
            let encoded = urlencoding::encode(&serialized);
            url = format!("{url}?input={encoded}");
        }

        let response = self
            .http
            .get(&url)
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        let value = read_value(response)?;
        let data = value
            .get("result")
            .and_then(|result| result.get("data"))
            .cloned()
            .unwrap_or(Value::Null);

        serde_json::from_value(data).map_err(|error| ApiError::Message(error.to_string()))
    }

    fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .send()
            .map_err(|_| ApiError::Unreachable)?;

        let value = read_value(response)?;

        serde_json::from_value(value).map_err(|error| ApiError::Message(error.to_string()))
    }
}

impl Default for KloucyClient {
    fn default() -> Self {
        Self::new()
    }
}

fn read_value(response: reqwest::blocking::Response) -> Result<Value, ApiError> {
    let status = response.status();
    let body = response.text().map_err(|_| ApiError::Unreachable)?;
    let value: Value = serde_json::from_str(&body).unwrap_or(Value::Null);

    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err(ApiError::Unauthorized);
    }

    if let Some(error) = value.get("error") {
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Request failed");
        return Err(ApiError::Message(message.to_string()));
    }

    Ok(value)
}
