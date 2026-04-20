use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{
    OpenCaptureUiError, RenderDocInstallation, normalize_capture_path, resolve_path_from_cwd,
};

use super::{
    CaptureInput, CapturePostActionOutputs, CapturePostActions, ExportActionsError,
    ExportActionsRequest, ExportBindingsIndexError, ExportBindingsIndexRequest,
    ExportBundleRequest, ExportBundleResponse,
};

#[derive(Debug, Error)]
pub enum ExportBundleError {
    #[error("export actions failed: {0}")]
    Actions(#[from] ExportActionsError),
    #[error("export bindings index failed: {0}")]
    Bindings(#[from] ExportBindingsIndexError),
    #[error("save thumbnail failed: {0}")]
    SaveThumbnail(std::io::Error),
    #[error("open capture UI failed: {0}")]
    OpenCaptureUi(#[from] OpenCaptureUiError),
}

fn default_thumbnail_output_path(actions_jsonl_path: &str) -> PathBuf {
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

        let post_actions = self.apply_capture_post_actions(
            cwd,
            Path::new(&capture_path),
            &actions.actions_jsonl_path,
            &req.post_actions,
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
            post_actions,
        })
    }

    fn apply_capture_post_actions(
        &self,
        cwd: &Path,
        capture_path: &Path,
        actions_jsonl_path: &str,
        post_actions: &CapturePostActions,
    ) -> Result<CapturePostActionOutputs, ExportBundleError> {
        let mut outputs = CapturePostActionOutputs::default();

        if post_actions.save_thumbnail {
            let output_path = post_actions
                .thumbnail_output_path
                .as_deref()
                .map(|path| resolve_path_from_cwd(cwd, path))
                .unwrap_or_else(|| default_thumbnail_output_path(actions_jsonl_path));

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(ExportBundleError::SaveThumbnail)?;
            }

            self.save_thumbnail(capture_path, &output_path)
                .map_err(ExportBundleError::SaveThumbnail)?;
            outputs.thumbnail_output_path = Some(output_path.display().to_string());
        }

        if post_actions.open_capture_ui {
            let child = self.open_capture_in_ui(capture_path)?;
            outputs.ui_pid = Some(child.id());
        }

        Ok(outputs)
    }
}
