//! The agents the composer can start.
//!
//! Kloucy does not ask the user to register agents up front: the composer offers
//! what is installed on the machine, and the first message that gets an answer
//! stores the agent record behind it.

use crate::api::Agent;
use crate::detect::DetectedAgent;

/// One entry in the composer's agent menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentChoice {
    /// The stored agent, when a previous message already created one.
    pub agent_id: Option<String>,
    pub command: String,
    pub name: String,
    pub provider: String,
}

/// The flag that makes a provider answer once and exit, instead of opening its
/// interactive session.
fn one_shot(provider: &str) -> Option<&'static str> {
    match provider {
        "claude-code" => Some("-p"),
        "codex" => Some("exec"),
        "opencode" => Some("run"),
        _ => None,
    }
}

/// The command to run for a provider's executable.
fn command_for(provider: &str, executable: &str) -> String {
    match one_shot(provider) {
        Some(flag) => format!("{executable} {flag}"),
        None => executable.to_string(),
    }
}

/// Whether a stored command carries its own arguments, in which case it is kept
/// as it was written instead of being replaced by the one-shot form.
fn has_arguments(command: &str) -> bool {
    command.split_whitespace().count() > 1
}

/// The agents the composer can start, one per provider: what is installed on
/// this machine first, then stored agents for providers detection missed.
pub fn choices(agents: &[Agent], detected: &[DetectedAgent]) -> Vec<AgentChoice> {
    let mut choices: Vec<AgentChoice> = Vec::new();

    for cli in detected {
        let stored = agents
            .iter()
            .find(|agent| agent.provider == cli.provider && agent.status != "disabled");

        let stored_command = stored
            .and_then(stored_command)
            .filter(|command| has_arguments(command));

        choices.push(AgentChoice {
            agent_id: stored.map(|agent| agent.id.clone()),
            command: stored_command
                .unwrap_or_else(|| command_for(&cli.provider, &cli.executable_path)),
            name: stored
                .map(|agent| agent.name.clone())
                .unwrap_or_else(|| cli.name.clone()),
            provider: cli.provider.clone(),
        });
    }

    for agent in agents {
        if agent.status == "disabled"
            || choices
                .iter()
                .any(|choice| choice.provider == agent.provider)
        {
            continue;
        }

        let Some(command) = stored_command(agent) else {
            continue;
        };

        choices.push(AgentChoice {
            agent_id: Some(agent.id.clone()),
            command,
            name: agent.name.clone(),
            provider: agent.provider.clone(),
        });
    }

    choices.sort_by_key(|choice| choice.name.to_lowercase());
    choices
}

/// The command an agent record would run, if it has one.
fn stored_command(agent: &Agent) -> Option<String> {
    agent
        .command_template
        .clone()
        .or_else(|| agent.executable_path.clone())
        .filter(|command| !command.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(id: &str, name: &str, provider: &str, command: Option<&str>, status: &str) -> Agent {
        Agent {
            command_template: command.map(str::to_string),
            executable_path: None,
            id: id.to_string(),
            name: name.to_string(),
            provider: provider.to_string(),
            status: status.to_string(),
        }
    }

    fn cli(name: &str, provider: &str, path: &str) -> DetectedAgent {
        DetectedAgent {
            executable_path: path.to_string(),
            name: name.to_string(),
            provider: provider.to_string(),
        }
    }

    #[test]
    fn offers_installed_agents_with_a_one_shot_command() {
        let choices = choices(&[], &[cli("Codex", "codex", "/usr/bin/codex")]);

        assert_eq!(
            choices,
            vec![AgentChoice {
                agent_id: None,
                command: "/usr/bin/codex exec".to_string(),
                name: "Codex".to_string(),
                provider: "codex".to_string(),
            }]
        );
    }

    #[test]
    fn links_a_stored_agent_to_its_provider() {
        let stored = agent(
            "agent-1",
            "Codex",
            "codex",
            Some("/opt/codex exec"),
            "ready",
        );
        let choices = choices(&[stored], &[cli("Codex", "codex", "/usr/bin/codex")]);

        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].agent_id, Some("agent-1".to_string()));
        assert_eq!(choices[0].command, "/opt/codex exec");
    }

    #[test]
    fn replaces_a_bare_stored_command_with_the_one_shot_form() {
        let stored = agent("agent-1", "Codex", "codex", Some("codex"), "ready");
        let choices = choices(&[stored], &[cli("Codex", "codex", "/usr/bin/codex")]);

        assert_eq!(choices[0].command, "/usr/bin/codex exec");
    }

    #[test]
    fn keeps_stored_agents_detection_missed() {
        let stored = agent("agent-1", "Custom", "custom", Some("custom run"), "ready");
        let choices = choices(&[stored], &[]);

        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].agent_id, Some("agent-1".to_string()));
    }

    #[test]
    fn skips_disabled_agents_that_are_not_installed() {
        let stored = agent("agent-1", "Codex", "codex", Some("codex exec"), "disabled");

        assert!(choices(&[stored], &[]).is_empty());
    }

    #[test]
    fn sorts_by_name() {
        let choices = choices(
            &[],
            &[
                cli("OpenCode", "opencode", "/usr/bin/opencode"),
                cli("Codex", "codex", "/usr/bin/codex"),
            ],
        );

        assert_eq!(
            choices
                .iter()
                .map(|choice| choice.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Codex", "OpenCode"]
        );
    }
}
