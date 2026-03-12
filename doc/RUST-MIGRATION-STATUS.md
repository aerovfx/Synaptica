# Đánh giá tiến trình chuyển đổi Node → Rust

**Trạng thái:** Đã loại bỏ thư mục `server/` (Node). Backend chạy hoàn toàn trên Rust (`server-rs/`). `pnpm dev` và `paperclipai run` khởi động Rust server. Cấu hình backend (pool DB, scheduler) theo env — xem `doc/RUST-BACKEND-OPTIMIZATION-PLAN.md` (Phase 1 đã áp dụng).

## Tổng quan

| Hạng mục | Trạng thái | Ghi chú |
|----------|------------|--------|
| **API đọc (GET)** | ~95% trên Rust | GET theo id cho goal, project, agent, issue, approval đã có |
| **API ghi (POST/PATCH/DELETE)** | ~87% trên Rust | + secrets, assets, invites, members, join-requests, attachments, export/import, heartbeat, config-revisions, runtime-state, task-sessions, invoke (stub), skills, board-claim |
| **Auth** | Đã có (Rust) | Session stub (get-session), Bearer agent API key middleware, board-mutation-guard (RequireBoard) |
| **UI** | 100% Rust serve | Build từ React+Vite, Rust serve static |
| **Heartbeat / Adapters** | API + scheduler + runner | wakeup/invoke tạo run + chạy adapter (process/http); runs list/get/events/cancel; scheduler (timer) |
| **Tiến độ tổng thể** | **~88%** | Rust: + heartbeat (wakeup, runs, events, cancel, scheduler) |

---

## Chi tiết endpoint

### Đã có trên Rust ✅

| Method | Path | Ghi chú |
|--------|------|--------|
| GET | /api/health | |
| GET | /api/companies | |
| GET | /api/companies/:id | |
| GET | /api/companies/:id/goals | list |
| GET | /api/companies/:id/projects | list |
| GET | /api/companies/:id/agents | list |
| GET | /api/companies/:id/issues | list |
| GET | /api/issues/:id | by id |
| GET | /api/companies/:id/dashboard | |
| GET | /api/companies/:id/activity | |
| POST | /api/companies | create (issue_prefix auto) |
| PATCH | /api/companies/:id | update name, description, status |
| POST | /api/companies/:id/archive | archive company |
| DELETE | /api/companies/:id | delete company |
| GET | /api/companies/:id/stats | thống kê (agents, projects, goals, issues, approvals, spend) |
| GET | /api/companies/:id/export | export JSON (company + goals, projects, agents, issues) |
| POST | /api/companies/import | import (create company từ JSON) |
| GET | /api/companies/:id/goals | list |
| GET | /api/goals/:id | by id |
| POST | /api/companies/:id/goals | create |
| PATCH | /api/goals/:id | update |
| DELETE | /api/goals/:id | delete |
| GET | /api/companies/:id/projects | list |
| GET | /api/projects/:id | by id |
| POST | /api/companies/:id/projects | create |
| PATCH | /api/projects/:id | update |
| DELETE | /api/projects/:id | delete project |
| GET | /api/projects/:id/workspaces | list workspaces |
| POST | /api/projects/:id/workspaces | create workspace |
| GET | /api/projects/:id/workspaces/:workspace_id | get workspace |
| PATCH | /api/projects/:id/workspaces/:workspace_id | update workspace |
| DELETE | /api/projects/:id/workspaces/:workspace_id | delete workspace |
| GET | /api/companies/:id/agents | list |
| GET | /api/agents/me | identity (header X-Agent-Id cho đến khi có auth) |
| GET | /api/agents/:id | by id |
| POST | /api/companies/:id/agents | create |
| PATCH | /api/agents/:id | update |
| POST | /api/agents/:id/pause | set status paused |
| POST | /api/agents/:id/resume | set status idle |
| POST | /api/agents/:id/terminate | set status terminated |
| GET | /api/agents/:id/keys | list API keys |
| POST | /api/agents/:id/keys | create key (trả key 1 lần) |
| DELETE | /api/agents/:id/keys/:key_id | revoke key |
| POST | /api/agents/:id/heartbeat | cập nhật last_heartbeat_at |
| GET | /api/agents/:id/config-revisions | list revisions |
| GET | /api/agents/:id/runtime-state | get state |
| PATCH | /api/agents/:id/runtime-state | upsert state |
| GET | /api/agents/:id/task-sessions | list sessions |
| POST | /api/agents/:id/invoke | tạo run + chạy adapter (process/http), trả run |
| POST | /api/companies/:id/issues | create |
| PATCH | /api/issues/:id | update |
| POST | /api/issues/:id/checkout | claim + start (X-Agent-Id hoặc body) |
| POST | /api/issues/:id/release | release task |
| GET | /api/issues/:id/comments | list comments |
| POST | /api/issues/:id/comments | add comment |
| GET | /api/issues/:id/approvals | list approvals linked to issue |
| POST | /api/issues/:id/approvals | link approval (body: approvalId) |
| DELETE | /api/issues/:id/approvals/:approval_id | unlink |
| GET | /api/issues/:id/attachments | list attachments |
| POST | /api/issues/:id/attachments | link asset (body: assetId) |
| DELETE | /api/issues/:id/attachments/:attachment_id | unlink |
| GET | /api/companies/:id/approvals | list |
| POST | /api/companies/:id/approvals | create approval |
| GET | /api/approvals/:id | by id |
| POST | /api/approvals/:id/approve | approve |
| POST | /api/approvals/:id/reject | reject |
| POST | /api/approvals/:id/request-revision | yêu cầu sửa |
| POST | /api/approvals/:id/resubmit | resubmit (optional payload) |
| GET | /api/approvals/:id/comments | list comments |
| POST | /api/approvals/:id/comments | add comment |
| GET | /api/approvals/:id/issues | issues linked to approval |
| POST | /api/companies/:id/cost-events | create |
| GET | /api/companies/:id/costs/summary | summary |
| GET | /api/companies/:id/costs/by-agent | by agent |
| GET | /api/companies/:id/costs/by-project | by project |
| PATCH | /api/companies/:id | update (gồm budget_monthly_cents) |
| GET | /api/companies/:id/secrets | list secrets |
| POST | /api/companies/:id/secrets | create secret |
| GET | /api/secrets/:id | get secret (metadata) |
| PATCH | /api/secrets/:id | update description |
| POST | /api/secrets/:id/rotate | tạo version mới |
| DELETE | /api/secrets/:id | delete secret |
| GET | /api/companies/:id/assets | list assets |
| POST | /api/companies/:id/assets | create (body: contentBase64, contentType, originalFilename) |
| GET | /api/assets/:id | get asset metadata |
| GET | /api/assets/:id/content | tải file (cần ASSETS_PATH) |
| DELETE | /api/assets/:id | delete asset |
| GET | /api/companies/:id/invites | list invites |
| POST | /api/companies/:id/invites | create (trả token 1 lần) |
| GET | /api/invites/:token | get invite by token |
| GET | /api/companies/:id/members | list memberships |
| GET | /api/companies/:id/join-requests | list join requests |
| GET | /api/join-requests/:id | get join request |
| GET | /api/companies/:id/sidebar-badges | pending approvals + open issues |
| GET | /api/llm-config | text từ PAPERCLIP_LLM_CONFIG |
| GET | /api/skills/index | list skill ids (static) |
| GET | /api/skills/:id | nội dung SKILL.md (SKILLS_DIR) |
| POST | /api/board/claim | stub (local_trusted) |
| GET | /api/auth/get-session | board session (local_trusted → null) |
| POST | /api/agents/:id/wakeup | tạo run queued + spawn runner (process/http) |
| GET | /api/companies/:id/heartbeat-runs | list runs (agentId, limit) |
| GET | /api/heartbeat-runs/:id | get run |
| GET | /api/heartbeat-runs/:id/events | run log (afterSeq, limit) |
| GET | /api/heartbeat-runs/:id/log | log stub (content rỗng nếu chưa lưu) |
| POST | /api/heartbeat-runs/:id/cancel | set status cancelled |

### Đã có trên Rust (Auth) ✅

| Tính năng | Ghi chú |
|-----------|--------|
| GET /api/auth/get-session | Trả `{ data: null }` (local_trusted); UI không cần login |
| Agent API key | `Authorization: Bearer <key>` → hash, tra `agent_api_keys`, set Actor::Agent; invalid → 401 |
| Board-mutation-guard | Extractor `RequireBoard`: route chỉ board (approve/reject, archive/delete company, keys, pause/resume/terminate agent) → agent gọi trả 403 |

### Chưa có / tùy chọn

- Session thật (Better Auth / cookie) khi chạy authenticated mode: get-session hiện luôn trả null.

### Chưa port (phức tạp, có thể làm sau)

- **Adapters**
  - **Trên Rust** (`server-rs/src/runner.rs`): `process`, `http` — chạy khi wakeup/invoke.
  - **Legacy (Node bridge):** khi set `PAPERCLIP_PROJECT_ROOT` (đường dẫn repo), các type `claude_local`, `codex_local`, `cursor`, `openclaw_gateway`, `opencode_local`, `pi_local` chạy qua script `cli/src/run-legacy-adapter.ts` (pnpm exec tsx).
  - **Chưa port thuần Rust:** code trong `packages/adapters/*` vẫn là Node; có thể port từng adapter sang Rust sau.
- **Storage**: local_disk, Cloud run
- **Secrets**: local_encrypted, provider registry
- **Realtime**: WebSocket live events
- **Embedded Postgres**: khởi động PGlite/embedded khi không có DATABASE_URL

### Tham khảo: OpenFang

**[OpenFang](https://github.com/RightNow-AI/openfang)** — Open-source Agent Operating System viết hoàn toàn bằng Rust ([RightNow-AI/openfang](https://github.com/RightNow-AI/openfang), Apache-2.0 / MIT). Tham khảo khi port thêm adapters hoặc thiết kế runner:

- **Kiến trúc:** 14 crates (~137k LOC), một binary ~32MB. Crate liên quan adapters: `openfang-channels` (40 channel adapters).
- **Channel adapters:** 40 adapters (Telegram, Discord, Slack, WhatsApp, Teams, IRC, Matrix, webhooks, …) — trait/channel thống nhất, rate limiting, DM/group policies.
- **Docs:** [openfang.sh/docs](https://www.openfang.sh/docs) — [Architecture](https://www.openfang.sh/docs/architecture), [Channel Adapters](https://www.openfang.sh/docs/channel-adapters), Configuration, Security.

Áp dụng cho tình huống hiện tại: Rust (server-rs) chỉ chạy **process** và **http**; các adapter phức tạp (claude-local, cursor, codex, openclaw-gateway, …) vẫn ở Node (packages/adapters). Có thể xem cách OpenFang tổ chức `openfang-channels` khi quyết định port từng adapter hoặc giữ gọi ngoại vi (subprocess/HTTP).

---

## Đã loại bỏ / đã cập nhật

| Thành phần | Trạng thái |
|------------|------------|
| **server/** | Đã xóa |
| **scripts/dev-runner.mjs** | Đã sửa: gọi `cargo run` trong server-rs, set UI_DIST |
| **pnpm-workspace.yaml** | Đã bỏ `server` |
| **package.json** | dev chạy dev-runner (Rust); thêm dev:build-ui; bỏ dev:server |
| **cli** (run command) | Đã sửa: spawn Rust (`cargo run`), bỏ dependency @paperclipai/server |
| **AGENTS.md / DEVELOPING.md** | Đã cập nhật theo Rust |

Giữ nguyên: **packages/db**, **packages/shared**, **ui/**, **packages/adapters/** (code giữ lại, chưa dùng cho đến khi có adapter runtime trên Rust).

---

## Kế hoạch loại bỏ Node (khi đã thay thế đủ)

1. **Phase A — Chỉ đọc + UI trên Rust (hiện tại)**  
   - Chạy: build UI + `cargo run` trong server-rs.  
   - Node vẫn cần nếu muốn tạo/sửa company, issue, agent, approval, cost.

2. **Phase B — Port tối thiểu để bỏ Node**  
   - Thêm trên Rust: POST/PATCH companies, goals, projects, agents; POST/PATCH issues, checkout, release, comments; GET/POST approvals (list, approve, reject); GET/POST costs (summary, cost-events).  
   - Auth: local_trusted = không kiểm tra; optional: Bearer API key cho agent.  
   - Sau Phase B: có thể xóa **server/** và chuyển dev/CLI sang Rust.

3. **Phase C — Tùy chọn**  
   - Port heartbeat + adapters sang Rust → có thể xóa hoặc archive **packages/adapters** hoặc giữ để tham chiếu.  
   - Port embedded Postgres, storage, secrets, WebSocket nếu cần.

---

## Lệnh sau khi đã loại bỏ Node

- **Dev**: `pnpm dev` → (1) chạy migration nếu cần, (2) build UI nếu cần, (3) `cd server-rs && cargo run`.  
- **CLI run**: `paperclipai run` → spawn Rust binary (server-rs) thay vì import Node server.  
- **Migration DB**: vẫn `DATABASE_URL=... pnpm db:migrate` (Drizzle trong packages/db).
- **Legacy adapters** (claude_local, cursor, codex, …): set `PAPERCLIP_PROJECT_ROOT` = đường dẫn tuyệt đối tới repo (chứa `cli/`, `packages/adapters/`) để Rust gọi được `cli/src/run-legacy-adapter.ts`.
