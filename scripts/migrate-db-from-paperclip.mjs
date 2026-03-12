#!/usr/bin/env node
/**
 * Migrate database from Paperclip (source) to Synaptica (target).
 *
 * Source: Paperclip DB — from PAPERCLIP_SOURCE_DATABASE_URL, or from Paperclip
 * config (PAPERCLIP_SOURCE_CONFIG path, or ~/.paperclip/instances/default/config.json).
 * If using embedded Postgres, that instance must be running or use explicit URL.
 *
 * Target: Synaptica DB — from DATABASE_URL. Should be empty or will be overwritten.
 *
 * Usage:
 *   # Source = Paperclip embedded (default instance), target = current DATABASE_URL
 *   DATABASE_URL=postgres://user:pass@localhost:5432/synaptica pnpm exec node scripts/migrate-db-from-paperclip.mjs
 *
 *   # Source = explicit URL (e.g. Paperclip's Postgres)
 *   PAPERCLIP_SOURCE_DATABASE_URL=postgres://paperclip:paperclip@127.0.0.1:54329/paperclip \
 *   DATABASE_URL=postgres://user:pass@localhost:5432/synaptica \
 *   pnpm exec node scripts/migrate-db-from-paperclip.mjs
 *
 *   # Source = config from Paperclip repo
 *   PAPERCLIP_SOURCE_CONFIG=/Users/you/Synaptica/paperclip/.paperclip/config.json \
 *   DATABASE_URL=postgres://... pnpm exec node scripts/migrate-db-from-paperclip.mjs
 */

import { existsSync, readFileSync, mkdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

function expandHome(p) {
  if (p === "~" || p.startsWith("~/")) {
    return path.resolve(process.env.HOME || "", p.slice(1) || "");
  }
  return path.resolve(p);
}

function resolveSourceConnectionString() {
  const envUrl = process.env.PAPERCLIP_SOURCE_DATABASE_URL?.trim();
  if (envUrl) return { value: envUrl, source: "PAPERCLIP_SOURCE_DATABASE_URL" };

  const configPath =
    process.env.PAPERCLIP_SOURCE_CONFIG?.trim() ||
    path.join(process.env.HOME || "", ".paperclip", "instances", "default", "config.json");
  const resolvedConfigPath = expandHome(configPath);

  if (existsSync(resolvedConfigPath)) {
    let config;
    try {
      config = JSON.parse(readFileSync(resolvedConfigPath, "utf8"));
    } catch (e) {
      console.error("Could not parse Paperclip config at", resolvedConfigPath, e.message);
      process.exit(1);
    }
    const db = config?.database;
    if (db?.mode === "postgres" && typeof db?.connectionString === "string" && db.connectionString.trim()) {
      return { value: db.connectionString.trim(), source: `config: ${resolvedConfigPath}` };
    }
    const port = typeof db?.embeddedPostgresPort === "number" ? db.embeddedPostgresPort : 54329;
    return {
      value: `postgres://paperclip:paperclip@127.0.0.1:${port}/paperclip`,
      source: `config (embedded): ${resolvedConfigPath}`,
    };
  }

  console.error("Source DB not found. Set PAPERCLIP_SOURCE_DATABASE_URL or PAPERCLIP_SOURCE_CONFIG to Paperclip config.json.");
  console.error("Example: PAPERCLIP_SOURCE_DATABASE_URL=postgres://paperclip:paperclip@127.0.0.1:54329/paperclip");
  process.exit(1);
}

function resolveTargetConnectionString() {
  const url = process.env.DATABASE_URL?.trim();
  if (!url) {
    console.error("Target DB required. Set DATABASE_URL (Synaptica connection string).");
    process.exit(1);
  }
  return url;
}

async function main() {
  console.log("[migrate-db] Paperclip → Synaptica\n");

  const source = resolveSourceConnectionString();
  const target = resolveTargetConnectionString();

  console.log("Source:", source.source);
  console.log("Target: DATABASE_URL");
  console.log("");

  const backupDir = path.join(repoRoot, "scripts", "backups");
  mkdirSync(backupDir, { recursive: true });

  let runDatabaseBackup, runDatabaseRestore, formatDatabaseBackupResult;
  try {
    const db = await import("@paperclipai/db");
    runDatabaseBackup = db.runDatabaseBackup;
    runDatabaseRestore = db.runDatabaseRestore;
    formatDatabaseBackupResult = db.formatDatabaseBackupResult;
  } catch (e) {
    console.error("Load @paperclipai/db failed. Build first: pnpm --filter @paperclipai/db build");
    process.exit(1);
  }

  console.log("Step 1/2: Backup from Paperclip...");
  let backupFile;
  try {
    const result = await runDatabaseBackup({
      connectionString: source.value,
      backupDir,
      retentionDays: 1,
      filenamePrefix: "paperclip-to-synaptica",
      includeMigrationJournal: true,
    });
    backupFile = result.backupFile;
    console.log("  ", formatDatabaseBackupResult(result));
  } catch (e) {
    console.error("Backup failed:", e.message);
    console.error("Ensure Paperclip DB is reachable (e.g. start Paperclip server once if using embedded).");
    process.exit(1);
  }

  console.log("\nStep 2/2: Restore into Synaptica (DATABASE_URL)...");
  try {
    await runDatabaseRestore({
      connectionString: target,
      backupFile,
    });
    console.log("  Restore OK.");
  } catch (e) {
    console.error("Restore failed:", e.message);
    process.exit(1);
  }

  console.log("\nDone. Run migrations on Synaptica if needed:");
  console.log("  DATABASE_URL=... pnpm db:migrate");
  console.log("Then start Synaptica: pnpm dev");
}

main();
