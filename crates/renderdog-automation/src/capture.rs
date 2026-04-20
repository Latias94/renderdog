use std::{ffi::OsString, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CaptureLaunchError, CaptureLaunchRequest as CommandCaptureLaunchRequest, OpenCaptureUiError,
    RenderDocInstallation, default_artifacts_dir, resolve_path_from_cwd,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LaunchCaptureRequest {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub artifacts_dir: Option<String>,
    #[serde(default)]
    pub capture_template_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LaunchCaptureResponse {
    pub target_ident: u32,
    pub capture_file_template: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveThumbnailRequest {
    pub capture_path: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveThumbnailResponse {
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenCaptureUiRequest {
    pub capture_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenCaptureUiResponse {
    pub capture_path: String,
    pub pid: u32,
}

#[derive(Debug, Error)]
pub enum LaunchCaptureError {
    #[error("failed to create artifacts dir: {0}")]
    CreateArtifactsDir(std::io::Error),
    #[error("launch capture failed: {0}")]
    Launch(#[from] CaptureLaunchError),
}

impl RenderDocInstallation {
    pub fn launch_capture_in_cwd(
        &self,
        cwd: &Path,
        req: &LaunchCaptureRequest,
    ) -> Result<LaunchCaptureResponse, LaunchCaptureError> {
        let artifacts_dir = req
            .artifacts_dir
            .as_deref()
            .map(|path| resolve_path_from_cwd(cwd, path))
            .unwrap_or_else(|| default_artifacts_dir(cwd));
        std::fs::create_dir_all(&artifacts_dir).map_err(LaunchCaptureError::CreateArtifactsDir)?;

        let capture_file_template = req
            .capture_template_name
            .as_deref()
            .map(|name| artifacts_dir.join(format!("{name}.rdc")));

        let res = self.launch_capture(&CommandCaptureLaunchRequest {
            executable: resolve_path_from_cwd(cwd, &req.executable),
            args: req.args.iter().map(OsString::from).collect(),
            working_dir: req
                .working_dir
                .as_deref()
                .map(|path| resolve_path_from_cwd(cwd, path)),
            capture_file_template: capture_file_template.clone(),
        })?;

        Ok(LaunchCaptureResponse {
            target_ident: res.target_ident,
            capture_file_template: capture_file_template.map(|path| path.display().to_string()),
            stdout: res.stdout,
            stderr: res.stderr,
        })
    }

    pub fn save_thumbnail_in_cwd(
        &self,
        cwd: &Path,
        req: &SaveThumbnailRequest,
    ) -> Result<SaveThumbnailResponse, std::io::Error> {
        let capture_path = resolve_path_from_cwd(cwd, &req.capture_path);
        let output_path = resolve_path_from_cwd(cwd, &req.output_path);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.save_thumbnail(&capture_path, &output_path)?;

        Ok(SaveThumbnailResponse {
            output_path: output_path.display().to_string(),
        })
    }

    pub fn open_capture_ui_in_cwd(
        &self,
        cwd: &Path,
        req: &OpenCaptureUiRequest,
    ) -> Result<OpenCaptureUiResponse, OpenCaptureUiError> {
        let capture_path = resolve_path_from_cwd(cwd, &req.capture_path);
        let child = self.open_capture_in_ui(&capture_path)?;

        Ok(OpenCaptureUiResponse {
            capture_path: capture_path.display().to_string(),
            pid: child.id(),
        })
    }
}
