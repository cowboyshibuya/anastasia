#!/usr/bin/env bun

import { $ } from "bun";
import { resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
$.cwd(root);

// Ensure cargo is in PATH if installed in standard ~/.cargo/bin
const home = process.env.HOME || "";
const cargoBin = `${home}/.cargo/bin`;
const currentPath = process.env.PATH || "";
const pathWithCargo = currentPath.includes(cargoBin)
  ? currentPath
  : `${cargoBin}:${currentPath}`;

$.env({ ...process.env, PATH: pathWithCargo });

const args = process.argv.slice(2);
const skipRust = args.includes("--skip-rust") || args.includes("--ts-only");
const skipTs = args.includes("--skip-ts") || args.includes("--rust-only");
const quick = args.includes("--quick");

console.log("🚀 Starting Anastasia test & verification suite...\n");

async function runStep(name: string, fn: () => Promise<any>): Promise<void> {
  const start = Date.now();
  process.stdout.write(`⏳ [${name}] Running... \r`);
  try {
    await fn();
    const duration = ((Date.now() - start) / 1000).toFixed(2);
    console.log(`✅ [${name}] Passed in ${duration}s`);
  } catch (error) {
    const duration = ((Date.now() - start) / 1000).toFixed(2);
    console.error(`❌ [${name}] Failed after ${duration}s\n`);
    throw error;
  }
}

try {
  if (!skipTs) {
    await runStep("TypeScript Unit Tests", async () => {
      await $`bun test`;
    });
  }

  if (!skipRust) {
    await runStep("Protocol Bindings Check", async () => {
      await $`cargo run -p anastasia-protocol --bin export_types -- --check`;
    });

    await runStep("Rust Workspace Check", async () => {
      await $`cargo check --all-targets`;
    });

    await runStep("Rust Unit Tests", async () => {
      if (quick) {
        await $`cargo test --lib`;
      } else {
        await $`cargo test --all-targets`;
      }
    });
  }

  console.log("\n🎉 All checks and tests passed successfully!");
} catch {
  console.error("\n💥 Test suite failed.");
  process.exit(1);
}
