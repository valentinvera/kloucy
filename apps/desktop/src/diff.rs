//! The change a thread has in its checkout.
//!
//! This reads `git diff HEAD` (tracked work, staged or not) and lists untracked
//! files as additions, so the panel can show a whole change without touching
//! the index or committing anything.

use std::path::Path;
use std::process::Command;

/// One file in the change, with the lines the panel draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDiff {
    pub added: usize,
    pub lines: Vec<DiffLine>,
    pub path: String,
    pub removed: usize,
}

/// How one line of a diff reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Added,
    Context,
    Hunk,
    Meta,
    Removed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    pub text: String,
}

/// How much the checkout has changed, without the lines themselves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChangeStat {
    pub added: usize,
    pub files: usize,
    pub removed: usize,
}

/// How many lines of one untracked file the panel reads.
const MAX_UNTRACKED_LINES: usize = 200;

/// How many untracked files the panel lists.
const MAX_UNTRACKED_FILES: usize = 40;

/// The change in `root`: tracked files against `HEAD`, plus untracked files as
/// additions. A directory that is not a repository has no change.
pub fn read(root: &Path) -> Result<Vec<FileDiff>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "HEAD", "--no-color", "--unified=3"])
        .output();

    let mut files = match output {
        Ok(output) if output.status.success() => parse(&String::from_utf8_lossy(&output.stdout)),
        // A repository without a first commit has no HEAD to diff against.
        Ok(_) => Vec::new(),
        Err(_) => return Err("That checkout could not be read.".to_string()),
    };

    files.extend(untracked(root));
    files.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(files)
}

/// Count what changed in `root` without reading the change itself. None means
/// the directory is not a repository.
pub fn stat(root: &Path) -> Option<ChangeStat> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "HEAD", "--numstat", "--no-color"])
        .output()
        .ok()?;

    // A repository without a first commit has no HEAD; its worktree still counts.
    let output = if output.status.success() {
        output
    } else {
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["diff", "--numstat", "--no-color"])
            .output()
            .ok()?
    };

    let mut stat = parse_numstat(&String::from_utf8_lossy(&output.stdout));

    for path in untracked_paths(root) {
        let Ok(content) = std::fs::read_to_string(root.join(&path)) else {
            continue;
        };

        stat.files += 1;
        stat.added += content.lines().count();
    }

    Some(stat)
}

/// Parse a unified diff, grouping lines by the file they belong to.
fn parse(diff: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();

    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            files.push(FileDiff {
                added: 0,
                lines: Vec::new(),
                path: file_path(line).unwrap_or_default(),
                removed: 0,
            });
            continue;
        }

        let Some(file) = files.last_mut() else {
            continue;
        };

        if line.starts_with("index ")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
            || line.starts_with("new file mode")
            || line.starts_with("deleted file mode")
        {
            continue;
        }

        let (kind, text) = if line.starts_with("@@") {
            (LineKind::Hunk, line.to_string())
        } else if let Some(added) = line.strip_prefix('+') {
            file.added += 1;
            (LineKind::Added, added.to_string())
        } else if let Some(removed) = line.strip_prefix('-') {
            file.removed += 1;
            (LineKind::Removed, removed.to_string())
        } else if let Some(context) = line.strip_prefix(' ') {
            (LineKind::Context, context.to_string())
        } else {
            (LineKind::Meta, line.to_string())
        };

        file.lines.push(DiffLine { kind, text });
    }

    files
}

/// The `b` path of a `diff --git a/x b/x` line.
fn file_path(line: &str) -> Option<String> {
    let rest = line.strip_prefix("diff --git ")?;
    let path = rest.rsplit_once(" b/")?.1;

    Some(path.to_string())
}

/// Add up `git diff --numstat` lines. A binary file prints `-` for both
/// counts; it still counts as a changed file.
fn parse_numstat(output: &str) -> ChangeStat {
    let mut stat = ChangeStat::default();

    for line in output.lines() {
        let mut fields = line.split('\t');
        let added = fields.next().unwrap_or_default();
        let removed = fields.next().unwrap_or_default();

        stat.files += 1;
        stat.added += added.parse::<usize>().unwrap_or_default();
        stat.removed += removed.parse::<usize>().unwrap_or_default();
    }

    stat
}

/// The paths git reports as untracked.
fn untracked_paths(root: &Path) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--others", "--exclude-standard"])
        .output()
    else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|path| !path.is_empty())
        .map(str::to_string)
        .collect()
}

/// Untracked files, read as additions.
fn untracked(root: &Path) -> Vec<FileDiff> {
    untracked_paths(root)
        .into_iter()
        .take(MAX_UNTRACKED_FILES)
        .filter_map(|path| {
            let content = crate::files::read(root, &path).ok()?;
            let lines = content
                .lines()
                .take(MAX_UNTRACKED_LINES)
                .map(|line| DiffLine {
                    kind: LineKind::Added,
                    text: line.to_string(),
                })
                .collect::<Vec<_>>();

            Some(FileDiff {
                added: lines.len(),
                lines,
                path: path.to_string(),
                removed: 0,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
diff --git a/src/main.rs b/src/main.rs
index 1111111..2222222 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
-    println!(\"old\");
+    println!(\"new\");
+    println!(\"more\");
 }
diff --git a/README.md b/README.md
index 3333333..4444444 100644
--- a/README.md
+++ b/README.md
@@ -1 +1 @@
-# Kloucy
+# Kloucy desktop
";

    #[test]
    fn groups_lines_by_file() {
        let files = parse(SAMPLE);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[0].added, 2);
        assert_eq!(files[0].removed, 1);
        assert_eq!(files[1].path, "README.md");
        assert_eq!(files[1].added, 1);
        assert_eq!(files[1].removed, 1);
    }

    #[test]
    fn keeps_hunks_and_context() {
        let files = parse(SAMPLE);
        let kinds = files[0]
            .lines
            .iter()
            .map(|line| line.kind)
            .collect::<Vec<_>>();

        assert!(kinds.contains(&LineKind::Hunk));
        assert!(kinds.contains(&LineKind::Context));
        assert!(kinds.contains(&LineKind::Added));
        assert!(kinds.contains(&LineKind::Removed));
    }

    #[test]
    fn adds_up_a_numstat() {
        let stat = parse_numstat("12\t3\tsrc/main.rs\n-\t-\tassets/logo.png\n");

        assert_eq!(stat.files, 2);
        assert_eq!(stat.added, 12);
        assert_eq!(stat.removed, 3);
    }

    #[test]
    fn reads_the_b_side_of_a_diff_header() {
        assert_eq!(
            file_path("diff --git a/app/page.tsx b/app/page.tsx"),
            Some("app/page.tsx".to_string())
        );
        assert_eq!(file_path("not a diff"), None);
    }
}
