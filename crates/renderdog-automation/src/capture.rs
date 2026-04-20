use std::{ffi::OsString, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::renderdoccmd::{
    CaptureLaunchError as CommandCaptureLaunchError,
    CaptureLaunchRequest as CommandCaptureLaunchRequest,
};
use crate::{
    OpenCaptureUiError, RenderDocInstallation, ToolInvocationError, default_artifacts_dir,
    resolve_path_from_cwd,
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
    #[error(transparent)]
    Tool(Box<ToolInvocationError>),
    #[error("renderdoccmd returned invalid target ident: {0}")]
    InvalidTargetIdent(i32),
}

impl From<CommandCaptureLaunchError> for LaunchCaptureError {
    fn from(value: CommandCaptureLaunchError) -> Self {
        match value {
            CommandCaptureLaunchError::Tool(err) => Self::Tool(err),
            CommandCaptureLaunchError::InvalidTargetIdent(code) => Self::InvalidTargetIdent(code),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedLaunchCaptureRequest {
    command: CommandCaptureLaunchRequest,
    capture_file_template: Option<String>,
}

impl RenderDocInstallation {
    pub fn launch_capture_in_cwd(
        &self,
        cwd: &Path,
        req: &LaunchCaptureRequest,
    ) -> Result<LaunchCaptureResponse, LaunchCaptureError> {
        let prepared = self.prepare_launch_capture_request(cwd, req)?;
        self.launch_capture_prepared(&prepared)
    }

    pub(crate) fn prepare_launch_capture_request(
        &self,
        cwd: &Path,
        req: &LaunchCaptureRequest,
    ) -> Result<PreparedLaunchCaptureRequest, LaunchCaptureError> {
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

        Ok(PreparedLaunchCaptureRequest {
            command: CommandCaptureLaunchRequest {
                executable: resolve_path_from_cwd(cwd, &req.executable),
                args: req.args.iter().map(OsString::from).collect(),
                working_dir: req
                    .working_dir
                    .as_deref()
                    .map(|path| resolve_path_from_cwd(cwd, path)),
                capture_file_template: capture_file_template.clone(),
            },
            capture_file_template: capture_file_template.map(|path| path.display().to_string()),
        })
    }

    pub(crate) fn launch_capture_prepared(
        &self,
        req: &PreparedLaunchCaptureRequest,
    ) -> Result<LaunchCaptureResponse, LaunchCaptureError> {
        let res = self.launch_capture(&req.command)?;

        Ok(LaunchCaptureResponse {
            target_ident: res.target_ident,
            capture_file_template: req.capture_file_template.clone(),
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

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn make_temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "renderdog-automation-capture-test-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    #[test]
    fn prepare_launch_capture_request_resolves_relative_paths() {
        let cwd = make_temp_dir();
        let install = RenderDocInstallation {
            root_dir: PathBuf::from("/renderdoc"),
            qrenderdoc_exe: PathBuf::from("/renderdoc/qrenderdoc"),
            renderdoccmd_exe: PathBuf::from("/renderdoc/renderdoccmd"),
        };
        let req = LaunchCaptureRequest {
            executable: "bin/app".to_string(),
            args: vec!["--flag".to_string()],
            working_dir: Some("run".to_string()),
            artifacts_dir: Some("captures".to_string()),
            capture_template_name: Some("capture_{frame}".to_string()),
        };

        let prepared = install
            .prepare_launch_capture_request(&cwd, &req)
            .expect("prepare should succeed");
        let expected_template_path = cwd.join("captures").join("capture_{frame}.rdc");

        assert_eq!(prepared.command.executable, cwd.join("bin/app"));
        assert_eq!(prepared.command.args, vec![OsString::from("--flag")]);
        assert_eq!(prepared.command.working_dir, Some(cwd.join("run")));
        assert_eq!(
            prepared.command.capture_file_template,
            Some(expected_template_path.clone())
        );
        assert_eq!(
            prepared.capture_file_template,
            Some(expected_template_path.display().to_string())
        );
        assert!(cwd.join("captures").is_dir());

        std::fs::remove_dir_all(&cwd).expect("cleanup should succeed");
    }

    #[test]
    fn prepare_launch_capture_request_uses_default_artifacts_dir() {
        let cwd = make_temp_dir();
        let install = RenderDocInstallation {
            root_dir: PathBuf::from("/renderdoc"),
            qrenderdoc_exe: PathBuf::from("/renderdoc/qrenderdoc"),
            renderdoccmd_exe: PathBuf::from("/renderdoc/renderdoccmd"),
        };
        let req = LaunchCaptureRequest {
            executable: "app".to_string(),
            args: Vec::new(),
            working_dir: None,
            artifacts_dir: None,
            capture_template_name: None,
        };

        let prepared = install
            .prepare_launch_capture_request(&cwd, &req)
            .expect("prepare should succeed");

        assert_eq!(prepared.command.executable, cwd.join("app"));
        assert!(default_artifacts_dir(&cwd).is_dir());
        assert_eq!(prepared.command.capture_file_template, None);
        assert_eq!(prepared.capture_file_template, None);

        std::fs::remove_dir_all(&cwd).expect("cleanup should succeed");
    }
}
