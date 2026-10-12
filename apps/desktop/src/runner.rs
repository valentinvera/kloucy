use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::api::KloucyClient;

/// Run an agent command in the given directory, streaming its output to the
/// session logs and marking the session finished when it exits.
pub fn run_agent(client: KloucyClient, session_id: String, command: String, cwd: String) {
    let _ = client.append_log(&session_id, "system", &format!("$ {command}"));

    let spawned = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .current_dir(&cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match spawned {
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
    let _ = client.finish_session(&session_id, if success { "done" } else { "failed" });
}
