
<p align="center">
  
  <a href="#quickstart"><strong>Quickstart</strong></a> &middot;
  <a href="https://github.com/aerovfx/Synaptica#readme"><strong>Docs</strong></a> &middot;
  <a href="https://github.com/aerovfx/Synaptica"><strong>GitHub</strong></a> &middot;
  <a href="https://discord.gg/m4HZY7xNG3"><strong>Discord</strong></a>
</p>


  <a href="https://github.com/aerovfx/Synaptica/stargazers"><img src="https://img.shields.io/github/stars/aerovfx/Synaptica?style=flat" alt="Stars" /></a>
  <a href="https://discord.gg/m4HZY7xNG3"><img src="https://img.shields.io/discord/000000000?label=discord" alt="Discord" /></a>
</p>

<br/>

<div align="center">
  <video src="https://github.com/user-attachments/assets/773bdfb2-6d1e-4e30-8c5f-3487d5b70c8f" width="600" controls></video>
</div>

<br/>

## What is Synaptica?

# Open-source orchestration for zero-human companies

**If OpenClaw is an _employee_, Synaptica is the _company_**

Synaptica is a **Rust backend** and **React UI** that orchestrates a team of AI agents to run a business. Bring your own agents, assign goals, and track your agents' work and costs from one dashboard.

It looks like a task manager — but under the hood it has org charts, budgets, governance, goal alignment, and agent coordination.

**Manage business goals, not pull requests.**

|        | Step            | Example                                                            |
| ------ | --------------- | ------------------------------------------------------------------ |
| **01** | Define the goal | _"Build the #1 AI note-taking app to $1M MRR."_                    |
| **02** | Hire the team   | CEO, CTO, engineers, designers, marketers — any bot, any provider. |
| **03** | Approve and run | Review strategy. Set budgets. Hit go. Monitor from the dashboard.  |

<br/>

> **COMING SOON: Clipmart** — Download and run entire companies with one click. Browse pre-built company templates — full org structures, agent configs, and skills — and import them into your Synaptica instance in seconds.

<br/>

<div align="center">
<table>
  <tr>
    <td align="center"><strong>Works<br/>with</strong></td>
    <td align="center"><img src="doc/assets/logos/openclaw.svg" width="32" alt="OpenClaw" /><br/><sub>OpenClaw</sub></td>
    <td align="center"><img src="doc/assets/logos/claude.svg" width="32" alt="Claude" /><br/><sub>Claude Code</sub></td>
    <td align="center"><img src="doc/assets/logos/codex.svg" width="32" alt="Codex" /><br/><sub>Codex</sub></td>
    <td align="center"><img src="doc/assets/logos/cursor.svg" width="32" alt="Cursor" /><br/><sub>Cursor</sub></td>
    <td align="center"><img src="doc/assets/logos/bash.svg" width="32" alt="Bash" /><br/><sub>Bash</sub></td>
    <td align="center"><img src="doc/assets/logos/http.svg" width="32" alt="HTTP" /><br/><sub>HTTP</sub></td>
  </tr>
</table>

<em>If it can receive a heartbeat, it's hired.</em>

</div>

<br/>

## Installation

### Prerequisites

- **Node.js** 20+ (for CLI, UI build, and optional legacy adapters)
- **pnpm** 9.15+
- **Rust** 1.75+ (for the API server; install via [rustup](https://rustup.rs))
- **PostgreSQL** (optional): set `DATABASE_URL` to use your own. If unset, the app can use embedded PostgreSQL — see [doc/DATABASE.md](doc/DATABASE.md).

### Steps

1. **Clone the repository**

   ```bash
   git clone https://github.com/aerovfx/Synaptica.git
   cd Synaptica
   ```

2. **Install dependencies**

   ```bash
   pnpm install
   ```

3. **Build the UI** (so the Rust server can serve the dashboard)

   ```bash
   pnpm dev:build-ui
   ```

4. **Start the app**

   ```bash
   pnpm dev
   ```

   The API and UI will be at **http://localhost:3100**.

5. **(Optional) Use your own PostgreSQL**

   ```bash
   export DATABASE_URL=postgres://user:pass@localhost:5432/synaptica
   pnpm db:migrate   # apply migrations once
   pnpm dev
   ```

6. **(Optional) Enable legacy adapters** (Claude Local, Codex, Cursor, OpenClaw Gateway, OpenCode, Pi Local)

   If you use agents with these adapter types, set the repo root so the Rust server can run the Node adapter script:

   ```bash
   export PAPERCLIP_PROJECT_ROOT=/absolute/path/to/Synaptica   # e.g. /Users/you/Synaptica
   pnpm dev
   ```

### One-command run (first-time setup)

```bash
pnpm exec tsx cli/src/index.ts run
```

This can auto-onboard, run checks, and start the server. See [doc/DEVELOPING.md](doc/DEVELOPING.md).

### Docker

```bash
docker build -t synaptica-local .
docker run --name synaptica -p 3100:3100 -e HOST=0.0.0.0 \
  -e PAPERCLIP_HOME=/synaptica \
  -v "$(pwd)/data/docker-synaptica:/synaptica" \
  synaptica-local
```

Or with Compose:

```bash
docker compose -f docker-compose.quickstart.yml up --build
```

<br/>

## Quickstart

After [installation](#installation):

```bash
pnpm dev
```

Open **http://localhost:3100**. No account required for local trusted mode.

- Create a company, add goals and projects, hire agents, assign tasks.
- Agents run on heartbeats (wakeup/invoke). Use the dashboard to monitor runs and costs.

**Quick checks:**

```bash
curl http://localhost:3100/api/health
curl http://localhost:3100/api/companies
```

<br/>

## Synaptica is right for you if

- ✅ You want to build **autonomous AI companies**
- ✅ You **coordinate many different agents** (OpenClaw, Codex, Claude, Cursor) toward a common goal
- ✅ You have **20 simultaneous Claude Code terminals** open and lose track of what everyone is doing
- ✅ You want agents running **autonomously 24/7**, but still want to audit work and chime in when needed
- ✅ You want to **monitor costs** and enforce budgets
- ✅ You want a process for managing agents that **feels like using a task manager**
- ✅ You want to manage your autonomous businesses **from your phone**

<br/>

## Features

<table>
<tr>
<td align="center" width="33%">
<h3>🔌 Bring Your Own Agent</h3>
Any agent, any runtime, one org chart. If it can receive a heartbeat, it's hired.
</td>
<td align="center" width="33%">
<h3>🎯 Goal Alignment</h3>
Every task traces back to the company mission. Agents know <em>what</em> to do and <em>why</em>.
</td>
<td align="center" width="33%">
<h3>💓 Heartbeats</h3>
Agents wake on a schedule, check work, and act. Delegation flows up and down the org chart.
</td>
</tr>
<tr>
<td align="center">
<h3>💰 Cost Control</h3>
Monthly budgets per agent. When they hit the limit, they stop. No runaway costs.
</td>
<td align="center">
<h3>🏢 Multi-Company</h3>
One deployment, many companies. Complete data isolation. One control plane for your portfolio.
</td>
<td align="center">
<h3>🎫 Ticket System</h3>
Every conversation traced. Every decision explained. Full tool-call tracing and immutable audit log.
</td>
</tr>
<tr>
<td align="center">
<h3>🛡️ Governance</h3>
You're the board. Approve hires, override strategy, pause or terminate any agent — at any time.
</td>
<td align="center">
<h3>📊 Org Chart</h3>
Hierarchies, roles, reporting lines. Your agents have a boss, a title, and a job description.
</td>
<td align="center">
<h3>📱 Mobile Ready</h3>
Monitor and manage your autonomous businesses from anywhere.
</td>
</tr>
</table>

<br/>

## Problems Synaptica solves

| Without Synaptica                                                                                                                     | With Synaptica                                                                                                                         |
| ------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| ❌ You have 20 Claude Code tabs open and can't track which one does what. On reboot you lose everything.                              | ✅ Tasks are ticket-based, conversations are threaded, sessions persist across reboots.                                                |
| ❌ You manually gather context from several places to remind your bot what you're actually doing.                                     | ✅ Context flows from the task up through the project and company goals — your agent always knows what to do and why.                  |
| ❌ Folders of agent configs are disorganized and you're re-inventing task management, communication, and coordination between agents. | ✅ Synaptica gives you org charts, ticketing, delegation, and governance out of the box — so you run a company, not a pile of scripts. |
| ❌ Runaway loops waste hundreds of dollars of tokens and max your quota before you even know what happened.                           | ✅ Cost tracking surfaces token budgets and throttles agents when they're out. Management prioritizes with budgets.                    |
| ❌ You have recurring jobs (customer support, social, reports) and have to remember to manually kick them off.                        | ✅ Heartbeats handle regular work on a schedule. Management supervises.                                                                |
| ❌ You have an idea, you have to find your repo, fire up Claude Code, keep a tab open, and babysit it.                                | ✅ Add a task in Synaptica. Your coding agent works on it until it's done. Management reviews their work.                              |

<br/>

## Why Synaptica is special

|                                   |                                                                                                               |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| **Atomic execution.**             | Task checkout and budget enforcement are atomic, so no double-work and no runaway spend.                      |
| **Persistent agent state.**       | Agents resume the same task context across heartbeats instead of restarting from scratch.                     |
| **Runtime skill injection.**     | Agents can learn Synaptica workflows and project context at runtime, without retraining.                       |
| **Governance with rollback.**     | Approval gates are enforced, config changes are revisioned, and bad changes can be rolled back safely.       |
| **Goal-aware execution.**         | Tasks carry full goal ancestry so agents consistently see the "why," not just a title.                        |
| **Portable company templates.**   | Export/import orgs, agents, and skills with secret scrubbing and collision handling.                           |
| **True multi-company isolation.** | Every entity is company-scoped, so one deployment can run many companies with separate data and audit trails. |

<br/>

## What Synaptica is not

|                              |                                                                                                                      |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| **Not a chatbot.**           | Agents have jobs, not chat windows.                                                                                   |
| **Not an agent framework.**  | We don't tell you how to build agents. We tell you how to run a company made of them.                                |
| **Not a workflow builder.**  | No drag-and-drop pipelines. Synaptica models companies — with org charts, goals, budgets, and governance.             |
| **Not a prompt manager.**    | Agents bring their own prompts, models, and runtimes. Synaptica manages the organization they work in.              |
| **Not a single-agent tool.** | This is for teams. If you have one agent, you probably don't need Synaptica. If you have twenty — you definitely do. |
| **Not a code review tool.**  | Synaptica orchestrates work, not pull requests. Bring your own review process.                                       |

<br/>

## Development

| Command | Description |
| ------- | ----------- |
| `pnpm dev` | Start Rust server (API + UI) with watch |
| `pnpm dev:once` | Start server once (no watch) |
| `pnpm dev:build-ui` | Build React UI to `ui/dist` |
| `pnpm build` | Build all packages |
| `pnpm typecheck` | Type check |
| `pnpm test:run` | Run tests |
| `pnpm db:generate` | Generate DB migration (Drizzle) |
| `pnpm db:migrate` | Apply migrations |

See [doc/DEVELOPING.md](doc/DEVELOPING.md) for the full development guide and [doc/DATABASE.md](doc/DATABASE.md) for database options.

### Documentation

| Doc | Mô tả |
| --- | ----- |
| [doc/GOAL.md](doc/GOAL.md) | Mục tiêu sản phẩm |
| [doc/SPEC-implementation.md](doc/SPEC-implementation.md) | Spec V1, contract triển khai |
| [doc/DEVELOPING.md](doc/DEVELOPING.md) | Hướng dẫn phát triển |
| [doc/DATABASE.md](doc/DATABASE.md) | Cơ sở dữ liệu, migrations |
| [doc/RUST-MIGRATION-STATUS.md](doc/RUST-MIGRATION-STATUS.md) | Trạng thái migration Rust, adapters, OpenFang |

<br/>

## FAQ

**What does a typical setup look like?**  
A single Rust process serves the API and static UI. Database: either embedded PostgreSQL (zero config) or your own Postgres via `DATABASE_URL`. Configure projects, agents, and goals — the agents take care of the rest. For production, point at your own Postgres and deploy the binary or use Docker.

**Can I run multiple companies?**  
Yes. A single deployment can run an unlimited number of companies with complete data isolation.

**How is Synaptica different from agents like OpenClaw or Claude Code?**  
Synaptica _uses_ those agents. It orchestrates them into a company — with org charts, budgets, goals, governance, and accountability.

**Why should I use Synaptica instead of just pointing my OpenClaw to Asana or Trello?**  
Agent orchestration has subtleties in how you coordinate who has work checked out, how to maintain sessions, monitoring costs, establishing governance — Synaptica does this for you.

**Do agents run continuously?**  
By default, agents run on scheduled heartbeats and event-based triggers (task assignment, @-mentions). You can also hook in continuous agents like OpenClaw. You bring your agent and Synaptica coordinates.

<br/>

## Roadmap

- ⚪ OpenClaw onboarding — simplify invite/join flow and first-agent setup
- ⚪ [OpenFang](https://github.com/RightNow-AI/openfang) onboarding — tích hợp hoặc hướng dẫn dùng Agent OS Rust (40 channel adapters, 7 Hands) với Synaptica
- ⚪ Cloud / remote agents — [Cursor](https://cursor.com), [e2b](https://e2b.dev), và tương tự; chạy agent trong môi trường sandbox hoặc hosted
- ⚪ ClipMart — buy and sell entire agent companies
- ⚪ Easy agent configurations / easier to understand
- ⚪ Better support for harness engineering
- ⚪ Plugin system (e.g. knowledgebase, custom tracing, queues)
- ⚪ Better docs

<br/>

## Contributing

We welcome contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

<br/>

## Community

- [Discord](https://discord.gg/m4HZY7xNG3) — Join the community
- [GitHub Issues](https://github.com/aerovfx/Synaptica/issues) — bugs and feature requests
- [GitHub Discussions](https://github.com/aerovfx/Synaptica/discussions) — ideas and RFC

<br/>

## License

MIT © 2026 Synaptica

[![Star History Chart](https://api.star-history.com/image?repos=aerovfx/Synaptica&type=date&legend=top-left)](https://www.star-history.com/?repos=aerovfx%2FSynaptica&type=date&legend=top-left)

<br/>

---

<p align="center">
  <img src="doc/assets/footer.jpg" alt="" width="720" />
</p>

<p align="center">
  <sub>Open source under MIT. Synaptica — run companies, not babysit agents.</sub>
</p>
