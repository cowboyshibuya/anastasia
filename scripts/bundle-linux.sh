#!/usr/bin/env bash

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

target_dir="${CARGO_TARGET_DIR:-target}"
# The cargo package keeps its upstream name so merges from Waku stay cheap; only
# what the user types and sees is Anastasia.
version="$(cargo metadata --no-deps --format-version 1 | sed -n 's/.*"name":"waku","version":"\([^"]*\)".*/\1/p')"
target_triple="$(rustc -vV | sed -n 's/^host: //p')"
package="anastasia-${version}-${target_triple}"
archive="$target_dir/release/$package.tar.gz"
staging="$(mktemp -d)"
trap 'rm -rf -- "$staging"' EXIT

cargo build --locked --release --package waku --bin waku --package waku-daemon --bin waku-daemon

package_dir="$staging/$package"
install -Dm755 "$target_dir/release/waku" "$package_dir/bin/anastasia"
# Keeps its upstream file name: `daemon_executable_path` looks for a sibling
# `waku-daemon`, which is also cargo's own bin name in the dev loop.
install -Dm755 "$target_dir/release/waku-daemon" "$package_dir/bin/waku-daemon"
install -Dm644 resources/linux/app.anastasia.desktop \
  "$package_dir/share/applications/app.anastasia.desktop"
install -Dm644 website/public/app-icon.png \
  "$package_dir/share/icons/hicolor/256x256/apps/app.anastasia.png"
install -Dm644 LICENSE "$package_dir/share/licenses/anastasia/LICENSE"

mkdir -p "$(dirname "$archive")"
tar -C "$staging" -czf "$archive" "$package"
printf 'Created %s\n' "$archive"
