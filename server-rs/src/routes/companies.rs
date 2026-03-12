use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::RequireBoard;
use crate::models::company::Company;

/// Inserts default agents (CEO, CTO, CFO, COO, Engineer) for a new company so the initial UI shows them.
async fn insert_default_agents(
    pool: &PgPool,
    company_id: Uuid,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    let ceo_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO agents (id, company_id, name, role, title, status, adapter_type, adapter_config, runtime_config, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)",
    )
    .bind(ceo_id)
    .bind(company_id)
    .bind("CEO Agent")
    .bind("ceo")
    .bind("Chief Executive Officer")
    .bind("idle")
    .bind("process")
    .bind(&json!({ "command": "echo", "args": ["hello from ceo"] }))
    .bind(&json!({}))
    .bind(&json!({}))
    .bind(now)
    .execute(pool)
    .await?;

    let cto_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO agents (id, company_id, name, role, title, status, reports_to, adapter_type, adapter_config, runtime_config, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)",
    )
    .bind(cto_id)
    .bind(company_id)
    .bind("CTO Agent")
    .bind("cto")
    .bind("Chief Technology Officer")
    .bind("idle")
    .bind(ceo_id)
    .bind("process")
    .bind(&json!({ "command": "echo", "args": ["hello from cto"] }))
    .bind(&json!({}))
    .bind(&json!({}))
    .bind(now)
    .execute(pool)
    .await?;

    let cfo_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO agents (id, company_id, name, role, title, status, reports_to, adapter_type, adapter_config, runtime_config, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)",
    )
    .bind(cfo_id)
    .bind(company_id)
    .bind("CFO Agent")
    .bind("cfo")
    .bind("Chief Financial Officer")
    .bind("idle")
    .bind(ceo_id)
    .bind("process")
    .bind(&json!({ "command": "echo", "args": ["hello from cfo"] }))
    .bind(&json!({}))
    .bind(&json!({}))
    .bind(now)
    .execute(pool)
    .await?;

    let coo_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO agents (id, company_id, name, role, title, status, reports_to, adapter_type, adapter_config, runtime_config, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)",
    )
    .bind(coo_id)
    .bind(company_id)
    .bind("COO Agent")
    .bind("coo")
    .bind("Chief Operating Officer")
    .bind("idle")
    .bind(ceo_id)
    .bind("process")
    .bind(&json!({ "command": "echo", "args": ["hello from coo"] }))
    .bind(&json!({}))
    .bind(&json!({}))
    .bind(now)
    .execute(pool)
    .await?;

    let _engineer_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO agents (id, company_id, name, role, title, status, reports_to, adapter_type, adapter_config, runtime_config, permissions, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)",
    )
    .bind(_engineer_id)
    .bind(company_id)
    .bind("Engineer Agent")
    .bind("engineer")
    .bind("Software Engineer")
    .bind("idle")
    .bind(ceo_id)
    .bind("process")
    .bind(&json!({ "command": "echo", "args": ["hello from engineer"] }))
    .bind(&json!({}))
    .bind(&json!({}))
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

/// GET /api/companies — list all companies
pub async fn list_companies(State(pool): State<PgPool>) -> Result<Json<Vec<Company>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, Company>(
        "SELECT id, name, description, status, issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents, require_board_approval_for_new_agents, brand_color, created_at, updated_at FROM companies ORDER BY created_at",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct CompanyIdParam {
    pub company_id: String,
}

/// GET /api/companies/:companyId
pub async fn get_company(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Company>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, Company>(
        "SELECT id, name, description, status, issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents, require_board_approval_for_new_agents, brand_color, created_at, updated_at FROM companies WHERE id = $1",
    )
    .bind(&params.company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Company not found".to_string()))?;
    Ok(Json(row))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyBody {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

fn derive_issue_prefix(name: &str) -> String {
    let s: String = name.to_uppercase().chars().filter(|c| c.is_ascii_alphabetic()).take(3).collect();
    if s.is_empty() { "CMP".to_string() } else { s }
}

/// POST /api/companies
pub async fn create_company(
    State(pool): State<PgPool>,
    Json(body): Json<CreateCompanyBody>,
) -> Result<(StatusCode, Json<Company>), (StatusCode, String)> {
    let status = body.status.as_deref().unwrap_or("active");
    let base = derive_issue_prefix(&body.name);
    for attempt in 0u32..100 {
        let prefix = if attempt == 0 {
            base.clone()
        } else {
            format!("{}{}", base, "A".repeat(attempt as usize))
        };
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let res = sqlx::query_as::<_, Company>(
            "INSERT INTO companies (id, name, description, status, issue_prefix, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6) RETURNING id, name, description, status, issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents, require_board_approval_for_new_agents, brand_color, created_at, updated_at",
        )
        .bind(id)
        .bind(&body.name)
        .bind(&body.description)
        .bind(status)
        .bind(&prefix)
        .bind(now)
        .fetch_optional(&pool)
        .await;
        let row = match res {
            Ok(Some(r)) => r,
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if db_err.is_unique_violation() {
                        continue;
                    }
                }
                return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
            }
            _ => continue,
        };
        if let Err(e) = insert_default_agents(&pool, row.id, now).await {
            // Best-effort: company is created even if default agents fail.
            eprintln!("failed to insert default agents for company {}: {}", row.id, e);
        }
        return Ok((StatusCode::CREATED, Json(row)));
    }
    Err((StatusCode::CONFLICT, "Could not allocate unique issue prefix".to_string()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCompanyBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub budget_monthly_cents: Option<i32>,
}

/// PATCH /api/companies/:companyId
pub async fn update_company(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
    Json(body): Json<UpdateCompanyBody>,
) -> Result<Json<Company>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Company>(
        "UPDATE companies SET name = COALESCE($2, name), description = COALESCE($3, description), status = COALESCE($4, status), budget_monthly_cents = COALESCE($5, budget_monthly_cents), updated_at = $6 WHERE id = $1 RETURNING id, name, description, status, issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents, require_board_approval_for_new_agents, brand_color, created_at, updated_at",
    )
    .bind(&params.company_id)
    .bind(body.name.as_deref())
    .bind(body.description.as_deref())
    .bind(body.status.as_deref())
    .bind(body.budget_monthly_cents)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Company not found".to_string()))?;
    Ok(Json(row))
}

/// POST /api/companies/:companyId/archive
pub async fn archive_company(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<Company>, (StatusCode, String)> {
    let now = chrono::Utc::now();
    let row = sqlx::query_as::<_, Company>(
        "UPDATE companies SET status = 'archived', updated_at = $2 WHERE id = $1 RETURNING id, name, description, status, issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents, require_board_approval_for_new_agents, brand_color, created_at, updated_at",
    )
    .bind(&params.company_id)
    .bind(now)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Company not found".to_string()))?;
    Ok(Json(row))
}

/// DELETE /api/companies/:companyId
pub async fn delete_company(
    _guard: RequireBoard,
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM companies WHERE id = $1")
        .bind(&params.company_id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Company not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyStats {
    pub company_id: String,
    pub agents_count: i64,
    pub projects_count: i64,
    pub goals_count: i64,
    pub issues_count: i64,
    pub pending_approvals_count: i64,
    pub month_spend_cents: i64,
    pub budget_monthly_cents: i32,
}

/// GET /api/companies/stats — global stats keyed by company id (paperclip parity).
pub async fn list_companies_stats(
    State(pool): State<PgPool>,
) -> Result<Json<std::collections::HashMap<String, CompanyStats>>, (StatusCode, String)> {
    let companies = sqlx::query_scalar::<_, Uuid>("SELECT id FROM companies ORDER BY created_at")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut out = std::collections::HashMap::new();
    for cid in companies {
        let cid_s = cid.to_string();
        let agents_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM agents WHERE company_id = $1")
            .bind(cid)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let projects_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM projects WHERE company_id = $1")
            .bind(cid)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let goals_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM goals WHERE company_id = $1")
            .bind(cid)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let issues_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM issues WHERE company_id = $1")
            .bind(cid)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let pending_approvals_count: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM approvals WHERE company_id = $1 AND status = 'pending'",
        )
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let month_spend_cents: i64 = sqlx::query_scalar(
            "SELECT coalesce(sum(cost_cents), 0)::bigint FROM cost_events WHERE company_id = $1 AND occurred_at >= date_trunc('month', now())",
        )
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let budget_monthly_cents: i32 = sqlx::query_scalar("SELECT budget_monthly_cents FROM companies WHERE id = $1")
            .bind(cid)
            .fetch_one(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        out.insert(
            cid_s.clone(),
            CompanyStats {
                company_id: cid_s.clone(),
                agents_count,
                projects_count,
                goals_count,
                issues_count,
                pending_approvals_count,
                month_spend_cents,
                budget_monthly_cents,
            },
        );
    }
    Ok(Json(out))
}

/// GET /api/companies/:companyId/stats
pub async fn get_company_stats(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<CompanyStats>, (StatusCode, String)> {
    let cid = &params.company_id;
    let _company = sqlx::query_scalar::<_, (i32,)>(
        "SELECT budget_monthly_cents FROM companies WHERE id = $1",
    )
    .bind(cid)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Company not found".to_string()))?;

    let agents_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM agents WHERE company_id = $1")
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let projects_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM projects WHERE company_id = $1")
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let goals_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM goals WHERE company_id = $1")
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let issues_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM issues WHERE company_id = $1")
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let pending_approvals_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM approvals WHERE company_id = $1 AND status = 'pending'",
    )
    .bind(cid)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let month_spend_cents: i64 = sqlx::query_scalar(
        "SELECT coalesce(sum(cost_cents), 0)::bigint FROM cost_events WHERE company_id = $1 AND occurred_at >= date_trunc('month', now())",
    )
    .bind(cid)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let budget_monthly_cents: i32 = sqlx::query_scalar("SELECT budget_monthly_cents FROM companies WHERE id = $1")
        .bind(cid)
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CompanyStats {
        company_id: cid.clone(),
        agents_count,
        projects_count,
        goals_count,
        issues_count,
        pending_approvals_count,
        month_spend_cents,
        budget_monthly_cents,
    }))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyExport {
    pub company: Company,
    pub goals: Vec<serde_json::Value>,
    pub projects: Vec<serde_json::Value>,
    pub agents: Vec<serde_json::Value>,
    pub issues: Vec<serde_json::Value>,
}

/// GET /api/companies/:companyId/export
pub async fn export_company(
    State(pool): State<PgPool>,
    Path(params): Path<CompanyIdParam>,
) -> Result<Json<CompanyExport>, (StatusCode, String)> {
    let company = sqlx::query_as::<_, Company>(
        "SELECT id, name, description, status, issue_prefix, issue_counter, budget_monthly_cents, spent_monthly_cents, require_board_approval_for_new_agents, brand_color, created_at, updated_at FROM companies WHERE id = $1",
    )
    .bind(&params.company_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Company not found".to_string()))?;
    let goals = sqlx::query_as::<_, (serde_json::Value,)>( "SELECT row_to_json(g) FROM (SELECT id, company_id, title, description, level, status, parent_id, owner_agent_id, created_at, updated_at FROM goals WHERE company_id = $1) g")
        .bind(&params.company_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter().map(|(v,)| v).collect();
    let projects = sqlx::query_as::<_, (serde_json::Value,)>("SELECT row_to_json(p) FROM (SELECT id, company_id, goal_id, name, description, status, lead_agent_id, target_date, color, created_at, updated_at FROM projects WHERE company_id = $1) p")
        .bind(&params.company_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter().map(|(v,)| v).collect();
    let agents = sqlx::query_as::<_, (serde_json::Value,)>("SELECT row_to_json(a) FROM (SELECT id, company_id, name, role, title, icon, status, reports_to, capabilities, adapter_type, created_at, updated_at FROM agents WHERE company_id = $1) a")
        .bind(&params.company_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter().map(|(v,)| v).collect();
    let issues = sqlx::query_as::<_, (serde_json::Value,)>("SELECT row_to_json(i) FROM (SELECT id, company_id, project_id, goal_id, parent_id, title, description, status, priority, created_at, updated_at FROM issues WHERE company_id = $1) i")
        .bind(&params.company_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter().map(|(v,)| v).collect();
    Ok(Json(CompanyExport { company, goals, projects, agents, issues }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCompanyBody {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

/// POST /api/companies/import — create company from payload (minimal: name, description, status)
pub async fn import_company(
    State(pool): State<PgPool>,
    Json(body): Json<ImportCompanyBody>,
) -> Result<(StatusCode, Json<Company>), (StatusCode, String)> {
    create_company(
        State(pool),
        Json(CreateCompanyBody {
            name: body.name,
            description: body.description,
            status: body.status,
        }),
    )
    .await
}

/// GET /api/companies when no DB: return 503
pub async fn companies_no_db() -> (StatusCode, &'static str) {
    (StatusCode::SERVICE_UNAVAILABLE, "DATABASE_URL not set; use Node server or set DATABASE_URL")
}
