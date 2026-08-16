#!/usr/bin/env bun

import { watch, type FSWatcher } from "node:fs";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const cargo = process.env.CARGO || `${process.env.HOME}/.cargo/bin/cargo`;
const isMacOS = process.platform === "darwin";
const appPath = join(root, "target/debug/Anastasia Debug.app");
const watchers: FSWatcher[] = [];
let app: ReturnType<typeof Bun.spawn> | undefined;
let timer: ReturnType<typeof setTimeout> | undefined;
let rebuilding = false;
let queued = false;
let stopping = false;

async function stop() {
  const running = app;
  app = undefined;
  if (running?.exitCode === null) {
    if (isMacOS) Bun.spawnSync(["pkill", "-TERM", "-x", "anastasia-debug"]);
    running.kill("SIGTERM");
    await running.exited;
  }
}

async function rebuild() {
  if (rebuilding || stopping) {
    queued = true;
    return;
  }
  rebuilding = true;
  do {
    queued = false;
    await stop();
    console.log("[anastasia-dev] Building and launching…");
    const command = isMacOS
      ? [join(root, "scripts/bundle-debug.sh")]
      : [cargo, "build", "-p", "anastasia"];
    const build = Bun.spawn(command, {
      cwd: root,
      env: { ...process.env, CARGO: cargo },
      stdout: "inherit",
      stderr: "inherit",
    });
    if ((await build.exited) !== 0) continue;
    app = Bun.spawn(isMacOS ? ["open", "-n", "-W", appPath] : [join(root, "target/debug/anastasia")], {
      cwd: root,
      env: process.env,
      stdout: "inherit",
      stderr: "inherit",
    });
  } while (queued && !stopping);
  rebuilding = false;
}

function schedule() {
  clearTimeout(timer);
  timer = setTimeout(() => void rebuild(), 500);
}

for (const path of ["apps/anastasia/src", "crates"]) {
  watchers.push(watch(join(root, path), { recursive: true }, schedule));
}
watchers.push(
  watch(root, (_event, file) => {
    if (file && ["Cargo.toml", "Cargo.lock"].includes(file.toString())) schedule();
  }),
);

async function cleanup() {
  if (stopping) return;
  stopping = true;
  clearTimeout(timer);
  watchers.forEach((watcher) => watcher.close());
  await stop();
}

process.on("SIGINT", () => void cleanup());
process.on("SIGTERM", () => void cleanup());
await rebuild();
