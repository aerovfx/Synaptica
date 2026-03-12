//! Heartbeat scheduler: periodically enqueue timer wakeups for agents with intervalSec.

use sqlx::PgPool;
use uuid::Uuid;

/// Run one scheduler tick: find agents due for timer wakeup and create queued heartbeat_runs.
pub async fn run_heartbeat_scheduler_tick(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
        r#"
        SELECT a.id, a.company_id FROM agents a
        WHERE a.status IN ('idle', 'paused')
        AND (a.runtime_config->'heartbeat'->>'enabled') = 'true'
        AND (a.runtime_config->'heartbeat'->'intervalSec') IS NOT NULL
        AND (a.runtime_config->'heartbeat'->'intervalSec')::int > 0
        AND NOT EXISTS (
            SELECT 1 FROM heartbeat_runs r WHERE r.agent_id = a.id AND r.status IN ('queued', 'running')
        )
        AND (
            a.last_heartbeat_at IS NULL
            OR a.last_heartbeat_at + ((a.runtime_config->'heartbeat'->'intervalSec')::int::text || ' seconds')::interval < now()
        )
        "#,
    )
    .fetch_all(pool)
    .await?;

    let now = chrono::Utc::now();
    for (agent_id, company_id) in rows {
        let run_id = Uuid::new_v4();
        if sqlx::query(
            r#"
            INSERT INTO heartbeat_runs (
                id, company_id, agent_id, invocation_source, trigger_detail, status, created_at, updated_at
            ) VALUES ($1, $2, $3, 'timer', 'system', 'queued', $4, $4)
            "#,
        )
        .bind(run_id)
        .bind(company_id)
        .bind(agent_id)
        .bind(now)
        .execute(pool)
        .await
        .is_ok()
        {
            tracing::debug!("scheduler: enqueued timer run {} for agent {}", run_id, agent_id);
        }
    }
    Ok(())
}
