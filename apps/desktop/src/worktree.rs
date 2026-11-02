//! Isolated checkouts for threads.
//!
//! A thread that works in a git repository gets its own worktree and branch, so
//! two threads — or a thread and the person — never edit the same files. The
//! thread keeps the path; the repository registry is git's own.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A checkout a thread works in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worktree {
    pub branch: String,
    pub path: String,
}

/// Create a worktree of `repo` under Kloucy's data directory, on a new branch
/// named after the prompt. `seed` keeps two threads with the same prompt apart.
pub fn create(
    repo: &Path,
    title: &str,
    seed: &str,
    base: Option<&str>,
) -> Result<Worktree, String> {
    let root = repository_root(repo)?;
    let slug = slug(title);
    let branch = format!("kloucy/{slug}-{seed}");
    let path = directory().join(format!("{slug}-{seed}"));

    if path.exists() {
        return Err("That worktree already exists.".to_string());
    }

    fs::create_dir_all(directory())
        .map_err(|_| "Kloucy could not create its worktrees.".to_string())?;

    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["worktree", "add", "-b", &branch])
        .arg(&path)
        .arg(base.unwrap_or("HEAD"))
        .output()
        .map_err(|_| "Kloucy could not run git.".to_string())?;

    if !output.status.success() {
        let _ = fs::remove_dir_all(&path);
        return Err("Kloucy could not create a worktree for this thread.".to_string());
    }

    Ok(Worktree {
        branch,
        path: path.display().to_string(),
    })
}

/// Where Kloucy keeps the worktrees of its threads.
fn directory() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    home.join(".local/share/kloucy/worktrees")
}

/// The top of the repository `path` belongs to.
fn repository_root(path: &Path) -> Result<PathBuf, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| "Kloucy could not run git.".to_string())?;

    if !output.status.success() {
        return Err("This project is not a git repository.".to_string());
    }

    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(PathBuf::from(root))
}

/// A branch-friendly name for a prompt.
fn slug(title: &str) -> String {
    let mut slug = String::new();

    for character in title.chars() {
        if slug.len() >= 24 {
            break;
        }

        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }

    let slug = slug.trim_matches('-').to_string();

    if slug.is_empty() {
        "thread".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "kloucy-worktree-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("scratch directory");

        path
    }

    fn repository(name: &str) -> PathBuf {
        let path = scratch(name);

        let git = |arguments: &[&str]| {
            let status = Command::new("git")
                .arg("-C")
                .arg(&path)
                .args(arguments)
                .status()
                .expect("git");

            assert!(status.success(), "git {arguments:?}");
        };

        git(&["init", "-q"]);
        git(&["config", "user.email", "test@kloucy.dev"]);
        git(&["config", "user.name", "Kloucy"]);
        fs::write(path.join("README.md"), "# test\n").expect("file");
        git(&["add", "."]);
        git(&["commit", "-qm", "first"]);

        path
    }

    #[test]
    fn slugs_a_prompt() {
        assert_eq!(
            slug("Responde solo con la palabra tarea."),
            "responde-solo-con-la-pal"
        );
        assert_eq!(slug("  "), "thread");
        assert_eq!(slug("hola"), "hola");
    }

    #[test]
    fn creates_a_worktree_on_its_own_branch() {
        let root = repository("create");
        let worktree = create(&root, "Fix the parser", "abc123", None).expect("a worktree");

        assert!(Path::new(&worktree.path).join("README.md").exists());
        assert!(worktree.branch.starts_with("kloucy/fix-the-parser-"));

        let status = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(["worktree", "remove", "--force", &worktree.path])
            .status()
            .expect("git");
        assert!(status.success());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_a_directory_that_is_not_a_repository() {
        let path = scratch("plain");

        assert!(create(&path, "Nope", "seed", None).is_err());

        let _ = fs::remove_dir_all(path);
    }
}
