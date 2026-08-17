#!/usr/bin/env bun
//
// Sign update archives and (re)generate the Sparkle appcast for a directory.
//
// Usage:
//   bun scripts/appcast.ts <updates-dir>
//
// <updates-dir> holds the packaged archives (e.g. Anastasia-0.3.0.zip) plus any
// older archives so Sparkle can build binary deltas. appcast.xml is written
// into that directory. The private EdDSA key is read from SPARKLE_PRIVATE_KEY
// when set, otherwise from the login keychain (see RELEASING.md).
//
// Env overrides:
//   SPARKLE_BIN                dir containing the Sparkle tools
//   SPARKLE_PRIVATE_KEY        EdDSA private key (CI; otherwise the keychain)
//   ANASTASIA_DOWNLOAD_URL_PREFIX   base URL for enclosure links
import { $ } from "bun";
import { existsSync, readdirSync } from "node:fs";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const projectRoot = resolve(import.meta.dir, "..");

/** Base for a release's assets on GitHub. `release.ts` appends the tag, since
 *  each release's assets live under their own `download/v<version>/` path. */
export const defaultDownloadUrlPrefix =
  "https://github.com/cowboyshibuya/anastasia/releases/download/";

/** Locate Sparkle's `generate_appcast`: SPARKLE_BIN first, then the pinned
 *  distribution scripts/bundle.sh caches under .anastasia-cache, then PATH. */
export function findGenerateAppcast(): string | null {
  const fromEnv = process.env.SPARKLE_BIN;
  if (fromEnv) {
    const candidate = join(fromEnv, "generate_appcast");
    if (existsSync(candidate)) return candidate;
  }

  const cacheRoot = join(projectRoot, ".anastasia-cache", "sparkle");
  if (existsSync(cacheRoot)) {
    const versionOrder = new Intl.Collator("en", { numeric: true });
    const versions = readdirSync(cacheRoot)
      .filter((name) => !name.startsWith("."))
      .sort((a, b) => versionOrder.compare(b, a));
    for (const version of versions) {
      const candidate = join(cacheRoot, version, "bin", "generate_appcast");
      if (existsSync(candidate)) return candidate;
    }
  }

  return Bun.which("generate_appcast");
}

/** Sign the archives in `updatesDir` and (re)write appcast.xml. */
export async function generateAppcast(
  updatesDir: string,
  downloadUrlPrefix: string,
): Promise<void> {
  const generator = findGenerateAppcast();
  if (!generator) {
    throw new Error(
      "generate_appcast not found. Run scripts/bundle.sh once to populate " +
        ".anastasia-cache/sparkle, or set SPARKLE_BIN to a Sparkle tools bin/ dir.",
    );
  }
  console.log(`Using: ${generator}`);
  // Same prefix for both: archives and the Anastasia-<version>.md release notes are
  // served from the same origin. The notes prefix makes generate_appcast emit
  // <sparkle:releaseNotesLink> for any notes file matching an archive name.
  // On a developer machine the key comes from the login keychain and
  // generate_appcast finds it unaided. CI has no keychain, so the key arrives in
  // SPARKLE_PRIVATE_KEY and is handed over as a file: `--ed-key-file -` would
  // read stdin, but Bun's shell has no stdin plumbing for it. The file lives in
  // a 0700 temp directory and is removed even if generation throws.
  const privateKey = process.env.SPARKLE_PRIVATE_KEY?.trim();
  let keyDirectory: string | undefined;
  try {
    let keyArguments: string[] = [];
    if (privateKey) {
      keyDirectory = await mkdtemp(join(tmpdir(), "anastasia-sparkle-"));
      const keyPath = join(keyDirectory, "ed25519");
      await writeFile(keyPath, `${privateKey}\n`, { mode: 0o600 });
      keyArguments = ["--ed-key-file", keyPath];
    }
    await $`${[
      generator,
      "--download-url-prefix",
      downloadUrlPrefix,
      "--release-notes-url-prefix",
      downloadUrlPrefix,
      ...keyArguments,
      updatesDir,
    ]}`;
  } finally {
    if (keyDirectory) {
      await rm(keyDirectory, { force: true, recursive: true });
    }
  }
  console.log(`Wrote ${join(updatesDir, "appcast.xml")}`);
}

if (import.meta.main) {
  const updatesDir = process.argv[2];
  if (!updatesDir) {
    console.error("usage: bun scripts/appcast.ts <updates-dir>");
    process.exit(1);
  }
  const prefix =
    process.env.ANASTASIA_DOWNLOAD_URL_PREFIX ?? defaultDownloadUrlPrefix;
  await generateAppcast(updatesDir, prefix);
}
