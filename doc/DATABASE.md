# Database

Paperclip uses PostgreSQL via [Drizzle ORM](https://orm.drizzle.team/). There are three ways to run the database, from simplest to most production-ready.

## 1. Embedded PostgreSQL — zero config

If you don't set `DATABASE_URL`, the server automatically starts an embedded PostgreSQL instance and manages a local data directory.

```sh
pnpm dev
```

That's it. On first start the server:

1. Creates a `~/.paperclip/instances/default/db/` directory for storage
2. Ensures the `paperclip` database exists
3. Runs migrations automatically for empty databases
4. Starts serving requests

Data persists across restarts in `~/.paperclip/instances/default/db/`. To reset local dev data, delete that directory.

If you need to apply pending migrations manually, run:

```sh
pnpm db:migrate
```

When `DATABASE_URL` is unset, this command targets the current embedded PostgreSQL instance for your active Paperclip config/instance.

This mode is ideal for local development and one-command installs.

Docker note: the Docker quickstart image also uses embedded PostgreSQL by default. Persist `/paperclip` to keep DB state across container restarts (see `doc/DOCKER.md`).

## 2. Local PostgreSQL (Docker)

For a full PostgreSQL server locally, use the included Docker Compose setup. From the repo root, ensure `.env` exists with at least `BETTER_AUTH_SECRET` (see `.env.example`), then:

```sh
docker compose up -d db
```

This starts only PostgreSQL 17 on `localhost:5432` (user `paperclip`, password `paperclip`, database `paperclip`). Set the connection string in `.env`:

```sh
cp .env.example .env
# .env should include:
# DATABASE_URL=postgres://paperclip:paperclip@localhost:5432/paperclip
```

Run migrations (required so the app sees tables like `agents`). The migrate script does not read the repo `.env`, so pass the URL explicitly:

```sh
DATABASE_URL=postgres://paperclip:paperclip@localhost:5432/paperclip pnpm db:migrate
```

Start the server (the dev runner loads `.env` from repo root and passes vars to the Rust server, so `DATABASE_URL` in `.env` is used):

```sh
pnpm dev
```

**Connecting with `psql`:** When Postgres runs in Docker it listens on TCP (`localhost:5432`), not the default Unix socket. Use TCP explicitly:

```sh
psql "postgres://paperclip:paperclip@localhost:5432/paperclip"
# or
psql -h localhost -p 5432 -U paperclip -d paperclip
```

If you see `connection to server on socket ... failed: No such file or directory`, the DB is likely in Docker—use one of the commands above (and ensure `docker compose up -d db` is running).

**If you see `permission denied for database paperclip`:**

1. **Check what is on port 5432.** Another Postgres (e.g. from `paperclip/docker-compose.yml`, Homebrew, or system) may be using 5432, so your migrate command talks to that server instead of `synaptica-db-1`:

   ```sh
   docker ps -a --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
   lsof -i :5432
   ```

   If another process is using 5432, the `synaptica-db-1` container may have failed to start (check `docker ps -a`). Stop the other Postgres (e.g. `docker stop <other-db-container>` or `brew services stop postgresql`), then:

   ```sh
   docker start synaptica-db-1
   # wait a few seconds
   DATABASE_URL=postgres://paperclip:paperclip@localhost:5432/paperclip pnpm db:migrate
   ```

2. **If only `synaptica-db-1` is on 5432** but migrate still fails, another process may be bound to 5432. Run migration from a container on the same Docker network (so it connects to the `db` service). Use a copy of the repo inside the container so Linux gets its own `node_modules` (avoids esbuild platform errors when mounting from macOS):

   ```sh
   docker run --rm --network synaptica_default \
     -v "$(pwd):/source:ro" -v synaptica_migrate_app:/app -w /app \
     -e DATABASE_URL=postgres://paperclip:paperclip@db:5432/paperclip \
     node:20-alpine sh -c "cp -r /source/. /app/ && rm -rf /app/node_modules /app/packages/*/node_modules 2>/dev/null; npm install -g pnpm && pnpm install && pnpm db:migrate"
   ```

3. **Otherwise** use a clean Postgres and re-run migrations:

   ```sh
   docker compose down
   docker volume rm synaptica_pgdata 2>/dev/null || docker volume rm $(docker volume ls -q | grep pgdata) 2>/dev/null || true
   docker compose up -d db
   # wait a few seconds, then:
   DATABASE_URL=postgres://paperclip:paperclip@localhost:5432/paperclip pnpm db:migrate
   ```

If your Compose project name is not `synaptica`, the volume may be named `<project>_pgdata`; run `docker volume ls` to confirm.

## 3. Hosted PostgreSQL (Supabase)

For production, use a hosted PostgreSQL provider. [Supabase](https://supabase.com/) is a good option with a free tier.

### Setup

1. Create a project at [database.new](https://database.new)
2. Go to **Project Settings > Database > Connection string**
3. Copy the URI and replace the password placeholder with your database password

### Connection string

Supabase offers two connection modes:

**Direct connection** (port 5432) — use for migrations and one-off scripts:

```
postgres://postgres.[PROJECT-REF]:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:5432/postgres
```

**Connection pooling via Supavisor** (port 6543) — use for the application:

```
postgres://postgres.[PROJECT-REF]:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres
```

### Configure

Set `DATABASE_URL` in your `.env`:

```sh
DATABASE_URL=postgres://postgres.[PROJECT-REF]:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres
```

If using connection pooling (port 6543), the `postgres` client must disable prepared statements. Update `packages/db/src/client.ts`:

```ts
export function createDb(url: string) {
  const sql = postgres(url, { prepare: false });
  return drizzlePg(sql, { schema });
}
```

### Push the schema

```sh
# Use the direct connection (port 5432) for schema changes
DATABASE_URL=postgres://postgres.[PROJECT-REF]:[PASSWORD]@...5432/postgres \
  npx drizzle-kit push
```

### Free tier limits

- 500 MB database storage
- 200 concurrent connections
- Projects pause after 1 week of inactivity

See [Supabase pricing](https://supabase.com/pricing) for current details.

## Migrating database from Paperclip to Synaptica

If you have an existing Paperclip project (e.g. at `Synaptica/paperclip`) and want to use that data in Synaptica:

1. **Source (Paperclip)** — choose one:
   - **Embedded Postgres:** Ensure Paperclip’s embedded instance is running (e.g. start Paperclip once: `cd paperclip && pnpm dev`, then stop it). The script will use `~/.paperclip/instances/default/config.json` and the port from config (default `54329`).
   - **Explicit URL:** Set `PAPERCLIP_SOURCE_DATABASE_URL` to Paperclip’s connection string, e.g.  
     `postgres://paperclip:paperclip@127.0.0.1:54329/paperclip`
   - **Config path:** Set `PAPERCLIP_SOURCE_CONFIG` to the path of Paperclip’s `config.json` (e.g. `paperclip/.paperclip/config.json` or `~/.paperclip/instances/default/config.json`).

3. **Target (Synaptica):** Set `DATABASE_URL` to the Synaptica Postgres URL (empty database or one you want to overwrite).

4. **Run the migration:**
   ```sh
   DATABASE_URL=postgres://user:pass@localhost:5432/synaptica pnpm db:migrate-from-paperclip
   ```
   Or with explicit source:
   ```sh
   PAPERCLIP_SOURCE_DATABASE_URL=postgres://paperclip:paperclip@127.0.0.1:54329/paperclip \
   DATABASE_URL=postgres://user:pass@localhost:5432/synaptica \
   pnpm db:migrate-from-paperclip
   ```

5. **Apply pending migrations** (if Synaptica has newer schema):
   ```sh
   DATABASE_URL=postgres://user:pass@localhost:5432/synaptica pnpm db:migrate
   ```

6. Start Synaptica with the same `DATABASE_URL`: `pnpm dev`.

Backup files are written to `scripts/backups/` with prefix `paperclip-to-synaptica-*.sql`.

## Switching between modes

The database mode is controlled by `DATABASE_URL`:

| `DATABASE_URL` | Mode |
|---|---|
| Not set | Embedded PostgreSQL (`~/.paperclip/instances/default/db/`) |
| `postgres://...localhost...` | Local Docker PostgreSQL |
| `postgres://...supabase.com...` | Hosted Supabase |

Your Drizzle schema (`packages/db/src/schema/`) stays the same regardless of mode.

### Connection pool (Rust server)

When using `DATABASE_URL`, the Rust backend (`server-rs`) uses a connection pool. You can tune it with:

- `DB_POOL_MAX_SIZE` (default `10`) — max connections in the pool
- `DB_POOL_ACQUIRE_TIMEOUT_SECS` (default `5`) — timeout when acquiring a connection
- `DB_POOL_IDLE_TIMEOUT_SECS` (optional) — idle timeout for connections

See **Environment variables (server-rs)** in `doc/DEVELOPING.md` for the full list.

## Secret storage

Paperclip stores secret metadata and versions in:

- `company_secrets`
- `company_secret_versions`

For local/default installs, the active provider is `local_encrypted`:

- Secret material is encrypted at rest with a local master key.
- Default key file: `~/.paperclip/instances/default/secrets/master.key` (auto-created if missing).
- CLI config location: `~/.paperclip/instances/default/config.json` under `secrets.localEncrypted.keyFilePath`.

Optional overrides:

- `PAPERCLIP_SECRETS_MASTER_KEY` (32-byte key as base64, hex, or raw 32-char string)
- `PAPERCLIP_SECRETS_MASTER_KEY_FILE` (custom key file path)

Strict mode to block new inline sensitive env values:

```sh
PAPERCLIP_SECRETS_STRICT_MODE=true
```

You can set strict mode and provider defaults via:

```sh
pnpm paperclipai configure --section secrets
```

Inline secret migration command:

```sh
pnpm secrets:migrate-inline-env --apply
```
