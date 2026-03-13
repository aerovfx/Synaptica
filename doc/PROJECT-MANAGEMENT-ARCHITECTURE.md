# Kiến trúc quản lý dự án (Trello/Jira-style)

Tài liệu mô tả kiến trúc mở rộng cho quản lý dự án kiểu Kanban/Sprint trong phạm vi **Company**, tích hợp với stack hiện tại: **Rust + Axum + PostgreSQL**, và thiết kế sẵn cho **Redis** (cache, pub/sub) và **WebSocket** (realtime).

## 1. Kiến trúc tổng thể

```
        Client (Web / Mobile)
                │
                │ HTTP / WebSocket
                ▼
           Axum Server
                │
    ┌───────────┼───────────┬─────────────────┐
    ▼           ▼           ▼                 ▼
 Auth      Project      Board/Sprint      Task (Issues)
    │           │           │                 │
    └───────────┴───────────┴────────┬────────┘
                                     ▼
                                PostgreSQL
                                     │
                        (optional)   ▼
                                  Redis
                          (cache + pub/sub + session)
```

- Mọi entity (board, sprint, task) đều **scoped theo company**.
- **Project** đã có sẵn; Board/Sprint gắn với project (hoặc company).
- **Task** = Issue hiện tại; bổ sung trường `board_id`, `board_column_id`, `sprint_id`, `position` để dùng trên Kanban.

## 2. Cấu trúc server-rs (Rust)

Cấu trúc hiện tại đã theo hướng modular (mỗi domain một file routes). Mở rộng thêm:

```
server-rs/src/
├── main.rs
├── config.rs
├── db.rs
├── routes/
│   ├── mod.rs
│   ├── boards.rs      # Kanban boards, columns
│   ├── sprints.rs     # Sprints (Agile)
│   ├── projects.rs
│   ├── issues.rs      # + move issue (column + position)
│   └── ...
├── models/
│   ├── board.rs
│   ├── sprint.rs
│   └── ...
└── ...
```

Luồng xử lý: **Router → Handler → (Service) → DB (sqlx)**. Có thể tách Repository/Service khi logic phức tạp hơn.

## 3. Database (PostgreSQL)

### Bảng mới

- **boards**: `id`, `company_id`, `project_id` (nullable), `name`, `type` (kanban, backlog), `created_at`, `updated_at`
- **board_columns**: `id`, `board_id`, `name`, `position` (double, cho thứ tự cột)
- **sprints**: `id`, `board_id`, `name`, `start_date`, `end_date`, `status` (planned, active, completed), `created_at`, `updated_at`

### Mở rộng bảng issues

- `board_id` (uuid, nullable, FK → boards)
- `board_column_id` (uuid, nullable, FK → board_columns)
- `sprint_id` (uuid, nullable, FK → sprints)
- `position` (double, nullable): thứ tự thẻ trong cột (Trello-style fractional indexing)

Khi `board_id`/`board_column_id`/`sprint_id`/`position` = null, issue hoạt động như hiện tại (không gắn Kanban).

## 4. Thuật toán Drag & Drop (Trello-style)

Dùng **fractional position** để tránh reorder toàn bộ list:

- Task A position 1.0, Task B 2.0, Task C 3.0.
- Kéo B lên giữa A và C: `position = (1.0 + 2.0) / 2 = 1.5`.
- Chỉ cập nhật một bản ghi; không cần cập nhật hàng loạt.

## 5. API (Company-scoped)

- `GET/POST /companies/:company_id/boards` — list, create board
- `GET/PATCH/DELETE /boards/:id` — get, update, delete board
- `GET/POST /boards/:id/columns` — list, create column
- `PATCH/DELETE /boards/:board_id/columns/:column_id` — update, delete column
- `GET/POST /companies/:company_id/sprints` hoặc `GET/POST /boards/:board_id/sprints` — list, create sprint
- `GET/PATCH/DELETE /sprints/:id` — get, update, delete sprint
- `PATCH /issues/:id` — hỗ trợ `boardColumnId`, `sprintId`, `position` để move thẻ / gán sprint

Board/Sprint đều kiểm tra `company_id` khi đọc/ghi.

## 6. Realtime (WebSocket)

- Hiện tại đã có **LiveEventBus** và `/companies/:company_id/events/ws`.
- Khi move issue / cập nhật board: emit event (ví dụ `issue_moved`, `board_updated`) qua bus để client cập nhật Kanban realtime.
- Tương lai: Redis Pub/Sub để scale nhiều instance; mỗi instance subscribe và forward vào WebSocket local.

## 7. Redis (tùy chọn)

- **Cache**: GET board (columns + issues) cache trong Redis, TTL ngắn; invalidate khi có PATCH/POST.
- **Pub/Sub**: Khi có thay đổi board/issue, publish message; WebSocket server subscribe và push tới client.
- Cấu hình: `REDIS_URL` (optional); nếu không set thì chạy không Redis (chỉ Postgres + in-memory LiveEventBus).

## 8. Security & Auth

- Giữ nguyên cơ chế hiện tại: board (human) và agent API keys, company-scoped.
- Mọi route board/sprint/issue kiểm tra company và quyền truy cập (board hoặc agent).

## 9. Scaling

- **Horizontal**: Nhiều instance Axum phía sau load balancer; Redis làm session/cache và pub/sub chung.
- **PostgreSQL**: Connection pooling (sqlx pool đã dùng); đọc replica nếu cần.

## 10. Tham chiếu

- `doc/SPEC-implementation.md` — V1 scope và data model.
- `doc/DATABASE.md` — Cấu hình Postgres và migration.
- `server-rs/src/routes/` — REST API hiện tại.
- `packages/db/src/schema/` — Drizzle schema và migration.
