/**
 * One-off script: renames agents so that no two agents in the same company
 * share the same URL slug (derived from name). Duplicates are renamed to
 * "Name (2)", "Name (3)", etc. so slugs become unique.
 *
 * Run from repo root: pnpm db:dedupe-agent-slugs
 * Or: pnpm --filter @paperclipai/db dedupe-agent-slugs
 *
 * Uses same DB as migrations (DATABASE_URL or embedded postgres).
 */
import postgres from "postgres";
import { normalizeAgentUrlKey } from "@paperclipai/shared";
import { resolveMigrationConnection } from "../migration-runtime.js";

/** Slug from name (matches backend and shared: Vietnamese -> không dấu, then slugify). */
function normalizeSlug(name: string): string {
  return normalizeAgentUrlKey(name) ?? "";
}

type AgentRow = { id: string; company_id: string; name: string; created_at: Date };

async function main(): Promise<void> {
  const resolved = await resolveMigrationConnection();
  console.log(`Using DB: ${resolved.source}`);

  const sql = postgres(resolved.connectionString, { max: 1 });
  try {
    const rows = await sql<AgentRow[]>`
      SELECT id, company_id, name, created_at
      FROM agents
      WHERE status != 'terminated'
      ORDER BY company_id, created_at
    `;

    const byKey = new Map<string, AgentRow[]>();
    for (const r of rows) {
      const slug = normalizeSlug(r.name);
      if (!slug) continue;
      const key = `${r.company_id}:${slug}`;
      let list = byKey.get(key);
      if (!list) {
        list = [];
        byKey.set(key, list);
      }
      list.push(r);
    }

    let updated = 0;
    for (const [, list] of byKey) {
      if (list.length <= 1) continue;
      const baseName = list[0]!.name;
      for (let i = 1; i < list.length; i++) {
        const agent = list[i]!;
        const newName = `${baseName} (${i + 1})`;
        await sql`UPDATE agents SET name = ${newName}, updated_at = now() WHERE id = ${agent.id}::uuid`;
        console.log(`Renamed agent ${agent.id}: "${agent.name}" -> "${newName}"`);
        updated++;
      }
    }

    if (updated === 0) {
      console.log("No duplicate slugs found.");
    } else {
      console.log(`Done. Renamed ${updated} agent(s).`);
    }
  } finally {
    await sql.end();
    await resolved.stop();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
