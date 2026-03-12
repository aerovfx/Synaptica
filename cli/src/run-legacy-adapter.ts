/**
 * Standalone entrypoint for running a legacy (Node) adapter from Rust.
 * Rust runner spawns this with env: PAPERCLIP_RUN_ID, PAPERCLIP_AGENT_ID, PAPERCLIP_COMPANY_ID,
 * PAPERCLIP_API_URL, AGENT_NAME, ADAPTER_TYPE, ADAPTER_CONFIG_JSON, RUNTIME_JSON (optional).
 * Stdout: one JSON object per line. {"t":"log","s":"stdout"|"stderr","m":string} or {"t":"result","r":AdapterExecutionResult}.
 * Stderr: free for debug. Exit code: 0 if result.exitCode===0, else 1.
 */
import { execute as claudeLocalExecute } from "@paperclipai/adapter-claude-local/server";
import { execute as codexLocalExecute } from "@paperclipai/adapter-codex-local/server";
import { execute as cursorLocalExecute } from "@paperclipai/adapter-cursor-local/server";
import { execute as openclawGatewayExecute } from "@paperclipai/adapter-openclaw-gateway/server";
import { execute as opencodeLocalExecute } from "@paperclipai/adapter-opencode-local/server";
import { execute as piLocalExecute } from "@paperclipai/adapter-pi-local/server";
import type { AdapterExecutionResult, AdapterExecutionContext } from "@paperclipai/adapter-utils";

const LEGACY_REGISTRY: Record<string, (ctx: AdapterExecutionContext) => Promise<AdapterExecutionResult>> = {
  claude_local: claudeLocalExecute,
  codex_local: codexLocalExecute,
  cursor: cursorLocalExecute,
  openclaw_gateway: openclawGatewayExecute,
  opencode_local: opencodeLocalExecute,
  pi_local: piLocalExecute,
};

function out(line: object): void {
  process.stdout.write(JSON.stringify(line) + "\n");
}

async function main(): Promise<void> {
  const runId = process.env.PAPERCLIP_RUN_ID ?? "";
  const agentId = process.env.PAPERCLIP_AGENT_ID ?? "";
  const companyId = process.env.PAPERCLIP_COMPANY_ID ?? "";
  const agentName = process.env.AGENT_NAME ?? "";
  const adapterType = process.env.ADAPTER_TYPE ?? "";
  const adapterConfigJson = process.env.ADAPTER_CONFIG_JSON ?? "{}";
  const runtimeJson = process.env.RUNTIME_JSON ?? "{}";

  if (!runId || !agentId || !companyId || !adapterType) {
    process.stderr.write("Missing required env: PAPERCLIP_RUN_ID, PAPERCLIP_AGENT_ID, PAPERCLIP_COMPANY_ID, ADAPTER_TYPE\n");
    out({ t: "result", r: { exitCode: 1, signal: null, timedOut: false, errorMessage: "Missing required env" } });
    process.exit(1);
  }

  const executeFn = LEGACY_REGISTRY[adapterType];
  if (!executeFn) {
    process.stderr.write(`Unknown legacy adapter type: ${adapterType}\n`);
    out({ t: "result", r: { exitCode: 1, signal: null, timedOut: false, errorMessage: `Adapter type not supported: ${adapterType}` } });
    process.exit(1);
  }

  let adapterConfig: Record<string, unknown>;
  try {
    adapterConfig = JSON.parse(adapterConfigJson) as Record<string, unknown>;
  } catch {
    process.stderr.write("Invalid ADAPTER_CONFIG_JSON\n");
    out({ t: "result", r: { exitCode: 1, signal: null, timedOut: false, errorMessage: "Invalid ADAPTER_CONFIG_JSON" } });
    process.exit(1);
  }

  let runtimeParams: Record<string, unknown> = {};
  try {
    runtimeParams = (JSON.parse(runtimeJson) as Record<string, unknown>) ?? {};
  } catch {
    // optional
  }

  const ctx: AdapterExecutionContext = {
    runId,
    agent: {
      id: agentId,
      companyId,
      name: agentName,
      adapterType,
      adapterConfig,
    },
    runtime: {
      sessionId: (runtimeParams.sessionId as string) ?? null,
      sessionParams: (runtimeParams.sessionParams as Record<string, unknown>) ?? null,
      sessionDisplayId: (runtimeParams.sessionDisplayId as string) ?? null,
      taskKey: (runtimeParams.taskKey as string) ?? null,
    },
    config: adapterConfig,
    context: {},
    onLog: async (stream, chunk) => {
      out({ t: "log", s: stream, m: chunk });
    },
  };

  try {
    const result = await executeFn(ctx);
    out({ t: "result", r: result });
    process.exit(result.exitCode === 0 ? 0 : 1);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    process.stderr.write(`${message}\n`);
    out({
      t: "result",
      r: {
        exitCode: 1,
        signal: null,
        timedOut: false,
        errorMessage: message,
      },
    });
    process.exit(1);
  }
}

void main();
