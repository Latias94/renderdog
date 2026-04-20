use std::time::Instant;

use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use renderdog_automation as renderdog;

use crate::{
    paths::resolve_base_cwd,
    types::{
        ReplayListTexturesRequest as ReplayListTexturesToolRequest,
        ReplayPickPixelRequest as ReplayPickPixelToolRequest,
        ReplaySaveOutputsPngRequest as ReplaySaveOutputsPngToolRequest,
        ReplaySaveTexturePngRequest as ReplaySaveTexturePngToolRequest,
    },
};

use super::{RenderdogMcpServer, require_installation, tool_result};

#[tool_router(router = replay_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_replay_list_textures",
        description = "List textures in a .rdc capture via `qrenderdoc --python` replay (headless)."
    )]
    async fn replay_list_textures(
        &self,
        Parameters(req): Parameters<ReplayListTexturesToolRequest>,
    ) -> Result<Json<renderdog::ReplayListTexturesResponse>, String> {
        let tool = "renderdoc_replay_list_textures";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay list textures",
            install.replay_list_textures(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            textures = res.textures.len(),
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_replay_pick_pixel",
        description = "Pick a pixel from a texture in a .rdc capture via `qrenderdoc --python` replay."
    )]
    async fn replay_pick_pixel(
        &self,
        Parameters(req): Parameters<ReplayPickPixelToolRequest>,
    ) -> Result<Json<renderdog::ReplayPickPixelResponse>, String> {
        let tool = "renderdoc_replay_pick_pixel";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            capture_path = %req.inner.capture_path,
            texture_index = req.inner.texture_index,
            x = req.inner.x,
            y = req.inner.y,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay pick pixel",
            install.replay_pick_pixel(&cwd, &req.inner),
        )?;

        tracing::info!(tool = tool, elapsed_ms = start.elapsed().as_millis(), "ok");
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_replay_save_texture_png",
        description = "Save a texture to PNG from a .rdc capture via `qrenderdoc --python` replay."
    )]
    async fn replay_save_texture_png(
        &self,
        Parameters(req): Parameters<ReplaySaveTexturePngToolRequest>,
    ) -> Result<Json<renderdog::ReplaySaveTexturePngResponse>, String> {
        let tool = "renderdoc_replay_save_texture_png";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            capture_path = %req.inner.capture_path,
            texture_index = req.inner.texture_index,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay save texture PNG",
            install.replay_save_texture_png(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            output_path = %res.output_path,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_replay_save_outputs_png",
        description = "Save current pipeline output textures (color RTs + optional depth) to PNG via `qrenderdoc --python` replay (headless)."
    )]
    async fn replay_save_outputs_png(
        &self,
        Parameters(req): Parameters<ReplaySaveOutputsPngToolRequest>,
    ) -> Result<Json<renderdog::ReplaySaveOutputsPngResponse>, String> {
        let tool = "renderdoc_replay_save_outputs_png";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            capture_path = %req.inner.capture_path,
            event_id = req.inner.event_id.unwrap_or(0),
            include_depth = req.inner.include_depth,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay save outputs PNG",
            install.replay_save_outputs_png(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            images = res.outputs.len(),
            "ok"
        );
        Ok(Json(res))
    }
}
