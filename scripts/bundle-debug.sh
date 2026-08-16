#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TARGET=${CARGO_TARGET_DIR:-"$ROOT/target"}
APP="$TARGET/debug/Anastasia Debug.app"

"${CARGO:-cargo}" build -p anastasia
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$TARGET/debug/anastasia" "$APP/Contents/MacOS/anastasia-debug"
cp "$ROOT/dist/macos/Info.debug.plist" "$APP/Contents/Info.plist"
cp "$ROOT/dist/macos/icon-1024.png" "$APP/Contents/Resources/anastasia.png"
codesign --force --deep --sign - "$APP" >/dev/null
