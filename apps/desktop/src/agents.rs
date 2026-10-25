//! The agents the composer can start.
//!
//! Kloucy does not ask the user to register agents up front: the composer offers
//! what is installed on the machine, and the first message that gets an answer
//! stores the agent record behind it.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
        // `--json` makes the CLI answer as events, so the reply and the trace
        // arrive apart instead of sharing one stream of text.
        "codex" => Some("exec --json"),
        "opencode" => Some("run"),
        _ => None,
    }
}

/// The command to run for a provider's executable.
fn command_for(provider: &str, executable: &str) -> String {
    let command = match one_shot(provider) {
        Some(flag) => format!("{executable} {flag}"),
        None => executable.to_string(),
    };

    with_event_flag(provider, command)
}

/// The flag that makes a provider report as events, when it has one.
fn event_flag(provider: &str) -> Option<&'static str> {
    match provider {
        "codex" => Some("--json"),
        _ => None,
    }
}

/// Add the events flag to a command that does not ask for it yet.
fn with_event_flag(provider: &str, command: String) -> String {
    match event_flag(provider) {
        Some(flag) if !command.contains(flag) => format!("{command} {flag}"),
        _ => command,
    }
}

/// Whether a stored command carries its own arguments, in which case it is kept
/// as it was written instead of being replaced by the one-shot form.
fn has_arguments(command: &str) -> bool {
    command.split_whitespace().count() > 1
}

/// A model an agent can be asked to run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelChoice {
    /// The effort this model runs at when nothing is picked.
    pub default_effort: Option<String>,
    /// The efforts this model accepts, when it has a ladder of its own.
    pub efforts: Vec<String>,
    pub id: String,
    pub label: String,
}

/// What an installed agent can run, as far as its CLI will say.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelCatalog {
    pub efforts: Vec<String>,
    pub fast: bool,
    pub models: Vec<ModelChoice>,
}

/// Ask an installed CLI what it can run. A CLI that cannot list its models, or
/// that fails, simply offers none and the picker keeps the default model.
pub fn model_catalog(provider: &str, executable: &str) -> ModelCatalog {
    match provider {
        "codex" => run_capture(executable, &["debug", "models"])
            .map(|json| parse_codex_models(&json))
            .unwrap_or_default(),
        "opencode" => opencode_catalog(executable),
        "claude-code" => ModelCatalog {
            efforts: ["low", "medium", "high", "max"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            fast: false,
            models: ["sonnet", "opus", "haiku"]
                .into_iter()
                .map(|id| ModelChoice {
                    default_effort: None,
                    efforts: Vec::new(),
                    id: id.to_string(),
                    label: id.to_string(),
                })
                .collect(),
        },
        _ => ModelCatalog::default(),
    }
}

/// Run a CLI and collect its stdout, giving up after a few seconds: a probe must
/// never leave the picker waiting on a command that will not come back.
fn run_capture(executable: &str, args: &[&str]) -> Option<String> {
    let mut child = Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buffer = String::new();
        let _ = stdout.read_to_string(&mut buffer);
        buffer
    });

    let deadline = Instant::now() + Duration::from_secs(12);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(50));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };

    let output = reader.join().ok()?;

    status.success().then_some(output)
}

/// The models OpenCode can run: its own listing first, then the model cache it
/// keeps on disk, which is what is left when the listing cannot answer.
fn opencode_catalog(executable: &str) -> ModelCatalog {
    // The cache names each model's effort options, which the listing does not,
    // so it is asked first and the CLI is the fallback.
    let cached = read_cache_file("models.json")
        .zip(read_data_file("auth.json"))
        .map(|(cache, auth)| parse_opencode_cache(&cache, &auth))
        .unwrap_or_default();

    if !cached.models.is_empty() {
        return cached;
    }

    for args in [vec!["models", "--standalone"], vec!["models"]] {
        if let Some(text) = run_capture(executable, &args) {
            let catalog = parse_opencode_models(&text);

            if !catalog.models.is_empty() {
                return catalog;
            }
        }
    }

    ModelCatalog::default()
}

/// A file under the XDG cache or data directory.
fn read_cache_file(name: &str) -> Option<String> {
    std::fs::read_to_string(
        xdg_base("XDG_CACHE_HOME", ".cache")?
            .join("opencode")
            .join(name),
    )
    .ok()
}

/// A file OpenCode keeps in the XDG data directory.
fn read_data_file(name: &str) -> Option<String> {
    std::fs::read_to_string(
        xdg_base("XDG_DATA_HOME", ".local/share")?
            .join("opencode")
            .join(name),
    )
    .ok()
}

fn xdg_base(variable: &str, fallback: &str) -> Option<PathBuf> {
    std::env::var(variable)
        .ok()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(fallback))
        })
}

/// Read OpenCode's local model cache: the providers it is signed into plus its
/// own free ones, which is the same set its `models` command prints.
pub fn parse_opencode_cache(cache: &str, auth: &str) -> ModelCatalog {
    let Ok(registry) = serde_json::from_str::<serde_json::Value>(cache) else {
        return ModelCatalog::default();
    };
    let signed_into: Vec<String> = serde_json::from_str::<serde_json::Value>(auth)
        .ok()
        .and_then(|value| {
            value
                .as_object()
                .map(|map| map.keys().cloned().collect::<Vec<_>>())
        })
        .unwrap_or_default();

    let mut models = Vec::new();

    for (provider, entry) in registry.as_object().into_iter().flatten() {
        if provider != "opencode" && !signed_into.iter().any(|signed| signed == provider) {
            continue;
        }

        for (key, model) in entry["models"].as_object().into_iter().flatten() {
            let Some(model_id) = model["id"].as_str().or(Some(key.as_str())) else {
                continue;
            };

            let id = if model_id.contains('/') {
                model_id.to_string()
            } else {
                format!("{provider}/{model_id}")
            };

            let efforts = model["reasoning_options"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|option| option["type"].as_str() == Some("effort"))
                .flat_map(|option| {
                    option["values"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|value| value.as_str().map(str::to_string))
                })
                .collect();

            models.push(ModelChoice {
                default_effort: None,
                efforts,
                id,
                label: model["name"].as_str().unwrap_or(model_id).to_string(),
            });
        }
    }

    ModelCatalog {
        models,
        ..ModelCatalog::default()
    }
}

/// Codex reports its catalog as JSON (`codex debug models`).
pub fn parse_codex_models(json: &str) -> ModelCatalog {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return ModelCatalog::default();
    };

    let mut catalog = ModelCatalog::default();

    for model in value["models"].as_array().into_iter().flatten() {
        let Some(id) = model["slug"].as_str() else {
            continue;
        };

        let levels = model["supported_reasoning_levels"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|level| level["effort"].as_str().map(str::to_string))
            .collect::<Vec<_>>();

        catalog.models.push(ModelChoice {
            default_effort: model["default_reasoning_level"]
                .as_str()
                .map(str::to_string),
            efforts: levels,
            id: id.to_string(),
            label: model["display_name"].as_str().unwrap_or(id).to_string(),
        });

        for level in model["supported_reasoning_levels"]
            .as_array()
            .into_iter()
            .flatten()
        {
            if let Some(effort) = level["effort"].as_str() {
                if !catalog.efforts.iter().any(|known| known == effort) {
                    catalog.efforts.push(effort.to_string());
                }
            }
        }

        if !catalog.fast
            && model["service_tiers"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|tier| tier["id"].as_str() == Some("priority"))
        {
            catalog.fast = true;
        }
    }

    catalog
}

/// OpenCode prints one `provider/model` id per line (`opencode models`).
pub fn parse_opencode_models(text: &str) -> ModelCatalog {
    let models = text
        .lines()
        .map(str::trim)
        .filter(|line| line.contains('/'))
        .map(|line| ModelChoice {
            default_effort: None,
            efforts: Vec::new(),
            id: line.to_string(),
            label: line.rsplit('/').next().unwrap_or(line).to_string(),
        })
        .collect();

    ModelCatalog {
        models,
        ..ModelCatalog::default()
    }
}

/// The arguments that ask an agent for a model, an effort and fast mode.
pub fn model_args(
    provider: &str,
    model: Option<&str>,
    effort: Option<&str>,
    fast: bool,
) -> Vec<String> {
    let mut args = Vec::new();

    if let Some(model) = model {
        let flag = if provider == "codex" { "-m" } else { "--model" };
        let model = match (provider, effort) {
            // OpenCode asks for a model variant in the model id itself.
            ("opencode", Some(effort)) => format!("{model}#{effort}"),
            _ => model.to_string(),
        };
        args.extend([flag.to_string(), quote(&model)]);

        if provider == "claude-code" {
            if let Some(effort) = effort {
                args.extend(["--effort".to_string(), quote(effort)]);
            }

            return args;
        }
    }

    if provider == "codex" {
        if let Some(effort) = effort {
            args.extend([
                "-c".to_string(),
                format!("model_reasoning_effort={}", quote(effort)),
            ]);
        }

        if fast {
            args.extend(["-c".to_string(), "service_tier='priority'".to_string()]);
        }
    }

    args
}

fn quote(value: &str) -> String {
    crate::runner::shell_quote(value)
}

/// The command that continues an agent-side session, when the CLI can.
///
/// Only providers whose resume flag is known get one; the rest start a fresh
/// run, and the thread still keeps the earlier turns on screen.
pub fn resume_command(provider: &str, command: &str, agent_session: &str) -> Option<String> {
    if provider != "codex" || !command.contains("exec") {
        return None;
    }

    let command = command.trim_end();
    let id = format!("'{agent_session}'");

    match command.strip_suffix("--json") {
        Some(base) => Some(format!("{} resume {id} --json", base.trim_end())),
        None => Some(format!("{command} resume {id}")),
    }
}

/// The agents the composer can start, one per provider, and only the ones that
/// are actually installed: an agent record for a CLI that is not here cannot
/// run, so offering it would be a dead entry in the picker.
pub fn choices(agents: &[Agent], detected: &[DetectedAgent]) -> Vec<AgentChoice> {
    let mut choices: Vec<AgentChoice> = Vec::new();

    for cli in detected {
        let stored = agents
            .iter()
            .find(|agent| agent.provider == cli.provider && agent.status != "disabled");

        // The stored form wins, but the answer file still has to be asked for
        // or the thread would show the agent's whole trace.
        let stored_command = stored
            .and_then(stored_command)
            .filter(|command| has_arguments(command))
            .map(|command| with_event_flag(&cli.provider, command));

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
                command: "/usr/bin/codex exec --json".to_string(),
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
        assert_eq!(choices[0].command, "/opt/codex exec --json");
    }

    #[test]
    fn replaces_a_bare_stored_command_with_the_one_shot_form() {
        let stored = agent("agent-1", "Codex", "codex", Some("codex"), "ready");
        let choices = choices(&[stored], &[cli("Codex", "codex", "/usr/bin/codex")]);

        assert_eq!(choices[0].command, "/usr/bin/codex exec --json");
    }

    #[test]
    fn hides_agents_that_are_not_installed() {
        let stored = agent("agent-1", "Custom", "custom", Some("custom run"), "ready");

        assert!(choices(&[stored], &[]).is_empty());
    }

    #[test]
    fn skips_disabled_agents_that_are_not_installed() {
        let stored = agent("agent-1", "Codex", "codex", Some("codex exec"), "disabled");

        assert!(choices(&[stored], &[]).is_empty());
    }

    #[test]
    fn resumes_a_conversation_when_the_cli_can() {
        assert_eq!(
            resume_command("codex", "/usr/bin/codex exec --json", "thread-1"),
            Some("/usr/bin/codex exec resume 'thread-1' --json".to_string())
        );
        assert_eq!(
            resume_command("claude-code", "/usr/bin/claude -p", "thread-1"),
            None
        );
    }

    #[test]
    fn reads_the_codex_model_catalog() {
        let json = r#"{"models":[
            {"slug":"gpt-6.1-sol","display_name":"GPT-6.1-Sol","default_reasoning_level":"high",
             "supported_reasoning_levels":[{"effort":"low"},{"effort":"high"}],
             "service_tiers":[{"id":"priority","name":"Fast"}]},
            {"slug":"gpt-6-sol","display_name":"GPT-6-Sol",
             "supported_reasoning_levels":[{"effort":"medium"}],
             "service_tiers":[]}
        ]}"#;
        let catalog = parse_codex_models(json);

        assert_eq!(
            catalog
                .models
                .iter()
                .map(|model| model.label.as_str())
                .collect::<Vec<_>>(),
            vec!["GPT-6.1-Sol", "GPT-6-Sol"]
        );
        assert_eq!(catalog.efforts, vec!["low", "high", "medium"]);
        assert_eq!(catalog.models[0].default_effort, Some("high".to_string()));
        assert_eq!(catalog.models[1].default_effort, None);
        assert!(catalog.fast);
    }

    #[test]
    fn reads_the_opencode_model_list() {
        let catalog =
            parse_opencode_models("opencode-go/gpt-6-luna\n\nopencode-go/glm-5.3\nnoise\n");

        assert_eq!(
            catalog
                .models
                .iter()
                .map(|model| (model.id.as_str(), model.label.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("opencode-go/gpt-6-luna", "gpt-6-luna"),
                ("opencode-go/glm-5.3", "glm-5.3")
            ]
        );
    }

    #[test]
    fn reads_the_opencode_cache_for_signed_in_providers() {
        let cache = r#"{
            "opencode": {"models": {
                "opencode/big": {"id":"opencode/big","name":"Big Pickle",
                                 "reasoning_options":[{"type":"effort","values":["low","high"]}]},
                "bare": {"id":"bare","name":"Bare"}
            }},
            "openrouter": {"models": {"openrouter/sonnet": {"id":"openrouter/sonnet","name":"Sonnet"}}},
            "someone-else": {"models": {"someone-else/x": {"id":"someone-else/x","name":"X"}}}
        }"#;
        let catalog = parse_opencode_cache(cache, r#"{"openrouter": {"type": "api"}}"#);

        assert_eq!(
            catalog
                .models
                .iter()
                .map(|model| (model.id.as_str(), model.label.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("opencode/big", "Big Pickle"),
                ("opencode/bare", "Bare"),
                ("openrouter/sonnet", "Sonnet")
            ]
        );
        assert_eq!(catalog.models[0].efforts, vec!["low", "high"]);
        assert!(catalog.models[1].efforts.is_empty());
    }

    #[test]
    fn asks_codex_for_model_effort_and_fast_mode() {
        assert_eq!(
            model_args("codex", Some("gpt-6.1-sol"), Some("high"), true),
            vec![
                "-m",
                "'gpt-6.1-sol'",
                "-c",
                "model_reasoning_effort='high'",
                "-c",
                "service_tier='priority'"
            ]
        );
        assert!(model_args("codex", None, None, false).is_empty());
    }

    #[test]
    fn asks_other_clis_for_a_model() {
        assert_eq!(
            model_args(
                "opencode",
                Some("opencode-go/gpt-6-luna"),
                Some("high"),
                true
            ),
            vec!["--model", "'opencode-go/gpt-6-luna#high'"]
        );
        assert_eq!(
            model_args("opencode", Some("opencode-go/glm-5.3"), None, false),
            vec!["--model", "'opencode-go/glm-5.3'"]
        );
        assert_eq!(
            model_args("claude-code", Some("sonnet"), Some("high"), false),
            vec!["--model", "'sonnet'", "--effort", "'high'"]
        );
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
