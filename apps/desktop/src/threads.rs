//! The conversations the desktop remembers between runs.
//!
//! Transcripts live in the API; this is only the index needed to reopen one: the
//! runs that make it up, the agent-side session that carries their context, and
//! where it ran.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// How many conversations are kept, newest first.
pub const KEPT: usize = 20;

/// A conversation as it is stored on disk.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredThread {
    /// The agent's own session id, which is what makes it resumable.
    #[serde(default)]
    pub agent_session: Option<String>,
    /// The runs that make up the conversation, oldest first.
    pub session_ids: Vec<String>,
    pub title: String,
    /// Unix seconds of the last turn.
    #[serde(default)]
    pub updated_at: u64,
    pub workspace_id: String,
}

impl StoredThread {
    /// The run that opened the conversation: its identity across turns.
    pub fn key(&self) -> Option<&str> {
        self.session_ids.first().map(String::as_str)
    }
}

/// Read the stored conversations, newest first. A missing or unreadable file is
/// an empty history, never an error the app has to show.
pub fn load() -> Vec<StoredThread> {
    let Some(path) = path() else {
        return Vec::new();
    };

    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };

    serde_json::from_str(&text).unwrap_or_default()
}

/// Write the conversations back, best effort.
pub fn save(conversations: &[StoredThread]) {
    let Some(path) = path() else {
        return;
    };

    let Some(parent) = path.parent() else {
        return;
    };

    if fs::create_dir_all(parent).is_err() {
        return;
    }

    if let Ok(text) = serde_json::to_string(conversations) {
        let _ = fs::write(path, text);
    }
}

/// Put a conversation at the front, replacing the entry it already has. A
/// conversation with no runs has nothing to reopen, so it is not stored.
pub fn upsert(conversations: &mut Vec<StoredThread>, thread: StoredThread) {
    let Some(key) = thread.key().map(str::to_string) else {
        return;
    };

    conversations.retain(|stored| stored.key() != Some(key.as_str()));
    conversations.insert(0, thread);
    conversations.truncate(KEPT);
}

fn path() -> Option<PathBuf> {
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

    Some(base.join("kloucy").join("threads.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thread(title: &str, first: &str, turns: usize) -> StoredThread {
        StoredThread {
            agent_session: None,
            session_ids: (0..turns).map(|index| format!("{first}-{index}")).collect(),
            title: title.to_string(),
            updated_at: 0,
            workspace_id: "workspace-1".to_string(),
        }
    }

    #[test]
    fn a_conversation_is_known_by_its_first_run() {
        let stored = thread("Hola", "a", 3);

        assert_eq!(stored.key(), Some("a-0"));
        assert_eq!(StoredThread::default().key(), None);
    }

    #[test]
    fn round_trips_through_json() {
        let stored = thread("Fibonacci", "a", 2);
        let text = serde_json::to_string(&vec![stored.clone()]).expect("serializes");

        let back = serde_json::from_str::<Vec<StoredThread>>(&text).expect("deserializes");

        assert_eq!(back, vec![stored]);
    }

    #[test]
    fn a_new_turn_moves_its_conversation_to_the_front() {
        let mut threads = vec![thread("Uno", "a", 1), thread("Dos", "b", 1)];

        upsert(&mut threads, thread("Uno", "a", 2));

        assert_eq!(
            threads
                .iter()
                .map(|stored| (stored.title.as_str(), stored.session_ids.len()))
                .collect::<Vec<_>>(),
            vec![("Uno", 2), ("Dos", 1)]
        );
    }

    #[test]
    fn keeps_the_history_bounded() {
        let mut threads = Vec::new();

        for index in 0..KEPT + 5 {
            upsert(
                &mut threads,
                thread(&format!("Charla {index}"), &format!("s{index}"), 1),
            );
        }

        assert_eq!(threads.len(), KEPT);
        assert_eq!(threads[0].title, format!("Charla {}", KEPT + 4));
    }

    #[test]
    fn a_conversation_without_runs_is_not_stored() {
        let mut threads = vec![thread("Hola", "a", 1)];
        let empty = thread("Vacía", "b", 0);

        upsert(&mut threads, empty);

        assert_eq!(threads, vec![thread("Hola", "a", 1)]);
    }
}
