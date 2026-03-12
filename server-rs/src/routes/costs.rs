use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::RequireBoard;
use crate::models::cost_event::CostEvent;
use crate::models::company::Company;
use crate::models::agent::Agent;

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCostEventBody {
    pub agent_id: String,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub goal_id: Option<String>,
    pub billing_code: Option<String>,
    pub provider: String,
    pub model: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_cents: i32,
    pub occurred_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSummaryResponse {
    pub company_id: String,
    pub month_spend_cents: i64,
    pub total_spend_cents: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostByAgentRow {
    pub agent_id: String,
    pub spend_cents: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostByProjectRow {
    pub project_id: Option<String>,
    pub spend_cents: i64,
}

/// POST /api/companies/:companyId/cost-events
pub async fn create_cost_event(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<CreateCostEventBody>,
) -> Result<(StatusCode, Json<CostEvent>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let agent_id: Uuid = Uuid::parse_str(&body.agent_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid agent_id".to_string()))?;
    let issue_id: Option<Uuid> = body.issue_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let project_id: Option<Uuid> = body.project_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let goal_id: Option<Uuid> = body.goal_id.as_ref().and_then(|s| Uuid::parse_str(s).ok());
    let occurred_at = body
        .occurred_at
        .as_ref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&chrono::Utc)))
        .unwrap_or_else(chrono::Utc::now);
    let input_tokens = body.input_tokens.unwrap_or(0);
    let output_tokens = body.output_tokens.unwrap_or(0);
    let row = sqlx::query_as::<_, CostEvent>(
        "INSERT INTO cost_events (id, company_id, agent_id, issue_id, project_id, goal_id, billing_code, provider, model, input_tokens, output_tokens, cost_cents, occurred_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING id, company_id, agent_id, issue_id, project_id, goal_id, billing_code, provider, model, input_tokens, output_tokens, cost_cents, occurred_at, created_at",
    )
    .bind(id)
    .bind(&params.company_id)
    .bind(agent_id)
    .bind(issue_id)
    .bind(project_id)
    .bind(goal_id)
    .bind(&body.billing_code)
    .bind(&body.provider)
    .bind(&body.model)
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(body.cost_cents)
    .bind(occurred_at)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(row)))
}

/// GET /api/companies/:companyId/costs/summary
pub async fn get_costs_summary(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<CostSummaryResponse>, (StatusCode, String)> {
    let company_id = &params.company_id;
    let month_spend: i64 = sqlx::query_scalar(
        "SELECT coalesce(sum(cost_cents), 0)::bigint FROM cost_events WHERE company_id = $1 AND occurred_at >= date_trunc('month', now())",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let total_spend: i64 = sqlx::query_scalar(
        "SELECT coalesce(sum(cost_cents), 0)::bigint FROM cost_events WHERE company_id = $1",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(CostSummaryResponse {
        company_id: company_id.clone(),
        month_spend_cents: month_spend,
        total_spend_cents: total_spend,
    }))
}

/// GET /api/companies/:companyId/costs/by-agent
pub async fn get_costs_by_agent(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CostByAgentRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (Uuid, i64)>(
        "SELECT agent_id, sum(cost_cents)::bigint FROM cost_events WHERE company_id = $1 GROUP BY agent_id",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, spend)| CostByAgentRow {
                agent_id: id.to_string(),
                spend_cents: spend,
            })
            .collect(),
    ))
}

/// GET /api/companies/:companyId/costs/by-project
pub async fn get_costs_by_project(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Vec<CostByProjectRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (Option<Uuid>, i64)>(
        "SELECT project_id, sum(cost_cents)::bigint FROM cost_events WHERE company_id = $1 GROUP BY project_id",
    )
    .bind(&params.company_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, spend)| CostByProjectRow {
                project_id: id.map(|u| u.to_string()),
                spend_cents: spend,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBudgetBody {
    pub budget_monthly_cents: Option<i32>,
}

/// PATCH /api/companies/:companyId/budgets
pub async fn patch_company_budgets(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<UpdateBudgetBody>,
) -> Result<Json<Company>, (StatusCode, String)> {
    let cents = body.budget_monthly_cents.unwrap_or(0);
    if cents < 0 {
        return Err((StatusCode::BAD_REQUEST, "budget_monthly_cents must be non-negative".to_string()));
    }
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Company>(
        "UPDATE companies SET budget_monthly_cents = $2, updated_at = $3 WHERE id = $1 RETURNING id, name, description, status, issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents, require_board_approval_for_new_agents, brand_color, created_at, updated_at",
    )
    .bind(&params.company_id)
    .bind(cents)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Company not found".to_string()))?;
    Ok(Json(row))
}

#[derive(Deserialize)]
pub struct AgentIdParam {
    pub id: String,
}

/// PATCH /api/agents/:agentId/budgets
pub async fn patch_agent_budgets(
    State(pool): State<PgPool>,
    Path(params): Path<AgentIdParam>,
    Json(body): Json<UpdateBudgetBody>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let cents = body.budget_monthly_cents.unwrap_or(0);
    if cents < 0 {
        return Err((StatusCode::BAD_REQUEST, "budget_monthly_cents must be non-negative".to_string()));
    }
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Agent>(
        "UPDATE agents SET budget_monthly_cents = $2, updated_at = $3 WHERE id = $1 RETURNING id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, adapter_config, runtime_config, budget_monthly_cents, spent_monthly_cents, permissions, last_heartbeat_at, metadata, created_at, updated_at",
    )
    .bind(&params.id)
    .bind(cents)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Agent not found".to_string()))?;
    Ok(Json(row))
}

pub async fn costs_no_db() -> (StatusCode, &'static str) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "DATABASE_URL not set",
    )
}
