mod activity;
mod agents;
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
mod members;
mod misc;
mod projects;
mod secrets;
mod workspaces;

use axum::middleware;
use axum::routing::{delete, get, post};
use axum::Router;
use sqlx::PgPool;

use crate::auth;

use self::activity::list_activity;
use self::agents::{create_agent, create_agent_key, get_agent, get_agent_me, get_runtime_state, heartbeat_agent, invoke_agent, list_agent_keys, list_agents, list_config_revisions, list_task_sessions, pause_agent, resume_agent, revoke_agent_key, terminate_agent, update_agent, update_runtime_state};
use self::companies::{archive_company, create_company, delete_company, export_company, get_company, get_company_stats, import_company, list_companies, update_company};
use self::dashboard::dashboard;
use self::goals::{create_goal, delete_goal, get_goal, list_goals, update_goal};
use self::health::health;
use self::issues::{add_issue_attachment, add_issue_comment, checkout_issue, create_issue, delete_issue_attachment, get_issue, link_issue_approval, list_issue_approvals, list_issue_attachments, list_issue_comments, list_issues, release_issue, unlink_issue_approval, update_issue};
use self::projects::{create_project, delete_project, get_project, list_projects, update_project};
use self::workspaces::{create_workspace, delete_workspace, get_workspace, list_workspaces, update_workspace};
use self::approvals::{add_approval_comment, approve_approval, create_approval, get_approval, list_approval_comments, list_approval_issues, list_approvals, reject_approval, request_revision_approval, resubmit_approval};
use self::costs::{create_cost_event, get_costs_by_agent, get_costs_by_project, get_costs_summary};
use self::secrets::{create_secret, delete_secret, get_secret, list_secrets, rotate_secret, update_secret};
use self::assets::{create_asset, delete_asset, get_asset, get_asset_content, list_assets};
use self::invites::{create_invite, get_invite_by_token, list_invites};
use self::members::list_members;
use self::join_requests::{get_join_request, list_join_requests};
use self::heartbeats::{cancel_run, get_run, get_run_log, list_run_events, list_runs, wakeup_agent};
use self::misc::{board_claim, get_session, get_skill, llm_config, sidebar_badges, skills_index};

pub fn api_routes(pool: PgPool) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/companies", get(list_companies).post(create_company))
        .route("/companies/:company_id", get(get_company).patch(update_company).delete(delete_company))
        .route("/companies/:company_id/archive", post(archive_company))
        .route("/companies/:company_id/stats", get(get_company_stats))
        .route("/companies/:company_id/export", get(export_company))
        .route("/companies/import", post(import_company))
        .route("/companies/:company_id/goals", get(list_goals).post(create_goal))
        .route("/goals/:id", get(get_goal).patch(update_goal).delete(delete_goal))
        .route("/companies/:company_id/projects", get(list_projects).post(create_project))
        .route("/projects/:id", get(get_project).patch(update_project).delete(delete_project))
        .route("/projects/:id/workspaces", get(list_workspaces).post(create_workspace))
        .route("/projects/:id/workspaces/:workspace_id", get(get_workspace).patch(update_workspace).delete(delete_workspace))
        .route("/companies/:company_id/agents", get(list_agents).post(create_agent))
        .route("/agents/me", get(get_agent_me))
        .route("/agents/:id", get(get_agent).patch(update_agent))
        .route("/agents/:id/pause", post(pause_agent))
        .route("/agents/:id/resume", post(resume_agent))
        .route("/agents/:id/terminate", post(terminate_agent))
        .route("/agents/:id/keys", get(list_agent_keys).post(create_agent_key))
        .route("/agents/:id/keys/:key_id", delete(revoke_agent_key))
        .route("/agents/:id/heartbeat", post(heartbeat_agent))
        .route("/agents/:id/config-revisions", get(list_config_revisions))
        .route("/agents/:id/runtime-state", get(get_runtime_state).patch(update_runtime_state))
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
        .route("/companies/:company_id/secrets", get(list_secrets).post(create_secret))
        .route("/secrets/:id", get(get_secret).patch(update_secret).delete(delete_secret))
        .route("/secrets/:id/rotate", post(rotate_secret))
        .route("/companies/:company_id/assets", get(list_assets).post(create_asset))
        .route("/assets/:id", get(get_asset).delete(delete_asset))
        .route("/assets/:id/content", get(get_asset_content))
        .route("/companies/:company_id/invites", get(list_invites).post(create_invite))
        .route("/invites/:token", get(get_invite_by_token))
        .route("/companies/:company_id/members", get(list_members))
        .route("/companies/:company_id/join-requests", get(list_join_requests))
        .route("/join-requests/:id", get(get_join_request))
        .route("/companies/:company_id/sidebar-badges", get(sidebar_badges))
        .route("/llm-config", get(llm_config))
        .route("/skills/index", get(skills_index))
        .route("/skills/:id", get(get_skill))
        .route("/board/claim", post(board_claim))
        .route("/auth/get-session", get(get_session))
        .route("/companies/:company_id/cost-events", post(create_cost_event))
        .route("/companies/:company_id/costs/summary", get(get_costs_summary))
        .route("/companies/:company_id/costs/by-agent", get(get_costs_by_agent))
        .route("/companies/:company_id/costs/by-project", get(get_costs_by_project))
        .route("/companies/:company_id/dashboard", get(dashboard))
        .route("/companies/:company_id/activity", get(list_activity))
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth::actor_middleware))
        .with_state(pool)
}

/// Routes when DATABASE_URL is not set (health only; others return 503)
pub fn api_routes_no_db() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/companies", get(companies::companies_no_db))
        .route("/companies/:company_id", get(companies::companies_no_db))
        .route("/companies/:company_id/archive", post(companies::companies_no_db))
        .route("/companies/:company_id/stats", get(companies::companies_no_db))
        .route("/companies/:company_id/goals", get(goals::goals_no_db))
        .route("/companies/:company_id/projects", get(projects::projects_no_db))
        .route("/projects/:id", get(projects::projects_no_db))
        .route("/projects/:id/workspaces", get(workspaces::workspaces_no_db))
        .route("/projects/:id/workspaces/:workspace_id", get(workspaces::workspaces_no_db))
        .route("/companies/:company_id/agents", get(agents::agents_no_db))
        .route("/agents/me", get(agents::agents_no_db))
        .route("/agents/:id", get(agents::agents_no_db))
        .route("/agents/:id/keys", get(agents::agents_no_db))
        .route("/companies/:company_id/issues", get(issues::issues_no_db))
        .route("/companies/:company_id/approvals", get(approvals::approvals_no_db))
        .route("/approvals/:id", get(approvals::approvals_no_db))
        .route("/companies/:company_id/cost-events", post(costs::costs_no_db))
        .route("/companies/:company_id/costs/summary", get(costs::costs_no_db))
        .route("/companies/:company_id/costs/by-agent", get(costs::costs_no_db))
        .route("/companies/:company_id/costs/by-project", get(costs::costs_no_db))
        .route("/companies/:company_id/secrets", get(secrets::secrets_no_db))
        .route("/companies/:company_id/assets", get(assets::assets_no_db))
        .route("/companies/:company_id/invites", get(invites::invites_no_db))
        .route("/companies/:company_id/members", get(members::members_no_db))
        .route("/companies/:company_id/join-requests", get(join_requests::join_requests_no_db))
        .route("/companies/:company_id/sidebar-badges", get(misc::sidebar_badges_no_db))
        .route("/companies/:company_id/export", get(companies::companies_no_db))
        .route("/companies/import", post(companies::companies_no_db))
        .route("/companies/:company_id/dashboard", get(dashboard::dashboard_no_db))
        .route("/companies/:company_id/activity", get(activity::activity_no_db))
}
