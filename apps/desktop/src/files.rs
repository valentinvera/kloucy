//! The files a thread can read from its checkout.
//!
//! The panel only needs a list of paths and the text of one file at a time, so
//! this stays out of the runner and the API: it reads the checkout that is on
//! this machine, with git when the checkout is a repository and a bounded walk
//! when it is not.

use std::fs;
use std::path::{Component, Path};
use std::process::Command;

/// The largest file the panel opens. Anything bigger is a binary artifact or a
/// generated bundle, and reading it helps nobody.
pub const MAX_BYTES: u64 = 200 * 1024;

/// How many entries a walk of a non-repository checkout keeps.
const MAX_WALK_ENTRIES: usize = 5000;

/// Directories a walk never descends into.
const SKIPPED_DIRECTORIES: [&str; 8] = [
    ".git",
    ".next",
    "build",
    "dist",
    "node_modules",
    "target",
    "vendor",
    "zig-cache",
];

/// The paths in `root`, tracked by git when it is a repository, walked when it
/// is not. Sorted, so the panel always reads the same way.
pub fn list(root: &Path) -> Vec<String> {
    if let Some(paths) = git_paths(root) {
        return paths;
    }

    walk(root)
}

/// Read `relative` inside `root`, refusing paths that leave the checkout and
/// files that are too large or not text.
pub fn read(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);

    if !stays_inside(root, &path) {
        return Err("That file is outside the checkout.".to_string());
    }

    let metadata = fs::metadata(&path).map_err(|_| "That file could not be read.".to_string())?;

    if !metadata.is_file() {
        return Err("That is not a file.".to_string());
    }

    if metadata.len() > MAX_BYTES {
        return Err("That file is too large to open.".to_string());
    }

    let bytes = fs::read(&path).map_err(|_| "That file could not be read.".to_string())?;

    if bytes.iter().take(8192).any(|byte| *byte == 0) {
        return Err("That file is binary.".to_string());
    }

    String::from_utf8(bytes).map_err(|_| "That file is not UTF-8.".to_string())
}

/// The syntect token for a path, when its extension names a language the
/// highlighter knows. An unknown extension renders as plain text.
pub fn language(path: &str) -> Option<&'static str> {
    let extension = Path::new(path).extension()?.to_str()?.to_lowercase();

    Some(match extension.as_str() {
        "bash" | "sh" | "zsh" => "bash",
        "c" | "h" => "c",
        "cpp" | "cc" | "hpp" => "cpp",
        "css" => "css",
        "go" => "go",
        "html" | "htm" => "html",
        "java" => "java",
        "js" | "jsx" | "mjs" | "cjs" => "js",
        "json" => "json",
        "kt" => "kotlin",
        "md" | "mdx" => "md",
        "py" => "py",
        "rs" => "rs",
        "sql" => "sql",
        "swift" => "swift",
        "toml" => "toml",
        "ts" | "tsx" | "mts" | "cts" => "ts",
        "xml" => "xml",
        "yaml" | "yml" => "yaml",
        _ => return None,
    })
}

/// The tracked and untracked files of a git repository, or nothing when the
/// directory is not one.
fn git_paths(root: &Path) -> Option<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut paths = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    paths.sort();
    paths.dedup();

    Some(paths)
}

/// A bounded walk of a checkout git does not know about.
fn walk(root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    let mut pending = vec![root.to_path_buf()];

    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };

        for entry in entries.flatten() {
            if paths.len() >= MAX_WALK_ENTRIES {
                break;
            }

            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            let name = entry.file_name().to_string_lossy().into_owned();

            if file_type.is_dir() {
                if !SKIPPED_DIRECTORIES.contains(&name.as_str()) {
                    pending.push(entry.path());
                }

                continue;
            }

            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };

            paths.push(relative.to_string_lossy().into_owned());
        }
    }

    paths.sort();
    paths
}

/// Whether `path` stays under `root`, after resolving the parts that do not
/// need the filesystem.
fn stays_inside(root: &Path, path: &Path) -> bool {
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return false;
    }

    let Ok(root) = root.canonicalize() else {
        return false;
    };

    let Ok(path) = path.canonicalize() else {
        return false;
    };

    path.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn scratch(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "kloucy-files-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));

        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("scratch directory");

        path
    }

    fn write(root: &Path, name: &str, contents: &[u8]) {
        let path = root.join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("parent directory");

        let mut file = fs::File::create(path).expect("file");
        file.write_all(contents).expect("write");
    }

    #[test]
    fn lists_a_directory_that_is_not_a_repository() {
        let root = scratch("walk");
        write(&root, "src/main.rs", b"fn main() {}\n");
        write(&root, "README.md", b"# hi\n");
        write(&root, "node_modules/pkg/index.js", b"module\n");

        let paths = list(&root);

        assert_eq!(paths, vec!["README.md", "src/main.rs"]);
    }

    #[test]
    fn reads_a_file_inside_the_checkout() {
        let root = scratch("read");
        write(&root, "src/lib.rs", b"pub fn answer() -> u32 { 42 }\n");

        assert_eq!(
            read(&root, "src/lib.rs").expect("a text file reads"),
            "pub fn answer() -> u32 { 42 }\n"
        );
    }

    #[test]
    fn refuses_a_path_that_leaves_the_checkout() {
        let root = scratch("escape");
        write(&root, "inside.txt", b"ok\n");

        assert!(read(&root, "../inside.txt").is_err());
        assert!(read(&root, "/etc/passwd").is_err());
    }

    #[test]
    fn refuses_binary_and_oversized_files() {
        let root = scratch("binary");
        write(&root, "image.bin", &[0, 1, 2, 3]);
        write(&root, "big.txt", &vec![b'a'; MAX_BYTES as usize + 1]);

        assert_eq!(read(&root, "image.bin"), Err("That file is binary.".into()));
        assert_eq!(
            read(&root, "big.txt"),
            Err("That file is too large to open.".into())
        );
    }

    #[test]
    fn names_the_language_of_a_path() {
        assert_eq!(language("src/main.rs"), Some("rs"));
        assert_eq!(language("app/page.tsx"), Some("ts"));
        assert_eq!(language("Cargo.toml"), Some("toml"));
        assert_eq!(language("LICENSE"), None);
    }
}
