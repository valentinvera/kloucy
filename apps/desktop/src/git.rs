//! Reading which branch a checkout has checked out, without shelling out to git.
//!
//! Kloucy only needs the label, so this reads `.git/HEAD` directly and handles
//! the two ways a checkout can point at its git directory: a real directory, or
//! the pointer file a linked worktree uses.

use std::fs;
use std::path::{Path, PathBuf};

/// What can be said about a checkout from the machine the desktop runs on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Checkout {
    /// No workspace is selected, so there is nothing to look at.
    None,
    /// The workspace path does not exist here.
    Missing,
    /// The path exists, but it is not a git checkout.
    NotRepository,
    /// The branch, or short commit, checked out there.
    Branch(String),
}

impl Checkout {
    /// How the checkout reads in a chip.
    pub fn label(&self) -> &str {
        match self {
            Checkout::Branch(branch) => branch,
            Checkout::Missing => "Path not here",
            Checkout::NotRepository => "Not a repository",
            Checkout::None => "No checkout",
        }
    }

    /// Why the checkout reads the way it does, for the menu.
    pub fn detail(&self) -> &'static str {
        match self {
            Checkout::Branch(_) => "Checked out on this machine.",
            Checkout::Missing => "This checkout is not on this machine.",
            Checkout::NotRepository => "This checkout is not a git repository.",
            Checkout::None => "Select a workspace first.",
        }
    }

    /// The branch name, when there is one to copy or attach.
    pub fn branch(&self) -> Option<&str> {
        match self {
            Checkout::Branch(branch) => Some(branch),
            _ => None,
        }
    }
}

/// Describe the checkout at `path`.
pub fn describe(path: &Path) -> Checkout {
    if !path.exists() {
        return Checkout::Missing;
    }

    match current_branch(path) {
        Some(branch) => Checkout::Branch(branch),
        None => Checkout::NotRepository,
    }
}

/// The branch, or the short commit when the checkout is detached.
pub fn current_branch(path: &Path) -> Option<String> {
    let git_dir = git_dir(path)?;
    let head = fs::read_to_string(git_dir.join("HEAD")).ok()?;

    parse_head(&head)
}

/// The branch name, or short commit, in the contents of a `HEAD` file.
pub fn parse_head(head: &str) -> Option<String> {
    let head = head.trim();

    if let Some(reference) = head.strip_prefix("ref: ") {
        let name = reference
            .strip_prefix("refs/heads/")
            .unwrap_or(reference)
            .trim();

        return (!name.is_empty()).then(|| name.to_string());
    }

    if head.len() >= 7 && head.chars().all(|character| character.is_ascii_hexdigit()) {
        return Some(head[..7].to_string());
    }

    None
}

/// Where a checkout's git directory lives.
fn git_dir(path: &Path) -> Option<PathBuf> {
    let dot_git = path.join(".git");

    if dot_git.is_dir() {
        return Some(dot_git);
    }

    let pointer = fs::read_to_string(&dot_git).ok()?;
    let target = pointer.trim().strip_prefix("gitdir:")?.trim();
    let target = PathBuf::from(target);

    Some(if target.is_absolute() {
        target
    } else {
        path.join(target)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("kloucy-git-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("scratch directory");

        path
    }

    #[test]
    fn reads_a_branch_out_of_head() {
        assert_eq!(
            parse_head("ref: refs/heads/feat/mobile-shared-spaces\n"),
            Some("feat/mobile-shared-spaces".to_string())
        );
        assert_eq!(parse_head("ref: refs/heads/main"), Some("main".to_string()));
    }

    #[test]
    fn falls_back_to_a_short_commit_when_detached() {
        assert_eq!(
            parse_head("3f9c1a2b8e4d5c6f7a8b9c0d1e2f3a4b5c6d7e8f"),
            Some("3f9c1a2".to_string())
        );
    }

    #[test]
    fn ignores_heads_it_cannot_read() {
        assert_eq!(parse_head(""), None);
        assert_eq!(parse_head("not a head"), None);
        assert_eq!(parse_head("ref: "), None);
        assert_eq!(parse_head("abc"), None);
    }

    #[test]
    fn reads_a_branch_from_a_checkout() {
        let repo = scratch_dir("repo");
        fs::create_dir_all(repo.join(".git")).expect("git directory");
        fs::write(repo.join(".git/HEAD"), "ref: refs/heads/plata\n").expect("head");

        assert_eq!(current_branch(&repo), Some("plata".to_string()));

        fs::remove_dir_all(&repo).ok();
    }

    #[test]
    fn follows_a_worktree_pointer() {
        let root = scratch_dir("worktree");
        let checkout = root.join("checkout");
        let git_dir = root.join("git-worktrees/plata");
        fs::create_dir_all(&checkout).expect("checkout");
        fs::create_dir_all(&git_dir).expect("git directory");
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/feat/composer\n").expect("head");
        fs::write(
            checkout.join(".git"),
            format!("gitdir: {}\n", git_dir.display()),
        )
        .expect("pointer");

        assert_eq!(current_branch(&checkout), Some("feat/composer".to_string()));

        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn reports_nothing_outside_a_checkout() {
        let plain = scratch_dir("plain");

        assert_eq!(current_branch(&plain), None);
        assert_eq!(current_branch(&plain.join("missing")), None);

        fs::remove_dir_all(&plain).ok();
    }

    #[test]
    fn describes_what_it_finds() {
        let plain = scratch_dir("describe");

        assert_eq!(describe(&plain.join("missing")), Checkout::Missing);
        assert_eq!(describe(&plain), Checkout::NotRepository);

        fs::create_dir_all(plain.join(".git")).expect("git directory");
        fs::write(plain.join(".git/HEAD"), "ref: refs/heads/plata\n").expect("head");
        assert_eq!(describe(&plain), Checkout::Branch("plata".to_string()));

        fs::remove_dir_all(&plain).ok();
    }

    #[test]
    fn reads_checkouts_as_labels() {
        assert_eq!(Checkout::Branch("main".to_string()).label(), "main");
        assert_eq!(Checkout::Branch("main".to_string()).branch(), Some("main"));
        assert_eq!(Checkout::Missing.label(), "Path not here");
        assert_eq!(Checkout::Missing.branch(), None);
        assert_eq!(Checkout::None.label(), "No checkout");
    }
}
