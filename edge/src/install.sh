#!/bin/sh
# Anastasia (native) headless installer.
#
#   curl -fsSL https://raw.githubusercontent.com/cowboyshibuya/anastasia/main/edge/src/install.sh | sh
#
# Installs the self-contained native binary (no runtime deps) to
# ~/.anastasia/app, puts `anastasia` on PATH, and runs it as a local-only
# systemd user service that survives reboots. Signing in is optional and
# enables sync after a restart. Re-running
# upgrades in place; ~/.anastasia state is preserved.
#
set -eu

REPO="${ANASTASIA_GITHUB_REPO:-cowboyshibuya/anastasia}"
API="${ANASTASIA_RELEASE_API_URL:-https://api.github.com/repos/$REPO/releases/latest}"

# --- platform ---------------------------------------------------------------
os="$(uname -s)"
arch="$(uname -m)"
case "$os" in
  Linux) plat=linux ;;
  Darwin)
    echo "anastasia install: on macOS, download the desktop app instead:" >&2
    echo "  https://github.com/$REPO/releases/latest" >&2
    exit 1
    ;;
  *)
    echo "anastasia install: unsupported OS '$os' — only Linux for now." >&2
    exit 1
    ;;
esac
case "$arch" in
  x86_64 | amd64) arch=x86_64 ;;
  aarch64 | arm64) arch=aarch64 ;;
  *)
    echo "anastasia install: unsupported architecture '$arch'." >&2
    exit 1
    ;;
esac

# --- download ----------------------------------------------------------------
ver="$(curl -fsSL "$API" | sed -n 's/^[[:space:]]*"tag_name":[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/p' | head -1)"
[ -n "$ver" ] || { echo "anastasia install: could not resolve latest version" >&2; exit 1; }
file="anastasia-$ver-$plat-$arch.tar.gz"
download_base="https://github.com/$REPO/releases/download/v$ver"
data_root="$HOME/.anastasia"
app_root="$data_root/app"
dest="$app_root/$ver"

if [ -x "$dest/anastasia" ]; then
  echo "anastasia $ver already downloaded — relinking."
else
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  echo "downloading anastasia $ver ($plat-$arch)…"
  curl -fSL --progress-bar "$download_base/$file" -o "$tmp/$file"
  mkdir -p "$dest"
  tar -xzf "$tmp/$file" -C "$dest" --strip-components=1
fi

ln -sfn "$dest" "$app_root/current"
mkdir -p "$HOME/.local/bin"
ln -sf "$app_root/current/anastasia" "$HOME/.local/bin/anastasia"

# --- service -----------------------------------------------------------------
# The daemon is useful before auth: without a saved session it serves the local
# profile. Login only changes which profile the next daemon start selects.

service=manual
if command -v systemctl >/dev/null 2>&1 && [ -n "${XDG_RUNTIME_DIR:-}" ]; then
  mkdir -p "$HOME/.config/systemd/user"
  cat >"$HOME/.config/systemd/user/anastasia.service" <<'UNIT'
[Unit]
Description=Anastasia native headless engine
After=network-online.target
StartLimitIntervalSec=60
StartLimitBurst=5

[Service]
ExecStart=%h/.anastasia/app/current/anastasia headless
Restart=on-failure
RestartSec=5
EnvironmentFile=-%h/.anastasia/env

[Install]
WantedBy=default.target
UNIT
  systemctl --user daemon-reload
  systemctl --user enable anastasia
  systemctl --user restart anastasia
  service=running
  # Keep the user manager (and the engine) running without an active login.
  loginctl enable-linger "$USER" 2>/dev/null \
    || sudo -n loginctl enable-linger "$USER" 2>/dev/null \
    || echo "warn: could not enable linger — the engine stops when you log out (run: sudo loginctl enable-linger $USER)"
else
  echo "warn: systemd user session not available — run the engine manually with: anastasia headless"
fi

# --- agent CLIs ---------------------------------------------------------------
command -v claude >/dev/null 2>&1 || \
  echo "note: Claude Code CLI not found — install it with: curl -fsSL https://claude.ai/install.sh | bash"

case ":$PATH:" in
  *":$HOME/.local/bin:"*) path_hint="" ;;
  *) path_hint=' (add ~/.local/bin to your PATH)' ;;
esac

echo ""
echo "✓ anastasia $ver installed$path_hint"
echo ""
case "$service" in
  running)
    echo "the engine is running with the new version (local-only unless sync is enabled)."
    echo "  systemctl --user status anastasia    check the service"
    echo ""
    echo "optional sync (local sessions stay local):"
    echo "  systemctl --user stop anastasia"
    echo "  anastasia login"
    echo "  systemctl --user restart anastasia"
    ;;
  manual)
    echo "next: run the local-only engine with \`anastasia headless\`."
    echo "optional sync: run \`anastasia login\` before starting the engine."
    ;;
esac
