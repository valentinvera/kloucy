use serde::Deserialize;

/// The authenticated user as returned by `GET /api/users/me`.
#[derive(Clone, Debug, Deserialize)]
pub struct CurrentUser {
    pub email: String,
    pub name: String,
}

/// The connection and authentication state against the Kloucy API.
#[derive(Clone, Debug)]
pub enum SessionStatus {
    Checking,
    SignedIn(CurrentUser),
    SignedOut,
    Unreachable,
}

/// Resolve the API base URL from the environment, defaulting to local.
pub fn base_url() -> String {
    std::env::var("KLOUCY_API_URL").unwrap_or_else(|_| "http://localhost:3001".to_string())
}

/// Fetch the current session user from the Kloucy API.
///
/// This is a blocking call; run it on a background executor.
pub fn fetch_session(base_url: &str) -> SessionStatus {
    let url = format!("{base_url}/api/users/me");

    match reqwest::blocking::get(&url) {
        Ok(response) if response.status().is_success() => response
            .json::<CurrentUser>()
            .map_or(SessionStatus::Unreachable, SessionStatus::SignedIn),
        Ok(response) if response.status() == reqwest::StatusCode::UNAUTHORIZED => {
            SessionStatus::SignedOut
        }
        _ => SessionStatus::Unreachable,
    }
}
