use std::path::Path;

use thiserror::Error;

use crate::{RenderDocInstallation, resolve_path_string_from_cwd};

use super::{
    ExportActionsError, ExportActionsRequest, ExportBindingsIndexError, ExportBindingsIndexRequest,
    ExportBundleRequest, ExportBundleResponse,
};

#[derive(Debug, Error)]
pub enum ExportBundleError {
    #[error("export actions failed: {0}")]
    Actions(#[from] ExportActionsError),
    #[error("export bindings index failed: {0}")]
    Bindings(#[from] ExportBindingsIndexError),
}

impl RenderDocInstallation {
    pub fn export_bundle_jsonl(
        &self,
        cwd: &Path,
        req: &ExportBundleRequest,
    ) -> Result<ExportBundleResponse, ExportBundleError> {
        let capture_path = resolve_path_string_from_cwd(cwd, &req.capture_path);
        let output_dir = resolve_path_string_from_cwd(cwd, &req.output_dir);

        let actions = self.export_actions_jsonl(
            cwd,
            &ExportActionsRequest {
                capture_path: capture_path.clone(),
                output_dir: output_dir.clone(),
                basename: req.basename.clone(),
                only_drawcalls: req.only_drawcalls,
                marker_prefix: req.marker_prefix.clone(),
                event_id_min: req.event_id_min,
                event_id_max: req.event_id_max,
                name_contains: req.name_contains.clone(),
                marker_contains: req.marker_contains.clone(),
                case_sensitive: req.case_sensitive,
            },
        )?;

        let bindings = self.export_bindings_index_jsonl(
            cwd,
            &ExportBindingsIndexRequest {
                capture_path: capture_path.clone(),
                output_dir: output_dir.clone(),
                basename: req.basename.clone(),
                marker_prefix: req.marker_prefix.clone(),
                event_id_min: req.event_id_min,
                event_id_max: req.event_id_max,
                name_contains: req.name_contains.clone(),
                marker_contains: req.marker_contains.clone(),
                case_sensitive: req.case_sensitive,
                include_cbuffers: req.include_cbuffers,
                include_outputs: req.include_outputs,
            },
        )?;

        Ok(ExportBundleResponse {
            capture_path,

            actions_jsonl_path: actions.actions_jsonl_path,
            actions_summary_json_path: actions.summary_json_path,
            total_actions: actions.total_actions,
            drawcall_actions: actions.drawcall_actions,

            bindings_jsonl_path: bindings.bindings_jsonl_path,
            bindings_summary_json_path: bindings.summary_json_path,
            total_drawcalls: bindings.total_drawcalls,
        })
    }
}
