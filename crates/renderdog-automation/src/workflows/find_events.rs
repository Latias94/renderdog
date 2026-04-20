use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJobError, QRenderDocJsonJobRequest};

use super::{FindEventsRequest, FindEventsResponse};

#[derive(Debug, Error)]
pub enum FindEventsError {
    #[error("failed to create scripts dir: {0}")]
    CreateScriptsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("failed to parse JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<QRenderDocJsonJobError> for FindEventsError {
    fn from(value: QRenderDocJsonJobError) -> Self {
        match value {
            QRenderDocJsonJobError::CreateScriptsDir(err) => Self::CreateScriptsDir(err),
            QRenderDocJsonJobError::WriteScript(err) => Self::WriteScript(err),
            QRenderDocJsonJobError::WriteRequest(err) => Self::WriteRequest(err),
            QRenderDocJsonJobError::QRenderDocPython(err) => Self::QRenderDocPython(err),
            QRenderDocJsonJobError::ReadResponse(err) => Self::ReadResponse(err),
            QRenderDocJsonJobError::ParseJson(err) => Self::ParseJson(err),
            QRenderDocJsonJobError::ScriptError(err) => Self::ScriptError(err),
        }
    }
}

impl RenderDocInstallation {
    pub fn find_events(
        &self,
        cwd: &Path,
        req: &FindEventsRequest,
    ) -> Result<FindEventsResponse, FindEventsError> {
        let req = FindEventsRequest {
            capture_path: crate::resolve_path_string_from_cwd(cwd, &req.capture_path),
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(
            cwd,
            &QRenderDocJsonJobRequest {
                run_dir_prefix: "find_events",
                script_file_name: "find_events_json.py",
                script_content: FIND_EVENTS_JSON_PY,
                request: &req,
            },
        )
        .map_err(FindEventsError::from)
    }
}

const FIND_EVENTS_JSON_PY: &str = include_str!("../../scripts/find_events_json.py");
