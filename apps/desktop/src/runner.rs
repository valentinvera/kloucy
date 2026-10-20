use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::api::KloucyClient;

/// Tracks the agent processes the desktop has started, so a session stop can
/// terminate the local process instead of only updating the API.
#[derive(Default)]
pub struct RunRegistry {
    processes: Mutex<HashMap<String, u32>>,
    stopped: Mutex<HashSet<String>>,
}

impl RunRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn register(&self, session_id: &str, pid: u32) {
        if let Ok(mut stopped) = self.stopped.lock() {
            stopped.remove(session_id);
        }

        if let Ok(mut processes) = self.processes.lock() {
            processes.insert(session_id.to_string(), pid);
        }
    }

    fn complete(&self, session_id: &str) {
        if let Ok(mut processes) = self.processes.lock() {
            processes.remove(session_id);
        }
    }

    /// Terminate the agent process for a session, if one is running locally.
    pub fn stop(&self, session_id: &str) -> bool {
        let pid = self
            .processes
            .lock()
            .ok()
            .and_then(|mut processes| processes.remove(session_id));

        let Some(pid) = pid else {
            return false;
        };

        if let Ok(mut stopped) = self.stopped.lock() {
            stopped.insert(session_id.to_string());
        }

        terminate(pid);
        true
    }

    fn take_stopped(&self, session_id: &str) -> bool {
        self.stopped
            .lock()
            .map(|mut stopped| stopped.remove(session_id))
            .unwrap_or(false)
    }
}

#[cfg(unix)]
fn terminate(pid: u32) {
    // The child is started as its own process group leader, so signalling the
    // group also stops anything the agent spawned.
    unsafe {
        libc::kill(-(pid as i32), libc::SIGTERM);
    }
}

#[cfg(not(unix))]
fn terminate(_pid: u32) {}

/// The command to run for a prompt.
///
/// A template may place the prompt with `{prompt}`; otherwise the prompt is
/// appended as the last argument, which is what one-shot agent commands expect
/// (`codex exec … "the prompt"`). The prompt is always single-quoted, so it
/// reaches the agent as one argument and never as shell syntax.
pub fn invocation(command: &str, prompt: &str) -> String {
    let quoted = shell_quote(prompt);

    if command.contains("{prompt}") {
        return command.replace("{prompt}", &quoted);
    }

    format!("{command} {quoted}")
}

/// Quote a value for `sh -c`, where single quotes take everything literally.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// Run an agent command in the given directory, streaming its output to the
/// session logs, marking the session finished, and creating a review when the
/// run changed files.
pub fn run_agent(
    registry: Arc<RunRegistry>,
    client: KloucyClient,
    workspace_id: String,
    session_id: String,
    command: String,
    cwd: String,
) {
    let _ = client.append_log(&session_id, "system", &format!("$ {command}"));

    let mut process = Command::new("sh");
    process
        .arg("-c")
        .arg(&command)
        .current_dir(&cwd)
        // Some agents read stdin when it is not a terminal and would wait
        // forever for input the window cannot give them.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        process.process_group(0);
    }

    let mut child = match process.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = client.append_log(
                &session_id,
                "system",
                &format!("Failed to start agent command: {error}"),
            );
            let _ = client.finish_session(&session_id, "failed");
            return;
        }
    };

    registry.register(&session_id, child.id());

    let stderr_handle = child.stderr.take().map(|stderr| {
        let stderr_client = client.clone();
        let stderr_session = session_id.clone();
        std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                let _ = stderr_client.append_log(&stderr_session, "stderr", &line);
            }
        })
    });

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = client.append_log(&session_id, "stdout", &line);
        }
    }

    if let Some(handle) = stderr_handle {
        let _ = handle.join();
    }

    let success = child.wait().map(|status| status.success()).unwrap_or(false);
    registry.complete(&session_id);

    if registry.take_stopped(&session_id) {
        let _ = client.append_log(
            &session_id,
            "system",
            "Session stopped; the agent process was terminated.",
        );
        return;
    }

    let _ = client.finish_session(&session_id, if success { "done" } else { "failed" });

    create_review_for_changes(&client, &workspace_id, &session_id, &cwd);
}

fn create_review_for_changes(
    client: &KloucyClient,
    workspace_id: &str,
    session_id: &str,
    cwd: &str,
) {
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("status")
        .arg("--porcelain")
        .output()
    else {
        return;
    };

    let changed = String::from_utf8_lossy(&output.stdout);
    let trimmed = changed.trim();

    if trimmed.is_empty() {
        return;
    }

    let _ = client.append_log(session_id, "system", "Changes detected; created a review.");
    let _ = client.create_review(workspace_id, session_id, trimmed);
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn spawn_group(command: &str) -> std::process::Child {
        let mut process = Command::new("sh");
        process
            .arg("-c")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        use std::os::unix::process::CommandExt;

        process.process_group(0);

        process.spawn().expect("agent command should start")
    }

    #[test]
    fn stop_terminates_the_running_process() {
        let registry = RunRegistry::new();
        let mut child = spawn_group("sleep 30");

        registry.register("session-1", child.id());
        assert!(registry.stop("session-1"));
        assert!(registry.take_stopped("session-1"));

        let status = child.wait().expect("process should be reapable");
        assert!(!status.success());
    }

    #[test]
    fn stop_without_a_running_process_is_a_no_op() {
        let registry = RunRegistry::new();

        assert!(!registry.stop("session-1"));
        assert!(!registry.take_stopped("session-1"));
    }

    #[test]
    fn register_clears_a_previous_stop() {
        let registry = RunRegistry::new();
        let mut child = spawn_group("sleep 30");

        registry.register("session-1", child.id());
        assert!(registry.stop("session-1"));
        let _ = child.wait();

        let mut next = spawn_group("true");
        registry.register("session-1", next.id());
        assert!(!registry.take_stopped("session-1"));
        let _ = next.wait();
    }

    #[test]
    fn appends_the_prompt_to_a_plain_command() {
        assert_eq!(
            invocation("codex exec --full-auto", "fix the login"),
            "codex exec --full-auto 'fix the login'"
        );
    }

    #[test]
    fn fills_a_prompt_placeholder() {
        assert_eq!(
            invocation("opencode run {prompt}", "hola"),
            "opencode run 'hola'"
        );
    }

    #[test]
    fn keeps_prompt_text_out_of_the_shell() {
        assert_eq!(
            invocation("agent", "it's $HOME; rm -rf /"),
            r"agent 'it'\''s $HOME; rm -rf /'"
        );
    }
}
