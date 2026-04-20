mod capture;
mod diagnostics;
mod export;
mod find;
mod replay;
mod workflows;

use std::{fmt::Display, path::PathBuf, time::Instant};

use rmcp::{handler::server::router::tool::ToolRouter, tool_handler};

use renderdog_automation as renderdog;

use crate::types::CwdRequest;

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

pub(super) struct ToolRun {
    tool: &'static str,
    start: Instant,
}

impl ToolRun {
    pub(super) fn start<F>(tool: &'static str, log_start: F) -> Self
    where
        F: FnOnce(),
    {
        log_start();
        Self {
            tool,
            start: Instant::now(),
        }
    }

    pub(super) fn with_install<T, E, F>(&self, action: &'static str, op: F) -> Result<T, String>
    where
        E: Display,
        F: FnOnce(&renderdog::RenderDocInstallation) -> Result<T, E>,
    {
        let install = require_installation(self.tool)?;
        self.result(action, op(&install))
    }

    pub(super) fn with_install_and_cwd<Req, T, E, F>(
        &self,
        action: &'static str,
        req: CwdRequest<Req>,
        op: F,
    ) -> Result<T, String>
    where
        E: Display,
        F: FnOnce(&renderdog::RenderDocInstallation, PathBuf, Req) -> Result<T, E>,
    {
        let install = require_installation(self.tool)?;
        let (cwd, req) = req.into_parts()?;
        self.result(action, op(&install, cwd, req))
    }

    pub(super) fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    fn result<T, E>(&self, action: &'static str, result: Result<T, E>) -> Result<T, String>
    where
        E: Display,
    {
        tool_result(self.tool, action, result)
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
