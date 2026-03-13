import type { CreateConfigValues } from "@paperclipai/adapter-utils";

export function buildOpenFangGatewayConfig(v: CreateConfigValues): Record<string, unknown> {
  const baseUrl = typeof v.url === "string" ? v.url.trim() : "";
  const extra = v as unknown as Record<string, unknown>;
  const apiKey = typeof extra.openfangApiKey === "string" ? extra.openfangApiKey.trim() : "";
  const out: Record<string, unknown> = { baseUrl };
  if (apiKey) out.apiKey = apiKey;
  return out;
}
