# Packaging

## Linux (implemented)

```sh
scripts/package-linux.sh            # release build (thin LTO, stripped)
PROFILE=debug scripts/package-linux.sh   # fast smoke package
```

Produces `target/package/anastasia-<version>-linux-<arch>.tar.gz` containing:

- `anastasia` — the binary (headed by default; `anastasia headless` runs the engine alone)
- `anastasia.desktop` — XDG desktop entry
- `anastasia.png` — 1024×1024 Anastasia app icon
- `install.sh` — installs into `~/.local/{bin,share/applications,share/icons}`

The release profile in the root `Cargo.toml` sets `lto = "thin"` and
`strip = "symbols"` for distribution builds.

## macOS

```sh
scripts/package-macos.sh    # → target/package/anastasia-<version>-macos-<arch>.dmg
```

Builds the release binary, assembles `Anastasia.app` (Info.plist + icns), ad-hoc
signs it (set `CODESIGN_IDENTITY` for a real Developer ID), and wraps it in a
dmg. The auto-update tarball retains an internal `Anastasia.app` path so older
installed builds can update into Anastasia. CI runs this on tags
(`.github/workflows/release.yml`). The manual steps it automates, for reference
(run on a macOS host — gpui needs Metal; no cross-build from Linux):

1. Build the universal (or per-arch) binary:
   ```sh
   cargo build --release -p anastasia --target aarch64-apple-darwin
   cargo build --release -p anastasia --target x86_64-apple-darwin
   lipo -create -output anastasia \
     target/aarch64-apple-darwin/release/anastasia \
     target/x86_64-apple-darwin/release/anastasia
   ```
2. Assemble the bundle:
   ```sh
   mkdir -p Anastasia.app/Contents/{MacOS,Resources}
   cp anastasia Anastasia.app/Contents/MacOS/anastasia
   sed "s/__VERSION__/$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')/" \
     dist/macos/Info.plist > Anastasia.app/Contents/Info.plist
   ```
3. Icon: generate `anastasia.icns` from `dist/macos/icon-1024.png` (the macOS-shaped
   variant of the artwork — squircle mask, margins, and shadow pre-baked, since
   `sips` can't apply an alpha mask) and place it at
   `Anastasia.app/Contents/Resources/anastasia.icns`:
   ```sh
   mkdir anastasia.iconset && sips -z 256 256 dist/macos/icon-1024.png --out anastasia.iconset/icon_256x256.png
   iconutil -c icns anastasia.iconset -o Anastasia.app/Contents/Resources/anastasia.icns
   ```
4. Sign + notarize (required for distribution):
   ```sh
   codesign --deep --force --options runtime --sign "Developer ID Application: …" Anastasia.app
   xcrun notarytool submit Anastasia.zip --keychain-profile … --wait
   xcrun stapler staple Anastasia.app
   ```
5. Ship as a `.dmg` (`hdiutil create -volname Anastasia -srcfolder Anastasia.app -ov -format UDZO Anastasia.dmg`).
