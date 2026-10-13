use std::env;
use std::path::{Path, PathBuf};

/// A known agent CLI Kloucy can look for on the local PATH.
struct KnownAgent {
    binary: &'static str,
    name: &'static str,
    provider: &'static str,
}

const KNOWN_AGENTS: [KnownAgent; 7] = [
    KnownAgent {
        binary: "claude",
        name: "Claude Code",
        provider: "claude-code",
    },
    KnownAgent {
        binary: "codex",
        name: "Codex",
        provider: "codex",
    },
    KnownAgent {
        binary: "cursor-agent",
        name: "Cursor Agent",
        provider: "cursor-agent",
    },
    KnownAgent {
        binary: "gemini",
        name: "Gemini CLI",
        provider: "gemini",
    },
    KnownAgent {
        binary: "goose",
        name: "Goose",
        provider: "goose",
    },
    KnownAgent {
        binary: "aider",
        name: "Aider",
        provider: "aider",
    },
    KnownAgent {
        binary: "opencode",
        name: "OpenCode",
        provider: "opencode",
    },
];

/// A project profile guessed from the entries of a workspace directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectProfile {
    pub package_manager: Option<String>,
    pub project_type: Option<String>,
    pub runtime: Option<String>,
}

/// Signals checked in order, so a repository with several ecosystems resolves to
/// the first one listed here.
const PROJECT_SIGNALS: [(&str, &str, &str, &str); 13] = [
    // file, project type, package manager, runtime
    ("bun.lock", "node", "bun", "bun"),
    ("bun.lockb", "node", "bun", "bun"),
    ("pnpm-lock.yaml", "node", "pnpm", "node"),
    ("yarn.lock", "node", "yarn", "node"),
    ("package-lock.json", "node", "npm", "node"),
    ("Cargo.toml", "rust", "cargo", "rust"),
    ("go.mod", "go", "go", "go"),
    ("uv.lock", "python", "uv", "python"),
    ("poetry.lock", "python", "poetry", "python"),
    ("Pipfile.lock", "python", "pipenv", "python"),
    ("requirements.txt", "python", "pip", "python"),
    ("Gemfile", "ruby", "bundler", "ruby"),
    ("composer.json", "php", "composer", "php"),
];

/// Guess a project profile from directory entry names. Reading names only is
/// enough; nothing in the project is executed.
pub fn detect_project(files: &[String]) -> Option<ProjectProfile> {
    let (_, project_type, package_manager, runtime) = PROJECT_SIGNALS
        .iter()
        .find(|(file, ..)| files.iter().any(|candidate| candidate == file))?;

    Some(ProjectProfile {
        package_manager: Some((*package_manager).to_string()),
        project_type: Some((*project_type).to_string()),
        runtime: Some((*runtime).to_string()),
    })
}

/// The names of the entries in a directory, or nothing when it cannot be read.
pub fn read_project_files(path: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(path) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect()
}

/// An agent CLI found on the local PATH.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedAgent {
    pub executable_path: String,
    pub name: String,
    pub provider: String,
}

/// Look for known agent CLIs on the given PATH value.
///
/// Detection only resolves executables; it never runs them, so an agent is
/// never started without the user registering it first.
pub fn detect_agents(path: &str) -> Vec<DetectedAgent> {
    KNOWN_AGENTS
        .iter()
        .filter_map(|known| {
            let executable = find_on_path(known.binary, path)?;

            Some(DetectedAgent {
                executable_path: executable.to_string_lossy().into_owned(),
                name: known.name.to_string(),
                provider: known.provider.to_string(),
            })
        })
        .collect()
}

/// Look for known agent CLIs on the current PATH.
pub fn detect_installed_agents() -> Vec<DetectedAgent> {
    detect_agents(&env::var("PATH").unwrap_or_default())
}

fn find_on_path(binary: &str, path: &str) -> Option<PathBuf> {
    env::split_paths(path)
        .filter(|directory| !directory.as_os_str().is_empty())
        .map(|directory| directory.join(binary))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch_dir(name: &str) -> PathBuf {
        let directory =
            env::temp_dir().join(format!("kloucy-detect-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("scratch directory should be created");
        directory
    }

    #[cfg(unix)]
    fn write_executable(directory: &Path, name: &str) {
        use std::os::unix::fs::PermissionsExt;

        let path = directory.join(name);
        fs::write(&path, "#!/bin/sh\n").expect("fake binary should be written");

        let mut permissions = fs::metadata(&path)
            .expect("fake binary should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("fake binary should be executable");
    }

    #[cfg(unix)]
    #[test]
    fn finds_a_known_agent_on_the_path() {
        let directory = scratch_dir("finds");
        write_executable(&directory, "codex");

        let found = detect_agents(&directory.to_string_lossy());

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].provider, "codex");
        assert_eq!(found[0].name, "Codex");
        assert!(found[0].executable_path.ends_with("codex"));

        let _ = fs::remove_dir_all(&directory);
    }

    #[cfg(unix)]
    #[test]
    fn ignores_unknown_and_non_executable_files() {
        let directory = scratch_dir("ignores");
        write_executable(&directory, "not-an-agent");
        fs::write(directory.join("claude"), "not executable")
            .expect("plain file should be written");

        assert!(detect_agents(&directory.to_string_lossy()).is_empty());

        let _ = fs::remove_dir_all(&directory);
    }

    #[cfg(unix)]
    #[test]
    fn searches_every_path_entry() {
        let empty = scratch_dir("empty");
        let populated = scratch_dir("populated");
        write_executable(&populated, "aider");

        let path = env::join_paths([empty.as_path(), populated.as_path()])
            .expect("paths should join")
            .to_string_lossy()
            .into_owned();
        let found = detect_agents(&path);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].provider, "aider");

        let _ = fs::remove_dir_all(&empty);
        let _ = fs::remove_dir_all(&populated);
    }

    #[test]
    fn returns_nothing_for_an_empty_path() {
        assert!(detect_agents("").is_empty());
    }

    #[test]
    fn detects_a_node_project_from_its_lockfile() {
        let profile = detect_project(&["package.json".to_string(), "bun.lock".to_string()])
            .expect("a node project");

        assert_eq!(profile.project_type.as_deref(), Some("node"));
        assert_eq!(profile.package_manager.as_deref(), Some("bun"));
        assert_eq!(profile.runtime.as_deref(), Some("bun"));
    }

    #[test]
    fn detects_rust_and_ignores_unknown_projects() {
        let profile = detect_project(&["Cargo.toml".to_string()]).expect("a rust project");

        assert_eq!(profile.package_manager.as_deref(), Some("cargo"));
        assert!(detect_project(&["README.md".to_string()]).is_none());
        assert!(detect_project(&[]).is_none());
    }

    #[test]
    fn reads_directory_entries_for_detection() {
        let directory = scratch_dir("profile");
        fs::write(directory.join("Cargo.toml"), "[package]").expect("signal file");

        let files = read_project_files(&directory.to_string_lossy());
        let profile = detect_project(&files).expect("a rust project");

        assert_eq!(profile.runtime.as_deref(), Some("rust"));

        let _ = fs::remove_dir_all(&directory);
    }

    #[test]
    fn reads_nothing_from_a_missing_directory() {
        assert!(read_project_files("/kloucy/does-not-exist").is_empty());
    }
}
