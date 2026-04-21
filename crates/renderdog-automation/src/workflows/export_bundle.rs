use std::path::{Path, PathBuf};

use thiserror::Error;

use super::{
    BundleExportArtifacts, CapturePostActionOutputs, CapturePostActions, ExportActionsRequest,
    ExportBindingsIndexRequest, ExportBundleRequest, ExportBundleResponse,
};
use crate::{OpenCaptureUiError, QRenderDocJobError, RenderDocInstallation, resolve_path_from_cwd};

#[derive(Debug, Error)]
pub enum ExportBundleError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("export job failed: {0}")]
    Job(#[from] QRenderDocJobError),
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

#[derive(Debug, Clone)]
struct PreparedCapturePostActions {
    thumbnail_output_path: Option<PathBuf>,
    open_capture_ui: bool,
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

impl PreparedCapturePostActions {
    fn resolved_in_cwd(
        cwd: &Path,
        actions_jsonl_path: &str,
        post_actions: &CapturePostActions,
    ) -> Self {
        let thumbnail_output_path = post_actions.save_thumbnail.then(|| {
            post_actions
                .thumbnail_output_path
                .as_deref()
                .map(|path| resolve_path_from_cwd(cwd, path))
                .unwrap_or_else(|| default_thumbnail_output_path(actions_jsonl_path))
        });

        Self {
            thumbnail_output_path,
            open_capture_ui: post_actions.open_capture_ui,
        }
    }
}

impl RenderDocInstallation {
    pub fn export_bundle(
        &self,
        cwd: &Path,
        req: &ExportBundleRequest,
    ) -> Result<ExportBundleResponse, ExportBundleError> {
        let prepared = PreparedBundleExport::resolved_in_cwd(cwd, req)
            .map_err(ExportBundleError::CreateOutputDir)?;

        let actions = self.export_actions(cwd, &prepared.actions)?;
        let bindings = self.export_bindings_index(cwd, &prepared.bindings)?;

        let post_action_plan = PreparedCapturePostActions::resolved_in_cwd(
            cwd,
            &actions.artifacts.actions_jsonl_path,
            &prepared.post_actions,
        );
        let post_actions =
            self.run_capture_post_actions(prepared.capture_path(), &post_action_plan)?;

        Ok(prepared.into_response(actions, bindings, post_actions))
    }

    fn run_capture_post_actions(
        &self,
        capture_path: &Path,
        plan: &PreparedCapturePostActions,
    ) -> Result<CapturePostActionOutputs, ExportBundleError> {
        let mut outputs = CapturePostActionOutputs::default();

        if let Some(output_path) = &plan.thumbnail_output_path {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent).map_err(ExportBundleError::SaveThumbnail)?;
            }

            self.save_thumbnail(capture_path, &output_path)
                .map_err(ExportBundleError::SaveThumbnail)?;
            outputs.thumbnail_output_path = Some(output_path.display().to_string());
        }

        if plan.open_capture_ui {
            let child = self.open_capture_in_ui(capture_path)?;
            outputs.ui_pid = Some(child.id());
        }

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{PreparedCapturePostActions, default_thumbnail_output_path};
    use crate::CapturePostActions;

    #[test]
    fn default_thumbnail_output_path_uses_actions_artifact_basename() {
        let path = default_thumbnail_output_path("/tmp/out/frame.actions.jsonl");

        assert_eq!(path, Path::new("/tmp/out/frame.thumb.png"));
    }

    #[test]
    fn prepared_post_actions_resolve_explicit_thumbnail_path_in_cwd() {
        let plan = PreparedCapturePostActions::resolved_in_cwd(
            Path::new("/tmp/project"),
            "/tmp/out/frame.actions.jsonl",
            &CapturePostActions {
                save_thumbnail: true,
                thumbnail_output_path: Some("thumbs/frame.png".to_string()),
                open_capture_ui: true,
            },
        );

        assert_eq!(
            plan.thumbnail_output_path,
            Some(PathBuf::from("/tmp/project/thumbs/frame.png"))
        );
        assert!(plan.open_capture_ui);
    }

    #[test]
    fn prepared_post_actions_default_thumbnail_path_follows_actions_output() {
        let plan = PreparedCapturePostActions::resolved_in_cwd(
            Path::new("/tmp/project"),
            "/tmp/out/frame.actions.jsonl",
            &CapturePostActions {
                save_thumbnail: true,
                thumbnail_output_path: None,
                open_capture_ui: false,
            },
        );

        assert_eq!(
            plan.thumbnail_output_path,
            Some(PathBuf::from("/tmp/out/frame.thumb.png"))
        );
        assert!(!plan.open_capture_ui);
    }

    #[test]
    fn prepared_post_actions_ignore_thumbnail_path_when_thumbnail_disabled() {
        let plan = PreparedCapturePostActions::resolved_in_cwd(
            Path::new("/tmp/project"),
            "/tmp/out/frame.actions.jsonl",
            &CapturePostActions {
                save_thumbnail: false,
                thumbnail_output_path: Some("thumbs/frame.png".to_string()),
                open_capture_ui: true,
            },
        );

        assert_eq!(plan.thumbnail_output_path, None);
        assert!(plan.open_capture_ui);
    }
}
