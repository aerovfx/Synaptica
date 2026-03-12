import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import * as p from "@clack/prompts";
import pc from "picocolors";
import { bootstrapCeoInvite } from "./auth-bootstrap-ceo.js";
import { onboard } from "./onboard.js";
import { doctor } from "./doctor.js";
import { loadPaperclipEnvFile } from "../config/env.js";
import { configExists, resolveConfigPath } from "../config/store.js";
import type { PaperclipConfig } from "../config/schema.js";
import { readConfig } from "../config/store.js";
import {
  describeLocalInstancePaths,
  resolvePaperclipHomeDir,
  resolvePaperclipInstanceId,
} from "../config/home.js";

interface RunOptions {
  config?: string;
  instance?: string;
  repair?: boolean;
  yes?: boolean;
}

interface StartedServer {
  apiUrl: string;
  databaseUrl: string;
  host: string;
  listenPort: number;
}

export async function runCommand(opts: RunOptions): Promise<void> {
  const instanceId = resolvePaperclipInstanceId(opts.instance);
  process.env.PAPERCLIP_INSTANCE_ID = instanceId;

  const homeDir = resolvePaperclipHomeDir();
  fs.mkdirSync(homeDir, { recursive: true });

  const paths = describeLocalInstancePaths(instanceId);
  fs.mkdirSync(paths.instanceRoot, { recursive: true });

  const configPath = resolveConfigPath(opts.config);
  process.env.PAPERCLIP_CONFIG = configPath;
  loadPaperclipEnvFile(configPath);

  p.intro(pc.bgCyan(pc.black(" paperclipai run ")));
  p.log.message(pc.dim(`Home: ${paths.homeDir}`));
  p.log.message(pc.dim(`Instance: ${paths.instanceId}`));
  p.log.message(pc.dim(`Config: ${configPath}`));

  if (!configExists(configPath)) {
    if (!process.stdin.isTTY || !process.stdout.isTTY) {
      p.log.error("No config found and terminal is non-interactive.");
      p.log.message(`Run ${pc.cyan("paperclipai onboard")} once, then retry ${pc.cyan("paperclipai run")}.`);
      process.exit(1);
    }

    p.log.step("No config found. Starting onboarding...");
    await onboard({ config: configPath, invokedByRun: true });
  }

  p.log.step("Running doctor checks...");
  const summary = await doctor({
    config: configPath,
    repair: opts.repair ?? true,
    yes: opts.yes ?? true,
  });

  if (summary.failed > 0) {
    p.log.error("Doctor found blocking issues. Not starting server.");
    process.exit(1);
  }

  const config = readConfig(configPath);
  if (!config) {
    p.log.error(`No config found at ${configPath}.`);
    process.exit(1);
  }

  p.log.step("Starting Paperclip server (Rust)...");
  const { startedServer, exitPromise } = await startRustServer();

  if (shouldGenerateBootstrapInviteAfterStart(config)) {
    p.log.step("Generating bootstrap CEO invite");
    await bootstrapCeoInvite({
      config: configPath,
      dbUrl: startedServer.databaseUrl,
      baseUrl: resolveBootstrapInviteBaseUrl(config, startedServer),
    });
  }

  await exitPromise;
}

function resolveBootstrapInviteBaseUrl(
  config: PaperclipConfig,
  startedServer: StartedServer,
): string {
  const explicitBaseUrl =
    process.env.PAPERCLIP_PUBLIC_URL ??
    process.env.PAPERCLIP_AUTH_PUBLIC_BASE_URL ??
    process.env.BETTER_AUTH_URL ??
    process.env.BETTER_AUTH_BASE_URL ??
    (config.auth.baseUrlMode === "explicit" ? config.auth.publicBaseUrl : undefined);

  if (typeof explicitBaseUrl === "string" && explicitBaseUrl.trim().length > 0) {
    return explicitBaseUrl.trim().replace(/\/+$/, "");
  }

  return startedServer.apiUrl.replace(/\/api$/, "");
}

const PORT = Number(process.env.PORT) || 3100;
const HOST = process.env.HOST || "127.0.0.1";

async function waitForHealth(apiUrl: string, maxAttempts = 30): Promise<void> {
  const url = `${apiUrl}/api/health`;
  for (let i = 0; i < maxAttempts; i++) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // ignore
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`Server did not become ready at ${url}`);
}

async function startRustServer(): Promise<{
  startedServer: StartedServer;
  exitPromise: Promise<void>;
}> {
  const projectRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
  const serverRsDir = path.join(projectRoot, "server-rs");
  if (!fs.existsSync(path.join(serverRsDir, "Cargo.toml"))) {
    throw new Error(
      `server-rs not found at ${serverRsDir}. Run from repo root or install Paperclip with Rust server.`,
    );
  }

  const { spawn } = await import("node:child_process");
  const child = spawn("cargo", ["run"], {
    cwd: serverRsDir,
    stdio: "inherit",
    env: {
      ...process.env,
      UI_DIST: process.env.UI_DIST || path.join(projectRoot, "ui", "dist"),
    },
    shell: process.platform === "win32",
  });

  const apiUrl = `http://${HOST}:${PORT}`;
  await waitForHealth(apiUrl);

  const exitPromise = new Promise<void>((resolve) => {
    child.on("exit", (code, signal) => {
      if (signal) process.kill(process.pid, signal);
      else process.exit(code ?? 0);
      resolve();
    });
  });

  const startedServer: StartedServer = {
    apiUrl,
    databaseUrl: process.env.DATABASE_URL || "",
    host: HOST,
    listenPort: PORT,
  };

  return { startedServer, exitPromise };
}

function shouldGenerateBootstrapInviteAfterStart(config: PaperclipConfig): boolean {
  return config.server.deploymentMode === "authenticated" && config.database.mode === "embedded-postgres";
}
