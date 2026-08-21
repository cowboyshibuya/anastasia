//! Helpers shared by the provider drivers: the Computer Use configuration
//! each provider needs handed to it differently, stderr triage, and tool-name
//! classification.

use std::fs;

use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::{Path, PathBuf};

use anyhow::{Context as _, anyhow};
use serde_json::Value;

use super::computer_use as computer_use_runtime;
use crate::driver::DriverEventSender;
use crate::model::{ActivityKind, CacheUsage, ProviderKind};

/// The context-window occupancy of one API call from a Claude-wire `usage`
/// object (Claude Code and Amp share the format): prompt (fresh + cached) plus
/// output. Multi-call messages carry per-call `iterations`; the last one is
/// the live context, and summed outer fields would double-count cache reads.
pub(super) fn claude_context_tokens(usage: &Value) -> Option<u64> {
    let call = last_call(usage);
    let field = |name: &str| call.get(name).and_then(Value::as_u64).unwrap_or(0);
    let total = field("input_tokens")
        + field("cache_read_input_tokens")
        + field("cache_creation_input_tokens")
        + field("output_tokens");
    (total > 0).then_some(total)
}

/// The cached/fresh split of the same call `claude_context_tokens` measures.
///
/// Read separately rather than folded into the total because the two answer
/// different questions: the total is how full the window is, this is how much of
/// it had to be paid for at full price. A resumed session re-reads its whole
/// transcript as `cache_creation`, so this is what makes an avoidable relaunch
/// visible instead of merely suspected.
pub(super) fn claude_cache_usage(usage: &Value) -> Option<CacheUsage> {
    let call = last_call(usage);
    let field = |name: &str| call.get(name).and_then(Value::as_u64).unwrap_or(0);
    let cache = CacheUsage {
        read: field("cache_read_input_tokens"),
        created: field("cache_creation_input_tokens"),
    };
    (cache.read + cache.created > 0).then_some(cache)
}

/// Multi-call messages carry per-call `iterations`; the last one is the live
/// state, and summed outer fields would double-count cache reads.
fn last_call(usage: &Value) -> &Value {
    usage
        .get("iterations")
        .and_then(Value::as_array)
        .and_then(|iterations| iterations.last())
        .unwrap_or(usage)
}

#[derive(Clone)]
pub(super) enum HeadlessComputerUseConfig {
    OpenCode {
        base: computer_use_runtime::ComputerUseConfig,
        config_content: String,
    },
    Grok {
        base: computer_use_runtime::ComputerUseConfig,
        grok_home: PathBuf,
        auth_path: Option<PathBuf>,
        rules: String,
    },
}

pub(super) struct HeadlessComputerUseRuntime {
    runtime: computer_use_runtime::ComputerUseRuntime,
    pub(super) config: HeadlessComputerUseConfig,
}

impl HeadlessComputerUseRuntime {
    pub(super) fn start(provider: ProviderKind, events: DriverEventSender) -> anyhow::Result<Self> {
        let runtime = computer_use_runtime::ComputerUseRuntime::start(events)?;
        let config = match provider {
            ProviderKind::OpenCode => {
                let existing = match std::env::var("OPENCODE_CONFIG_CONTENT") {
                    Ok(content) => Some(content),
                    Err(std::env::VarError::NotPresent) => None,
                    Err(std::env::VarError::NotUnicode(_)) => {
                        return Err(anyhow!("OPENCODE_CONFIG_CONTENT is not valid UTF-8"));
                    }
                };
                let base = runtime.config.clone();
                let config_content = build_opencode_computer_use_config(
                    existing.as_deref(),
                    &base.server_path,
                    &base.repl_path,
                    &base.skill_path,
                    &base.process_directory,
                )?;
                HeadlessComputerUseConfig::OpenCode {
                    base,
                    config_content,
                }
            }
            ProviderKind::Grok => build_grok_computer_use_config(runtime.config.clone())?,
            _ => return Err(anyhow!("Computer Use is not supported by this driver")),
        };
        Ok(Self { runtime, config })
    }

    pub(super) fn stop(&self) {
        self.runtime.stop();
    }

    pub(super) fn grok_home(&self) -> Option<&Path> {
        match &self.config {
            HeadlessComputerUseConfig::Grok { grok_home, .. } => Some(grok_home),
            HeadlessComputerUseConfig::OpenCode { .. } => None,
        }
    }
}

fn build_opencode_computer_use_config(
    existing: Option<&str>,
    server_path: &Path,
    repl_path: &Path,
    skill_path: &Path,
    process_directory: &Path,
) -> anyhow::Result<String> {
    let mut config = existing
        .map(serde_json::from_str::<Value>)
        .transpose()
        .context("OPENCODE_CONFIG_CONTENT is invalid JSON")?
        .unwrap_or_else(|| serde_json::json!({}));
    let root = config
        .as_object_mut()
        .ok_or_else(|| anyhow!("OPENCODE_CONFIG_CONTENT must contain a JSON object"))?;
    let mcp = root
        .entry("mcp")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow!("OPENCODE_CONFIG_CONTENT.mcp must be a JSON object"))?;
    mcp.insert(
        "waku_js_repl".into(),
        serde_json::json!({
            "type": "local",
            "command": [repl_path.display().to_string()],
            "enabled": true,
            "environment": {
                "ANASTASIA_COMPUTER_USE_SERVER": server_path.display().to_string(),
                "ANASTASIA_COMPUTER_USE_PROCESS_DIRECTORY": process_directory.display().to_string(),
            },
        }),
    );
    let instructions = root
        .entry("instructions")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or_else(|| anyhow!("OPENCODE_CONFIG_CONTENT.instructions must be a JSON array"))?;
    let skill_path = skill_path.display().to_string();
    if !instructions
        .iter()
        .any(|instruction| instruction.as_str() == Some(&skill_path))
    {
        instructions.push(Value::String(skill_path));
    }
    serde_json::to_string(&config).context("could not encode OpenCode Computer Use configuration")
}

/// Adds Anastasia's composed instructions to an OpenCode server's configuration.
///
/// OpenCode's `instructions` array takes file paths, so the composed text is
/// written out and referenced — the same shape Computer Use already uses, and it
/// merges with a Computer Use entry rather than replacing it.
pub(super) fn opencode_harness_environment(
    launch: &crate::harness::HarnessLaunch,
    environment: &[(String, String)],
) -> anyhow::Result<Vec<(String, String)>> {
    let text = launch
        .instructions
        .as_deref()
        .ok_or_else(|| anyhow!("this harness launch carries no instruction text"))?;
    let path = crate::harness::write_instructions_file(text)?;
    let existing = environment
        .iter()
        .find(|(key, _)| key == "OPENCODE_CONFIG_CONTENT")
        .map(|(_, value)| value.as_str())
        .filter(|value| !value.is_empty());
    let content = add_opencode_instruction(existing, &path)?;
    let mut merged = environment
        .iter()
        .filter(|(key, _)| key != "OPENCODE_CONFIG_CONTENT")
        .cloned()
        .collect::<Vec<_>>();
    merged.push(("OPENCODE_CONFIG_CONTENT".to_owned(), content));
    Ok(merged)
}

pub(super) fn opencode_alabasta_environment(
    environment: &[(String, String)],
) -> anyhow::Result<Vec<(String, String)>> {
    let bridge_path = crate::computer_use::alabasta_bridge_path()?;
    let existing = environment
        .iter()
        .find(|(key, _)| key == "OPENCODE_CONFIG_CONTENT")
        .map(|(_, value)| value.as_str())
        .filter(|value| !value.is_empty());
    let mut config = existing
        .map(serde_json::from_str::<Value>)
        .transpose()
        .context("OPENCODE_CONFIG_CONTENT is invalid JSON")?
        .unwrap_or_else(|| serde_json::json!({}));
    let root = config
        .as_object_mut()
        .ok_or_else(|| anyhow!("OPENCODE_CONFIG_CONTENT must contain a JSON object"))?;
    let mcp = root
        .entry("mcp")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow!("OPENCODE_CONFIG_CONTENT.mcp must be a JSON object"))?;
    let mut env = serde_json::Map::new();
    if let Ok(address) = std::env::var(anastasia_protocol::DAEMON_ADDRESS_ENV) {
        env.insert("ANASTASIA_DAEMON_ADDRESS".into(), Value::String(address));
    }
    if let Ok(token) = std::env::var(anastasia_protocol::DAEMON_TOKEN_ENV) {
        env.insert("ANASTASIA_DAEMON_TOKEN".into(), Value::String(token));
    }
    mcp.insert(
        "alabasta".into(),
        serde_json::json!({
            "type": "local",
            "command": [bridge_path.display().to_string()],
            "enabled": true,
            "environment": env,
        }),
    );
    let content = serde_json::to_string(&config)
        .context("could not encode OpenCode Alabasta configuration")?;
    let mut merged = environment
        .iter()
        .filter(|(key, _)| key != "OPENCODE_CONFIG_CONTENT")
        .cloned()
        .collect::<Vec<_>>();
    merged.push(("OPENCODE_CONFIG_CONTENT".to_owned(), content));
    Ok(merged)
}

/// Appends one path to `OPENCODE_CONFIG_CONTENT.instructions`, idempotently.
fn add_opencode_instruction(existing: Option<&str>, path: &Path) -> anyhow::Result<String> {
    let mut config = existing
        .map(serde_json::from_str::<Value>)
        .transpose()
        .context("OPENCODE_CONFIG_CONTENT is invalid JSON")?
        .unwrap_or_else(|| serde_json::json!({}));
    let root = config
        .as_object_mut()
        .ok_or_else(|| anyhow!("OPENCODE_CONFIG_CONTENT must contain a JSON object"))?;
    let instructions = root
        .entry("instructions")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or_else(|| anyhow!("OPENCODE_CONFIG_CONTENT.instructions must be a JSON array"))?;
    let path = path.display().to_string();
    if !instructions
        .iter()
        .any(|instruction| instruction.as_str() == Some(&path))
    {
        instructions.push(Value::String(path));
    }
    serde_json::to_string(&config).context("could not encode the OpenCode Ponytail configuration")
}

/// The environment that hands OpenCode its Computer Use configuration.
pub(super) fn opencode_computer_use_environment(
    config: &HeadlessComputerUseConfig,
) -> Vec<(String, String)> {
    let HeadlessComputerUseConfig::OpenCode {
        base,
        config_content,
    } = config
    else {
        return Vec::new();
    };
    vec![
        ("OPENCODE_CONFIG_CONTENT".to_owned(), config_content.clone()),
        (
            "ANASTASIA_COMPUTER_USE_SERVER".to_owned(),
            base.server_path.display().to_string(),
        ),
        (
            "ANASTASIA_COMPUTER_USE_PROCESS_DIRECTORY".to_owned(),
            base.process_directory.display().to_string(),
        ),
    ]
}

fn build_grok_computer_use_config(
    base: computer_use_runtime::ComputerUseConfig,
) -> anyhow::Result<HeadlessComputerUseConfig> {
    let source_home = match std::env::var_os("GROK_HOME") {
        Some(home) => PathBuf::from(home),
        None => dirs::home_dir()
            .ok_or_else(|| anyhow!("home directory is unavailable"))?
            .join(".grok"),
    };
    let grok_home = base.process_directory.join("grok-home");
    fs::create_dir(&grok_home).with_context(|| {
        format!(
            "could not create isolated Grok home {}",
            grok_home.display()
        )
    })?;
    fs::set_permissions(&grok_home, fs::Permissions::from_mode(0o700)).with_context(|| {
        format!(
            "could not secure isolated Grok home {}",
            grok_home.display()
        )
    })?;
    if source_home.is_dir() {
        for entry in fs::read_dir(&source_home)
            .with_context(|| format!("could not read Grok home {}", source_home.display()))?
        {
            let entry = entry?;
            let name = entry.file_name();
            if matches!(
                name.to_str(),
                Some("config.toml" | "auth.json" | "auth.json.lock")
            ) {
                continue;
            }
            symlink(entry.path(), grok_home.join(name)).with_context(|| {
                format!(
                    "could not mirror Grok runtime resource {}",
                    entry.path().display()
                )
            })?;
        }
    }
    let existing = match fs::read_to_string(source_home.join("config.toml")) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "could not read {}",
                    source_home.join("config.toml").display()
                )
            });
        }
    };
    let config_content = build_grok_computer_use_toml(existing.as_deref(), &base)?;
    fs::write(grok_home.join("config.toml"), config_content).with_context(|| {
        format!(
            "could not write isolated Grok config {}",
            grok_home.join("config.toml").display()
        )
    })?;
    let auth_path = std::env::var_os("GROK_AUTH_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            let path = source_home.join("auth.json");
            path.is_file().then_some(path)
        });
    let rules = fs::read_to_string(&base.skill_path).with_context(|| {
        format!(
            "could not read Anastasia Computer Use skill {}",
            base.skill_path.display()
        )
    })?;
    Ok(HeadlessComputerUseConfig::Grok {
        base,
        grok_home,
        auth_path,
        rules,
    })
}

fn build_grok_computer_use_toml(
    existing: Option<&str>,
    base: &computer_use_runtime::ComputerUseConfig,
) -> anyhow::Result<String> {
    let mut root = match existing.filter(|content| !content.trim().is_empty()) {
        Some(content) => {
            toml::from_str::<toml::Table>(content).context("Grok config.toml is invalid TOML")?
        }
        None => toml::Table::new(),
    };
    let mcp_servers = root
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or_else(|| anyhow!("Grok config.toml mcp_servers must be a table"))?;
    let mut environment = toml::Table::new();
    environment.insert(
        "ANASTASIA_COMPUTER_USE_SERVER".into(),
        toml::Value::String(base.server_path.display().to_string()),
    );
    environment.insert(
        "ANASTASIA_COMPUTER_USE_PROCESS_DIRECTORY".into(),
        toml::Value::String(base.process_directory.display().to_string()),
    );
    let mut server = toml::Table::new();
    server.insert(
        "command".into(),
        toml::Value::String(base.repl_path.display().to_string()),
    );
    server.insert("args".into(), toml::Value::Array(Vec::new()));
    server.insert("env".into(), toml::Value::Table(environment));
    server.insert("enabled".into(), toml::Value::Boolean(true));
    mcp_servers.insert("waku_js_repl".into(), toml::Value::Table(server));
    if let Ok(bridge_path) = crate::computer_use::alabasta_bridge_path() {
        let mut alabasta_env = toml::Table::new();
        if let Ok(address) = std::env::var(anastasia_protocol::DAEMON_ADDRESS_ENV) {
            alabasta_env.insert("ANASTASIA_DAEMON_ADDRESS".into(), toml::Value::String(address));
        }
        if let Ok(token) = std::env::var(anastasia_protocol::DAEMON_TOKEN_ENV) {
            alabasta_env.insert("ANASTASIA_DAEMON_TOKEN".into(), toml::Value::String(token));
        }
        let mut alabasta_server = toml::Table::new();
        alabasta_server.insert(
            "command".into(),
            toml::Value::String(bridge_path.display().to_string()),
        );
        alabasta_server.insert("args".into(), toml::Value::Array(Vec::new()));
        alabasta_server.insert("env".into(), toml::Value::Table(alabasta_env));
        alabasta_server.insert("enabled".into(), toml::Value::Boolean(true));
        mcp_servers.insert("alabasta".into(), toml::Value::Table(alabasta_server));
    }
    toml::to_string(&root).context("could not encode Grok Computer Use configuration")
}

/// Process arguments and environment shared by every Grok transport.
///
/// ACP now launches through the official SDK rather than a `Command`, so its
/// process configuration must be representable independently of either
/// process API. Keeping one source of truth also prevents Computer Use from
/// behaving differently between the headless and ACP drivers.
pub(super) fn grok_computer_use_launch_configuration(
    config: Option<&HeadlessComputerUseConfig>,
) -> (Vec<String>, Vec<(String, String)>) {
    if let Some(HeadlessComputerUseConfig::Grok {
        base,
        grok_home,
        auth_path,
        rules,
    }) = config
    {
        let args = vec![format!("--rules={rules}")];
        let mut environment = vec![
            ("GROK_HOME".to_owned(), grok_home.display().to_string()),
            (
                "ANASTASIA_COMPUTER_USE_SERVER".to_owned(),
                base.server_path.display().to_string(),
            ),
            (
                "ANASTASIA_COMPUTER_USE_PROCESS_DIRECTORY".to_owned(),
                base.process_directory.display().to_string(),
            ),
        ];
        if let Some(auth_path) = auth_path {
            environment.push(("GROK_AUTH_PATH".to_owned(), auth_path.display().to_string()));
        }
        (args, environment)
    } else {
        (Vec::new(), Vec::new())
    }
}

pub(super) fn provider_stderr_error(lines: Vec<String>) -> Option<String> {
    let first_error = lines
        .iter()
        .find(|line| line.to_ascii_lowercase().contains("error"))?
        .trim();

    // CLI parsers can echo a rejected multi-line argument in full. The first
    // diagnostic already identifies the failure; forwarding the rest would
    // turn provider stderr into an enormous assistant message.
    if first_error.to_ascii_lowercase().starts_with("error:") {
        return Some(truncate_error(first_error, 400));
    }

    let mut message = String::new();
    let first_error_index = lines
        .iter()
        .position(|line| line.trim() == first_error)
        .unwrap_or_default();
    for line in lines.iter().skip(first_error_index).take(6) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !message.is_empty() {
            message.push('\n');
        }
        message.push_str(line);
        if message.chars().count() >= 800 {
            break;
        }
    }
    Some(truncate_error(&message, 800))
}

fn truncate_error(message: &str, max_chars: usize) -> String {
    if message.chars().count() <= max_chars {
        return message.to_owned();
    }
    let mut truncated = message.chars().take(max_chars).collect::<String>();
    truncated.push('…');
    truncated
}

pub(super) fn classify_tool(name: &str) -> ActivityKind {
    ActivityKind::from_tool_name(name)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// The split has to come from the same call the total does, or a multi-call
    /// message reports one turn's occupancy against another turn's cache state.
    #[test]
    fn cache_usage_reads_the_last_call_and_ignores_uncached_ones() {
        let warm = serde_json::json!({
            "input_tokens": 12,
            "cache_read_input_tokens": 9_000,
            "cache_creation_input_tokens": 1_000,
            "output_tokens": 40
        });
        let cache = claude_cache_usage(&warm).unwrap();
        assert_eq!((cache.read, cache.created), (9_000, 1_000));
        assert_eq!(cache.hit_ratio(), Some(0.9));

        // A resumed session writes its whole transcript fresh: 0% reused.
        let cold = serde_json::json!({
            "input_tokens": 12,
            "cache_creation_input_tokens": 80_000,
            "output_tokens": 40
        });
        assert_eq!(claude_cache_usage(&cold).unwrap().hit_ratio(), Some(0.0));

        // `iterations` wins over the summed outer fields.
        let multi = serde_json::json!({
            "cache_read_input_tokens": 1,
            "cache_creation_input_tokens": 99,
            "iterations": [
                {"cache_read_input_tokens": 5, "cache_creation_input_tokens": 5},
                {"cache_read_input_tokens": 30, "cache_creation_input_tokens": 10}
            ]
        });
        let cache = claude_cache_usage(&multi).unwrap();
        assert_eq!((cache.read, cache.created), (30, 10));

        // Nothing cacheable reported at all is absent, not zero — a provider
        // that never reports the split must not render as a 0% cache miss.
        let none = serde_json::json!({"input_tokens": 12, "output_tokens": 40});
        assert!(claude_cache_usage(&none).is_none());
    }

    fn computer_use_config() -> computer_use_runtime::ComputerUseConfig {
        computer_use_runtime::ComputerUseConfig {
            server_path: PathBuf::from("/tmp/Anastasia Computer Use"),
            repl_path: PathBuf::from("/Applications/Anastasia.app/Contents/Resources/waku_js_repl"),
            skill_path: PathBuf::from(
                "/Applications/Anastasia.app/Contents/Resources/skills/anastasia-computer-use/SKILL.md",
            ),
            process_directory: PathBuf::from("/tmp/anastasia-computer-use/session"),
        }
    }

    #[test]
    fn todo_tools_are_plans_not_file_writes() {
        assert_eq!(classify_tool("TodoWrite"), ActivityKind::Plan);
        assert_eq!(classify_tool("todo_write"), ActivityKind::Plan);
        assert_eq!(classify_tool("apply_patch"), ActivityKind::FileChange);
        assert_eq!(classify_tool("read"), ActivityKind::FileRead);
        assert_eq!(classify_tool("ReadFile"), ActivityKind::FileRead);
        assert_eq!(classify_tool("grep"), ActivityKind::FileSearch);
        assert_eq!(classify_tool("glob"), ActivityKind::FileSearch);
        assert_eq!(classify_tool("ls"), ActivityKind::FileList);
        assert_eq!(classify_tool("websearch"), ActivityKind::Search);
        assert_eq!(classify_tool("create_thread"), ActivityKind::Tool);
        assert_eq!(classify_tool("read_mcp_resource"), ActivityKind::Tool);
        assert_eq!(classify_tool("list_threads"), ActivityKind::Tool);
    }

    #[test]
    fn opencode_computer_use_config_preserves_existing_inline_config() {
        let content = build_opencode_computer_use_config(
            Some(
                r#"{
                    "mcp": {
                        "existing": {
                            "type": "local",
                            "command": ["existing-server"],
                            "enabled": true
                        }
                    },
                    "instructions": ["existing.md"],
                    "plugin": ["existing-plugin"]
                }"#,
            ),
            Path::new("/Applications/Anastasia Computer Use"),
            Path::new("/Applications/Anastasia.app/Contents/Resources/waku_js_repl"),
            Path::new(
                "/Applications/Anastasia.app/Contents/Resources/skills/anastasia-computer-use/SKILL.md",
            ),
            Path::new("/tmp/waku computer use/session"),
        )
        .unwrap();
        let value: Value = serde_json::from_str(&content).unwrap();

        assert_eq!(
            value
                .pointer("/mcp/existing/command/0")
                .and_then(Value::as_str),
            Some("existing-server")
        );
        assert_eq!(
            value
                .pointer("/mcp/waku_js_repl/command/0")
                .and_then(Value::as_str),
            Some("/Applications/Anastasia.app/Contents/Resources/waku_js_repl")
        );
        assert_eq!(
            value
                .pointer("/mcp/waku_js_repl/environment/ANASTASIA_COMPUTER_USE_SERVER")
                .and_then(Value::as_str),
            Some("/Applications/Anastasia Computer Use")
        );
        assert_eq!(
            value.get("instructions").and_then(Value::as_array).unwrap(),
            &[
                Value::String("existing.md".into()),
                Value::String(
                    "/Applications/Anastasia.app/Contents/Resources/skills/anastasia-computer-use/SKILL.md"
                        .into(),
                ),
            ]
        );
        assert_eq!(
            value.pointer("/plugin/0").and_then(Value::as_str),
            Some("existing-plugin")
        );
        assert!(value.pointer("/mcp/waku_computer_use").is_none());
    }

    #[test]
    fn grok_computer_use_config_preserves_existing_config_and_replaces_waku_server() {
        let content = build_grok_computer_use_toml(
            Some(
                r#"
                    default_model = "grok-code-fast"

                    [mcp_servers.existing]
                    command = "existing-server"

                    [mcp_servers.waku_js_repl]
                    command = "stale-server"
                "#,
            ),
            &computer_use_config(),
        )
        .unwrap();
        let value: toml::Value = toml::from_str(&content).unwrap();

        assert_eq!(
            value.get("default_model").and_then(toml::Value::as_str),
            Some("grok-code-fast")
        );
        assert_eq!(
            value
                .get("mcp_servers")
                .and_then(|mcp| mcp.get("existing"))
                .and_then(|server| server.get("command"))
                .and_then(toml::Value::as_str),
            Some("existing-server")
        );
        let server = value
            .get("mcp_servers")
            .and_then(|mcp| mcp.get("waku_js_repl"))
            .unwrap();
        assert_eq!(
            server.get("command").and_then(toml::Value::as_str),
            Some("/Applications/Anastasia.app/Contents/Resources/waku_js_repl")
        );
        assert_eq!(
            server
                .get("env")
                .and_then(|env| env.get("ANASTASIA_COMPUTER_USE_SERVER"))
                .and_then(toml::Value::as_str),
            Some("/tmp/Anastasia Computer Use")
        );
    }

    #[test]
    fn grok_computer_use_command_is_process_scoped_and_loads_rules() {
        let config = HeadlessComputerUseConfig::Grok {
            base: computer_use_config(),
            grok_home: PathBuf::from("/tmp/anastasia-computer-use/session/grok-home"),
            auth_path: Some(PathBuf::from("/Users/test/.grok/auth.json")),
            rules: "Anastasia Computer Use rules".into(),
        };
        let (arguments, environment) = grok_computer_use_launch_configuration(Some(&config));
        assert_eq!(arguments, ["--rules=Anastasia Computer Use rules"]);
        let environment = environment.into_iter().collect::<HashMap<_, _>>();
        assert_eq!(
            environment.get("GROK_HOME"),
            Some(&"/tmp/anastasia-computer-use/session/grok-home".into())
        );
        assert_eq!(
            environment.get("GROK_AUTH_PATH"),
            Some(&"/Users/test/.grok/auth.json".into())
        );
    }

    #[test]
    fn provider_stderr_keeps_cli_argument_errors_compact() {
        let message = provider_stderr_error(vec![
            "error: unexpected argument '---".into(),
            "name: anastasia-computer-use".into(),
            "description: a very long bundled skill".into(),
            "---' found".into(),
            "tip: to pass it as a value, use '-- ---'".into(),
        ]);

        assert_eq!(message.as_deref(), Some("error: unexpected argument '---"));
    }

    #[test]
    fn provider_stderr_ignores_non_error_diagnostics() {
        assert_eq!(
            provider_stderr_error(vec!["warning: optional integration unavailable".into()]),
            None
        );
    }

    #[test]
    fn harness_instructions_join_opencode_without_evicting_computer_use() {
        let existing = serde_json::json!({
            "mcp": {"waku_js_repl": {"enabled": true}},
            "instructions": ["/skills/anastasia-computer-use/SKILL.md"],
        })
        .to_string();
        let harness = Path::new("/tmp/anastasia-harness/abc123/CONTEXT.md");

        let merged = add_opencode_instruction(Some(&existing), harness).unwrap();
        let value: Value = serde_json::from_str(&merged).unwrap();
        let instructions = value["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 2);
        assert_eq!(
            instructions[0].as_str(),
            Some("/skills/anastasia-computer-use/SKILL.md")
        );
        assert_eq!(instructions[1].as_str(), Some(harness.to_str().unwrap()));
        // The rest of the document is carried through, not rebuilt.
        assert_eq!(
            value["mcp"]["waku_js_repl"]["enabled"].as_bool(),
            Some(true)
        );

        // Applying it twice must not stack duplicate entries.
        let twice = add_opencode_instruction(Some(&merged), harness).unwrap();
        let value: Value = serde_json::from_str(&twice).unwrap();
        assert_eq!(value["instructions"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn an_absent_opencode_configuration_becomes_a_valid_one() {
        let merged = add_opencode_instruction(None, Path::new("/tmp/harness.md")).unwrap();
        let value: Value = serde_json::from_str(&merged).unwrap();
        assert_eq!(value["instructions"][0].as_str(), Some("/tmp/harness.md"));
    }

    #[test]
    fn opencode_alabasta_environment_configures_bridge_without_secrets() {
        let base_env = vec![("OPENCODE_CONFIG_CONTENT".to_string(), "{}".to_string())];
        if let Ok(env) = opencode_alabasta_environment(&base_env) {
            let (_, content) = env
                .iter()
                .find(|(k, _)| k == "OPENCODE_CONFIG_CONTENT")
                .unwrap();
            let value: Value = serde_json::from_str(content).unwrap();
            assert!(value["mcp"]["alabasta"]["enabled"].as_bool().unwrap());
            assert!(!content.contains("alab_sk_"));
        }
    }
}
