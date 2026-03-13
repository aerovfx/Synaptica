# Developing

This project can run fully in local dev without setting up PostgreSQL manually.

## Deployment Modes

For mode definitions and intended CLI behavior, see `doc/DEPLOYMENT-MODES.md`.

Current implementation status:

- canonical model: `local_trusted` and `authenticated` (with `private/public` exposure)

## Prerequisites

- Node.js 20+
- pnpm 9+
- Rust 1.75+ (for backend; `rustup` recommended)
- PostgreSQL (or embedded; see `doc/DATABASE.md`)

## Optimizing disk space (Node modules)

To reduce disk usage:

1. **Remove unused dependencies** — Root no longer keeps `cross-env` or `esbuild` (esbuild lives under `cli` as it is only used for the CLI build). Run `npx depcheck` at repo root or in a package to find other unused deps; remove only after confirming they are not used (e.g. in config or dynamic imports).
2. **Prune the pnpm store** — After removing deps, run `pnpm store prune` to remove packages from the store that are no longer referenced by any project. Uses the store at `pnpm store path` (often `~/.local/share/pnpm/store` or `~/Library/pnpm/store` on macOS).
3. **Clean install** — From repo root, `rm -rf node_modules && pnpm install` to get a minimal `node_modules` tree (pnpm already deduplicates via the store).

## Dependency Lockfile Policy

GitHub Actions owns `pnpm-lock.yaml`.

- Do not commit `pnpm-lock.yaml` in pull requests.
- Pull request CI validates dependency resolution when manifests change.
- Pushes to `master` regenerate `pnpm-lock.yaml` with `pnpm install --lockfile-only --no-frozen-lockfile`, commit it back if needed, and then run verification with `--frozen-lockfile`.

## Start Dev

From repo root:

```sh
pnpm install
pnpm dev:build-ui   # once, or when UI changes (builds ui/dist for Rust to serve)
pnpm dev
```

This starts the **Rust backend** (`server-rs/`) via `cargo run`. API + UI at `http://localhost:3100`. UI is served from `ui/dist` (build with `pnpm dev:build-ui`). Set `DATABASE_URL` for Postgres; see `doc/DATABASE.md` for embedded option.

Use `pnpm dev:once` to run without migration prompt.

### Environment variables (server-rs)

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `127.0.0.1` | Bind host |
| `PORT` | `3100` | Bind port |
| `DATABASE_URL` | — | PostgreSQL connection string (required for full API) |
| `UI_DIST` | `../ui/dist` if present | Directory containing built UI (`index.html` + assets) |
| `DB_POOL_MAX_SIZE` | `10` | Max connections in the PostgreSQL pool |
| `DB_POOL_ACQUIRE_TIMEOUT_SECS` | `5` | Timeout (seconds) when acquiring a connection from the pool |
| `DB_POOL_IDLE_TIMEOUT_SECS` | — | Idle timeout (seconds) for connections returned to the pool; unset = driver default |
| `SCHEDULER_INTERVAL_SECS` | `60` | Heartbeat scheduler tick interval (seconds); recommended 30–120 |
| `HTTP_BODY_MAX_BYTES` | `2097152` (2 MiB) | Max request body size (bytes) |
| `RUNNER_MAX_CONCURRENT_RUNS` | `0` | Max concurrent adapter runs (0 = unlimited) |
| `RUNNER_HTTP_MAX_TIMEOUT_MS` | `300000` (5 min) | Cap for HTTP adapter timeout (ms) |
| `RUNNER_PROCESS_MAX_TIMEOUT_SECS` | `86400` (24 h) | Cap for process adapter timeout (sec) |
| `CORS_ORIGINS` | — | Comma-separated allowed origins (e.g. `https://app.example.com`); empty/unset = allow any |
| `CONFIG_FILE` | — | Path to JSON config file; keys = env names (e.g. `PORT`, `HOST`), values = string or number; env overrides file |
| `RUST_LOG` | `info,tower_http=debug` | Log level (e.g. `info`, `debug`, `warn`) |

**Metrics:** `GET /api/metrics` returns Prometheus exposition format (counters: `paperclip_http_requests_total`, `paperclip_http_errors_total`; gauge: `paperclip_runner_active_runs` when DB is set).

For Tailscale/private auth, set env (e.g. `PAPERCLIP_DEPLOYMENT_MODE=authenticated`) before `pnpm dev`.

Allow additional private hostnames (for example custom Tailscale hostnames):

```sh
pnpm paperclipai allowed-hostname dotta-macbook-pro
```

## One-Command Local Run

For a first-time local install, you can bootstrap and run in one command:

```sh
pnpm paperclipai run
```

`paperclipai run` does:

1. auto-onboard if config is missing
2. `paperclipai doctor` with repair enabled
3. starts the server when checks pass

## Docker Quickstart (No local Node install)

Build and run Synaptica in Docker:

```sh
docker build -t paperclip-local .
docker run --name paperclip \
  -p 3100:3100 \
  -e HOST=0.0.0.0 \
  -e PAPERCLIP_HOME=/paperclip \
  -v "$(pwd)/data/docker-paperclip:/paperclip" \
  paperclip-local
```

Or use Compose:

```sh
docker compose -f docker-compose.quickstart.yml up --build
```

See `doc/DOCKER.md` for API key wiring (`OPENAI_API_KEY` / `ANTHROPIC_API_KEY`) and persistence details.

## Database in Dev (Auto-Handled)

For local development, leave `DATABASE_URL` unset.
The server will automatically use embedded PostgreSQL and persist data at:

- `~/.paperclip/instances/default/db`

Override home and instance:

```sh
PAPERCLIP_HOME=/custom/path PAPERCLIP_INSTANCE_ID=dev pnpm paperclipai run
```

No Docker or external database is required for this mode.

## Storage in Dev (Auto-Handled)

For local development, the default storage provider is `local_disk`, which persists uploaded images/attachments at:

- `~/.paperclip/instances/default/data/storage`

Configure storage provider/settings:

```sh
pnpm paperclipai configure --section storage
```

## Default Agent Workspaces

When a local agent run has no resolved project/session workspace, Synaptica falls back to an agent home workspace under the instance root:

- `~/.paperclip/instances/default/workspaces/<agent-id>`

This path honors `PAPERCLIP_HOME` and `PAPERCLIP_INSTANCE_ID` in non-default setups.

## Worktree-local Instances

When developing from multiple git worktrees, do not point two Synaptica servers at the same embedded PostgreSQL data directory.

Instead, create a repo-local Synaptica config plus an isolated instance for the worktree:

```sh
paperclipai worktree init
# or create the git worktree and initialize it in one step:
pnpm paperclipai worktree:make paperclip-pr-432
```

This command:

- writes repo-local files at `.paperclip/config.json` and `.paperclip/.env`
- creates an isolated instance under `~/.paperclip-worktrees/instances/<worktree-id>/`
- when run inside a linked git worktree, mirrors the effective git hooks into that worktree's private git dir
- picks a free app port and embedded PostgreSQL port
- by default seeds the isolated DB in `minimal` mode from your main instance via a logical SQL snapshot

Seed modes:

- `minimal` keeps core app state like companies, projects, issues, comments, approvals, and auth state, preserves schema for all tables, but omits row data from heavy operational history such as heartbeat runs, wake requests, activity logs, runtime services, and agent session state
- `full` makes a full logical clone of the source instance
- `--no-seed` creates an empty isolated instance

After `worktree init`, both the server and the CLI auto-load the repo-local `.paperclip/.env` when run inside that worktree, so normal commands like `pnpm dev`, `paperclipai doctor`, and `paperclipai db:backup` stay scoped to the worktree instance.

That repo-local env also sets `PAPERCLIP_IN_WORKTREE=true`, which the server can use for worktree-specific UI behavior such as an alternate favicon.

Print shell exports explicitly when needed:

```sh
paperclipai worktree env
# or:
eval "$(paperclipai worktree env)"
```

Useful options:

```sh
paperclipai worktree init --no-seed
paperclipai worktree init --seed-mode minimal
paperclipai worktree init --seed-mode full
paperclipai worktree init --from-instance default
paperclipai worktree init --from-data-dir ~/.paperclip
paperclipai worktree init --force
```

For project execution worktrees, Synaptica can also run a project-defined provision command after it creates or reuses an isolated git worktree. Configure this on the project's execution workspace policy (`workspaceStrategy.provisionCommand`). The command runs inside the derived worktree and receives `PAPERCLIP_WORKSPACE_*`, `PAPERCLIP_PROJECT_ID`, `PAPERCLIP_AGENT_ID`, and `PAPERCLIP_ISSUE_*` environment variables so each repo can bootstrap itself however it wants.

## Quick Health Checks

In another terminal:

```sh
curl http://localhost:3100/api/health
curl http://localhost:3100/api/companies
```

Expected:

- `/api/health` returns `{"status":"ok"}`
- `/api/companies` returns a JSON array

## Reset Local Dev Database

To wipe local dev data and start fresh:

```sh
rm -rf ~/.paperclip/instances/default/db
pnpm dev
```

## Optional: Use External Postgres

If you set `DATABASE_URL`, the server will use that instead of embedded PostgreSQL.

## Automatic DB Backups

Synaptica can run automatic DB backups on a timer. Defaults:

- enabled
- every 60 minutes
- retain 30 days
- backup dir: `~/.paperclip/instances/default/data/backups`

Configure these in:

```sh
pnpm paperclipai configure --section database
```

Run a one-off backup manually:

```sh
pnpm paperclipai db:backup
# or:
pnpm db:backup
```

Environment overrides:

- `PAPERCLIP_DB_BACKUP_ENABLED=true|false`
- `PAPERCLIP_DB_BACKUP_INTERVAL_MINUTES=<minutes>`
- `PAPERCLIP_DB_BACKUP_RETENTION_DAYS=<days>`
- `PAPERCLIP_DB_BACKUP_DIR=/absolute/or/~/path`

## Secrets in Dev

Agent env vars now support secret references. By default, secret values are stored with local encryption and only secret refs are persisted in agent config.

- Default local key path: `~/.paperclip/instances/default/secrets/master.key`
- Override key material directly: `PAPERCLIP_SECRETS_MASTER_KEY`
- Override key file path: `PAPERCLIP_SECRETS_MASTER_KEY_FILE`

Strict mode (recommended outside local trusted machines):

```sh
PAPERCLIP_SECRETS_STRICT_MODE=true
```

When strict mode is enabled, sensitive env keys (for example `*_API_KEY`, `*_TOKEN`, `*_SECRET`) must use secret references instead of inline plain values.

CLI configuration support:

- `pnpm paperclipai onboard` writes a default `secrets` config section (`local_encrypted`, strict mode off, key file path set) and creates a local key file when needed.
- `pnpm paperclipai configure --section secrets` lets you update provider/strict mode/key path and creates the local key file when needed.
- `pnpm paperclipai doctor` validates secrets adapter configuration and can create a missing local key file with `--repair`.

Migration helper for existing inline env secrets:

```sh
pnpm secrets:migrate-inline-env         # dry run
pnpm secrets:migrate-inline-env --apply # apply migration
```

## Company Deletion Toggle

Company deletion is intended as a dev/debug capability and can be disabled at runtime:

```sh
PAPERCLIP_ENABLE_COMPANY_DELETION=false
```

Default behavior:

- `local_trusted`: enabled
- `authenticated`: disabled

## CLI Client Operations

Synaptica CLI now includes client-side control-plane commands in addition to setup commands.

Quick examples:

```sh
pnpm paperclipai issue list --company-id <company-id>
pnpm paperclipai issue create --company-id <company-id> --title "Investigate checkout conflict"
pnpm paperclipai issue update <issue-id> --status in_progress --comment "Started triage"
```

Set defaults once with context profiles:

```sh
pnpm paperclipai context set --api-base http://localhost:3100 --company-id <company-id>
```

Then run commands without repeating flags:

```sh
pnpm paperclipai issue list
pnpm paperclipai dashboard get
```

See full command reference in `doc/CLI.md`.

## How agent skills are loaded

Skills are instructions and resources (each skill = one folder under `skills/` with a `SKILL.md`) that agents use when running. They are **injected automatically** by the adapter when a heartbeat/run starts, or **installed manually** via the CLI.

### 1. Automatic injection (heartbeat runs)

When Synaptica starts an agent run via a **local adapter** (Cursor, Codex, Claude Code, Pi, OpenCode), the adapter:

1. Resolves the repo **skills directory**: `packages/adapters/<adapter>/skills` (published) or repo root **`skills/`** (dev).
2. For each subfolder (e.g. `paperclip`, `aerovfx-frontend-design`), creates a **symlink** in the agent runtime’s skills directory if it doesn’t already exist.
3. The agent runtime (Cursor/Codex/Claude/Pi) then discovers and loads those skills from its own config dir; the run’s `cwd` is never modified.

| Adapter        | Skills target (symlink destination)     |
|----------------|-----------------------------------------|
| cursor-local   | `~/.cursor/skills/<skill-name>`         |
| codex-local    | `$CODEX_HOME/skills` (default `~/.codex/skills`) |
| claude-local   | Temp dir `/.claude/skills/` passed to Claude via `--add-dir` |
| pi-local       | `~/.pi/agent/skills/<skill-name>`       |
| opencode-local | `~/.claude/skills/<skill-name>`         |

So **any folder you add under repo `skills/`** (e.g. `skills/my-skill/SKILL.md`) is automatically offered to agents on the next run for that adapter; no server restart needed.

### 2. Manual install (CLI, no heartbeat)

For local use without the server (e.g. “run agent from terminal with Synaptica env”):

```sh
pnpm paperclipai agent local-cli <agent-id-or-shortname> --company-id <company-id>
```

This command:

- Creates an API key for the agent (or reuses an existing one).
- If `--install-skills` is set (default): symlinks repo `skills/` into **Codex** and **Claude** skills dirs (`~/.codex/skills`, `~/.claude/skills`).
- Prints the `PAPERCLIP_*` env vars to export so you can run the agent in a shell/IDE with Synaptica context.

Use `--no-install-skills` to skip skill install and only get the env vars.

### 3. API (read-only)

The server exposes skills for onboarding/docs, not for “loading” into the agent at runtime:

- `GET /api/skills/index` — list skill ids and display names.
- `GET /api/skills/:id` — raw markdown of `skills/<id>/SKILL.md` (uses `SKILLS_DIR` or default `skills/`).

Agents that already have skills injected (via §1 or §2) use their **local** skill copy; the API is for invites, onboarding text, or tooling that needs to show skill content.

### Summary

| Cách load skill vào agent | Khi nào dùng |
|---------------------------|--------------|
| **Tự động (symlink)**     | Mỗi lần chạy agent qua adapter (heartbeat); adapter symlink `skills/` → thư mục skills của Cursor/Codex/Claude/Pi. |
| **CLI `agent local-cli`** | Chạy agent tay (terminal/IDE); cần skill cho Codex/Claude thì dùng lệnh này để install skill vào `~/.codex/skills` và `~/.claude/skills`. |
| **API `/api/skills/:id`** | Chỉ đọc nội dung skill (markdown) cho onboarding/tài liệu; không “cài” skill vào agent. |

Thêm skill mới: tạo thư mục `skills/<tên-skill>/` với `SKILL.md` (frontmatter `name`, `description` + nội dung). Lần chạy agent tiếp theo (hoặc sau `agent local-cli` với `--install-skills`) sẽ có skill đó.

## OpenClaw Invite Onboarding Endpoints

Agent-oriented invite onboarding now exposes machine-readable API docs:

- `GET /api/invites/:token` returns invite summary plus onboarding and skills index links.
- `GET /api/invites/:token/onboarding` returns onboarding manifest details (registration endpoint, claim endpoint template, skill install hints).
- `GET /api/invites/:token/onboarding.txt` returns a plain-text onboarding doc intended for both human operators and agents (llm.txt-style handoff), including optional inviter message and suggested network host candidates.
- `GET /api/skills/index` lists available skill documents.
- `GET /api/skills/paperclip` returns the Synaptica heartbeat skill markdown.

## OpenClaw Join Smoke Test

Run the end-to-end OpenClaw join smoke harness:

```sh
pnpm smoke:openclaw-join
```

What it validates:

- invite creation for agent-only join
- agent join request using `adapterType=openclaw`
- board approval + one-time API key claim semantics
- callback delivery on wakeup to a dockerized OpenClaw-style webhook receiver

Required permissions:

- This script performs board-governed actions (create invite, approve join, wakeup another agent).
- In authenticated mode, run with board auth via `PAPERCLIP_AUTH_HEADER` or `PAPERCLIP_COOKIE`.

Optional auth flags (for authenticated mode):

- `PAPERCLIP_AUTH_HEADER` (for example `Bearer ...`)
- `PAPERCLIP_COOKIE` (session cookie header value)

## OpenClaw Docker UI One-Command Script

To boot OpenClaw in Docker and print a host-browser dashboard URL in one command:

```sh
pnpm smoke:openclaw-docker-ui
```

This script lives at `scripts/smoke/openclaw-docker-ui.sh` and automates clone/build/config/start for Compose-based local OpenClaw UI testing.

Pairing behavior for this smoke script:

- default `OPENCLAW_DISABLE_DEVICE_AUTH=1` (no Control UI pairing prompt for local smoke; no extra pairing env vars required)
- set `OPENCLAW_DISABLE_DEVICE_AUTH=0` to require standard device pairing

Model behavior for this smoke script:

- defaults to OpenAI models (`openai/gpt-5.2` + OpenAI fallback) so it does not require Anthropic auth by default

State behavior for this smoke script:

- defaults to isolated config dir `~/.openclaw-paperclip-smoke`
- resets smoke agent state each run by default (`OPENCLAW_RESET_STATE=1`) to avoid stale provider/auth drift

Networking behavior for this smoke script:

- auto-detects and prints a Synaptica host URL reachable from inside OpenClaw Docker
- default container-side host alias is `host.docker.internal` (override with `PAPERCLIP_HOST_FROM_CONTAINER` / `PAPERCLIP_HOST_PORT`)
- if Synaptica rejects container hostnames in authenticated/private mode, allow `host.docker.internal` via `pnpm paperclipai allowed-hostname host.docker.internal` and restart Synaptica
