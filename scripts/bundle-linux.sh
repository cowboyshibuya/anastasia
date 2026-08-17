#!/usr/bin/env bash

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

target_dir="${CARGO_TARGET_DIR:-target}"
# The cargo package keeps its name as anastasia
version="$(cargo metadata --no-deps --format-version 1 | sed -n 's/.*"name":"anastasia","version":"\([^"]*\)".*/\1/p')"
target_triple="$(rustc -vV | sed -n 's/^host: //p')"
package="anastasia-${version}-${target_triple}"
archive="$target_dir/release/$package.tar.gz"
staging="$(mktemp -d)"
trap 'rm -rf -- "$staging"' EXIT

cargo build --locked --release --package anastasia --bin anastasia --package anastasia-daemon --bin anastasia-daemon

package_dir="$staging/$package"
install -Dm755 "$target_dir/release/anastasia" "$package_dir/bin/anastasia"
# Daemon executable packaged with sibling anastasia-daemon
install -Dm755 "$target_dir/release/anastasia-daemon" "$package_dir/bin/anastasia-daemon"
install -Dm644 resources/linux/app.anastasia.desktop \
  "$package_dir/share/applications/app.anastasia.desktop"
install -Dm644 website/public/app-icon.png \
  "$package_dir/share/icons/hicolor/256x256/apps/app.anastasia.png"
install -Dm644 LICENSE "$package_dir/share/licenses/anastasia/LICENSE"

mkdir -p "$(dirname "$archive")"
tar -C "$staging" -czf "$archive" "$package"
printf 'Created %s\n' "$archive"
