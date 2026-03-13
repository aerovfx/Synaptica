mod activity;
mod adapters;
mod admin;
mod agents;
mod boards;
mod company_spaces;
mod company_departments;
mod company_posts;
mod company_classes;
mod dms;
mod events;
mod approvals;
mod assets;
mod companies;
mod costs;
mod dashboard;
mod goals;
mod heartbeats;
mod health;
mod invites;
mod issues;
mod join_requests;
mod labels;
mod llms;
mod members;
mod misc;
mod projects;
mod secrets;
mod sprints;
mod workspaces;

use axum::extract::FromRef;
use axum::middleware;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::config::Config;
use crate::metrics::MetricsGauge;
use crate::runner::RunnerLimits;
use crate::auth;
use sqlx::PgPool;

use self::activity::{create_activity, list_activity, list_issue_activity, list_issue_runs, list_run_issues};
use self::agents::{create_agent, create_agent_key, delete_agent, get_agent, get_agent_me, get_config_revision, get_org, get_runtime_state, heartbeat_agent, invoke_agent, list_agent_configurations, list_agent_keys, list_agents, list_config_revisions, list_task_sessions, pause_agent, reset_runtime_session, resume_agent, revoke_agent_key, rollback_config_revision, terminate_agent, update_agent, update_runtime_state};
use self::companies::{archive_company, create_company, delete_company, export_company, get_company, get_company_stats, import_company, list_companies, list_companies_stats, openclaw_invite_prompt, openfang_invite_prompt, update_company};
use self::dashboard::dashboard;
use self::goals::{create_goal, delete_goal, get_goal, list_goals, update_goal};
use self::health::health;
use self::issues::{add_issue_attachment, add_issue_comment, checkout_issue, create_issue, delete_issue_attachment, get_issue, link_issue_approval, list_issue_approvals, list_issue_attachments, list_issue_comments, list_issues, mark_issue_read, release_issue, unlink_issue_approval, update_issue};
use self::boards::{create_board, create_board_column, delete_board, delete_board_column, get_board, list_board_columns, list_boards, update_board, update_board_column};
use self::company_spaces::{create_company_space, delete_company_space, get_company_space, list_company_spaces, update_company_space};
use self::company_departments::{create_company_department, delete_company_department, get_company_department, list_company_departments, update_company_department};
use self::company_posts::{create_company_post, delete_company_post, get_company_post, list_company_posts, update_company_post};
use self::company_classes::{create_company_class, delete_company_class, get_company_class, list_company_classes, update_company_class};
use self::dms::{
    create_dms_document, list_dms_all, list_dms_documents, list_dms_incoming, list_dms_outgoing,
    upload_dms_document,
};
use self::projects::{create_project, delete_project, get_project, list_projects, update_project};
use self::sprints::{create_sprint, delete_sprint, get_sprint, list_sprints, update_sprint};
use self::workspaces::{create_workspace, delete_workspace, get_workspace, list_workspaces, update_workspace};
use self::approvals::{add_approval_comment, approve_approval, create_approval, get_approval, list_approval_comments, list_approval_issues, list_approvals, reject_approval, request_revision_approval, resubmit_approval};
use self::costs::{create_cost_event, get_costs_by_agent, get_costs_by_project, get_costs_summary, patch_agent_budgets, patch_company_budgets};
use self::secrets::{create_secret, delete_secret, get_secret, list_secret_providers, list_secrets, rotate_secret, update_secret};
use self::assets::{create_asset, delete_asset, get_asset, get_asset_content, list_assets};
use self::invites::{create_invite, get_invite_by_token, get_invite_onboarding, get_invite_onboarding_txt, list_invites, revoke_invite};
use self::join_requests::{approve_join_request, claim_join_request_api_key, get_join_request, list_join_requests, reject_join_request};
use self::members::{list_members, update_member_permissions};
use self::admin::{demote_instance_admin, get_user_company_access, promote_instance_admin, put_user_company_access};
use self::heartbeats::{cancel_run, get_run, get_run_log, list_run_events, list_runs, wakeup_agent};
use self::labels::{create_label, delete_label, list_labels};
use self::events::{company_events_ws, company_events_ws_no_db, LiveEventBus};
use self::adapters::{get_adapter_models, post_test_environment};
use self::llms::{llms_agent_configuration_adapter, llms_agent_configuration_index, llms_agent_icons};
use self::misc::{board_claim, get_board_claim, get_session, get_skill, llm_config, post_board_claim_claim, sidebar_badges, skills_index};

/// Shared state for API routes (pool + optional runner semaphore + runner limits + metrics + live bus).
#[derive(Clone)]
pub struct ApiState {
    pub pool: PgPool,
    pub runner_semaphore: Option<Arc<Semaphore>>,
    pub runner_limits: RunnerLimits,
    /// Gauge for active adapter runs (used by /api/metrics and runner).
    pub metrics_active_runs: Arc<MetricsGauge>,
    /// In-memory bus for company live events (WebSocket).
    pub live_bus: Arc<LiveEventBus>,
}

impl FromRef<ApiState> for PgPool {
    fn from_ref(state: &ApiState) -> PgPool {
        state.pool.clone()
    }
}

/// Build shared API state from pool and config (used when DATABASE_URL is set).
pub fn build_api_state(pool: PgPool, config: &Config) -> ApiState {
    let runner_semaphore = if config.runner_max_concurrent_runs > 0 {
        Some(Arc::new(Semaphore::new(config.runner_max_concurrent_runs)))
    } else {
        None
    };
    let runner_limits = RunnerLimits {
        max_http_timeout_ms: config.runner_http_max_timeout_ms,
        max_process_timeout_secs: config.runner_process_max_timeout_secs,
    };
    ApiState {
        pool,
        runner_semaphore,
        runner_limits,
        metrics_active_runs: Arc::new(MetricsGauge::new()),
        live_bus: Arc::new(LiveEventBus::new()),
    }
}

async fn serve_metrics(axum::extract::State(state): axum::extract::State<ApiState>) -> axum::response::Response {
    crate::metrics::metrics_handler(Some(state.metrics_active_runs.get())).await
}

async fn serve_metrics_no_db() -> axum::response::Response {
    crate::metrics::metrics_handler(None).await
}

pub fn api_routes(state: ApiState) -> Router<ApiState> {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(serve_metrics))
        .route("/companies", get(list_companies).post(create_company))
        .route("/companies/stats", get(list_companies_stats))
        .route("/companies/:company_id", get(get_company).patch(update_company).delete(delete_company))
        .route("/companies/:company_id/archive", post(archive_company))
        .route("/companies/:company_id/openclaw/invite-prompt", post(openclaw_invite_prompt))
        .route("/companies/:company_id/openfang/invite-prompt", post(openfang_invite_prompt))
        .route("/companies/:company_id/stats", get(get_company_stats))
        .route("/companies/:company_id/export", get(export_company))
        .route("/companies/import", post(import_company))
        .route("/companies/:company_id/goals", get(list_goals).post(create_goal))
        .route("/goals/:id", get(get_goal).patch(update_goal).delete(delete_goal))
        .route("/companies/:company_id/projects", get(list_projects).post(create_project))
        .route("/projects/:id", get(get_project).patch(update_project).delete(delete_project))
        .route("/companies/:company_id/boards", get(list_boards).post(create_board))
        .route("/companies/:company_id/boards/:board_id", get(get_board).patch(update_board).delete(delete_board))
        .route("/companies/:company_id/spaces", get(list_company_spaces).post(create_company_space))
        .route("/companies/:company_id/spaces/:space_id", get(get_company_space).patch(update_company_space).delete(delete_company_space))
        .route("/companies/:company_id/departments", get(list_company_departments).post(create_company_department))
        .route("/companies/:company_id/departments/:department_id", get(get_company_department).patch(update_company_department).delete(delete_company_department))
        .route("/companies/:company_id/posts", get(list_company_posts).post(create_company_post))
        .route("/companies/:company_id/posts/:post_id", get(get_company_post).patch(update_company_post).delete(delete_company_post))
        .route("/companies/:company_id/classes", get(list_company_classes).post(create_company_class))
        .route("/companies/:company_id/classes/:class_id", get(get_company_class).patch(update_company_class).delete(delete_company_class))
        .route("/companies/:company_id/dms", get(list_dms_all))
        .route("/companies/:company_id/dms/documents", get(list_dms_documents).post(create_dms_document))
        .route("/companies/:company_id/dms/documents/upload", post(upload_dms_document))
        .route("/companies/:company_id/dms/incoming", get(list_dms_incoming))
        .route("/companies/:company_id/dms/outgoing", get(list_dms_outgoing))
        .route("/companies/:company_id/boards/:board_id/columns", get(list_board_columns).post(create_board_column))
        .route("/companies/:company_id/boards/:board_id/columns/:column_id", patch(update_board_column).delete(delete_board_column))
        .route("/companies/:company_id/boards/:board_id/sprints", get(list_sprints).post(create_sprint))
        .route("/companies/:company_id/boards/:board_id/sprints/:sprint_id", get(get_sprint).patch(update_sprint).delete(delete_sprint))
        .route("/projects/:id/workspaces", get(list_workspaces).post(create_workspace))
        .route("/projects/:id/workspaces/:workspace_id", get(get_workspace).patch(update_workspace).delete(delete_workspace))
        .route("/companies/:company_id/agents", get(list_agents).post(create_agent))
        .route("/companies/:company_id/org", get(get_org))
        .route("/companies/:company_id/agent-configurations", get(list_agent_configurations))
        .route("/companies/:company_id/adapters/:adapter_type/models", get(get_adapter_models))
        .route("/companies/:company_id/adapters/:adapter_type/test-environment", post(post_test_environment))
        .route("/agents/me", get(get_agent_me))
        .route("/agents/:id", get(get_agent).patch(update_agent).delete(delete_agent))
        .route("/agents/:id/pause", post(pause_agent))
        .route("/agents/:id/resume", post(resume_agent))
        .route("/agents/:id/terminate", post(terminate_agent))
        .route("/agents/:id/keys", get(list_agent_keys).post(create_agent_key))
        .route("/agents/:id/keys/:key_id", delete(revoke_agent_key))
        .route("/agents/:id/heartbeat", post(heartbeat_agent))
        .route("/agents/:id/config-revisions", get(list_config_revisions))
        .route("/agents/:id/config-revisions/:revision_id", get(get_config_revision))
        .route("/agents/:id/config-revisions/:revision_id/rollback", post(rollback_config_revision))
        .route("/agents/:id/runtime-state", get(get_runtime_state).patch(update_runtime_state))
        .route("/agents/:id/runtime-state/reset-session", post(reset_runtime_session))
        .route("/agents/:id/task-sessions", get(list_task_sessions))
        .route("/agents/:id/invoke", post(invoke_agent))
        .route("/agents/:id/wakeup", post(wakeup_agent))
        .route("/companies/:company_id/heartbeat-runs", get(list_runs))
        .route("/heartbeat-runs/:id", get(get_run))
        .route("/heartbeat-runs/:id/events", get(list_run_events))
        .route("/heartbeat-runs/:id/log", get(get_run_log))
        .route("/heartbeat-runs/:id/cancel", post(cancel_run))
        .route("/companies/:company_id/issues", get(list_issues).post(create_issue))
        .route("/issues/:id", get(get_issue).patch(update_issue))
        .route("/issues/:id/checkout", post(checkout_issue))
        .route("/issues/:id/release", post(release_issue))
        .route("/issues/:id/read", post(mark_issue_read))
        .route("/issues/:id/comments", get(list_issue_comments).post(add_issue_comment))
        .route("/issues/:id/approvals", get(list_issue_approvals).post(link_issue_approval))
        .route("/issues/:id/approvals/:approval_id", delete(unlink_issue_approval))
        .route("/issues/:id/attachments", get(list_issue_attachments).post(add_issue_attachment))
        .route("/issues/:id/attachments/:attachment_id", delete(delete_issue_attachment))
        .route("/companies/:company_id/approvals", get(list_approvals).post(create_approval))
        .route("/approvals/:id", get(get_approval))
        .route("/approvals/:id/approve", post(approve_approval))
        .route("/approvals/:id/reject", post(reject_approval))
        .route("/approvals/:id/request-revision", post(request_revision_approval))
        .route("/approvals/:id/resubmit", post(resubmit_approval))
        .route("/approvals/:id/comments", get(list_approval_comments).post(add_approval_comment))
        .route("/approvals/:id/issues", get(list_approval_issues))
        .route("/companies/:company_id/labels", get(list_labels).post(create_label))
        .route("/labels/:label_id", delete(delete_label))
        .route("/companies/:company_id/secrets", get(list_secrets).post(create_secret))
        .route("/companies/:company_id/secret-providers", get(list_secret_providers))
        .route("/secrets/:id", get(get_secret).patch(update_secret).delete(delete_secret))
        .route("/secrets/:id/rotate", post(rotate_secret))
        .route("/companies/:company_id/assets", get(list_assets).post(create_asset))
        .route("/assets/:id", get(get_asset).delete(delete_asset))
        .route("/assets/:id/content", get(get_asset_content))
        .route("/companies/:company_id/invites", get(list_invites).post(create_invite))
        .route("/invites/:token", get(get_invite_by_token))
        .route("/invites/:token/onboarding", get(get_invite_onboarding))
        .route("/invites/:token/onboarding.txt", get(get_invite_onboarding_txt))
        .route("/invites/:invite_id/revoke", post(revoke_invite))
        .route("/companies/:company_id/members", get(list_members))
        .route("/companies/:company_id/members/:member_id/permissions", patch(update_member_permissions))
        .route("/companies/:company_id/events/ws", get(company_events_ws))
        .route("/companies/:company_id/join-requests", get(list_join_requests))
        .route("/companies/:company_id/join-requests/:request_id/approve", post(approve_join_request))
        .route("/companies/:company_id/join-requests/:request_id/reject", post(reject_join_request))
        .route("/join-requests/:id", get(get_join_request))
        .route("/join-requests/:id/claim-api-key", post(claim_join_request_api_key))
        .route("/admin/users/:user_id/company-access", get(get_user_company_access).put(put_user_company_access))
        .route("/admin/users/:user_id/promote-instance-admin", post(promote_instance_admin))
        .route("/admin/users/:user_id/demote-instance-admin", post(demote_instance_admin))
        .route("/companies/:company_id/sidebar-badges", get(sidebar_badges))
        .route("/llm-config", get(llm_config))
        .route("/llms/agent-configuration.txt", get(llms_agent_configuration_index))
        .route("/llms/agent-configuration/:adapter_type", get(llms_agent_configuration_adapter))
        .route("/llms/agent-icons.txt", get(llms_agent_icons))
        .route("/skills/index", get(skills_index))
        .route("/skills/:id", get(get_skill))
        .route("/board/claim", post(board_claim))
        .route("/board-claim/:token", get(get_board_claim))
        .route("/board-claim/:token/claim", post(post_board_claim_claim))
        .route("/auth/get-session", get(get_session))
        .route("/companies/:company_id/cost-events", post(create_cost_event))
        .route("/companies/:company_id/costs/summary", get(get_costs_summary))
        .route("/companies/:company_id/costs/by-agent", get(get_costs_by_agent))
        .route("/companies/:company_id/costs/by-project", get(get_costs_by_project))
        .route("/companies/:company_id/budgets", patch(patch_company_budgets))
        .route("/agents/:id/budgets", patch(patch_agent_budgets))
        .route("/companies/:company_id/dashboard", get(dashboard))
        .route("/companies/:company_id/activity", get(list_activity).post(create_activity))
        .route("/issues/:id/activity", get(list_issue_activity))
        .route("/issues/:id/runs", get(list_issue_runs))
        .route("/heartbeat-runs/:id/issues", get(list_run_issues))
        .route_layer(middleware::from_fn(crate::metrics::metrics_middleware))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth::actor_middleware))
        .route_layer(middleware::from_fn(auth::require_agent_company_scope))
        .with_state(state)
}

/// Routes when DATABASE_URL is not set (health only; others return 503)
pub fn api_routes_no_db() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(serve_metrics_no_db))
        .route("/companies", get(companies::companies_no_db).post(companies::companies_no_db))
        .route("/companies/stats", get(companies::companies_no_db))
        .route("/companies/:company_id", get(companies::companies_no_db).patch(companies::companies_no_db).delete(companies::companies_no_db))
        .route("/companies/:company_id/archive", post(companies::companies_no_db))
        .route("/companies/:company_id/stats", get(companies::companies_no_db))
        .route("/companies/:company_id/goals", get(goals::goals_no_db))
        .route("/companies/:company_id/projects", get(projects::projects_no_db))
        .route("/projects/:id", get(projects::projects_no_db))
        .route("/projects/:id/workspaces", get(workspaces::workspaces_no_db))
        .route("/projects/:id/workspaces/:workspace_id", get(workspaces::workspaces_no_db))
        .route("/companies/:company_id/agents", get(agents::agents_no_db))
        .route("/companies/:company_id/org", get(agents::agents_no_db))
        .route("/companies/:company_id/agent-configurations", get(agents::agents_no_db))
        .route("/companies/:company_id/adapters/:adapter_type/models", get(get_adapter_models))
        .route("/companies/:company_id/adapters/:adapter_type/test-environment", post(post_test_environment))
        .route("/agents/me", get(agents::agents_no_db))
        .route("/agents/:id", get(agents::agents_no_db).patch(agents::agents_no_db).delete(agents::agents_no_db))
        .route("/agents/:id/config-revisions", get(agents::agents_no_db))
        .route("/agents/:id/config-revisions/:revision_id", get(agents::agents_no_db))
        .route("/agents/:id/config-revisions/:revision_id/rollback", post(agents::agents_no_db))
        .route("/agents/:id/runtime-state", get(agents::agents_no_db).patch(agents::agents_no_db))
        .route("/agents/:id/runtime-state/reset-session", post(agents::agents_no_db))
        .route("/agents/:id/keys", get(agents::agents_no_db))
        .route("/companies/:company_id/labels", get(labels::labels_no_db).post(labels::labels_no_db))
        .route("/labels/:label_id", delete(labels::labels_no_db))
        .route("/companies/:company_id/issues", get(issues::issues_no_db).post(issues::issues_no_db))
        .route("/issues/:id/read", post(issues::issues_no_db))
        .route("/companies/:company_id/approvals", get(approvals::approvals_no_db))
        .route("/approvals/:id", get(approvals::approvals_no_db))
        .route("/companies/:company_id/cost-events", post(costs::costs_no_db))
        .route("/companies/:company_id/costs/summary", get(costs::costs_no_db))
        .route("/companies/:company_id/costs/by-agent", get(costs::costs_no_db))
        .route("/companies/:company_id/costs/by-project", get(costs::costs_no_db))
        .route("/companies/:company_id/budgets", patch(costs::costs_no_db))
        .route("/agents/:id/budgets", patch(costs::costs_no_db))
        .route("/companies/:company_id/secrets", get(secrets::secrets_no_db))
        .route("/companies/:company_id/secret-providers", get(secrets::secrets_no_db))
        .route("/companies/:company_id/assets", get(assets::assets_no_db))
        .route("/companies/:company_id/invites", get(invites::invites_no_db))
        .route("/companies/:company_id/members", get(members::members_no_db))
        .route("/companies/:company_id/events/ws", get(company_events_ws_no_db))
        .route("/companies/:company_id/join-requests", get(join_requests::join_requests_no_db))
        .route("/companies/:company_id/spaces", get(company_spaces::company_spaces_no_db).post(company_spaces::company_spaces_no_db))
        .route("/companies/:company_id/spaces/:space_id", get(company_spaces::company_spaces_no_db).patch(company_spaces::company_spaces_no_db).delete(company_spaces::company_spaces_no_db))
        .route("/companies/:company_id/departments", get(company_departments::company_departments_no_db).post(company_departments::company_departments_no_db))
        .route("/companies/:company_id/departments/:department_id", get(company_departments::company_departments_no_db).patch(company_departments::company_departments_no_db).delete(company_departments::company_departments_no_db))
        .route("/companies/:company_id/posts", get(company_posts::company_posts_no_db).post(company_posts::company_posts_no_db))
        .route("/companies/:company_id/posts/:post_id", get(company_posts::company_posts_no_db).patch(company_posts::company_posts_no_db).delete(company_posts::company_posts_no_db))
        .route("/companies/:company_id/classes", get(company_classes::company_classes_no_db).post(company_classes::company_classes_no_db))
        .route("/companies/:company_id/classes/:class_id", get(company_classes::company_classes_no_db).patch(company_classes::company_classes_no_db).delete(company_classes::company_classes_no_db))
        .route("/companies/:company_id/dms", get(dms::dms_no_db))
        .route("/companies/:company_id/dms/documents", get(dms::dms_no_db).post(dms::dms_no_db))
        .route("/companies/:company_id/dms/documents/upload", post(dms::dms_no_db))
        .route("/companies/:company_id/dms/incoming", get(dms::dms_no_db))
        .route("/companies/:company_id/dms/outgoing", get(dms::dms_no_db))
        .route("/companies/:company_id/sidebar-badges", get(misc::sidebar_badges_no_db))
        .route("/llms/agent-configuration.txt", get(llms::llms_agent_configuration_index))
        .route("/llms/agent-configuration/:adapter_type", get(llms::llms_agent_configuration_adapter))
        .route("/llms/agent-icons.txt", get(llms::llms_agent_icons))
        .route("/companies/:company_id/export", get(companies::companies_no_db))
        .route("/companies/import", post(companies::companies_no_db))
        .route("/companies/:company_id/dashboard", get(dashboard::dashboard_no_db))
        .route("/companies/:company_id/activity", get(activity::activity_no_db).post(activity::activity_no_db))
        .route("/issues/:id/activity", get(activity::activity_no_db))
        .route("/issues/:id/runs", get(activity::activity_no_db))
        .route("/heartbeat-runs/:id/issues", get(activity::activity_no_db))
}
