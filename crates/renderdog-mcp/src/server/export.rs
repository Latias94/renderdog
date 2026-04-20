use std::{path::Path, time::Instant};

use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use crate::{
    paths::{resolve_base_cwd, resolve_path_from_base},
    types::{
        ExportActionsRequest as ExportActionsToolRequest,
        ExportBindingsIndexRequest as ExportBindingsIndexToolRequest,
        ExportBundleRequest as ExportBundleToolRequest, ExportBundleResponse,
    },
};

use super::{RenderdogMcpServer, default_thumbnail_output_path, require_installation, tool_result};

#[tool_router(router = export_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_export_actions_jsonl",
        description = "Export an action/event tree from a .rdc capture into a searchable JSONL format via `qrenderdoc --python`."
    )]
    async fn export_actions_jsonl(
        &self,
        Parameters(req): Parameters<ExportActionsToolRequest>,
    ) -> Result<Json<renderdog_automation::ExportActionsResponse>, String> {
        let tool = "renderdoc_export_actions_jsonl";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "export actions",
            install.export_actions_jsonl(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            actions_jsonl_path = %res.actions_jsonl_path,
            total_actions = res.total_actions,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_export_bindings_index_jsonl",
        description = "Export a searchable bindings index (`*.bindings.jsonl`) via `qrenderdoc --python` for fast offline querying."
    )]
    async fn export_bindings_index_jsonl(
        &self,
        Parameters(req): Parameters<ExportBindingsIndexToolRequest>,
    ) -> Result<Json<renderdog_automation::ExportBindingsIndexResponse>, String> {
        let tool = "renderdoc_export_bindings_index_jsonl";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "export bindings index",
            install.export_bindings_index_jsonl(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            bindings_jsonl_path = %res.bindings_jsonl_path,
            total_drawcalls = res.total_drawcalls,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_export_bundle_jsonl",
        description = "Export both actions + bindings index from an existing .rdc capture, and optionally save a thumbnail/open qrenderdoc UI."
    )]
    async fn export_bundle_jsonl(
        &self,
        Parameters(req): Parameters<ExportBundleToolRequest>,
    ) -> Result<Json<ExportBundleResponse>, String> {
        let tool = "renderdoc_export_bundle_jsonl";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let bundle = tool_result(
            tool,
            "export bundle",
            install.export_bundle_jsonl(&cwd, &req.inner),
        )?;

        let mut thumbnail_output_path = None;
        if req.save_thumbnail {
            let thumb_path = req
                .thumbnail_output_path
                .map(|path| resolve_path_from_base(&cwd, &path).display().to_string())
                .unwrap_or_else(|| default_thumbnail_output_path(&bundle.actions_jsonl_path));
            if let Some(parent) = Path::new(&thumb_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|err| format!("create thumbnail output dir failed: {err}"))?;
            }
            tool_result(
                tool,
                "save thumbnail",
                install.save_thumbnail(Path::new(&bundle.capture_path), Path::new(&thumb_path)),
            )?;
            thumbnail_output_path = Some(thumb_path);
        }

        let mut ui_pid = None;
        if req.open_capture_ui {
            let child = tool_result(
                tool,
                "open capture UI",
                install.open_capture_in_ui(Path::new(&bundle.capture_path)),
            )?;
            ui_pid = Some(child.id());
        }

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            capture_path = %bundle.capture_path,
            actions_jsonl_path = %bundle.actions_jsonl_path,
            bindings_jsonl_path = %bundle.bindings_jsonl_path,
            total_actions = bundle.total_actions,
            total_drawcalls = bundle.total_drawcalls,
            "ok"
        );

        Ok(Json(ExportBundleResponse {
            bundle,
            thumbnail_output_path,
            ui_pid,
        }))
    }
}
