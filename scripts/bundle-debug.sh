#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TARGET=${CARGO_TARGET_DIR:-"$ROOT/target"}
APP="$TARGET/debug/Anastasia Debug.app"
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)

"${CARGO:-cargo}" build -p anastasia
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$TARGET/debug/anastasia" "$APP/Contents/MacOS/anastasia-debug"
sed "s/__VERSION__/$VERSION/g" "$ROOT/dist/macos/Info.debug.plist" > "$APP/Contents/Info.plist"
cp "$ROOT/dist/macos/icon-1024.png" "$APP/Contents/Resources/anastasia.png"
codesign --force --deep --sign - "$APP" >/dev/null
