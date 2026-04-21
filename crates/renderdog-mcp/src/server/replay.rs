use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use renderdog_automation as renderdog;

use super::{CwdRequest, RenderdogMcpServer, ToolRun};

#[tool_router(router = replay_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_replay_list_textures",
        description = "List textures in a .rdc capture via `qrenderdoc --python` replay (headless)."
    )]
    async fn replay_list_textures(
        &self,
        Parameters(req): Parameters<CwdRequest<renderdog::ReplayListTexturesRequest>>,
    ) -> Result<Json<renderdog::ReplayListTexturesResponse>, String> {
        let tool = "renderdoc_replay_list_textures";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        });
        let res = run.with_install_and_cwd("replay list textures", req, |install, cwd, req| {
            install.replay_list_textures(&cwd, &req)
        })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
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
        Parameters(req): Parameters<CwdRequest<renderdog::ReplayPickPixelRequest>>,
    ) -> Result<Json<renderdog::ReplayPickPixelResponse>, String> {
        let tool = "renderdoc_replay_pick_pixel";
        let run = ToolRun::start(tool, || {
            tracing::info!(
                tool = tool,
                capture_path = %req.inner.capture.capture_path,
                texture_index = req.inner.texture_index,
                x = req.inner.x,
                y = req.inner.y,
                "start"
            );
        });
        let res = run.with_install_and_cwd("replay pick pixel", req, |install, cwd, req| {
            install.replay_pick_pixel(&cwd, &req)
        })?;

        tracing::info!(tool = tool, elapsed_ms = run.elapsed_ms(), "ok");
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_replay_save_texture_png",
        description = "Save a texture to PNG from a .rdc capture via `qrenderdoc --python` replay."
    )]
    async fn replay_save_texture_png(
        &self,
        Parameters(req): Parameters<CwdRequest<renderdog::ReplaySaveTexturePngRequest>>,
    ) -> Result<Json<renderdog::ReplaySaveTexturePngResponse>, String> {
        let tool = "renderdoc_replay_save_texture_png";
        let run = ToolRun::start(tool, || {
            tracing::info!(
                tool = tool,
                capture_path = %req.inner.capture.capture_path,
                texture_index = req.inner.texture_index,
                "start"
            );
        });
        let res =
            run.with_install_and_cwd("replay save texture PNG", req, |install, cwd, req| {
                install.replay_save_texture_png(&cwd, &req)
            })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            output_path = %res.output.output_path,
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
        Parameters(req): Parameters<CwdRequest<renderdog::ReplaySaveOutputsPngRequest>>,
    ) -> Result<Json<renderdog::ReplaySaveOutputsPngResponse>, String> {
        let tool = "renderdoc_replay_save_outputs_png";
        let run = ToolRun::start(tool, || {
            tracing::info!(
                tool = tool,
                capture_path = %req.inner.capture.capture_path,
                event_id = req.inner.event_id.unwrap_or(0),
                include_depth = req.inner.include_depth,
                "start"
            );
        });
        let res =
            run.with_install_and_cwd("replay save outputs PNG", req, |install, cwd, req| {
                install.replay_save_outputs_png(&cwd, &req)
            })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            images = res.outputs.len(),
            "ok"
        );
        Ok(Json(res))
    }
}
