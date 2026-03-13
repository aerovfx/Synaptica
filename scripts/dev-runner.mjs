#!/usr/bin/env node
/**
 * Paperclip dev runner: migration check + Rust server (server-rs).
 * UI: build first with `pnpm --filter @paperclipai/ui build` (or set UI_DIST).
 * Loads .env from repo root so DATABASE_URL and other vars are available to the
 * Rust server (which runs with cwd=server-rs and would not see repo root .env).
 */
import { spawn } from "node:child_process";
import { createInterface } from "node:readline/promises";
import { stdin, stdout } from "node:process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { readFile } from "node:fs/promises";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");
const serverRsDir = path.join(repoRoot, "server-rs");

const mode = process.argv[2] === "watch" ? "watch" : "dev";
const cliArgs = process.argv.slice(3);

const env = { ...process.env };

/** Load KEY=VALUE from repo root .env into env (does not override existing). */
async function loadEnvFromRepoRoot(targetEnv) {
  const envPath = path.join(repoRoot, ".env");
  let content;
  try {
    content = await readFile(envPath, "utf8");
  } catch (e) {
    if (e.code === "ENOENT") return;
    throw e;
  }
  for (const line of content.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const eq = trimmed.indexOf("=");
    if (eq <= 0) continue;
    const key = trimmed.slice(0, eq).trim();
    let value = trimmed.slice(eq + 1).trim();
    if ((value.startsWith('"') && value.endsWith('"')) || (value.startsWith("'") && value.endsWith("'"))) {
      value = value.slice(1, -1).replace(/\\n/g, "\n");
    }
    if (key) targetEnv[key] = value;
  }
}

const pnpmBin = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

function formatPendingMigrationSummary(migrations) {
  if (migrations.length === 0) return "none";
  return migrations.length > 3
    ? `${migrations.slice(0, 3).join(", ")} (+${migrations.length - 3} more)`
    : migrations.join(", ");
}

async function runPnpm(args, options = {}) {
  return await new Promise((resolve, reject) => {
    const child = spawn(pnpmBin, args, {
      stdio: options.stdio ?? ["ignore", "pipe", "pipe"],
      env: options.env ?? process.env,
      shell: process.platform === "win32",
    });
    let stdoutBuffer = "";
    let stderrBuffer = "";
    if (child.stdout) child.stdout.on("data", (chunk) => { stdoutBuffer += String(chunk); });
    if (child.stderr) child.stderr.on("data", (chunk) => { stderrBuffer += String(chunk); });
    child.on("error", reject);
    child.on("exit", (code, signal) => {
      resolve({ code: code ?? 0, signal, stdout: stdoutBuffer, stderr: stderrBuffer });
    });
  });
}

async function maybePreflightMigrations() {
  if (mode !== "watch") return;
  if (process.env.PAPERCLIP_MIGRATION_PROMPT === "never") return;

  const status = await runPnpm(
    ["--filter", "@paperclipai/db", "exec", "tsx", "src/migration-status.ts", "--json"],
    { env },
  );
  if (status.code !== 0) {
    process.stderr.write(status.stderr || status.stdout);
    process.exit(status.code);
  }

  let payload;
  try {
    payload = JSON.parse(status.stdout.trim());
  } catch (error) {
    process.stderr.write(status.stderr || status.stdout);
    throw error;
  }

  if (payload.status !== "needsMigrations" || payload.pendingMigrations.length === 0) {
    return;
  }

  const autoApply = process.env.PAPERCLIP_MIGRATION_AUTO_APPLY === "true";
  let shouldApply = autoApply;

  if (!autoApply && stdin.isTTY && stdout.isTTY) {
    const prompt = createInterface({ input: stdin, output: stdout });
    try {
      const answer = (await prompt.question(
        `Apply pending migrations (${formatPendingMigrationSummary(payload.pendingMigrations)}) now? (y/N): `,
      ))
        .trim()
        .toLowerCase();
      shouldApply = answer === "y" || answer === "yes";
    } finally {
      prompt.close();
    }
  } else if (!autoApply) {
    shouldApply = true;
  }

  if (!shouldApply) return;

  const migrate = spawn(pnpmBin, ["db:migrate"], { stdio: "inherit", env, shell: process.platform === "win32" });
  const exit = await new Promise((resolve) => {
    migrate.on("exit", (code, signal) => resolve({ code: code ?? 0, signal }));
  });
  if (exit.signal) {
    process.kill(process.pid, exit.signal);
    return;
  }
  if (exit.code !== 0) {
    process.exit(exit.code);
  }
}

await loadEnvFromRepoRoot(env);

await maybePreflightMigrations();

if (mode === "watch") {
  env.PAPERCLIP_MIGRATION_PROMPT = "never";
}

console.log("[paperclip] dev: Rust backend (server-rs)");
const child = spawn(
  "cargo",
  ["run"],
  {
    cwd: serverRsDir,
    stdio: "inherit",
    env: { ...env, UI_DIST: env.UI_DIST || path.join(repoRoot, "ui", "dist") },
    shell: process.platform === "win32",
  },
);

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  process.exit(code ?? 0);
});
