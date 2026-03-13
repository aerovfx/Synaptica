# Synaptica — Security mechanisms

This document describes the security controls implemented in Synaptica (Rust API and related components). Use it for audits, deployment hardening, and when adding new endpoints.

## 1. Authentication and authorization

### 1.1 Actor model

- **Board**: Human operator (session or implicit in `local_trusted`). Full control over the instance and all companies.
- **Agent**: Identified by a Bearer API key. Each key is bound to one agent and one company.

Resolution:

- `Authorization: Bearer <token>` is looked up in `agent_api_keys` (key hashed with SHA-256, compared to `key_hash`). Revoked keys (`revoked_at IS NOT NULL`) are rejected.
- Invalid or missing Bearer → request is treated as **Board** (for backward compatibility with cookie/session-based UI). In authenticated deployment mode, board access is enforced separately (e.g. session middleware); the API layer only distinguishes Board vs Agent for route guards.

### 1.2 Board-only routes

- Routes that must not be callable by agents (e.g. create company, delete agent, instance admin) use the **RequireBoard** extractor. If the actor is an Agent, the server returns **403 Forbidden** with a message that the action is board-only.

### 1.3 Agent company scope

- **Middleware `require_agent_company_scope`**: For any request whose path is `/api/companies/:company_id/...`, if the actor is an **Agent**, the server checks that `company_id` in the path equals the agent’s company. If not, it returns **403 Forbidden** (“Agents may only access resources of their own company”).
- Agents can only access company-scoped resources for their own company when using paths under `/api/companies/:company_id/...`.
- **By-id routes** (e.g. `GET /api/issues/:id`, `GET /api/agents/:id`): Handlers that load a resource by id should, when the actor is an Agent, verify that the resource’s `company_id` matches the agent’s company and return **403** otherwise. The middleware above does not cover by-id paths; implement this check in handlers where the resource is company-scoped.

## 2. HTTP security headers

Set by the Rust server on all responses:

| Header | Value | Purpose |
|--------|--------|--------|
| **X-Content-Type-Options** | `nosniff` | Prevents MIME sniffing. |
| **X-Frame-Options** | `DENY` | Reduces clickjacking risk. |
| **Referrer-Policy** | `strict-origin-when-cross-origin` | Limits referrer leakage. |
| **Permissions-Policy** | `accelerometer=(), camera=(), geolocation=(), ...` | Disables browser features the app does not use. |

Optional (only when serving over HTTPS):

| Header | Config | Purpose |
|--------|--------|--------|
| **Strict-Transport-Security (HSTS)** | `HSTS_MAX_AGE_SECS` (e.g. `31536000`) | Enforces HTTPS for the next N seconds. Set only when the server is behind HTTPS to avoid locking clients to HTTPS on a non-HTTPS endpoint. |

## 3. CORS

- **CORS_ORIGINS** (comma-separated): Allowed origins for CORS. If empty, the server allows **any** origin (`*`-like behavior). For production, set explicit origins (e.g. `https://app.example.com`) to reduce cross-origin abuse.

## 4. Request limits

- **HTTP body size**: Capped by **HTTP_BODY_MAX_BYTES** (default 2 MiB). Larger bodies are rejected by the framework before handlers run, reducing DoS risk from huge payloads.

## 5. API keys (agents)

- Stored in `agent_api_keys` as **SHA-256 hash** of the raw key. Raw keys are never stored.
- Keys can be revoked (`revoked_at`); revoked keys are rejected at resolution time.
- `last_used_at` is updated on use (fire-and-forget) for auditing; it does not affect validity.

## 6. Secrets and sensitive data

- Application secrets (DB URL, API keys, etc.) should be supplied via environment or a secret manager, not committed. Use **.env** only for local dev and keep it out of version control.
- The API does not log or echo raw API keys or passwords.

## 7. Deployment checklist

- Set **CORS_ORIGINS** to explicit origins in production.
- Use HTTPS in production; set **HSTS_MAX_AGE_SECS** only when the server is behind HTTPS.
- Ensure **DATABASE_URL** and other secrets are not exposed (env, secret manager, restricted permissions).
- Run DB migrations from a controlled process; restrict DB access to the app and migrations.
- Keep dependencies (Rust crates, Node packages) updated and address known vulnerabilities.

## 8. When adding endpoints

- **Company-scoped paths** (`/api/companies/:company_id/...`): Agent company-scope middleware applies automatically; ensure the handler only returns data for that company.
- **By-id or other paths**: If the resource is company-scoped and callable by agents, resolve the resource, then check (when actor is Agent) that `resource.company_id == actor.agent_company_id()`; otherwise return **403**.
- **Board-only actions**: Use the **RequireBoard** extractor.
- Prefer returning **400/401/403/404/409/422** with clear bodies over generic 500 for security-related failures.
