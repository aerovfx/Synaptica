import type { AdapterExecutionContext, AdapterExecutionResult } from "@paperclipai/adapter-utils";
import { asString, parseObject } from "@paperclipai/adapter-utils/server-utils";

const OPENFANG_API_REF = "https://www.openfang.sh/docs/api-reference";

/**
 * OpenFang Gateway adapter execute.
 * Stub: returns failure with message to implement run endpoint per OpenFang API.
 * Config: baseUrl, apiKey (or auth header), model (optional).
 */
export async function execute(ctx: AdapterExecutionContext): Promise<AdapterExecutionResult> {
  const config = parseObject(ctx.agent.adapterConfig);
  const baseUrl = asString(config.baseUrl, "").trim() || asString(config.url, "").trim();
  const apiKey = asString(config.apiKey, "").trim() || asString(config.token, "").trim();

  const msg =
    "OpenFang gateway adapter: run endpoint not yet implemented. " +
    `Configure baseUrl and apiKey, then implement execute per ${OPENFANG_API_REF}`;
  await ctx.onLog?.("stderr", `[openfang_gateway] ${msg}\n`);

  return {
    exitCode: 1,
    signal: null,
    timedOut: false,
    errorMessage: `OpenFang gateway: implement run per ${OPENFANG_API_REF}`,
    errorCode: "not_implemented",
  };
}
