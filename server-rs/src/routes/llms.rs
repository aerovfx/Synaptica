//! LLM docs: GET /api/llms/agent-configuration.txt, agent-configuration/:adapter_type.txt, agent-icons.txt

use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::IntoResponse;

/// GET /api/llms/agent-configuration.txt — index of adapter config docs (stub).
pub async fn llms_agent_configuration_index() -> impl IntoResponse {
    const BODY: &str = r#"# Synaptica Agent Configuration Index

Installed adapters (Rust stub):
- No adapters registered in this build. Use Node server for full adapter list.

Related API endpoints:
- GET /api/companies/:companyId/agent-configurations
- GET /api/agents/:id/configuration
- POST /api/companies/:companyId/agent-hires

Agent identity references:
- GET /llms/agent-icons.txt

Notes:
- Sensitive values are redacted in configuration read APIs.
- New hires may be created in pending_approval state depending on company settings.
"#;
    (StatusCode::OK, [(CONTENT_TYPE, "text/plain; charset=utf-8")], BODY)
}

/// GET /api/llms/agent-icons.txt — list of allowed agent icon names (stub).
pub async fn llms_agent_icons() -> impl IntoResponse {
    const BODY: &str = r#"# Synaptica Agent Icon Names

Set the `icon` field on hire/create payloads to one of:
- bot
- cpu
- brain
- zap
- rocket
- code
- terminal
- shield
- eye
- search
- wrench
- hammer
- lightbulb
- sparkles
- star
- heart
- flame
- bug
- cog
- database
- globe
- lock
- mail
- message-square
- file-code
- git-branch
- package
- puzzle
- target
- wand
- atom
- circuit-board
- radar
- swords
- telescope
- microscope
- crown
- gem
- hexagon
- pentagon
- fingerprint

Example:
{ "name": "SearchOps", "role": "researcher", "icon": "search" }
"#;
    (StatusCode::OK, [(CONTENT_TYPE, "text/plain; charset=utf-8")], BODY)
}

#[derive(serde::Deserialize)]
pub struct AdapterTypeParam {
    pub adapter_type: String,
}

/// GET /api/llms/agent-configuration/:adapter_type — adapter-specific config doc (stub).
pub async fn llms_agent_configuration_adapter(
    Path(params): Path<AdapterTypeParam>,
) -> impl IntoResponse {
    let body = format!(
        "# {} agent configuration\n\nNo adapter-specific documentation registered in this build.\n",
        params.adapter_type
    );
    (StatusCode::OK, [(CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}
