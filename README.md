# Anastasia

Anastasia is a fast, local-first desktop workspace for Claude Code, Codex,
Gemini CLI, Cursor, Grok, Hermes, and Pi. It is built in Rust with GPUI and is
dark by default.

Downloads are coming soon. Anastasia does not ship analytics, a hosted sync
service, or inherited production credentials.

## Develop

Requirements: Rust, Bun, and the CLI for any agent you want to use.

```bash
bun run dev
```

On macOS this builds, ad-hoc signs, and launches `Anastasia Debug.app`; source
changes rebuild and relaunch it. Release builds use `scripts/package-macos.sh`
or `scripts/package-linux.sh`.

Gemini uses its native ACP transport:

```bash
gemini --acp
```

Your existing CLI authentication, settings, extensions, hooks, skills, and MCP
configuration remain owned by each provider.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the implementation overview.

Anastasia is derived from the MIT-licensed Comet/Zeron project. See
[LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
