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
    pub local_path: String,
    pub name: String,
    pub stage: String,
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
