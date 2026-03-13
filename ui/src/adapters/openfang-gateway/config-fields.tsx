import type { AdapterConfigFieldsProps } from "../types";
import { Field, DraftInput } from "../../components/agent-config-primitives";

const inputClass =
  "w-full rounded-md border border-border px-2.5 py-1.5 bg-transparent outline-none text-sm font-mono placeholder:text-muted-foreground/40";

export function OpenFangGatewayConfigFields({
  isCreate,
  values,
  set,
  config,
  eff,
  mark,
}: AdapterConfigFieldsProps) {
  const baseUrl = isCreate
    ? (values?.url ?? "")
    : eff("adapterConfig", "baseUrl", String(config.baseUrl ?? config.url ?? ""));
  const apiKey = isCreate
    ? String((values as unknown as Record<string, unknown>)?.openfangApiKey ?? "")
    : eff("adapterConfig", "apiKey", String(config.apiKey ?? config.token ?? ""));

  return (
    <>
      <Field label="Base URL" hint="OpenFang instance URL (e.g. https://openfang.example.com)">
        <DraftInput
          value={baseUrl}
          onCommit={(v) =>
            isCreate
              ? set!({ ...values!, url: v })
              : mark("adapterConfig", "baseUrl", v || undefined)
          }
          immediate
          className={inputClass}
          placeholder="https://openfang.example.com"
        />
      </Field>
      <Field label="API key (optional)" hint="Auth token if your OpenFang instance requires it">
        <DraftInput
          value={apiKey}
          onCommit={(v) => {
            if (isCreate && set && values) {
              set({
                ...(values as unknown as Record<string, unknown>),
                openfangApiKey: v,
              } as Partial<typeof values>);
            } else {
              mark("adapterConfig", "apiKey", v || undefined);
            }
          }}
          immediate
          type="password"
          className={inputClass}
          placeholder="Optional"
        />
      </Field>
    </>
  );
}
