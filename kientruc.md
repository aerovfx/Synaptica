# Phân tích kiến trúc hệ thống Synaptica

## 1. Vai trò tổng quan

**Synaptica** là **control plane** (mặt phẳng điều khiển) cho công ty vận hành bằng AI agent: một instance có thể quản lý **nhiều company**; mỗi company có agents, goals, tasks, budget và governance.

- **Control plane (phần này)**: đăng ký agent, org chart, task, budget, knowledge, heartbeat.
- **Execution**: agents chạy bên ngoài (process, HTTP, OpenClaw, …), "phone home" về control plane.

---

## 2. Kiến trúc runtime (V1)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         PAPERCLIP INSTANCE                               │
├─────────────────────────────────────────────────────────────────────────┤
│  server/           │  REST API, auth, orchestration, adapters           │
│  (Express)         │  Middleware: actor (board/agent), validation,      │
│                    │  board-mutation-guard, private-hostname-guard       │
├────────────────────┼────────────────────────────────────────────────────┤
│  ui/               │  Board operator UI (React + Vite)                   │
│  (React)           │  Company selector, dashboard, org, tasks, costs,   │
│                    │  approvals, activity; dev: Vite middleware         │
├────────────────────┼────────────────────────────────────────────────────┤
│  packages/db/      │  Drizzle schema, migrations, DB client (Postgres)   │
│  packages/shared/  │  Types, validators, constants, API paths            │
└────────────────────┴────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  DATA: PostgreSQL (embedded / Docker / hosted)                          │
│  STORAGE: local_disk (~/.paperclip/...) hoặc S3                          │
│  SECRETS: company_secrets + company_secret_versions (local_encrypted)    │
└─────────────────────────────────────────────────────────────────────────┘
```

- **Một process server** vừa phục vụ API vừa (trong V1) chạy scheduler/worker nhẹ: heartbeat trigger, stuck run, budget threshold, stale task.
- **UI** trong dev được serve qua dev middleware của server (cùng origin với API).

---

## 3. Cấu trúc repo và luồng xử lý

| Thư mục / package | Nội dung chính |
|-------------------|----------------|
| **server/** | Express app, routes, services, adapters, auth, storage, secrets, realtime (WebSocket live events). |
| **ui/** | React SPA: Dashboard, Companies, Org/Agents, Projects, Goals, Issues (Kanban), Approvals, Costs, Activity, Auth, Invite/Join. |
| **packages/db/** | Schema Drizzle (companies, agents, goals, projects, issues, approvals, cost_events, heartbeat_runs, activity_log, secrets, …), migrations, client. |
| **packages/shared/** | Shared types, validators, constants, API path constants. |

**Luồng request điển hình:**

1. HTTP → Express → `actorMiddleware` (board session hoặc agent API key) → `boardMutationGuard` (nếu cần) → route handler.
2. Route gọi **service** (companies, agents, issues, heartbeat, costs, approvals, activity, …).
3. Service dùng **Db** từ `@paperclipai/db` (Drizzle), có thể gọi **secrets**, **storage**, **adapters**.
4. Mutation ghi **activity_log**.
5. Realtime: `publishLiveEvent` → WebSocket cho UI (live updates).

---

## 4. Mô hình dữ liệu (tóm tắt)

- **Tenancy**: multi-company, single-tenant deployment; mọi bản ghi nghiệp vụ gắn `company_id`.
- **companies** → **agents** (reports_to cây), **goals** (parent_id hierarchy), **projects**, **issues** (parent_id, assignee, project, goal).
- **agent_api_keys**: bearer key → 1 agent, 1 company; hash lưu, plaintext chỉ hiện lúc tạo.
- **heartbeat_runs**, **heartbeat_run_events**: lịch/gọi heartbeat, trạng thái run, log.
- **cost_events**: theo agent/issue/project/goal; rollup theo tháng, budget enforcement (soft/hard → auto-pause).
- **approvals**: hire_agent, approve_ceo_strategy; board approve/reject.
- **activity_log**: actor_type (agent | user | system), action, entity_type, entity_id, details.
- **company_secrets** + **company_secret_versions**: secret refs trong adapter config, không lưu plaintext nhạy cảm.

Tất cả truy vấn và mutation đều kiểm tra **company boundary** (company_id, agent thuộc company).

---

## 5. Auth và quyền

- **Board**: session (Better Auth hoặc local_trusted implicit); toàn quyền trong instance, mọi mutation ghi activity_log.
- **Agent**: Bearer API key → đọc org/task/company của company mình; read/write task được assign, comment, report heartbeat/cost; không bypass approval, không sửa budget/keys trực tiếp.
- **Deployment modes**: `local_trusted` (mặc định, không login) và `authenticated` (session, private/public exposure) — chi tiết trong `doc/DEPLOYMENT-MODES.md`.

---

## 6. Adapters (heartbeat / execution)

- **Adapter**: giao diện invoke / status / cancel; config theo agent (process, http, openclaw-gateway, cursor, codex, claude-local, opencode-local, …).
- **Process adapter**: spawn process (command, args, cwd, env, timeout); stream log; cancel = SIGTERM → SIGKILL.
- **HTTP adapter**: gọi webhook/API; 2xx = accepted; có thể callback để cập nhật trạng thái bất đồng bộ.
- **Context**: `thin` (chỉ IDs, agent gọi API lấy context) hoặc `fat` (gửi kèm assignments, goal, budget, comments).
- Scheduler (trong server): theo `adapter_config` (enabled, intervalSec, maxConcurrentRuns); không gọi khi agent paused/terminated, run đang chạy, hoặc vượt hard budget.

---

## 7. API (REST)

- Base path: **`/api`** (trên app mount thêm prefix nếu có).
- Nhóm route chính: **health**, **companies**, **agents** (keys, heartbeat invoke), **projects**, **issues** (checkout/release, comments, attachments), **goals**, **approvals**, **secrets**, **costs**, **activity**, **dashboard**, **sidebar-badges**, **llms**, **access** (invites, join-requests, members, skills, board-claim), **assets**.
- Lỗi: 400, 401, 403, 404, 409 (conflict checkout), 422, 500.

---

## 8. UI (Board)

- **React + Vite**, design system (tokens, typography, status/priority) và design guide trong repo.
- Routes chính: `/`, `/companies`, `/companies/:id/org`, `/companies/:id/tasks`, `/companies/:id/agents/:agentId`, `/companies/:id/costs`, `/companies/:id/approvals`, `/companies/:id/activity`, auth, invite landing, board claim.
- Context: Company, Sidebar, Theme, Breadcrumb, Toast, Dialog, LiveUpdates (WebSocket).
- Adapter config UI: từng loại adapter (process, http, cursor, codex, claude-local, opencode, openclaw-gateway, …) có config fields riêng.

---

## 9. Database và vận hành

- **PostgreSQL**: không set `DATABASE_URL` → embedded Postgres (PGlite / embedded-postgres) tại `~/.paperclip/instances/default/db`.
- Migrations: Drizzle, `pnpm db:generate` / `pnpm db:migrate`; dev có thể tự apply khi khởi động.
- Storage: mặc định `local_disk` tại `~/.paperclip/instances/default/data/storage`.
- Worktree/local instances: `.paperclip/` trong repo, instance tách cho worktree; seed minimal/full/no-seed.

---

## 10. Tóm tắt luồng nghiệp vụ V1

1. Board tạo company, goals, agents (org tree), projects.
2. Agents nhận task qua **checkout** (atomic, single assignee), cập nhật status/comments.
3. **Heartbeat** được trigger bởi scheduler hoặc manual/wakeup; adapter invoke process hoặc HTTP; run được ghi và có thể cancel.
4. **Cost** được báo qua API; rollup theo agent/project/company; budget hard limit → auto-pause agent.
5. **Approvals** (hire, CEO strategy) do board duyệt; activity log ghi mọi mutation.

Toàn bộ thiết kế bám **SPEC-implementation.md** (V1), đồng bộ contract giữa **db** ↔ **shared** ↔ **server** ↔ **ui** và bảo toàn biên company cùng các invariant (single assignee, atomic checkout, approval gates, budget auto-pause, activity logging).

---

## 11. Backend Rust (chuyển đổi dần)

Đã có **scaffold backend Rust** tại `server-rs/` (Axum + SQLx) để chuyển đổi từ Node sang Rust theo từng phase:

- **Phase 1**: `GET /api/health`, `GET /api/companies`.
- **Phase 2**: thêm list goals, projects, agents, issues; get company by id; dashboard; activity. **UI**: React + Vite, build ra `ui/dist`, Rust serve static (một process: API + UI). Chạy: `cd ui && pnpm build && cd ../server-rs && DATABASE_URL=... cargo run` → http://127.0.0.1:3100. Các route ghi (POST/PATCH/DELETE) vẫn port dần — xem `doc/RUST-MIGRATION.md`.
- **Chạy**: `cd server-rs && DATABASE_URL=postgres://... cargo run` — xem chi tiết và kế hoạch phase trong `server-rs/README.md`.
- **Mục tiêu**: từng bước port hết routes, auth, adapters, heartbeat sang Rust rồi tắt server Node.
Kế hoạch chuyển đổi (trong server-rs/README.md)
Phase 1 (hiện tại): health + companies bằng Rust.
Phase 2: thêm goals, projects, agents, issues (list).
Phase 3: auth (session + API key) trong Rust, port nốt CRUD.
Phase 4: adapters, heartbeat, storage/secrets.
Phase 5: tắt Node, chỉ dùng Rust.