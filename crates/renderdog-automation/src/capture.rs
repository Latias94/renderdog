use std::{ffi::OsString, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::renderdoccmd::{
    CaptureLaunchCommand as CommandCaptureLaunchCommand,
    CaptureLaunchError as CommandCaptureLaunchError,
    CaptureLaunchOutcome as CommandCaptureLaunchOutcome,
};
use crate::{
    CaptureInput, CaptureRef, OpenCaptureUiError, OutputFile, OutputRef, RenderDocInstallation,
    TargetControlRef, ToolInvocationError, default_artifacts_dir, resolve_path_from_cwd,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureTargetRequest {
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
pub struct CaptureLaunchReport {
    #[serde(flatten)]
    pub target: TargetControlRef,
    pub capture_file_template: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SaveThumbnailRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: OutputFile,
}

pub type SaveThumbnailResponse = OutputRef;
pub type OpenCaptureUiRequest = CaptureInput;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenCaptureUiResponse {
    #[serde(flatten)]
    pub capture: CaptureRef,
    pub pid: u32,
}

#[derive(Debug, Error)]
pub enum CaptureTargetError {
    #[error("failed to create artifacts dir: {0}")]
    CreateArtifactsDir(std::io::Error),
    #[error("failed to launch capture target: {0}")]
    LaunchTarget(Box<ToolInvocationError>),
    #[error("renderdoccmd returned invalid target ident: {0}")]
    InvalidTargetIdent(i32),
}

impl From<CommandCaptureLaunchError> for CaptureTargetError {
    fn from(value: CommandCaptureLaunchError) -> Self {
        match value {
            CommandCaptureLaunchError::Tool(err) => Self::LaunchTarget(err),
            CommandCaptureLaunchError::InvalidTargetIdent(code) => Self::InvalidTargetIdent(code),
        }
    }
}

impl From<CommandCaptureLaunchOutcome> for CaptureLaunchReport {
    fn from(value: CommandCaptureLaunchOutcome) -> Self {
        Self {
            target: TargetControlRef::new(value.target_ident),
            capture_file_template: value.capture_file_template,
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedCaptureTarget {
    command: CommandCaptureLaunchCommand,
}

impl CaptureTargetRequest {
    pub(crate) fn resolved_in_cwd(
        &self,
        cwd: &Path,
    ) -> Result<ResolvedCaptureTarget, CaptureTargetError> {
        let artifacts_dir = self
            .artifacts_dir
            .as_deref()
            .map(|path| resolve_path_from_cwd(cwd, path))
            .unwrap_or_else(|| default_artifacts_dir(cwd));
        std::fs::create_dir_all(&artifacts_dir).map_err(CaptureTargetError::CreateArtifactsDir)?;

        let capture_file_template = self
            .capture_template_name
            .as_deref()
            .map(|name| artifacts_dir.join(format!("{name}.rdc")));

        Ok(ResolvedCaptureTarget {
            command: CommandCaptureLaunchCommand {
                executable: resolve_path_from_cwd(cwd, &self.executable),
                args: self.args.iter().map(OsString::from).collect(),
                working_dir: self
                    .working_dir
                    .as_deref()
                    .map(|path| resolve_path_from_cwd(cwd, path)),
                capture_file_template,
            },
        })
    }
}

impl RenderDocInstallation {
    pub(crate) fn launch_capture_target(
        &self,
        req: &ResolvedCaptureTarget,
    ) -> Result<CaptureLaunchReport, CaptureTargetError> {
        Ok(self.launch_capture(&req.command)?.into())
    }

    pub fn save_thumbnail_in_cwd(
        &self,
        cwd: &Path,
        req: &SaveThumbnailRequest,
    ) -> Result<SaveThumbnailResponse, std::io::Error> {
        let capture = req.capture.normalized_in_cwd(cwd);
        let output = req.output.resolved_in_cwd(cwd);
        let capture_path = Path::new(&capture.capture_path);
        let output_path = Path::new(&output.output_path);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        self.save_thumbnail(capture_path, output_path)?;

        Ok(OutputRef::new(output_path.display().to_string()))
    }

    pub fn open_capture_ui_in_cwd(
        &self,
        cwd: &Path,
        req: &OpenCaptureUiRequest,
    ) -> Result<OpenCaptureUiResponse, OpenCaptureUiError> {
        let capture = req.normalized_in_cwd(cwd);
        let capture_path = Path::new(&capture.capture_path);
        let child = self.open_capture_in_ui(capture_path)?;

        Ok(OpenCaptureUiResponse {
            capture: CaptureRef::new(capture_path.display().to_string()),
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

    use crate::renderdoccmd::CaptureLaunchError as CommandCaptureLaunchError;
    use serde_json::{Value, json};

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
    fn capture_target_request_resolves_relative_paths() {
        let cwd = make_temp_dir();
        let req = CaptureTargetRequest {
            executable: "bin/app".to_string(),
            args: vec!["--flag".to_string()],
            working_dir: Some("run".to_string()),
            artifacts_dir: Some("captures".to_string()),
            capture_template_name: Some("capture_{frame}".to_string()),
        };

        let resolved = req.resolved_in_cwd(&cwd).expect("resolve should succeed");
        let expected_template_path = cwd.join("captures").join("capture_{frame}.rdc");

        assert_eq!(resolved.command.executable, cwd.join("bin/app"));
        assert_eq!(resolved.command.args, vec![OsString::from("--flag")]);
        assert_eq!(resolved.command.working_dir, Some(cwd.join("run")));
        assert_eq!(
            resolved.command.capture_file_template,
            Some(expected_template_path.clone())
        );
        assert!(cwd.join("captures").is_dir());

        std::fs::remove_dir_all(&cwd).expect("cleanup should succeed");
    }

    #[test]
    fn capture_target_request_uses_default_artifacts_dir() {
        let cwd = make_temp_dir();
        let req = CaptureTargetRequest {
            executable: "app".to_string(),
            args: Vec::new(),
            working_dir: None,
            artifacts_dir: None,
            capture_template_name: None,
        };

        let resolved = req.resolved_in_cwd(&cwd).expect("resolve should succeed");

        assert_eq!(resolved.command.executable, cwd.join("app"));
        assert!(default_artifacts_dir(&cwd).is_dir());
        assert_eq!(resolved.command.capture_file_template, None);

        std::fs::remove_dir_all(&cwd).expect("cleanup should succeed");
    }

    #[test]
    fn capture_target_request_deserializes_optional_fields_with_defaults() {
        let req: CaptureTargetRequest = serde_json::from_value(json!({
            "executable": "game"
        }))
        .expect("deserialize should succeed");

        assert_eq!(req.executable, "game");
        assert!(req.args.is_empty());
        assert_eq!(req.working_dir, None);
        assert_eq!(req.artifacts_dir, None);
        assert_eq!(req.capture_template_name, None);
    }

    #[test]
    fn capture_target_error_preserves_launch_context() {
        let err = CaptureTargetError::from(CommandCaptureLaunchError::InvalidTargetIdent(-1));
        assert!(matches!(err, CaptureTargetError::InvalidTargetIdent(-1)));

        let err = CaptureTargetError::from(CommandCaptureLaunchError::Tool(Box::new(
            ToolInvocationError::NonZeroExit {
                program: "renderdoccmd".to_string(),
                args: vec!["capture".to_string()],
                cwd: None,
                status: 1,
                stdout: String::new(),
                stderr: "boom".to_string(),
            },
        )));
        assert!(matches!(err, CaptureTargetError::LaunchTarget(_)));
    }

    #[test]
    fn save_thumbnail_request_serializes_capture_and_output_flattened() {
        let req = SaveThumbnailRequest {
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            output: OutputFile {
                output_path: "/tmp/frame.png".to_string(),
            },
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(
            object.get("output_path"),
            Some(&Value::String("/tmp/frame.png".to_string()))
        );
        assert!(!object.contains_key("capture"));
        assert!(!object.contains_key("output"));
    }

    #[test]
    fn open_capture_ui_request_serializes_capture_flattened() {
        let req = OpenCaptureUiRequest {
            capture_path: "/tmp/frame.rdc".to_string(),
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert!(!object.contains_key("capture"));
    }

    #[test]
    fn save_thumbnail_response_serializes_output_flattened() {
        let response = SaveThumbnailResponse::new("/tmp/frame.png");

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("output_path"),
            Some(&Value::String("/tmp/frame.png".to_string()))
        );
    }

    #[test]
    fn capture_launch_report_serializes_flat_fields() {
        let response = CaptureLaunchReport {
            target: TargetControlRef::new(7),
            capture_file_template: Some("/tmp/capture.rdc".to_string()),
            stdout: "stdout".to_string(),
            stderr: "stderr".to_string(),
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("target_ident"),
            Some(&Value::Number(7_u32.into()))
        );
        assert_eq!(
            object.get("capture_file_template"),
            Some(&Value::String("/tmp/capture.rdc".to_string()))
        );
        assert_eq!(
            object.get("stdout"),
            Some(&Value::String("stdout".to_string()))
        );
        assert_eq!(
            object.get("stderr"),
            Some(&Value::String("stderr".to_string()))
        );
        assert!(!object.contains_key("target"));
    }

    #[test]
    fn open_capture_ui_response_serializes_capture_flattened() {
        let response = OpenCaptureUiResponse {
            capture: CaptureRef::new("/tmp/frame.rdc"),
            pid: 123,
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(object.get("pid"), Some(&Value::Number(123_u32.into())));
        assert!(!object.contains_key("capture"));
    }
}
