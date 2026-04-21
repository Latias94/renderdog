use std::path::{Path, PathBuf};

use thiserror::Error;

use super::{
    BundleExportArtifacts, CapturePostActionOutputs, CapturePostActions, ExportActionsRequest,
    ExportBindingsIndexRequest, ExportBundleRequest, ExportBundleResponse,
};
use crate::{
    OpenCaptureUiError, QRenderDocJsonError, RenderDocInstallation, resolve_path_from_cwd,
};

#[derive(Debug, Error)]
pub enum ExportBundleError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("export job failed: {0}")]
    Job(#[from] QRenderDocJsonError),
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

#[derive(Debug, Clone)]
struct PreparedBundleExport {
    actions: ExportActionsRequest,
    bindings: ExportBindingsIndexRequest,
    post_actions: CapturePostActions,
}

impl PreparedBundleExport {
    fn resolved_in_cwd(cwd: &Path, req: &ExportBundleRequest) -> Result<Self, std::io::Error> {
        let (capture, output) = req.output.normalized_for_capture(cwd, &req.capture)?;

        Ok(Self {
            actions: ExportActionsRequest {
                capture: capture.clone(),
                output: output.clone(),
                drawcall_scope: req.bundle.drawcall_scope,
                filter: req.bundle.filter.clone(),
            },
            bindings: ExportBindingsIndexRequest {
                capture,
                output,
                filter: req.bundle.filter.clone(),
                bindings: req.bundle.bindings,
            },
            post_actions: req.bundle.post_actions.clone(),
        })
    }

    fn capture_path(&self) -> &Path {
        Path::new(&self.actions.capture.capture_path)
    }

    fn into_response(
        self,
        actions: super::ExportActionsResponse,
        bindings: super::ExportBindingsIndexResponse,
        post_actions: CapturePostActionOutputs,
    ) -> ExportBundleResponse {
        ExportBundleResponse::from_parts(
            self.actions.capture.capture_path,
            BundleExportArtifacts::from_parts(actions, bindings, post_actions),
        )
    }
}

impl RenderDocInstallation {
    pub fn export_bundle_jsonl(
        &self,
        cwd: &Path,
        req: &ExportBundleRequest,
    ) -> Result<ExportBundleResponse, ExportBundleError> {
        let prepared = PreparedBundleExport::resolved_in_cwd(cwd, req)
            .map_err(ExportBundleError::CreateOutputDir)?;

        let actions = self.export_actions_jsonl(cwd, &prepared.actions)?;
        let bindings = self.export_bindings_index_jsonl(cwd, &prepared.bindings)?;

        let post_actions = self.apply_capture_post_actions(
            cwd,
            prepared.capture_path(),
            &actions.artifacts.actions_jsonl_path,
            &prepared.post_actions,
        )?;

        Ok(prepared.into_response(actions, bindings, post_actions))
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
