//! Applying the Ponytail harness policy to a session's agent process.
//!
//! Ponytail (<https://github.com/DietrichGebert/ponytail>, MIT) is vendored at a
//! pinned release under `resources/ponytail` and shipped inside the app bundle,
//! so a session never downloads or executes moving upstream code, and works
//! offline. See `resources/ponytail/README.anastasia.md`.
//!
//! Upstream's entire runtime effect is one instruction blob built from
//! `skills/ponytail/SKILL.md` and filtered to the active intensity. Anastasia
//! delivers it through the strongest channel each runtime exposes:
//!
//! | Runtime  | Channel                                    | Integration    |
//! |----------|--------------------------------------------|----------------|
//! | Claude   | `--plugin-dir` (session-scoped plugin)      | native         |
//! | Pi       | `--extension` (upstream's own Pi extension) | native         |
//! | Codex    | `-c developer_instructions=…`               | instructions   |
//! | OpenCode | `instructions[]` in `OPENCODE_CONFIG_CONTENT` | instructions |
//! | Grok     | `--rules=…`                                 | instructions   |
//!
//! Nothing here writes to `~/.claude`, `~/.codex`, `~/.config/opencode`, or the
//! user's repository. Codex and Claude both offer a native plugin system, but
//! Codex's is a global install plus a hook-trust prompt, so it stays an explicit
//! user action rather than something opening Anastasia does silently.
//!
//! Every failure is non-fatal: a resolver error yields an unhealthy
//! [`PonytailStatus`] and an empty launch, and the agent starts exactly as it
//! would with Ponytail off. Ponytail must never be the reason a session cannot run.

use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};

use anastasia_protocol::model::ProviderKind;
pub use anastasia_protocol::ponytail::{
    PonytailIntegration, PonytailMode, PonytailStatus, vendored_version,
};

/// Where upstream's ruleset lives inside the vendored copy.
const SKILL_RELATIVE_PATH: &str = "skills/ponytail/SKILL.md";
/// Upstream's Pi extension entry point.
const PI_EXTENSION_RELATIVE_PATH: &str = "pi-extension/index.js";

/// A resolved decision about how one session's process gets Ponytail.
///
/// `args` and `env` are applied verbatim by the driver, so every path travels as
/// its own argv entry and never through a shell — spaces, quotes and unicode in
/// an install path are safe by construction. `instructions` is for OpenCode,
/// whose channel is a JSON document rather than a command line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PonytailLaunch {
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub instructions: Option<String>,
    pub status: Option<PonytailStatus>,
}

impl PonytailLaunch {
    /// Ponytail is off. The process is launched exactly as it was before this
    /// module existed — no args, no env, no status badge.
    pub fn disabled() -> Self {
        Self::default()
    }

    fn failed(provider: ProviderKind, mode: PonytailMode, error: impl std::fmt::Display) -> Self {
        Self {
            status: Some(PonytailStatus::failed(provider, mode, error.to_string())),
            ..Self::default()
        }
    }
}

/// The vendored Ponytail directory inside the running app bundle.
pub fn root_path() -> anyhow::Result<PathBuf> {
    let executable = host_executable_path()?;
    let path = executable
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow!("Anastasia app bundle is malformed"))?
        .join("Resources")
        .join("ponytail");
    if !path.join(SKILL_RELATIVE_PATH).is_file() {
        return Err(anyhow!("Ponytail is missing from this Anastasia build"));
    }
    Ok(path)
}

fn host_executable_path() -> anyhow::Result<PathBuf> {
    std::env::var_os(anastasia_protocol::APP_EXECUTABLE_ENV)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| {
            std::env::current_exe().context("Anastasia executable path is unavailable")
        })
}

/// Whether a Node.js runtime is on the search path.
///
/// Upstream's lifecycle hooks and Pi extension are Node programs. Without Node
/// they stay quiet rather than erroring, so Anastasia checks first and falls
/// back to instruction injection instead of claiming a native integration that
/// would never fire.
pub fn node_available() -> bool {
    crate::command_env::find_executable("node").is_some()
}

/// The instruction text upstream would emit for `mode`.
pub fn instructions(mode: PonytailMode) -> anyhow::Result<String> {
    let path = root_path()?.join(SKILL_RELATIVE_PATH);
    let body = std::fs::read_to_string(&path)
        .with_context(|| format!("could not read the Ponytail ruleset at {}", path.display()))?;
    Ok(render_instructions(&body, mode))
}

/// Ports upstream's `filterSkillBodyForMode` (`hooks/ponytail-instructions.js`).
///
/// Only the intensity table rows and the worked examples are mode-specific, and
/// both are keyed by a mode name. A line whose label is not a mode — upstream's
/// example is `No unrequested abstractions: …` — is an ordinary rule and must
/// survive verbatim, which is why the label is parsed rather than pattern-matched
/// loosely.
fn render_instructions(body: &str, mode: PonytailMode) -> String {
    let mut rendered = format!("PONYTAIL MODE ACTIVE — level: {}\n\n", mode.id());
    for line in strip_frontmatter(body).lines() {
        match mode_label(line) {
            Some(labelled) if labelled != mode => continue,
            _ => {
                rendered.push_str(line);
                rendered.push('\n');
            }
        }
    }
    rendered
}

/// Drops a leading `---` frontmatter block, matching upstream's
/// `/^---[\s\S]*?---\s*/`.
fn strip_frontmatter(body: &str) -> &str {
    let Some(rest) = body.strip_prefix("---") else {
        return body;
    };
    let Some(end) = rest.find("---") else {
        return body;
    };
    rest[end + 3..].trim_start_matches(['\r', '\n', ' ', '\t'])
}

/// The mode a line is keyed to, for the two shapes upstream filters:
/// an intensity table row `| **full** | … |`, and a worked example `- full: …`.
/// `None` means the line is not mode-specific and is always kept.
fn mode_label(line: &str) -> Option<PonytailMode> {
    if let Some(rest) = line.strip_prefix('|') {
        let cell = rest.split('|').next()?.trim();
        let label = cell.strip_prefix("**")?.strip_suffix("**")?;
        return PonytailMode::from_id(label);
    }
    if let Some(rest) = line.strip_prefix('-') {
        // Upstream's `^-\s*([^:]+):` — the label may not span a colon, so a rule
        // like "Deletion over addition. Boring over clever" never matches.
        return PonytailMode::from_id(rest.split(':').next()?.trim());
    }
    None
}

/// Decides how `provider` receives Ponytail at `mode`, or that it does not.
///
/// Never returns an error: a caller starting an agent must not have to handle
/// one. Problems surface as an unhealthy status alongside an empty launch.
pub fn resolve(provider: ProviderKind, mode: Option<PonytailMode>) -> PonytailLaunch {
    let Some(mode) = mode else {
        return PonytailLaunch::disabled();
    };
    match provider {
        ProviderKind::Claude => resolve_claude(mode),
        ProviderKind::Pi => resolve_pi(mode),
        ProviderKind::Codex => resolve_codex(mode),
        ProviderKind::OpenCode | ProviderKind::Grok => resolve_instructions(provider, mode),
        // No instruction channel exists on these launch paths. Nothing is
        // attempted, so this is reported as unsupported rather than unhealthy.
        ProviderKind::Amp | ProviderKind::Cursor | ProviderKind::DeepSeek => PonytailLaunch {
            status: Some(PonytailStatus::unsupported(provider, mode)),
            ..PonytailLaunch::default()
        },
    }
}

/// Claude Code loads a plugin directory for one session only, which is the
/// strongest integration available anywhere: real `SessionStart`,
/// `SubagentStart` and `UserPromptSubmit` hooks, so subagents inherit the policy
/// and `/ponytail lite|full|ultra` still works inside the session.
fn resolve_claude(mode: PonytailMode) -> PonytailLaunch {
    let root = match root_path() {
        Ok(root) => root,
        Err(error) => return PonytailLaunch::failed(ProviderKind::Claude, mode, error),
    };
    if !node_available() {
        return claude_instruction_fallback(mode);
    }
    let state_directory = state_directory(mode);
    if let Err(error) = std::fs::create_dir_all(&state_directory) {
        return claude_instruction_fallback(mode).or_report(error);
    }
    PonytailLaunch {
        args: vec![
            "--plugin-dir".to_owned(),
            root.display().to_string(),
            "--append-system-prompt".to_owned(),
            // The hooks carry the ruleset on their own; this line only makes the
            // active level legible to `/ponytail` when it reports back.
            format!("Ponytail is active at level {}.", mode.id()),
        ],
        env: vec![
            ("PONYTAIL_DEFAULT_MODE".to_owned(), mode.id().to_owned()),
            // ponytail: upstream's hooks/ponytail-runtime.js treats PLUGIN_DATA as
            // "you are a hosted plugin": it relocates the .ponytail-active flag
            // file there instead of ~/.claude, and hooks/ponytail-activate.js
            // suppresses its "add a statusLine to ~/.claude/settings.json" nudge on
            // the same signal. Both are exactly what Anastasia wants — native hooks
            // with no writes outside our own temp directory and no agent prompting
            // the user to edit their global settings — and it needs no fork. If
            // upstream ever drops the variable, the no-Node fallback below already
            // covers the same ground at the instruction tier.
            (
                "PLUGIN_DATA".to_owned(),
                state_directory.display().to_string(),
            ),
        ],
        instructions: None,
        status: Some(PonytailStatus::active(
            ProviderKind::Claude,
            mode,
            PonytailIntegration::Native,
            Some(vendored_version().to_owned()),
        )),
    }
}

fn claude_instruction_fallback(mode: PonytailMode) -> PonytailLaunch {
    match instructions(mode) {
        Ok(text) => PonytailLaunch {
            args: vec!["--append-system-prompt".to_owned(), text],
            env: Vec::new(),
            instructions: None,
            status: Some(PonytailStatus {
                message: Some(tr!("ponytail.node_missing")),
                ..PonytailStatus::active(
                    ProviderKind::Claude,
                    mode,
                    PonytailIntegration::Instructions,
                    Some(vendored_version().to_owned()),
                )
            }),
        },
        Err(error) => PonytailLaunch::failed(ProviderKind::Claude, mode, error),
    }
}

/// Pi loads upstream's own extension, so Ponytail owns Pi's lifecycle the same
/// way it owns Claude's. Pi itself is a Node program, so there is no separate
/// missing-Node case to handle.
fn resolve_pi(mode: PonytailMode) -> PonytailLaunch {
    let root = match root_path() {
        Ok(root) => root,
        Err(error) => return PonytailLaunch::failed(ProviderKind::Pi, mode, error),
    };
    PonytailLaunch {
        args: vec![
            "--extension".to_owned(),
            root.join(PI_EXTENSION_RELATIVE_PATH).display().to_string(),
        ],
        env: vec![("PONYTAIL_DEFAULT_MODE".to_owned(), mode.id().to_owned())],
        instructions: None,
        status: Some(PonytailStatus::active(
            ProviderKind::Pi,
            mode,
            PonytailIntegration::Native,
            Some(vendored_version().to_owned()),
        )),
    }
}

/// Codex takes the ruleset as developer instructions. Its own plugin system
/// would be stronger, but installing into it means writing `~/.codex/config.toml`
/// and answering a hook-trust prompt, which is an explicit user action, not
/// something a session start may do on its own.
fn resolve_codex(mode: PonytailMode) -> PonytailLaunch {
    let text = match instructions(mode) {
        Ok(text) => text,
        Err(error) => return PonytailLaunch::failed(ProviderKind::Codex, mode, error),
    };
    PonytailLaunch {
        args: vec![
            "-c".to_owned(),
            codex_developer_instructions_override(&text, existing_codex_developer_instructions()),
        ],
        env: Vec::new(),
        instructions: None,
        status: Some(PonytailStatus::active(
            ProviderKind::Codex,
            mode,
            PonytailIntegration::Instructions,
            Some(vendored_version().to_owned()),
        )),
    }
}

/// `-c key=value` replaces rather than merges, so a `developer_instructions`
/// the user wrote in their own config would silently vanish. Read it and keep it
/// ahead of the ruleset.
fn existing_codex_developer_instructions() -> Option<String> {
    let path = dirs::home_dir()?.join(".codex").join("config.toml");
    let config: toml::Value = toml::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    Some(config.get("developer_instructions")?.as_str()?.to_owned())
}

/// Builds the `key=value` argument for Codex's `-c`.
///
/// The value is parsed as TOML, so it is *encoded* as TOML rather than pasted in
/// — quotes, newlines and non-ASCII in the ruleset would otherwise produce a
/// malformed override or, worse, inject unintended keys.
fn codex_developer_instructions_override(text: &str, existing: Option<String>) -> String {
    let value = match existing {
        Some(existing) if !existing.trim().is_empty() => format!("{existing}\n\n{text}"),
        _ => text.to_owned(),
    };
    format!("developer_instructions={}", toml::Value::String(value))
}

/// Runtimes whose channel takes the instruction text directly: OpenCode merges
/// it into its config document, Grok passes it as `--rules`.
fn resolve_instructions(provider: ProviderKind, mode: PonytailMode) -> PonytailLaunch {
    let text = match instructions(mode) {
        Ok(text) => text,
        Err(error) => return PonytailLaunch::failed(provider, mode, error),
    };
    let args = match provider {
        ProviderKind::Grok => vec![format!("--rules={text}")],
        _ => Vec::new(),
    };
    PonytailLaunch {
        args,
        env: Vec::new(),
        instructions: Some(text),
        status: Some(PonytailStatus::active(
            provider,
            mode,
            PonytailIntegration::Instructions,
            Some(vendored_version().to_owned()),
        )),
    }
}

/// Materializes the ruleset as a file for runtimes whose instruction channel
/// takes paths rather than text — currently OpenCode's `instructions` array.
///
/// Written under the mode's own state directory, so two sessions at the same
/// level share identical bytes and sessions at different levels never collide.
pub fn write_instructions_file(launch: &PonytailLaunch) -> anyhow::Result<PathBuf> {
    let text = launch
        .instructions
        .as_deref()
        .ok_or_else(|| anyhow!("this Ponytail launch carries no instruction text"))?;
    let mode = launch
        .status
        .as_ref()
        .map(|status| status.mode)
        .unwrap_or_default();
    let directory = state_directory(mode);
    std::fs::create_dir_all(&directory)
        .with_context(|| format!("could not create {}", directory.display()))?;
    let path = directory.join("ponytail.md");
    std::fs::write(&path, text).with_context(|| format!("could not write {}", path.display()))?;
    Ok(path)
}

/// Where upstream's hooks keep their mode flag, kept out of `~/.claude`.
///
/// Keyed by mode rather than by session: the file holds only the mode name, so
/// concurrent sessions at the same level write identical bytes, and sessions at
/// different levels never share a file.
fn state_directory(mode: PonytailMode) -> PathBuf {
    std::env::temp_dir()
        .join("anastasia-ponytail")
        .join(mode.id())
}

impl PonytailLaunch {
    /// Keeps a working launch but records why it is not the strongest one.
    fn or_report(mut self, error: impl std::fmt::Display) -> Self {
        if let Some(status) = self.status.as_mut() {
            status.message = Some(error.to_string());
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trimmed stand-in for upstream's SKILL.md carrying every shape the
    /// filter has to distinguish.
    const RULESET: &str = "\
---
name: ponytail
description: Lazy senior dev mode.
---

# Ponytail

## Rules

- No unrequested abstractions: no interface with one implementation.
- Deletion over addition. Boring over clever.

## Intensity

| Level | What change |
|-------|------------|
| **lite** | Nudges only. |
| **full** | The ladder enforced. Default. |
| **ultra** | Deletion is the first move. |

Example: \"Add a cache.\"
- lite: mention the stdlib option.
- full: use `@lru_cache`.
- ultra: question whether the cache is needed.
";

    #[test]
    fn instructions_keep_only_the_active_intensity() {
        let full = render_instructions(RULESET, PonytailMode::Full);
        assert!(full.contains("| **full** | The ladder enforced. Default. |"));
        assert!(!full.contains("**lite**"));
        assert!(!full.contains("**ultra**"));
        assert!(full.contains("- full: use `@lru_cache`."));
        assert!(!full.contains("- lite:"));
        assert!(!full.contains("- ultra:"));

        let ultra = render_instructions(RULESET, PonytailMode::Ultra);
        assert!(ultra.contains("| **ultra** | Deletion is the first move. |"));
        assert!(!ultra.contains("**full**"));
        assert!(ultra.contains("- ultra: question whether the cache is needed."));
    }

    #[test]
    fn instructions_keep_rules_whose_label_is_not_a_mode() {
        // The trap upstream calls out: a bullet like this looks like a worked
        // example but is an ordinary rule, and dropping it would quietly weaken
        // the policy at every level.
        for mode in PonytailMode::ALL {
            let rendered = render_instructions(RULESET, mode);
            assert!(
                rendered.contains(
                    "- No unrequested abstractions: no interface with one implementation."
                ),
                "{mode:?} dropped a non-mode rule"
            );
            assert!(rendered.contains("- Deletion over addition. Boring over clever."));
        }
    }

    #[test]
    fn instructions_announce_the_level_and_drop_frontmatter() {
        let rendered = render_instructions(RULESET, PonytailMode::Lite);
        assert!(rendered.starts_with("PONYTAIL MODE ACTIVE — level: lite\n\n"));
        assert!(!rendered.contains("description: Lazy senior dev mode."));
        assert!(rendered.contains("# Ponytail"));
    }

    #[test]
    fn a_ruleset_without_frontmatter_survives_intact() {
        assert_eq!(strip_frontmatter("# Ponytail\n"), "# Ponytail\n");
        // An opening fence with no closing one is not frontmatter either.
        assert_eq!(
            strip_frontmatter("---\nunterminated\n"),
            "---\nunterminated\n"
        );
    }

    #[test]
    fn the_real_vendored_ruleset_filters_by_intensity() {
        // The synthetic fixture above proves the filter; this proves it against
        // the bytes actually shipped, so a vendoring upgrade that reshapes the
        // intensity table fails here rather than silently sending every level
        // the same ruleset.
        const VENDORED: &str = include_str!("../../../resources/ponytail/skills/ponytail/SKILL.md");

        let rendered = PonytailMode::ALL.map(|mode| (mode, render_instructions(VENDORED, mode)));
        for (mode, text) in &rendered {
            assert!(
                text.starts_with(&format!("PONYTAIL MODE ACTIVE — level: {}", mode.id())),
                "{mode:?} is not announced"
            );
            assert!(!text.contains("description:"), "{mode:?} kept frontmatter");
            for other in PonytailMode::ALL.into_iter().filter(|other| other != mode) {
                assert!(
                    !text.contains(&format!("| **{}** |", other.id())),
                    "{mode:?} kept the {other:?} intensity row"
                );
            }
        }
        // Each level must actually differ, and each must keep its own row.
        assert_ne!(rendered[0].1, rendered[1].1);
        assert_ne!(rendered[1].1, rendered[2].1);
        for (mode, text) in &rendered {
            assert!(
                text.contains(&format!("| **{}** |", mode.id())),
                "{mode:?} lost its own intensity row"
            );
        }
    }

    #[test]
    fn disabled_ponytail_contributes_nothing_to_the_launch() {
        for provider in ProviderKind::ALL {
            assert_eq!(resolve(provider, None), PonytailLaunch::disabled());
        }
        // The regression guard that matters: off means an untouched process.
        assert!(PonytailLaunch::disabled().args.is_empty());
        assert!(PonytailLaunch::disabled().env.is_empty());
        assert!(PonytailLaunch::disabled().status.is_none());
    }

    #[test]
    fn the_resolver_agrees_with_the_ladder_settings_displays() {
        // Settings renders PonytailIntegration::for_provider without launching
        // anything, so the two must not drift apart.
        for provider in ProviderKind::ALL {
            let declared = PonytailIntegration::for_provider(provider);
            let resolved = resolve(provider, Some(PonytailMode::Full))
                .status
                .expect("an enabled policy always reports")
                .integration;
            if declared == PonytailIntegration::Unsupported {
                assert_eq!(resolved, declared, "{provider:?}");
            }
        }
    }

    #[test]
    fn providers_without_a_channel_are_unsupported_but_healthy() {
        for provider in [
            ProviderKind::Amp,
            ProviderKind::Cursor,
            ProviderKind::DeepSeek,
        ] {
            let launch = resolve(provider, Some(PonytailMode::Full));
            let status = launch.status.expect("unsupported providers still report");
            assert_eq!(status.integration, PonytailIntegration::Unsupported);
            // Nothing was attempted, so nothing failed.
            assert!(status.healthy);
            assert!(launch.args.is_empty(), "{provider:?} launch was modified");
            assert!(
                launch.env.is_empty(),
                "{provider:?} environment was modified"
            );
        }
    }

    #[test]
    fn codex_override_encodes_the_ruleset_as_toml() {
        let text = "quote \" backslash \\ newline\nunicode — ✓";
        let argument = codex_developer_instructions_override(text, None);
        let key_value = argument
            .strip_prefix("developer_instructions=")
            .expect("argument is a -c key=value pair");
        // Codex parses the value as TOML, so it must round-trip back to the
        // exact ruleset rather than truncating at the first quote.
        let parsed: toml::Value =
            toml::from_str(&format!("value = {key_value}")).expect("value is valid TOML");
        assert_eq!(parsed["value"].as_str(), Some(text));
    }

    #[test]
    fn codex_override_keeps_developer_instructions_the_user_wrote() {
        let argument =
            codex_developer_instructions_override("PONYTAIL", Some("Always use tabs.".into()));
        let key_value = argument.strip_prefix("developer_instructions=").unwrap();
        let parsed: toml::Value = toml::from_str(&format!("value = {key_value}")).unwrap();
        let value = parsed["value"].as_str().unwrap();
        assert!(
            value.starts_with("Always use tabs."),
            "user setting dropped"
        );
        assert!(value.contains("PONYTAIL"));

        // A blank existing value adds nothing but leading whitespace.
        let argument = codex_developer_instructions_override("PONYTAIL", Some("  ".into()));
        let key_value = argument.strip_prefix("developer_instructions=").unwrap();
        let parsed: toml::Value = toml::from_str(&format!("value = {key_value}")).unwrap();
        assert_eq!(parsed["value"].as_str(), Some("PONYTAIL"));
    }

    #[test]
    fn state_directories_do_not_collide_between_modes() {
        // Concurrent sessions at different levels must not fight over upstream's
        // mode flag file.
        let lite = state_directory(PonytailMode::Lite);
        let ultra = state_directory(PonytailMode::Ultra);
        assert_ne!(lite, ultra);
        assert_eq!(state_directory(PonytailMode::Lite), lite);
    }
}
