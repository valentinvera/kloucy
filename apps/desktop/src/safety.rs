//! The workspace safety rule that decides whether an agent command may start.

/// Whether `command` may run under `mode`, given the workspace's safe commands.
///
/// `open` runs anything. `guarded` only allows a command when a safe command
/// equals it or is a prefix of it, so listing `codex` allows
/// `codex exec --full-auto` but not `codexfoo`.
pub fn command_allowed(mode: &str, command: &str, safe_commands: &[String]) -> bool {
    if mode != "guarded" {
        return true;
    }

    let command = command.trim();

    if command.is_empty() {
        return false;
    }

    safe_commands.iter().any(|allowed| {
        let allowed = allowed.trim();
        !allowed.is_empty() && (command == allowed || command.starts_with(&format!("{allowed} ")))
    })
}

/// The workspace's safe commands with `command` added when it is missing.
pub fn with_safe_command(safe_commands: &[String], command: &str) -> Vec<String> {
    let command = command.trim();

    if command.is_empty()
        || safe_commands
            .iter()
            .any(|allowed| allowed.trim() == command)
    {
        return safe_commands.to_vec();
    }

    let mut next = safe_commands.to_vec();
    next.push(command.to_string());
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed(commands: &[&str]) -> Vec<String> {
        commands
            .iter()
            .map(|command| (*command).to_string())
            .collect()
    }

    #[test]
    fn open_workspaces_run_anything() {
        assert!(command_allowed("open", "codex exec --full-auto", &[]));
        assert!(command_allowed("", "anything", &[]));
    }

    #[test]
    fn guarded_workspaces_need_a_matching_safe_command() {
        let safe = allowed(&["codex"]);

        assert!(command_allowed("guarded", "codex", &safe));
        assert!(command_allowed("guarded", "codex exec --full-auto", &safe));
        assert!(!command_allowed("guarded", "codexfoo", &safe));
        assert!(!command_allowed("guarded", "aider --yes", &safe));
    }

    #[test]
    fn guarded_workspaces_refuse_without_safe_commands() {
        assert!(!command_allowed("guarded", "bun test", &[]));
        assert!(!command_allowed(
            "guarded",
            "bun test",
            &allowed(&["", "  "])
        ));
        assert!(!command_allowed("guarded", "", &allowed(&["bun"])));
    }

    #[test]
    fn adds_a_missing_safe_command() {
        let safe = allowed(&["cargo test"]);

        assert_eq!(
            with_safe_command(&safe, "codex exec --full-auto"),
            vec![
                "cargo test".to_string(),
                "codex exec --full-auto".to_string()
            ]
        );
    }

    #[test]
    fn does_not_add_a_command_it_already_allows() {
        let safe = allowed(&[" codex "]);

        assert_eq!(with_safe_command(&safe, "codex"), safe);
        assert_eq!(with_safe_command(&safe, "   "), safe);
    }

    #[test]
    fn matching_ignores_surrounding_whitespace() {
        let safe = allowed(&[" cargo test "]);

        assert!(command_allowed("guarded", "cargo test", &safe));
        assert!(command_allowed("guarded", "  cargo test --all", &safe));
    }
}
