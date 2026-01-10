use std::path::Path;

use thiserror::Error;

use crate::scripting::{QRenderDocJsonEnvelope, create_qrenderdoc_run_dir};
use crate::{
    QRenderDocPythonRequest, RenderDocInstallation, default_scripts_dir, write_script_file,
};

use super::{FindEventsRequest, FindEventsResponse, util::remove_if_exists};

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

impl From<crate::QRenderDocPythonError> for FindEventsError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn find_events(
        &self,
        cwd: &Path,
        req: &FindEventsRequest,
    ) -> Result<FindEventsResponse, FindEventsError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(FindEventsError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("find_events_json.py");
        write_script_file(&script_path, FIND_EVENTS_JSON_PY)
            .map_err(FindEventsError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "find_events")
            .map_err(FindEventsError::CreateScriptsDir)?;
        let request_path = run_dir.join("find_events_json.request.json");
        let response_path = run_dir.join("find_events_json.response.json");
        remove_if_exists(&response_path).map_err(FindEventsError::WriteRequest)?;

        let req = FindEventsRequest {
            capture_path: crate::resolve_path_string_from_cwd(cwd, &req.capture_path),
            ..req.clone()
        };

        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(FindEventsError::ParseJson)?,
        )
        .map_err(FindEventsError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;
        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(FindEventsError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<FindEventsResponse> =
            serde_json::from_slice(&bytes).map_err(FindEventsError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| FindEventsError::ScriptError("missing result".into()))
        } else {
            Err(FindEventsError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }
}

const FIND_EVENTS_JSON_PY: &str = include_str!("../../scripts/find_events_json.py");
