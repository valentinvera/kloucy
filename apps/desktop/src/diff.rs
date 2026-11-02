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
        } else if line.starts_with('+') {
            file.added += 1;
            (LineKind::Added, line[1..].to_string())
        } else if line.starts_with('-') {
            file.removed += 1;
            (LineKind::Removed, line[1..].to_string())
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

/// Untracked files, read as additions.
fn untracked(root: &Path) -> Vec<FileDiff> {
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
        .take(MAX_UNTRACKED_FILES)
        .filter_map(|path| {
            let content = crate::files::read(root, path).ok()?;
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
    fn reads_the_b_side_of_a_diff_header() {
        assert_eq!(
            file_path("diff --git a/app/page.tsx b/app/page.tsx"),
            Some("app/page.tsx".to_string())
        );
        assert_eq!(file_path("not a diff"), None);
    }
}
