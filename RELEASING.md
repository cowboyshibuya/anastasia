# Releasing Anastasia

Anastasia auto-updates with [Sparkle](https://sparkle-project.org), served from
this repository's own **GitHub releases** — the same arrangement Alabasta uses,
without a bucket or a CDN to maintain.

New users download a notarized **`.dmg`**; existing users get an in-app update
from the Sparkle archive. The app reads its feed from

```
https://github.com/cowboyshibuya/anastasia/releases/latest/download/appcast.xml
```

`/releases/latest/download/` always resolves to the newest **published**
(non-draft, non-prerelease) release, so the feed URL compiled into the app never
changes even though each build's assets live under their own tag.

Cutting a release is: bump the version, write the changelog section, push a
`v*` tag, publish the draft the workflow opens.

## Where the pieces live

- **Updater**: [`src/updater.rs`](src/updater.rs) loads the embedded
  Sparkle.framework at runtime and starts `SPUUpdater` with Anastasia's own user
  driver. Available updates appear in the sidebar footer; download, signature
  verification, install and relaunch stay Sparkle's. **Check for Updates…** is
  in the app menu, and Settings → General mirrors Sparkle's automatic-check
  setting.
- **Feed URL + public key**: [`resources/Info.plist`](resources/Info.plist)
  (`SUFeedURL`, `SUPublicEDKey`).
- **Framework embedding**: [`scripts/bundle.sh`](scripts/bundle.sh) — bump
  `sparkle_version` and `sparkle_sha256` together; the distribution caches under
  `.anastasia-cache/sparkle/`.
- **Release automation**: [`scripts/release.ts`](scripts/release.ts),
  [`scripts/appcast.ts`](scripts/appcast.ts),
  [`scripts/changelog.ts`](scripts/changelog.ts).
- **CI**: [`.github/workflows/release.yml`](.github/workflows/release.yml)
  builds Linux (x86_64, arm64) and macOS on a `v*` tag and opens a draft
  release.

## Two constraints worth knowing

**The archive's app bundle must carry `SUPublicEDKey`, and must be validly code
signed.** `generate_appcast` verifies Apple code signing before it will touch an
archive, and only emits `sparkle:edSignature` when the bundle names a public key
it can match. An app signed *before* the plist was edited fails both. If an
appcast comes out with no `edSignature`, this is why.

**One release per appcast.** Sparkle's `generate_appcast` applies a single
`--download-url-prefix` to every archive in the directory, but each GitHub
release serves its assets from its own tag path. So the appcast carries only the
release being cut. That also rules out binary deltas, which need the previous
archives staged alongside — updates are full downloads.

---

## One-time setup

Local releases need [Bun](https://bun.sh) and
[`create-dmg`](https://github.com/create-dmg/create-dmg):
`brew install bun create-dmg`. CI installs them itself.

### 1. Sparkle signing key

Updates are signed with an ed25519 key. The private half lives in the login
keychain; the public half ships in `Info.plist` as `SUPublicEDKey`.

The key already exists on this Mac and its public half is already in the plist.
To put it in CI, export it and paste it into the `SPARKLE_PRIVATE_KEY`
repository secret:

```sh
.anastasia-cache/sparkle/*/bin/generate_keys -x sparkle_private_key.txt
```

Back that file up in a password manager, then delete it.

> ⚠️ Lose the private key and **every existing install can never update again** —
> they only trust the matching public key. There is no recovery but a manual
> reinstall by each user.

On a fresh machine, restore it with `generate_keys -f sparkle_private_key.txt`,
and confirm with `generate_keys -p` that the printed key matches
`SUPublicEDKey`.

### 2. Developer ID signing + notarization

Local releases sign with `ANASTASIA_SIGNING_IDENTITY` (or `--signing-identity`)
and notarize through the `NOTARY` keychain profile:

```sh
xcrun notarytool store-credentials NOTARY \
  --key AuthKey_XXXX.p8 --key-id YOUR_KEY_ID --issuer YOUR_ISSUER_ID
```

CI does the same from repository secrets, and derives the signing identity from
the imported certificate rather than keeping it as a separate secret.

### 3. Repository secrets

| Secret | Purpose |
| --- | --- |
| `MACOS_CERT_P12` | **base64** Developer ID Application `.p12` |
| `MACOS_CERT_PASSWORD` | password for that `.p12` |
| `AC_API_KEY_P8` | App Store Connect API key, **raw `.p8` text** (not base64) |
| `AC_API_KEY_ID` | that key's ID |
| `AC_API_ISSUER_ID` | that key's issuer ID |
| `SPARKLE_PRIVATE_KEY` | EdDSA private key that signs the appcast |

The two Apple secrets are stored in *different* formats — base64 for the
certificate, raw text for the key. Decoding the `.p8` as base64 fails with
`invalidPrivateKeyContents`.

---

## Cutting a release

1. **Bump `version` in `Cargo.toml`** — the single source of truth.
   `CFBundleShortVersionString` is that version and `CFBundleVersion` is derived
   from it (`major*1e6 + minor*1e3 + patch`, so `0.3.1` → `3001`), which keeps
   Sparkle's build-number comparison monotonic with no manual counter.
2. **Write the release notes** — a `## [<version>]` section at the top of
   [`CHANGELOG.md`](CHANGELOG.md). It ships beside the archive and Sparkle shows
   it in the update prompt.
3. **Tag and push:**
   ```sh
   git tag v<version> && git push origin v<version>
   ```
4. **Publish the draft release** the workflow opens. Nothing updates until you
   do — `/releases/latest/` ignores drafts.

The macOS job runs `bun run release --local`, which builds and signs the app,
verifies the bundled JS REPL and computer-use helper, builds the DMG, notarizes
and staples DMG + app, zips the app for Sparkle, attaches the changelog section,
and writes the signed `appcast.xml`. Assets on the release:

- `Anastasia-<version>.dmg` — what new users download
- `Anastasia-<version>.zip` — what Sparkle downloads
- `Anastasia-<version>.md` — release notes
- `appcast.xml` — the feed
- `anastasia-<version>-{x86_64,aarch64}-unknown-linux-gnu.tar.gz`

To test: keep an older build, launch it, and choose **Check for Updates…**.

### Prereleases

A version like `0.4.0-beta.1` is built and published as normal, but
`/releases/latest/` skips prereleases, so it never reaches anyone on the stable
feed — mark the GitHub release as a prerelease and it stays invisible to the
updater.
