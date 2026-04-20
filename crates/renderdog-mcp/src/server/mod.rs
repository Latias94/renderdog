mod capture;
mod diagnostics;
mod export;
mod find;
mod replay;
mod workflows;

use std::{fmt::Display, path::Path};

use rmcp::{handler::server::router::tool::ToolRouter, tool_handler};

use renderdog_automation as renderdog;

#[derive(Clone)]
pub(crate) struct RenderdogMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for RenderdogMcpServer {}

impl RenderdogMcpServer {
    pub(crate) fn new() -> Self {
        Self {
            tool_router: Self::diagnostics_tool_router()
                + Self::capture_tool_router()
                + Self::export_tool_router()
                + Self::find_tool_router()
                + Self::replay_tool_router()
                + Self::workflows_tool_router(),
        }
    }
}

pub(super) fn tool_result<T, E>(
    tool: &'static str,
    action: &'static str,
    result: Result<T, E>,
) -> Result<T, String>
where
    E: Display,
{
    result.map_err(|err| {
        tracing::error!(tool = tool, action = action, "failed");
        tracing::debug!(tool = tool, action = action, err = %err, "details");
        format!("{action} failed: {err}")
    })
}

pub(super) fn require_installation(
    tool: &'static str,
) -> Result<renderdog::RenderDocInstallation, String> {
    tool_result(
        tool,
        "detect installation",
        renderdog::RenderDocInstallation::detect(),
    )
}

pub(super) fn default_thumbnail_output_path(actions_jsonl_path: &str) -> String {
    let actions_path = Path::new(actions_jsonl_path);
    let basename = actions_path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".actions.jsonl"))
        .or_else(|| actions_path.file_stem().and_then(|name| name.to_str()))
        .unwrap_or("capture");

    actions_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{basename}.thumb.png"))
        .display()
        .to_string()
}
