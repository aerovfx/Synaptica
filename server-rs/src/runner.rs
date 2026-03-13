//! Heartbeat run executor: runs process and http adapters for queued heartbeat_runs.

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use chrono::Utc;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::metrics::MetricsGauge;

const MAX_EXCERPT_BYTES: usize = 32 * 1024;
const DEFAULT_TIMEOUT_SEC: u64 = 900;
const DEFAULT_GRACE_SEC: u64 = 15;
const DEFAULT_HTTP_TIMEOUT_MS: u64 = 15_000;

/// Timeout caps for adapter runs (from config).
#[derive(Clone)]
pub struct RunnerLimits {
    pub max_http_timeout_ms: u64,
    pub max_process_timeout_secs: u64,
}

/// Spawns the run executor in the background. Call this after creating a queued run.
/// If `semaphore` is Some, acquires a permit before running (drops when done).
/// If `active_runs_gauge` is Some, increments on start and decrements when the run finishes.
pub fn spawn_run(
    pool: PgPool,
    run_id: Uuid,
    semaphore: Option<Arc<Semaphore>>,
    limits: RunnerLimits,
    active_runs_gauge: Option<Arc<MetricsGauge>>,
) {
    tokio::spawn(async move {
        let _active_guard = active_runs_gauge.as_ref().map(|g| g.clone().guard());
        let _permit = if let Some(sem) = semaphore {
            match sem.acquire_owned().await {
                Ok(p) => Some(p),
                Err(_) => {
                    tracing::warn!("heartbeat run {} semaphore closed", run_id);
                    return;
                }
            }
        } else {
            None
        };
        if let Err(e) = run_heartbeat_run(&pool, run_id, &limits).await {
            tracing::warn!("heartbeat run {} failed: {}", run_id, e);
        }
    });
}

/// Legacy (Node) adapter types — run via cli/src/run-legacy-adapter.ts when PAPERCLIP_PROJECT_ROOT is set.
const LEGACY_ADAPTER_TYPES: &[&str] = &[
    "claude_local",
    "codex_local",
    "cursor",
    "openclaw_gateway",
    "openfang_gateway",
    "opencode_local",
    "pi_local",
];

/// Loads run + agent, marks running, executes adapter (process/http/legacy), updates run on completion.
pub async fn run_heartbeat_run(
    pool: &PgPool,
    run_id: Uuid,
    limits: &RunnerLimits,
) -> Result<(), String> {
    let (_run, agent_id, company_id, agent_name, adapter_type, adapter_config) = {
        let row: Option<(
            String,  // status
            Uuid,    // agent_id
            Uuid,    // company_id
            String,  // agent name
            String,  // adapter_type
            Option<serde_json::Value>, // adapter_config
        )> = sqlx::query_as(
            "SELECT r.status, r.agent_id, r.company_id, a.name, a.adapter_type, a.adapter_config \
             FROM heartbeat_runs r JOIN agents a ON a.id = r.agent_id WHERE r.id = $1",
        )
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        let (status, agent_id, company_id, agent_name, adapter_type, adapter_config) =
            row.ok_or_else(|| "Run or agent not found".to_string())?;
        if status != "queued" {
            return Ok(());
        }
        let adapter_config = adapter_config.unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        ((), agent_id, company_id, agent_name, adapter_type, adapter_config)
    };

    let now = Utc::now();
    sqlx::query(
        "UPDATE heartbeat_runs SET status = 'running', started_at = $2, updated_at = $2 WHERE id = $1",
    )
    .bind(run_id)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let result = match adapter_type.as_str() {
        "process" => {
            run_process_adapter(pool, run_id, agent_id, company_id, &adapter_config, limits).await
        }
        "http" => {
            run_http_adapter(pool, run_id, agent_id, company_id, &adapter_config, limits).await
        }
        t if LEGACY_ADAPTER_TYPES.contains(&t) => {
            run_legacy_adapter(
                pool,
                run_id,
                agent_id,
                company_id,
                &agent_name,
                &adapter_type,
                &adapter_config,
            )
            .await
        }
        _ => {
            finish_run(
                pool,
                run_id,
                "failed",
                Some(format!("Adapter type not supported: {}", adapter_type)),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await
        }
    };

    if let Err(e) = result {
        let _ = finish_run(
            pool,
            run_id,
            "failed",
            Some(e.clone()),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;
        return Err(e);
    }
    Ok(())
}

fn excerpt(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        format!("...{}", s.get(s.len().saturating_sub(max_bytes)..).unwrap_or(""))
    }
}

async fn next_seq(pool: &PgPool, run_id: Uuid) -> Result<i32, String> {
    let v: Option<(i32,)> = sqlx::query_as(
        "SELECT COALESCE(MAX(seq), 0) + 1 FROM heartbeat_run_events WHERE run_id = $1",
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e: sqlx::Error| e.to_string())?;
    Ok(v.map(|(n,)| n).unwrap_or(1))
}

async fn emit_event(
    pool: &PgPool,
    company_id: Uuid,
    run_id: Uuid,
    agent_id: Uuid,
    seq: i32,
    event_type: &str,
    stream: Option<&str>,
    message: Option<&str>,
    payload: Option<serde_json::Value>,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO heartbeat_run_events (company_id, run_id, agent_id, seq, event_type, stream, message, payload) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(company_id)
    .bind(run_id)
    .bind(agent_id)
    .bind(seq)
    .bind(event_type)
    .bind(stream)
    .bind(message)
    .bind(payload.as_ref())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

async fn finish_run(
    pool: &PgPool,
    run_id: Uuid,
    status: &str,
    error: Option<String>,
    exit_code: Option<i32>,
    signal: Option<String>,
    stdout_excerpt: Option<String>,
    stderr_excerpt: Option<String>,
    usage_json: Option<serde_json::Value>,
    result_json: Option<serde_json::Value>,
) -> Result<(), String> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE heartbeat_runs SET status = $2, finished_at = $3, updated_at = $3, error = $4, \
         exit_code = $5, signal = $6, stdout_excerpt = $7, stderr_excerpt = $8, usage_json = $9, result_json = $10 \
         WHERE id = $1",
    )
    .bind(run_id)
    .bind(status)
    .bind(now)
    .bind(error.as_deref())
    .bind(exit_code)
    .bind(signal.as_deref())
    .bind(stdout_excerpt.as_deref())
    .bind(stderr_excerpt.as_deref())
    .bind(usage_json.as_ref())
    .bind(result_json.as_ref())
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn paperclip_env(agent_id: Uuid, company_id: Uuid, run_id: Uuid) -> HashMap<String, String> {
    let host = std::env::var("PAPERCLIP_LISTEN_HOST")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PAPERCLIP_LISTEN_PORT")
        .or_else(|_| std::env::var("PORT"))
        .unwrap_or_else(|_| "3100".to_string());
    let api_url = std::env::var("PAPERCLIP_API_URL").unwrap_or_else(|_| format!("http://{}:{}", host, port));
    let mut m = HashMap::new();
    m.insert("PAPERCLIP_AGENT_ID".to_string(), agent_id.to_string());
    m.insert("PAPERCLIP_COMPANY_ID".to_string(), company_id.to_string());
    m.insert("PAPERCLIP_API_URL".to_string(), api_url);
    m.insert("PAPERCLIP_RUN_ID".to_string(), run_id.to_string());
    m
}

async fn run_process_adapter(
    pool: &PgPool,
    run_id: Uuid,
    agent_id: Uuid,
    company_id: Uuid,
    config: &serde_json::Value,
    limits: &RunnerLimits,
) -> Result<(), String> {
    let cmd = config
        .get("command")
        .and_then(|c| c.as_str())
        .ok_or_else(|| "process adapter requires adapterConfig.command".to_string())?;
    let args: Vec<String> = config
        .get("args")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let cwd = config
        .get("cwd")
        .and_then(|c| c.as_str())
        .unwrap_or(".");
    let timeout_sec = config
        .get("timeoutSec")
        .and_then(|t| t.as_u64())
        .unwrap_or(DEFAULT_TIMEOUT_SEC);
    let timeout_sec = timeout_sec.min(limits.max_process_timeout_secs);
    let _grace_sec = config
        .get("graceSec")
        .and_then(|g| g.as_u64())
        .unwrap_or(DEFAULT_GRACE_SEC);

    let mut env: HashMap<String, String> = paperclip_env(agent_id, company_id, run_id);
    if let Some(env_obj) = config.get("env").and_then(|e| e.as_object()) {
        for (k, v) in env_obj {
            if let Some(s) = v.as_str() {
                env.insert(k.clone(), s.to_string());
            }
        }
    }

    let seq = next_seq(pool, run_id).await?;
    let payload = serde_json::json!({
        "adapterType": "process",
        "command": cmd,
        "commandArgs": args,
        "cwd": cwd,
        "env": env,
    });
    let (company_id_fetch, agent_id_fetch) = (company_id, agent_id);
    emit_event(
        pool,
        company_id_fetch,
        run_id,
        agent_id_fetch,
        seq,
        "adapter_invoke",
        None,
        None,
        Some(payload),
    )
    .await?;

    let mut child = Command::new(cmd)
        .args(&args)
        .current_dir(cwd)
        .envs(env)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let stdout_handle = child.stdout.take().ok_or("no stdout")?;
    let stderr_handle = child.stderr.take().ok_or("no stderr")?;

    let (stdout_tx, mut stdout_rx) = tokio::sync::mpsc::channel::<String>(64);
    let (stderr_tx, mut stderr_rx) = tokio::sync::mpsc::channel::<String>(64);

    let pool_stdout = pool.clone();
    let run_id_stdout = run_id;
    let company_id_stdout = company_id;
    let agent_id_stdout = agent_id;
    tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        let reader = tokio::io::BufReader::new(stdout_handle);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = stdout_tx.send(line.clone()).await;
            let seq = match next_seq(&pool_stdout, run_id_stdout).await {
                Ok(s) => s,
                _ => break,
            };
            let _ = emit_event(
                &pool_stdout,
                company_id_stdout,
                run_id_stdout,
                agent_id_stdout,
                seq,
                "log",
                Some("stdout"),
                None,
                Some(serde_json::json!({ "message": line })),
            )
            .await;
        }
    });

    let pool_stderr = pool.clone();
    let run_id_stderr = run_id;
    let company_id_stderr = company_id;
    let agent_id_stderr = agent_id;
    tokio::spawn(async move {
        use tokio::io::AsyncBufReadExt;
        let reader = tokio::io::BufReader::new(stderr_handle);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = stderr_tx.send(line.clone()).await;
            let seq = match next_seq(&pool_stderr, run_id_stderr).await {
                Ok(s) => s,
                _ => break,
            };
            let _ = emit_event(
                &pool_stderr,
                company_id_stderr,
                run_id_stderr,
                agent_id_stderr,
                seq,
                "log",
                Some("stderr"),
                None,
                Some(serde_json::json!({ "message": line })),
            )
            .await;
        }
    });

    let mut stdout_acc = String::new();
    let mut stderr_acc = String::new();
    let timeout_duration = if timeout_sec > 0 {
        Duration::from_secs(timeout_sec)
    } else {
        Duration::from_secs(86400 * 365)
    };

    let exit_status = loop {
        tokio::select! {
            status = child.wait() => {
                break status.map_err(|e| e.to_string())?;
            }
            _ = tokio::time::sleep(timeout_duration) => {
                if timeout_sec > 0 {
                    child.kill().await.ok();
                    finish_run(
                        pool,
                        run_id,
                        "timed_out",
                        Some("Process timed out".to_string()),
                        None,
                        None,
                        Some(excerpt(&stdout_acc, MAX_EXCERPT_BYTES)),
                        Some(excerpt(&stderr_acc, MAX_EXCERPT_BYTES)),
                        None,
                        None,
                    )
                    .await?;
                    return Ok(());
                }
            }
            chunk = stdout_rx.recv() => {
                if let Some(line) = chunk {
                    stdout_acc.push_str(&line);
                    stdout_acc.push('\n');
                }
            }
            chunk = stderr_rx.recv() => {
                if let Some(line) = chunk {
                    stderr_acc.push_str(&line);
                    stderr_acc.push('\n');
                }
            }
        }
    };

    while let Ok(line) = stdout_rx.try_recv() {
        stdout_acc.push_str(&line);
        stdout_acc.push('\n');
    }
    while let Ok(line) = stderr_rx.try_recv() {
        stderr_acc.push_str(&line);
        stderr_acc.push('\n');
    }

    let (status, exit_code, signal) = match exit_status.code() {
        Some(0) => ("succeeded", Some(0), None),
        Some(c) => ("failed", Some(c), None),
        None => {
            #[cfg(unix)]
            let sig = exit_status.signal().map(|s| format!("{}", s));
            #[cfg(not(unix))]
            let sig: Option<String> = None;
            ("failed", None, sig)
        }
    };

    let err_msg = if status == "failed" {
        Some(format!(
            "Process exited with code {:?} signal {:?}",
            exit_code,
            signal
        ))
    } else {
        None
    };

    finish_run(
        pool,
        run_id,
        status,
        err_msg,
        exit_code,
        signal,
        Some(excerpt(&stdout_acc, MAX_EXCERPT_BYTES)),
        Some(excerpt(&stderr_acc, MAX_EXCERPT_BYTES)),
        None,
        None,
    )
    .await
}

async fn run_http_adapter(
    pool: &PgPool,
    run_id: Uuid,
    agent_id: Uuid,
    company_id: Uuid,
    config: &serde_json::Value,
    limits: &RunnerLimits,
) -> Result<(), String> {
    let url = config
        .get("url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| "http adapter requires adapterConfig.url".to_string())?;
    let timeout_ms = config
        .get("timeoutMs")
        .and_then(|t| t.as_u64())
        .unwrap_or(DEFAULT_HTTP_TIMEOUT_MS);
    let timeout_ms = timeout_ms.min(limits.max_http_timeout_ms);

    let body = serde_json::json!({
        "runId": run_id.to_string(),
        "agentId": agent_id.to_string(),
        "companyId": company_id.to_string(),
        "context": {
            "runId": run_id.to_string(),
            "agentId": agent_id.to_string(),
            "companyId": company_id.to_string(),
        },
    });

    let seq = next_seq(pool, run_id).await?;
    let payload = serde_json::json!({
        "adapterType": "http",
        "url": url,
        "payload": body,
    });
    emit_event(
        pool,
        company_id,
        run_id,
        agent_id,
        seq,
        "adapter_invoke",
        None,
        None,
        Some(payload),
    )
    .await?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|e| e.to_string())?;

    let mut request = client.post(url).json(&body);
    if let Some(h) = config.get("headers").and_then(|h| h.as_object()) {
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                request = request.header(k.as_str(), s);
            }
        }
    }

    let response = request.send().await.map_err(|e| e.to_string())?;
    let status_code = response.status().as_u16();
    let body_bytes = response.bytes().await.map_err(|e| e.to_string())?;
    let result_json = serde_json::from_slice(&body_bytes).ok();

    if (200..300).contains(&status_code) {
        finish_run(
            pool,
            run_id,
            "succeeded",
            None,
            Some(0),
            None,
            None,
            None,
            None,
            result_json,
        )
        .await
    } else {
        let err_body = String::from_utf8_lossy(&body_bytes);
        let err_msg = format!("HTTP {} {}", status_code, err_body);
        finish_run(
            pool,
            run_id,
            "failed",
            Some(err_msg),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }
}

/// Runs a legacy (Node) adapter via cli/src/run-legacy-adapter.ts. Requires PAPERCLIP_PROJECT_ROOT.
async fn run_legacy_adapter(
    pool: &PgPool,
    run_id: Uuid,
    agent_id: Uuid,
    company_id: Uuid,
    agent_name: &str,
    adapter_type: &str,
    adapter_config: &serde_json::Value,
) -> Result<(), String> {
    let project_root = std::env::var("PAPERCLIP_PROJECT_ROOT").map_err(|_| {
        "Legacy adapters require PAPERCLIP_PROJECT_ROOT (path to repo root)".to_string()
    })?;
    let script_path = std::path::Path::new(&project_root)
        .join("cli")
        .join("src")
        .join("run-legacy-adapter.ts");
    if !script_path.exists() {
        return Err(format!(
            "Legacy adapter script not found: {}",
            script_path.display()
        ));
    }

    let mut env: HashMap<String, String> = paperclip_env(agent_id, company_id, run_id);
    env.insert("AGENT_NAME".to_string(), agent_name.to_string());
    env.insert("ADAPTER_TYPE".to_string(), adapter_type.to_string());
    env.insert(
        "ADAPTER_CONFIG_JSON".to_string(),
        adapter_config.to_string(),
    );
    env.insert("RUNTIME_JSON".to_string(), "{}".to_string());

    let seq = next_seq(pool, run_id).await?;
    let payload = serde_json::json!({
        "adapterType": adapter_type,
        "bridge": "node",
        "script": format!("{}", script_path.display()),
    });
    emit_event(
        pool,
        company_id,
        run_id,
        agent_id,
        seq,
        "adapter_invoke",
        None,
        None,
        Some(payload),
    )
    .await?;

    let mut child = Command::new("pnpm")
        .args(["exec", "tsx", script_path.to_str().unwrap()])
        .envs(env)
        .current_dir(&project_root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn legacy adapter: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "no stdout".to_string())?;
    let mut stderr = child.stderr.take().ok_or_else(|| "no stderr".to_string())?;

    let stderr_handle = tokio::spawn(async move {
        let mut v = Vec::new();
        let _ = tokio::io::AsyncReadExt::read_to_end(&mut stderr, &mut v).await;
        String::from_utf8_lossy(&v).to_string()
    });

    let (mut stdout_acc, mut result_from_script) = (String::new(), None::<LegacyResult>);
    let pool_stdout = pool.clone();
    let run_id_out = run_id;
    let company_id_out = company_id;
    let agent_id_out = agent_id;

    let reader = tokio::io::BufReader::new(stdout);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        stdout_acc.push_str(&line);
        stdout_acc.push('\n');
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(t) = v.get("t").and_then(|t| t.as_str()) {
                if t == "log" {
                    let stream = v.get("s").and_then(|s| s.as_str()).unwrap_or("stdout");
                    let msg = v.get("m").and_then(|m| m.as_str()).unwrap_or("");
                    let seq = match next_seq(&pool_stdout, run_id_out).await {
                        Ok(s) => s,
                        _ => break,
                    };
                    let _ = emit_event(
                        &pool_stdout,
                        company_id_out,
                        run_id_out,
                        agent_id_out,
                        seq,
                        "log",
                        Some(stream),
                        None,
                        Some(serde_json::json!({ "message": msg })),
                    )
                    .await;
                } else if t == "result" {
                    if let Some(r) = v.get("r") {
                        result_from_script = serde_json::from_value(r.clone()).ok();
                    }
                    break;
                }
            }
        }
    }

    let stderr_acc = stderr_handle.await.unwrap_or_default();

    let exit_status = child.wait().await.map_err(|e| e.to_string())?;
    let exit_code = exit_status.code();
    let (status, err_msg, result_json) = if let Some(r) = result_from_script {
        let st = if r.exit_code == Some(0) {
            "succeeded"
        } else {
            "failed"
        };
        let err = r.error_message.as_deref().map(String::from);
        let res = r.result_json.clone();
        (st, err, res)
    } else {
        let st = if exit_code == Some(0) {
            "succeeded"
        } else {
            "failed"
        };
        let err = exit_code
            .filter(|&c| c != 0)
            .map(|c| format!("Process exited with code {}", c));
        (st, err, None)
    };

    finish_run(
        pool,
        run_id,
        status,
        err_msg,
        exit_code,
        None,
        Some(excerpt(&stdout_acc, MAX_EXCERPT_BYTES)),
        Some(excerpt(&stderr_acc, MAX_EXCERPT_BYTES)),
        None,
        result_json,
    )
    .await
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct LegacyResult {
    #[serde(rename = "exitCode")]
    exit_code: Option<i32>,
    signal: Option<String>,
    #[serde(rename = "errorMessage")]
    error_message: Option<String>,
    #[serde(rename = "resultJson")]
    result_json: Option<serde_json::Value>,
}