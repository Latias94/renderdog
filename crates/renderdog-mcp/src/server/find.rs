use std::time::Instant;

use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use renderdog_automation as renderdog;

use crate::types::{
    FindEventsAndSaveOutputsPngRequest as FindEventsAndSaveOutputsPngToolRequest,
    FindEventsAndSaveOutputsPngResponse, FindEventsRequest as FindEventsToolRequest,
};

use super::{RenderdogMcpServer, require_installation, tool_result};

#[tool_router(router = find_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_find_events",
        description = "Find matching action events (event_id + marker_path) in a .rdc capture via `qrenderdoc --python`. Useful for quickly locating event IDs for later replay tools."
    )]
    async fn find_events(
        &self,
        Parameters(req): Parameters<FindEventsToolRequest>,
    ) -> Result<Json<renderdog::FindEventsResponse>, String> {
        let tool = "renderdoc_find_events";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let (cwd, req) = req.into_parts()?;
        let res = tool_result(tool, "find events", install.find_events(&cwd, &req))?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            total_matches = res.total_matches,
            truncated = res.truncated,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_find_events_and_save_outputs_png",
        description = "One-shot helper: find events by marker/name and save pipeline output textures (color RTs + optional depth) to PNG."
    )]
    async fn find_events_and_save_outputs_png(
        &self,
        Parameters(req): Parameters<FindEventsAndSaveOutputsPngToolRequest>,
    ) -> Result<Json<FindEventsAndSaveOutputsPngResponse>, String> {
        let tool = "renderdoc_find_events_and_save_outputs_png";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let (cwd, req) = req.into_parts()?;
        let res = tool_result(
            tool,
            "find events and save outputs PNG",
            install.find_events_and_save_outputs_png(&cwd, &req),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            selected_event_id = res.selected_event_id,
            images = res.replay.outputs.len(),
            "ok"
        );
        Ok(Json(res))
    }
}
