use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardResponse {
    pub company_id: String,
    pub agents: AgentCounts,
    pub tasks: TaskCounts,
    pub costs: CostSummary,
    pub pending_approvals: i64,
    pub stale_tasks: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCounts {
    pub active: i64,
    pub running: i64,
    pub paused: i64,
    pub error: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskCounts {
    pub open: i64,
    pub in_progress: i64,
    pub blocked: i64,
    pub done: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSummary {
    pub month_spend_cents: i64,
    pub month_budget_cents: i32,
    pub month_utilization_percent: f64,
}

/// GET /api/companies/:companyId/dashboard
pub async fn dashboard(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<DashboardResponse>, (StatusCode, String)> {
    let company_id = &params.company_id;

    let company = sqlx::query_scalar::<_, (i32,)>(
        "SELECT budget_monthly_cents FROM companies WHERE id = $1",
    )
    .bind(company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Company not found".to_string()))?;

    let budget = company.0;

    let agent_rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, count(*)::bigint FROM agents WHERE company_id = $1 GROUP BY status",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut agents_map = std::collections::HashMap::new();
    for (status, count) in agent_rows {
        let bucket = if status == "idle" { "active" } else { status.as_str() };
        *agents_map.entry(bucket.to_string()).or_insert(0i64) += count;
    }

    let task_rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, count(*)::bigint FROM issues WHERE company_id = $1 GROUP BY status",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut open = 0i64;
    let mut in_progress = 0i64;
    let mut blocked = 0i64;
    let mut done = 0i64;
    for (status, count) in task_rows {
        if status == "in_progress" {
            in_progress += count;
        } else if status == "blocked" {
            blocked += count;
        } else if status == "done" {
            done += count;
        }
        if status != "done" && status != "cancelled" {
            open += count;
        }
    }

    let pending_approvals: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM approvals WHERE company_id = $1 AND status = 'pending'",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let stale_cutoff = chrono::Utc::now() - chrono::Duration::hours(1);
    let stale_tasks: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM issues WHERE company_id = $1 AND status = 'in_progress' AND started_at < $2",
    )
    .bind(company_id)
    .bind(stale_cutoff)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let month_spend: i64 = sqlx::query_scalar(
        "SELECT coalesce(sum(cost_cents), 0)::bigint FROM cost_events WHERE company_id = $1 AND occurred_at >= date_trunc('month', now())",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let utilization = if budget > 0 {
        (month_spend as f64 / budget as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(DashboardResponse {
        company_id: company_id.clone(),
        agents: AgentCounts {
            active: *agents_map.get("active").unwrap_or(&0),
            running: *agents_map.get("running").unwrap_or(&0),
            paused: *agents_map.get("paused").unwrap_or(&0),
            error: *agents_map.get("error").unwrap_or(&0),
        },
        tasks: TaskCounts {
            open,
            in_progress,
            blocked,
            done,
        },
        costs: CostSummary {
            month_spend_cents: month_spend,
            month_budget_cents: budget,
            month_utilization_percent: (utilization * 100.0).round() / 100.0, // already in [0,100]
        },
        pending_approvals,
        stale_tasks,
    }))
}

pub async fn dashboard_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set",
    )
}
