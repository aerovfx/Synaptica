# AGENTS.md

Guidance for human and AI contributors working in this repository.

## 1. Purpose

Synaptica is a control plane for AI-agent companies.
The current implementation target is V1 and is defined in `doc/SPEC-implementation.md`.

## 2. Read This First

Before making changes, read in this order:

1. `doc/GOAL.md`
2. `doc/PRODUCT.md`
3. `doc/SPEC-implementation.md`
4. `doc/DEVELOPING.md`
5. `doc/DATABASE.md`

`doc/SPEC.md` is long-horizon product context.
`doc/SPEC-implementation.md` is the concrete V1 build contract.

## 3. Repo Map

- `server-rs/`: REST API (Rust, Axum + SQLx) and static UI serving
- `ui/`: React + Vite board UI (build output served by server-rs)
- `packages/db/`: Drizzle schema, migrations, DB clients
- `packages/shared/`: shared types, constants, validators, API path constants
- `doc/`: operational and product docs
- `skills/`: Agent skills (injected into Cursor/Codex/Claude/Pi when running via adapters). Includes:
  - Skills: `paperclip`, `paperclip-create-agent`, `create-agent-adapter`, `release`, `release-changelog`, `pr-report`, `para-memory-files`
  - Skills from [aerovfx/skills](https://github.com/aerovfx/skills) (prefix `aerovfx-`): algorithmic-art, brand-guidelines, canvas-design, claude-api, doc-coauthoring, docx, frontend-design, internal-comms, mcp-builder, pdf, pptx, skill-creator, slack-gif-creator, theme-factory, web-artifacts-builder, webapp-testing, xlsx. To refresh: clone that repo and copy `skills/*` into `skills/aerovfx-<name>`.

## 4. Dev Setup

Backend là Rust (`server-rs/`). Cần PostgreSQL: set `DATABASE_URL` hoặc dùng embedded (xem `doc/DATABASE.md`).

```sh
pnpm install
pnpm dev:build-ui   # một lần hoặc khi đổi UI
pnpm dev
```

`pnpm dev` chạy Rust server (script gọi `cargo run` trong server-rs). API + UI tại `http://localhost:3100`. Build UI trước nếu cần giao diện: `pnpm dev:build-ui`.

Quick checks:

```sh
curl http://localhost:3100/api/health
curl http://localhost:3100/api/companies
```

Reset local dev DB (khi dùng embedded): xem `doc/DATABASE.md`. Khi dùng Postgres bên ngoài: dùng `pnpm db:migrate` với `DATABASE_URL` tương ứng.

## 5. Core Engineering Rules

1. Keep changes company-scoped.
Every domain entity should be scoped to a company and company boundaries must be enforced in routes/services.

2. Keep contracts synchronized.
If you change schema/API behavior, update all impacted layers:
- `packages/db` schema and exports
- `packages/shared` types/constants/validators
- `server-rs` routes/handlers
- `ui` API clients and pages

3. Preserve control-plane invariants.
- Single-assignee task model
- Atomic issue checkout semantics
- Approval gates for governed actions
- Budget hard-stop auto-pause behavior
- Activity logging for mutating actions

4. Do not replace strategic docs wholesale unless asked.
Prefer additive updates. Keep `doc/SPEC.md` and `doc/SPEC-implementation.md` aligned.

## 6. Database Change Workflow

When changing data model:

1. Edit `packages/db/src/schema/*.ts`
2. Ensure new tables are exported from `packages/db/src/schema/index.ts`
3. Generate migration:

```sh
pnpm db:generate
```

4. Validate compile:

```sh
pnpm -r typecheck
```

Notes:
- `packages/db/drizzle.config.ts` reads compiled schema from `dist/schema/*.js`
- `pnpm db:generate` compiles `packages/db` first

## 7. Verification Before Hand-off

Run this full check before claiming done:

```sh
pnpm -r typecheck
pnpm test:run
pnpm build
```

If anything cannot be run, explicitly report what was not run and why.

## 8. API and Auth Expectations

- Base path: `/api`
- Board access is treated as full-control operator context
- Agent access uses bearer API keys (`agent_api_keys`), hashed at rest
- Agent keys must not access other companies (enforced by middleware for `/api/companies/:company_id/*`; see `doc/SECURITY.md`)

When adding endpoints:

- apply company access checks
- enforce actor permissions (board vs agent)
- write activity log entries for mutations
- return consistent HTTP errors (`400/401/403/404/409/422/500`)

## 9. UI Expectations

- Keep routes and nav aligned with available API surface
- Use company selection context for company-scoped pages
- Surface failures clearly; do not silently ignore API errors

## 10. Definition of Done

A change is done when all are true:

1. Behavior matches `doc/SPEC-implementation.md`
2. Typecheck, tests, and build pass
3. Contracts are synced across db/shared/server/ui
4. Docs updated when behavior or commands change
