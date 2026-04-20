use std::path::Path;

use thiserror::Error;

use crate::{RenderDocInstallation, normalize_capture_path};

use super::{
    CaptureInput, ExportActionsError, ExportActionsRequest, ExportBindingsIndexError,
    ExportBindingsIndexRequest, ExportBundleRequest, ExportBundleResponse,
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
        let capture_path = normalize_capture_path(cwd, &req.capture.capture_path);

        let actions = self.export_actions_jsonl(
            cwd,
            &ExportActionsRequest {
                capture: CaptureInput {
                    capture_path: capture_path.clone(),
                },
                output: req.output.clone(),
                drawcall_scope: req.drawcall_scope,
                filter: req.filter.clone(),
            },
        )?;

        let bindings = self.export_bindings_index_jsonl(
            cwd,
            &ExportBindingsIndexRequest {
                capture: CaptureInput {
                    capture_path: capture_path.clone(),
                },
                output: req.output.clone(),
                filter: req.filter.clone(),
                bindings: req.bindings,
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
