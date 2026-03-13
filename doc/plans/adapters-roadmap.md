# Adapters roadmap

Lộ trình mở rộng adapter types: OpenFang, API các model (Gemini, Grok, GPT), và tham chiếu các plan adapter khác.

## Hiện có (V1)

| Adapter | Mô tả ngắn | Trạng thái |
|---------|------------|------------|
| process | Lệnh tùy ý (subprocess) | Done |
| http | Gọi HTTP endpoint | Done |
| claude_local | Claude (local CLI) | Done |
| codex_local | Codex (local) | Done |
| opencode_local | OpenCode — multi-provider local | Done |
| pi_local | Pi (local) | Done |
| cursor | Cursor (local) | Done |
| openclaw_gateway | OpenClaw qua gateway API | Done |
| openfang_gateway | OpenFang Agent OS — multi-provider (Gemini, Grok, GPT, Claude, …) | Planned / In progress |

Tham chiếu: [SPEC-implementation.md](../SPEC-implementation.md), [cursor-cloud-adapter.md](./cursor-cloud-adapter.md).

---

## Giai đoạn tiếp theo

### 1. OpenFang (openfang_gateway)

- **Mục tiêu:** Adapter gọi OpenFang (gateway hoặc local) — [OpenFang](https://www.openfang.sh) hỗ trợ 20+ LLM (Gemini, Grok, GPT, Claude, DeepSeek, …), REST/WS/SSE, MCP, A2A.
- **Tasks:**
  - [x] Thêm `openfang_gateway` vào `AGENT_ADAPTER_TYPES` (packages/shared)
  - [x] Thêm vào `LEGACY_ADAPTER_TYPES` và runner (server-rs)
  - [x] Package `packages/adapters/openfang-gateway` (execute + test)
  - [x] Đăng ký trong CLI `run-legacy-adapter.ts`
  - [x] UI adapter module + registry + labels + OnboardingWizard + AgentConfigForm
  - [ ] Implement execute theo [OpenFang API Reference](https://www.openfang.sh/docs/api-reference) (run/execute endpoint)
- **Config gợi ý:** baseUrl, apiKey (hoặc header auth), model/provider (tùy chọn).

### 2. API trực tiếp (tùy chọn)

- **Gemini API:** Adapter type `gemini_api` — config API key, model id; gọi Google AI REST API.
- **Grok API:** Adapter type `grok_api` — config API key, model; gọi xAI API.
- **OpenAI / GPT API:** Adapter type `openai_api` hoặc `gpt_api` — config API key, model (GPT-4, GPT-4o hoặc phiên bản tương thích API). Ghi chú: "GPT 5.4" cần xác nhận phiên bản cụ thể.

Có thể triển khai từng adapter riêng hoặc mở rộng OpenCode (opencode_local) thêm provider. Thứ tự ưu tiên theo nhu cầu sản phẩm.

---

## Trạng thái theo adapter

| Adapter | Constant | Runner | Package | CLI | UI | Ghi chú |
|---------|----------|--------|---------|-----|----|---------|
| openfang_gateway | Done | Done | Done | Done | Done | Execute: stub / implement per API docs |
