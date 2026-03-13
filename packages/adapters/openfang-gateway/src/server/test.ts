import type {
  AdapterEnvironmentCheck,
  AdapterEnvironmentTestContext,
  AdapterEnvironmentTestResult,
} from "@paperclipai/adapter-utils";
import { asString, parseObject } from "@paperclipai/adapter-utils/server-utils";

function summarizeStatus(checks: AdapterEnvironmentCheck[]): AdapterEnvironmentTestResult["status"] {
  if (checks.some((c) => c.level === "error")) return "fail";
  if (checks.some((c) => c.level === "warn")) return "warn";
  return "pass";
}

function nonEmpty(value: unknown): string | null {
  return typeof value === "string" && value.trim().length > 0 ? value.trim() : null;
}

export async function testEnvironment(
  ctx: AdapterEnvironmentTestContext,
): Promise<AdapterEnvironmentTestResult> {
  const checks: AdapterEnvironmentCheck[] = [];
  const config = parseObject(ctx.config);
  const baseUrl = nonEmpty(config.baseUrl) ?? nonEmpty(config.url) ?? "";
  const apiKey = nonEmpty(config.apiKey) ?? nonEmpty(config.token) ?? null;

  if (!baseUrl) {
    checks.push({
      code: "openfang_base_url_missing",
      level: "error",
      message: "OpenFang gateway requires baseUrl (or url) in adapter config.",
      hint: "Set the base URL of your OpenFang instance (e.g. https://openfang.example.com).",
    });
  } else {
    try {
      const url = new URL(baseUrl);
      if (!["http:", "https:"].includes(url.protocol)) {
        checks.push({
          code: "openfang_base_url_scheme",
          level: "error",
          message: "baseUrl must use http or https.",
          detail: baseUrl,
        });
      } else {
        checks.push({
          code: "openfang_base_url_ok",
          level: "info",
          message: `Base URL configured: ${url.origin}`,
        });
      }
    } catch {
      checks.push({
        code: "openfang_base_url_invalid",
        level: "error",
        message: "baseUrl is not a valid URL.",
        detail: baseUrl,
      });
    }
  }

  if (!apiKey && baseUrl) {
    checks.push({
      code: "openfang_api_key_missing",
      level: "warn",
      message: "No apiKey (or token) set. Some OpenFang endpoints may require authentication.",
      hint: "Add apiKey in adapter config if your instance requires it.",
    });
  } else if (apiKey) {
    checks.push({
      code: "openfang_api_key_set",
      level: "info",
      message: "API key is set.",
    });
  }

  return {
    adapterType: ctx.adapterType,
    status: summarizeStatus(checks),
    checks,
    testedAt: new Date().toISOString(),
  };
}
