# Synaptica — Rust backend + UI

Backend API viết bằng Rust (Axum + SQLx), serve luôn UI React (build từ `ui/`). Một process phục vụ cả `/api` và giao diện web.

## Yêu cầu

- Rust 1.75+ (`rustup`).
- PostgreSQL (cùng schema với `packages/db`; migration chạy từ repo gốc: `DATABASE_URL=... pnpm db:migrate`).
- Node/pnpm để build UI (chỉ lúc build, không cần khi chạy).

## Chạy app đầy đủ (API + UI)

```bash
# 1. Build UI (từ repo gốc)
cd ui && pnpm build && cd ..

# 2. Chạy Rust server (từ repo gốc hoặc từ server-rs)
cd server-rs
export DATABASE_URL="postgres://paperclip1:paperclip%23Ceo1@localhost:5432/paperclip1"
cargo run
```

Mở **http://127.0.0.1:3100** — API tại `/api/*`, UI tại `/`. Nếu không set `UI_DIST`, server tự tìm `../ui/dist` (khi chạy từ `server-rs`). Có thể set `UI_DIST=/path/to/ui/dist` để chỉ định thư mục build.

## Chạy chỉ API (không UI)

Bỏ qua bước build UI; không set `UI_DIST`. Chỉ các route `/api` hoạt động. Đổi port/host bằng biến môi trường:

```bash
PORT=3200 HOST=0.0.0.0 cargo run
```

## API hiện có

| Method | Path | Ghi chú |
|--------|------|--------|
| GET | `/api/health` | Health + deployment info |
| GET | `/api/companies` | List companies |
| GET | `/api/companies/:id` | Get company by id |
| GET | `/api/companies/:id/goals` | List goals |
| GET | `/api/companies/:id/projects` | List projects |
| GET | `/api/companies/:id/agents` | List agents |
| GET | `/api/companies/:id/issues` | List issues |
| GET | `/api/companies/:id/dashboard` | Dashboard summary |
| GET | `/api/companies/:id/activity` | Activity log (200 mới nhất) |

JSON camelCase. Cần `DATABASE_URL`; không set thì các route cần DB trả 503. Các route POST/PATCH/DELETE (create company, update issue, v.v.) vẫn do Node cung cấp cho đến khi port thêm sang Rust — xem `doc/RUST-MIGRATION.md`.

## Dùng với UI React

Chạy Rust server với `DATABASE_URL` trỏ tới DB đã migrate. Mở UI (Vite hoặc build) và trỏ base URL API tới `http://127.0.0.1:3100` (hoặc port bạn đặt). Các route khác (agents, issues, …) vẫn do Node phục vụ cho đến khi port sang Rust.

## Kế hoạch chuyển đổi (gợi ý)

1. **Phase 1 (hiện tại)**  
   - Health + companies (Rust).  
   - Còn lại: Node.  
   - Có thể chạy 2 server (Rust cho vài route, Node cho phần còn lại) qua proxy hoặc chọn server theo path.

2. **Phase 2** ✅  
   - Đã có: GET list goals, projects, agents, issues theo `companyId`.  
   - Auth chưa port: coi mọi request là board.

3. **Phase 3**  
   - Auth (board session + agent API key) trong Rust.  
   - Port toàn bộ routes CRUD còn lại.

4. **Phase 4**  
   - Adapters (process, HTTP), heartbeat, scheduler.  
   - Storage, secrets (hoặc gọi service Node tạm).

5. **Phase 5**  
   - Tắt Node; Rust phục vụ toàn bộ API.  
   - UI có thể vẫn build bằng pnpm; chỉ đổi base URL API.

## Cấu trúc thư mục

```
server-rs/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs       # Entry, config, mount /api
    ├── config.rs      # DATABASE_URL, HOST, PORT
    ├── db.rs          # PgPool
    ├── models/        # Struct trùng schema DB, serde camelCase
    │   ├── mod.rs
    │   └── company.rs
    └── routes/
        ├── mod.rs
        ├── health.rs
        ├── companies.rs
        ├── goals.rs
        ├── projects.rs
        ├── agents.rs
        └── issues.rs
```

## Ghi chú

- Schema DB do Drizzle trong `packages/db` quản lý; Rust chỉ đọc/ghi bảng hiện có.  
- JSON response dùng `camelCase` để khớp Node và UI.  
- Chưa có auth: Rust coi mọi request là board, phù hợp dev/local; production cần thêm middleware auth.
