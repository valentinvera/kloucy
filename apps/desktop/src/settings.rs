//! Persisted desktop preferences. Kept deliberately tiny: one JSON file beside
//! the user's config, read at startup and written when something changes.

use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

/// Theme mode used when nothing was stored yet.
pub const DEFAULT_THEME_MODE: &str = "dark";

const THEME_MODE_KEY: &str = "themeMode";

/// Models starred in the picker, stored as `provider/model`.
const FAVORITE_MODELS_KEY: &str = "favoriteModels";

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

/// Where Kloucy runs messages that target no project.
pub fn chat_space() -> Option<PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".local").join("share"))
        })?;

    Some(base.join("kloucy").join("chats"))
}

/// The chat space, created as a git checkout the first time it is needed.
///
/// Agent CLIs refuse to run outside a repository, and having one here keeps
/// whatever they write out of the user's home directory.
pub fn ensure_chat_space() -> Option<PathBuf> {
    let path = chat_space()?;

    fs::create_dir_all(&path).ok()?;

    if !path.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(&path)
            .status();
    }

    Some(path)
}

/// The stored theme mode as a plain string, if it was readable.
pub fn load_theme_mode() -> Option<String> {
    let text = fs::read_to_string(config_path()?).ok()?;

    theme_mode_from_json(&text)
}

/// Persist the theme mode, leaving the other stored settings alone.
pub fn save_theme_mode(mode: &str) {
    update_document(|document| {
        document[THEME_MODE_KEY] = json!(mode);
    });
}

/// The models marked as favourites, as `provider/model`.
pub fn load_favorites() -> Vec<String> {
    let Ok(text) = fs::read_to_string(config_path().unwrap_or_default()) else {
        return Vec::new();
    };

    favorites_from_json(&text)
}

/// Persist the favourite models.
pub fn save_favorites(models: &[String]) {
    update_document(|document| {
        document[FAVORITE_MODELS_KEY] = json!(models);
    });
}

/// Read the document, let `update` change it, and write it back. Best effort:
/// an environment without a config directory simply keeps nothing.
fn update_document(update: impl FnOnce(&mut Value)) {
    let Some(path) = config_path() else {
        return;
    };

    let mut document = fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));

    update(&mut document);

    let Some(parent) = path.parent() else {
        return;
    };

    if fs::create_dir_all(parent).is_err() {
        return;
    }

    let _ = fs::write(path, document.to_string());
}

/// The favourite models listed in a stored document.
pub fn favorites_from_json(text: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };

    value
        .get(FAVORITE_MODELS_KEY)
        .and_then(Value::as_array)
        .map(|models| {
            models
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Read the theme mode out of a stored document.
pub fn theme_mode_from_json(text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;

    value
        .get(THEME_MODE_KEY)
        .and_then(Value::as_str)
        .map(str::to_string)
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
    fn setting_one_key_keeps_the_others() {
        let mut document: Value = serde_json::from_str(r#"{"themeMode":"light"}"#).unwrap();
        document[FAVORITE_MODELS_KEY] = json!(["codex/gpt-6.1-sol"]);

        assert_eq!(
            theme_mode_from_json(&document.to_string()),
            Some("light".to_string())
        );
        assert_eq!(
            favorites_from_json(&document.to_string()),
            vec!["codex/gpt-6.1-sol".to_string()]
        );
    }

    #[test]
    fn reads_favourites_leniently() {
        assert_eq!(
            favorites_from_json(r#"{"favoriteModels":["a/b","c/d"]}"#),
            vec!["a/b".to_string(), "c/d".to_string()]
        );
        assert!(favorites_from_json("{}").is_empty());
        assert!(favorites_from_json("not json").is_empty());
        assert!(favorites_from_json(r#"{"favoriteModels":[1,2]}"#).is_empty());
    }

    #[test]
    fn defaults_to_dark() {
        assert_eq!(DEFAULT_THEME_MODE, "dark");
    }

    #[test]
    fn keeps_chat_runs_out_of_the_home_directory() {
        let path = chat_space().expect("a data directory");

        assert!(path.ends_with("kloucy/chats"), "{path:?}");
    }
}
