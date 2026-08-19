//! Composing everything Anastasia wants to tell an agent before it starts.
//!
//! Anastasia has more than one thing to say at launch: a code-philosophy policy
//! (Ponytail) and, when a session is bound to a workspace, its compiled Alabasta
//! context. Each provider exposes exactly **one** instruction channel —
//! `--append-system-prompt`, `-c developer_instructions=`, `--rules=`, an
//! `instructions[]` entry — and passing two values for it does not merge them,
//! it silently keeps the last. That is why contributors here supply *text* and
//! never the channel argument itself: [`compose`] owns the channel and emits it
//! once.
//!
//! Ordering is authority ordering, decided by the caller: Alabasta's standing
//! rules bind harder than a code-style preference, so they come first and
//! Ponytail follows. A contributor that resolved to nothing contributes nothing,
//! and with every contributor empty the launch is byte-identical to one built
//! before any of this existed.

use std::path::{Path, PathBuf};

use anyhow::Context;

use anastasia_protocol::model::{InteractionMode, RuntimeMode, SessionGoal};
use crate::model::ProviderKind;

/// Plan Mode instructions injected into the provider's instruction channel.
pub const PLAN_MODE_INSTRUCTIONS: &str = "\
# Plan Mode Instructions
You are operating in Plan Mode.
1. Review and inspect the codebase to thoroughly analyze the user's request (read-only).
2. Do NOT write, edit, create, or delete any files.
3. Do NOT execute mutating or destructive commands.
4. Formulate a comprehensive implementation plan detailing:
   - Summary of the problem and proposed approach
   - Specific files to create, modify, or delete
   - Step-by-step implementation changes
   - Verification and testing plan
5. Ask any necessary clarifying questions if requirements are ambiguous.
6. Present the complete plan to the user and request their confirmation to proceed with implementation in Build mode.";

/// Generates a harness contribution when operating in Plan mode.
pub fn plan_contribution(
    interaction_mode: InteractionMode,
    mode: RuntimeMode,
) -> HarnessContribution {
    if interaction_mode == InteractionMode::Plan || mode == RuntimeMode::Plan {
        HarnessContribution {
            instructions: Some(PLAN_MODE_INSTRUCTIONS.to_owned()),
            ..HarnessContribution::default()
        }
    } else {
        HarnessContribution::default()
    }
}

/// Generates a harness contribution when an active session goal exists.
pub fn goal_contribution(goal: Option<&SessionGoal>) -> HarnessContribution {
    if let Some(goal) = goal {
        if !goal.paused && !goal.text.trim().is_empty() {
            return HarnessContribution {
                instructions: Some(format!(
                    "# Persistent Session Goal\n\
                     Objective: {}\n\
                     Systematically pursue and fulfill this primary objective across turns until completed.",
                    goal.text.trim()
                )),
                ..HarnessContribution::default()
            };
        }
    }
    HarnessContribution::default()
}

/// Separator between contributions in a composed instruction channel. A visible
/// rule keeps two rulesets from reading as one run-on document to the model.
const SEPARATOR: &str = "\n\n---\n\n";

/// One subsystem's request to shape an agent launch.
///
/// `args` and `env` are applied verbatim and are for channels that genuinely
/// accept repetition (`--plugin-dir`, `--extension`, MCP server registration).
/// Anything destined for the provider's shared instruction channel goes in
/// `instructions` so it can be merged rather than overwritten.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HarnessContribution {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub instructions: Option<String>,
}

impl HarnessContribution {
    pub fn is_empty(&self) -> bool {
        self.args.is_empty() && self.env.is_empty() && self.instructions.is_none()
    }
}

/// The composed result a driver applies to its child process.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HarnessLaunch {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    /// Composed instruction text for providers whose channel is a config
    /// document rather than a command-line argument (OpenCode). Already emitted
    /// as an argument for every other provider.
    pub instructions: Option<String>,
}

impl HarnessLaunch {
    /// Nothing to say. The process launches exactly as it would without any of
    /// this machinery.
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty() && self.env.is_empty() && self.instructions.is_none()
    }
}

/// Merges contributions into one launch, emitting the provider's instruction
/// channel exactly once.
///
/// Never fails: a launch is not the place to discover that a filesystem write
/// did not work. A channel that needs a file it could not write simply carries
/// no instructions, and the subsystem's own status reports why.
pub fn compose(provider: ProviderKind, contributions: Vec<HarnessContribution>) -> HarnessLaunch {
    let mut launch = HarnessLaunch::default();
    let mut texts = Vec::new();
    for contribution in contributions {
        launch.args.extend(contribution.args);
        launch.env.extend(contribution.env);
        if let Some(text) = contribution.instructions {
            if !text.trim().is_empty() {
                texts.push(text);
            }
        }
    }
    if texts.is_empty() {
        return launch;
    }
    let text = texts.join(SEPARATOR);

    match provider {
        ProviderKind::Claude => {
            launch.args.push("--append-system-prompt".to_owned());
            launch.args.push(text);
        }
        ProviderKind::Codex => {
            launch.args.push("-c".to_owned());
            launch.args.push(codex_developer_instructions(
                &text,
                existing_codex_developer_instructions(),
            ));
        }
        ProviderKind::Grok => {
            // One argv entry, so the text travels verbatim however it is punctuated.
            launch.args.push(format!("--rules={text}"));
        }
        ProviderKind::Pi => {
            // Pi takes instruction material as a skill file path.
            if let Ok(path) = write_instructions_file(&text) {
                launch.args.push("--skill".to_owned());
                launch.args.push(path.display().to_string());
            }
        }
        // OpenCode merges a file path into its own config document; the driver
        // owns that merge because it also has to preserve the user's config.
        ProviderKind::OpenCode => launch.instructions = Some(text),
        // No instruction channel exists on these launch paths.
        ProviderKind::Amp | ProviderKind::Cursor | ProviderKind::DeepSeek => {}
    }
    launch
}

/// Builds Codex's `-c developer_instructions=<value>` argument.
///
/// The value is parsed as TOML, so it is *encoded* as TOML rather than pasted
/// in — quotes, newlines and non-ASCII in a ruleset would otherwise produce a
/// malformed override or inject unintended keys.
fn codex_developer_instructions(text: &str, existing: Option<String>) -> String {
    let value = match existing {
        Some(existing) if !existing.trim().is_empty() => format!("{existing}{SEPARATOR}{text}"),
        _ => text.to_owned(),
    };
    format!("developer_instructions={}", toml::Value::String(value))
}

/// `-c key=value` replaces rather than merges, so a `developer_instructions` the
/// user wrote in their own config would silently vanish. Read it and keep it
/// ahead of everything Anastasia adds.
fn existing_codex_developer_instructions() -> Option<String> {
    let path = dirs::home_dir()?.join(".codex").join("config.toml");
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    Some(config.get("developer_instructions")?.as_str()?.to_owned())
}

/// Materializes instruction text for channels that take a path rather than a
/// string.
///
/// Named by a hash of its own content, so identical text from concurrent
/// sessions is one file with identical bytes and differing text never collides.
pub fn write_instructions_file(text: &str) -> anyhow::Result<PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let directory = std::env::temp_dir()
        .join("anastasia-harness")
        .join(format!("{:016x}", hasher.finish()));
    std::fs::create_dir_all(&directory)
        .with_context(|| format!("could not create {}", directory.display()))?;
    let path = directory.join("CONTEXT.md");
    write_if_changed(&path, text)?;
    Ok(path)
}

/// Rewriting an identical file would churn its mtime for every session sharing
/// it, and a concurrent reader could observe a truncated file mid-write.
fn write_if_changed(path: &Path, text: &str) -> anyhow::Result<()> {
    if std::fs::read_to_string(path).is_ok_and(|existing| existing == text) {
        return Ok(());
    }
    std::fs::write(path, text).with_context(|| format!("could not write {}", path.display()))
}

/// Applies a composed launch to a child process.
///
/// Args and environment are applied as-is — never through a shell — so a path
/// containing spaces, quotes or unicode stays one argv entry.
pub fn apply(command: &mut std::process::Command, launch: &HarnessLaunch) {
    command.args(&launch.args);
    for (key, value) in &launch.env {
        command.env(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(instructions: &str) -> HarnessContribution {
        HarnessContribution {
            instructions: Some(instructions.to_owned()),
            ..HarnessContribution::default()
        }
    }

    fn codex_value(launch: &HarnessLaunch) -> String {
        let index = launch
            .args
            .iter()
            .position(|arg| arg.starts_with("developer_instructions="))
            .expect("codex receives a developer_instructions override");
        let pair = launch.args[index]
            .strip_prefix("developer_instructions=")
            .unwrap();
        let parsed: toml::Value = toml::from_str(&format!("value = {pair}")).unwrap();
        parsed["value"].as_str().unwrap().to_owned()
    }

    #[test]
    fn two_contributors_share_one_codex_channel() {
        // The regression this whole module exists for: a second
        // `-c developer_instructions=` does not merge, it wins outright.
        let launch = compose(
            ProviderKind::Codex,
            vec![text("ALABASTA CONTEXT"), text("PONYTAIL RULES")],
        );
        assert_eq!(
            launch
                .args
                .iter()
                .filter(|arg| arg.starts_with("developer_instructions="))
                .count(),
            1,
            "more than one developer_instructions override"
        );
        let value = codex_value(&launch);
        assert!(value.contains("ALABASTA CONTEXT"));
        assert!(value.contains("PONYTAIL RULES"));
        // Authority order is preserved: the caller put Alabasta first.
        assert!(value.find("ALABASTA").unwrap() < value.find("PONYTAIL").unwrap());
    }

    #[test]
    fn two_contributors_share_one_claude_channel() {
        let launch = compose(
            ProviderKind::Claude,
            vec![text("ALABASTA CONTEXT"), text("PONYTAIL RULES")],
        );
        assert_eq!(
            launch
                .args
                .iter()
                .filter(|arg| arg.as_str() == "--append-system-prompt")
                .count(),
            1
        );
        let value = launch.args.last().unwrap();
        assert!(value.contains("ALABASTA CONTEXT") && value.contains("PONYTAIL RULES"));
    }

    #[test]
    fn grok_receives_one_rules_argument() {
        let launch = compose(ProviderKind::Grok, vec![text("FIRST"), text("SECOND")]);
        let rules = launch
            .args
            .iter()
            .filter(|arg| arg.starts_with("--rules="))
            .collect::<Vec<_>>();
        assert_eq!(rules.len(), 1);
        assert!(rules[0].contains("FIRST") && rules[0].contains("SECOND"));
    }

    #[test]
    fn nothing_to_say_leaves_the_command_untouched() {
        for provider in ProviderKind::ALL {
            // Both the no-contributor and the all-empty-contributor cases must
            // produce a launch indistinguishable from the pre-harness one.
            assert_eq!(compose(provider, Vec::new()), HarnessLaunch::disabled());
            assert_eq!(
                compose(
                    provider,
                    vec![HarnessContribution::default(), text("   \n  ")]
                ),
                HarnessLaunch::disabled(),
                "{provider:?} acted on whitespace-only instructions"
            );
        }
    }

    #[test]
    fn native_args_and_environment_survive_composition() {
        let native = HarnessContribution {
            args: vec!["--plugin-dir".to_owned(), "/a b/c".to_owned()],
            env: vec![("PONYTAIL_DEFAULT_MODE".to_owned(), "full".to_owned())],
            instructions: Some("LINE".to_owned()),
        };
        let launch = compose(ProviderKind::Claude, vec![native]);
        assert!(
            launch
                .args
                .windows(2)
                .any(|pair| pair[0] == "--plugin-dir" && pair[1] == "/a b/c")
        );
        assert_eq!(
            launch.env,
            vec![("PONYTAIL_DEFAULT_MODE".to_owned(), "full".to_owned())]
        );
    }

    #[test]
    fn a_user_written_codex_instruction_is_kept_ahead_of_ours() {
        let argument = codex_developer_instructions("OURS", Some("Always use tabs.".into()));
        let pair = argument.strip_prefix("developer_instructions=").unwrap();
        let parsed: toml::Value = toml::from_str(&format!("value = {pair}")).unwrap();
        let value = parsed["value"].as_str().unwrap();
        assert!(value.starts_with("Always use tabs."));
        assert!(value.contains("OURS"));

        // A blank existing value contributes nothing but noise.
        let argument = codex_developer_instructions("OURS", Some("  ".into()));
        let pair = argument.strip_prefix("developer_instructions=").unwrap();
        let parsed: toml::Value = toml::from_str(&format!("value = {pair}")).unwrap();
        assert_eq!(parsed["value"].as_str(), Some("OURS"));
    }

    #[test]
    fn codex_encoding_survives_quotes_newlines_and_unicode() {
        let hostile = "quote \" backslash \\ newline\nunicode — ✓";
        let argument = codex_developer_instructions(hostile, None);
        let pair = argument.strip_prefix("developer_instructions=").unwrap();
        let parsed: toml::Value = toml::from_str(&format!("value = {pair}")).unwrap();
        assert_eq!(parsed["value"].as_str(), Some(hostile));
    }

    #[test]
    fn applying_a_launch_never_goes_through_a_shell() {
        let hostile = "/Users/a b/\"quoted\"/$HOME/`whoami`/naïve — ✓";
        let launch = HarnessLaunch {
            args: vec!["--plugin-dir".to_owned(), hostile.to_owned()],
            env: vec![("PLUGIN_DATA".to_owned(), hostile.to_owned())],
            instructions: None,
        };
        let mut command = std::process::Command::new("/usr/bin/true");
        apply(&mut command, &launch);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        // Two entries, not five: no word splitting, no expansion, no quoting.
        assert_eq!(args, vec!["--plugin-dir".to_owned(), hostile.to_owned()]);
        let env = command
            .get_envs()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().into_owned(),
                    v.map(|v| v.to_string_lossy().into_owned()),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            env,
            vec![("PLUGIN_DATA".to_owned(), Some(hostile.to_owned()))]
        );
    }

    #[test]
    fn plan_mode_generates_instructions_and_composes_cleanly() {
        let plan = plan_contribution(InteractionMode::Plan, RuntimeMode::FullAccess);
        assert!(plan.instructions.is_some());
        assert!(plan.instructions.unwrap().contains("Plan Mode Instructions"));

        let build = plan_contribution(InteractionMode::Build, RuntimeMode::FullAccess);
        assert_eq!(build, HarnessContribution::default());

        let claude_launch = compose(
            ProviderKind::Claude,
            vec![plan_contribution(InteractionMode::Plan, RuntimeMode::Ask)],
        );
        assert_eq!(claude_launch.args[0], "--append-system-prompt");
        assert!(claude_launch.args[1].contains("Plan Mode Instructions"));

        let codex_launch = compose(
            ProviderKind::Codex,
            vec![plan_contribution(InteractionMode::Plan, RuntimeMode::FullAccess)],
        );
        assert_eq!(codex_launch.args[0], "-c");
        assert!(codex_launch.args[1].contains("Plan Mode Instructions"));
    }

    #[test]
    fn instruction_files_are_content_addressed_and_stable() {
        let first = write_instructions_file("alpha").unwrap();
        let again = write_instructions_file("alpha").unwrap();
        let other = write_instructions_file("beta").unwrap();
        assert_eq!(first, again, "identical text must reuse one file");
        assert_ne!(first, other, "different text must not collide");
        assert_eq!(std::fs::read_to_string(&first).unwrap(), "alpha");
    }
}
