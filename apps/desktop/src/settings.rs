//! Persisted desktop preferences. Kept deliberately tiny: one JSON file beside
//! the user's config, read at startup and written when something changes.

use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

/// Theme mode used when nothing was stored yet.
pub const DEFAULT_THEME_MODE: &str = "dark";

const THEME_MODE_KEY: &str = "themeMode";

/// Where the desktop keeps its preferences, if the environment says so.
fn config_path() -> Option<PathBuf> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".config"))
        })?;

    Some(base.join("kloucy").join("desktop.json"))
}

/// The stored theme mode as a plain string, if it was readable.
pub fn load_theme_mode() -> Option<String> {
    let path = config_path()?;
    let text = fs::read_to_string(path).ok()?;

    theme_mode_from_json(&text)
}

/// Persist the theme mode, ignoring environments without a config directory.
pub fn save_theme_mode(mode: &str) {
    let Some(path) = config_path() else {
        return;
    };

    let Some(parent) = path.parent() else {
        return;
    };

    if fs::create_dir_all(parent).is_err() {
        return;
    }

    let _ = fs::write(path, theme_mode_to_json(mode));
}

/// Read the theme mode out of a stored document.
pub fn theme_mode_from_json(text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;

    value
        .get(THEME_MODE_KEY)
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// The document written back to disk.
pub fn theme_mode_to_json(mode: &str) -> String {
    json!({ THEME_MODE_KEY: mode }).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_stored_mode() {
        assert_eq!(
            theme_mode_from_json(r#"{"themeMode":"light"}"#),
            Some("light".to_string())
        );
        assert_eq!(
            theme_mode_from_json(r#"{"themeMode":"dark","other":1}"#),
            Some("dark".to_string())
        );
    }

    #[test]
    fn ignores_documents_it_cannot_use() {
        assert_eq!(theme_mode_from_json("not json"), None);
        assert_eq!(theme_mode_from_json("{}"), None);
        assert_eq!(theme_mode_from_json(r#"{"themeMode":3}"#), None);
    }

    #[test]
    fn round_trips_through_storage() {
        for mode in ["dark", "light"] {
            assert_eq!(
                theme_mode_from_json(&theme_mode_to_json(mode)),
                Some(mode.to_string())
            );
        }
    }

    #[test]
    fn defaults_to_dark() {
        assert_eq!(DEFAULT_THEME_MODE, "dark");
    }
}
