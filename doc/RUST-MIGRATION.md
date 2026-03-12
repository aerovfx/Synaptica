# Chuyển đổi Backend sang Rust

## Mục tiêu

- **Backend**: toàn bộ API chạy trên Rust (Axum + SQLx).
- **UI**: giữ React + Vite; build ra static files, Rust serve cùng một process.

## Kiến trúc sau khi chuyển

```
┌─────────────────────────────────────────────────────────┐
│  paperclip-server (Rust binary)                         │
│  - GET/POST/PATCH/DELETE /api/*  → REST API             │
│  - GET /* (không phải /api)     → UI static (ui/dist)  │
│  - Fallback /*                  → index.html (SPA)      │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
              PostgreSQL (cùng schema Drizzle)
```

- **Build UI**: `pnpm build` trong `ui/` → output `ui/dist/`.
- **Chạy**: `cargo run` trong `server-rs/` (cần `DATABASE_URL`); binary đọc static từ `../ui/dist` hoặc env `UI_DIST`.

## Trạng thái endpoint (Rust)

### Đã có (Phase 1–2)

| Method | Path | Ghi chú |
|--------|------|--------|
| GET | /api/health | |
| GET | /api/companies | list |
| GET | /api/companies/:id/goals | list |
| GET | /api/companies/:id/projects | list |
| GET | /api/companies/:id/agents | list |
| GET | /api/companies/:id/issues | list |

### Cần thêm cho chạy UI đầy đủ

- **Companies**: GET /:id, POST /, PATCH /:id, POST /:id/archive, GET /stats
- **Goals**: GET /goals/:id, POST /companies/:id/goals, PATCH /goals/:id, DELETE /goals/:id
- **Projects**: GET /projects/:id, POST /companies/:id/projects, PATCH /projects/:id
- **Agents**: GET /agents/:id, POST /companies/:id/agents, PATCH /agents/:id, POST /agents/:id/pause, resume, GET/POST keys, POST heartbeat/invoke
- **Issues**: GET /issues/:id, POST /companies/:id/issues, PATCH /issues/:id, POST checkout, release, GET/POST comments
- **Dashboard**: GET /companies/:id/dashboard
- **Activity**: GET /companies/:id/activity
- **Approvals**: GET list, GET /:id, POST approve, reject, POST /companies/:id/approvals
- **Costs**: GET summary, by-agent, by-project, PATCH budgets, POST cost-events
- **Secrets**: GET list, POST create (stub nếu cần)
- **Assets**: POST upload, GET content (stub hoặc local file)
- **Access**: invites, join-requests, members (có thể stub cho local_trusted)
- **Sidebar badges**: GET /companies/:id/sidebar-badges
- **LLM**: GET /llms/* (text config) – stub

### Auth (local_trusted)

- V1 Rust: không kiểm tra session; mọi request coi là board. Agent API key (Bearer) có thể thêm sau.

## UI

- **Công nghệ**: React + Vite (giữ nguyên).
- **Build**: `cd ui && pnpm build` → `ui/dist/`.
- **Base URL API**: `/api` (cùng origin khi serve từ Rust).
- **Chạy dev**: có thể chạy `pnpm dev` trong `ui/` với proxy `/api` → `http://localhost:3100` để dev UI với Rust backend.

## Lệnh chạy (Rust backend + UI)

```bash
# 1. Build UI (một lần hoặc sau khi đổi UI)
cd ui && pnpm build && cd ..

# 2. Chạy Rust
cd server-rs
export DATABASE_URL="postgres://..."
cargo run
# Mở http://127.0.0.1:3100 — API + UI cùng origin
```

Rust server tự tìm `../ui/dist` nếu không set `UI_DIST`. Các route POST/PATCH/DELETE (create/update company, issue, agent, v.v.) hiện vẫn cần Node; chỉ GET list/detail và dashboard/activity đã có trên Rust.

## Migration DB

- Schema và migration vẫn do **Drizzle** (Node) quản lý trong `packages/db`.
- Chạy migration: từ repo root, `DATABASE_URL=... pnpm db:migrate`.
- Rust chỉ đọc/ghi bảng qua SQLx, không tạo schema.
