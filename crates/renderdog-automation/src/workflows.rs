use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::resolve_path_string_from_cwd;
use crate::scripting::{QRenderDocJsonEnvelope, create_qrenderdoc_run_dir};
use crate::{
    QRenderDocPythonRequest, RenderDocInstallation, default_scripts_dir, write_script_file,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureRequest {
    pub host: String,
    pub target_ident: u32,
    pub num_frames: u32,
    pub timeout_s: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureResponse {
    pub capture_path: String,
    pub frame_number: u32,
    pub api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportActionsRequest {
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,
    pub only_drawcalls: bool,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportActionsResponse {
    pub capture_path: String,
    pub actions_jsonl_path: String,
    pub summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsRequest {
    pub capture_path: String,
    pub only_drawcalls: bool,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,
    pub max_results: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FoundEvent {
    pub event_id: u32,
    pub parent_event_id: Option<u32>,
    pub depth: u32,
    pub name: String,
    pub flags: u64,
    pub flags_names: Vec<String>,
    pub marker_path: Vec<String>,
    pub marker_path_joined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsResponse {
    pub capture_path: String,
    pub total_matches: u64,
    pub truncated: bool,
    pub first_event_id: Option<u32>,
    pub last_event_id: Option<u32>,
    pub matches: Vec<FoundEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBindingsIndexRequest {
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,
    pub include_cbuffers: bool,
    pub include_outputs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBindingsIndexResponse {
    pub capture_path: String,
    pub bindings_jsonl_path: String,
    pub summary_json_path: String,
    pub total_drawcalls: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBundleRequest {
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,

    pub only_drawcalls: bool,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,

    pub include_cbuffers: bool,
    pub include_outputs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBundleResponse {
    pub capture_path: String,

    pub actions_jsonl_path: String,
    pub actions_summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,

    pub bindings_jsonl_path: String,
    pub bindings_summary_json_path: String,
    pub total_drawcalls: u64,
}

#[derive(Debug, Error)]
pub enum TriggerCaptureError {
    #[error("failed to create artifacts dir: {0}")]
    CreateArtifactsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to parse capture JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<crate::QRenderDocPythonError> for TriggerCaptureError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ExportActionsError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to parse export JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

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

impl From<crate::QRenderDocPythonError> for ExportActionsError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ExportBindingsIndexError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to parse export JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

#[derive(Debug, Error)]
pub enum ExportBundleError {
    #[error("export actions failed: {0}")]
    Actions(#[from] ExportActionsError),
    #[error("export bindings index failed: {0}")]
    Bindings(#[from] ExportBindingsIndexError),
}

fn remove_if_exists(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

impl From<crate::QRenderDocPythonError> for ExportBindingsIndexError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

impl From<crate::QRenderDocPythonError> for FindEventsError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn trigger_capture_via_target_control(
        &self,
        cwd: &Path,
        req: &TriggerCaptureRequest,
    ) -> Result<TriggerCaptureResponse, TriggerCaptureError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(TriggerCaptureError::CreateArtifactsDir)?;

        let script_path = scripts_dir.join("trigger_capture.py");
        write_script_file(&script_path, TRIGGER_CAPTURE_PY)
            .map_err(TriggerCaptureError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "trigger_capture")
            .map_err(TriggerCaptureError::CreateArtifactsDir)?;
        let request_path = run_dir.join("trigger_capture.request.json");
        let response_path = run_dir.join("trigger_capture.response.json");
        remove_if_exists(&response_path).map_err(TriggerCaptureError::WriteRequest)?;
        std::fs::write(
            &request_path,
            serde_json::to_vec(req).map_err(TriggerCaptureError::ParseJson)?,
        )
        .map_err(TriggerCaptureError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;
        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(TriggerCaptureError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<TriggerCaptureResponse> =
            serde_json::from_slice(&bytes).map_err(TriggerCaptureError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| TriggerCaptureError::ScriptError("missing result".into()))
        } else {
            Err(TriggerCaptureError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    pub fn export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, ExportActionsError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ExportActionsError::CreateOutputDir)?;

        let script_path = scripts_dir.join("export_actions_jsonl.py");
        write_script_file(&script_path, EXPORT_ACTIONS_JSONL_PY)
            .map_err(ExportActionsError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "export_actions_jsonl")
            .map_err(ExportActionsError::CreateOutputDir)?;
        let request_path = run_dir.join("export_actions_jsonl.request.json");
        let response_path = run_dir.join("export_actions_jsonl.response.json");
        remove_if_exists(&response_path).map_err(ExportActionsError::WriteRequest)?;

        let req = ExportActionsRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_dir: resolve_path_string_from_cwd(cwd, &req.output_dir),
            ..req.clone()
        };

        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ExportActionsError::ParseJson)?,
        )
        .map_err(ExportActionsError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;
        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(ExportActionsError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ExportActionsResponse> =
            serde_json::from_slice(&bytes).map_err(ExportActionsError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ExportActionsError::ScriptError("missing result".into()))
        } else {
            Err(ExportActionsError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

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
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
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

    pub fn export_bindings_index_jsonl(
        &self,
        cwd: &Path,
        req: &ExportBindingsIndexRequest,
    ) -> Result<ExportBindingsIndexResponse, ExportBindingsIndexError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ExportBindingsIndexError::CreateOutputDir)?;

        let script_path = scripts_dir.join("export_bindings_index_jsonl.py");
        write_script_file(&script_path, EXPORT_BINDINGS_INDEX_JSONL_PY)
            .map_err(ExportBindingsIndexError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "export_bindings_index_jsonl")
            .map_err(ExportBindingsIndexError::CreateOutputDir)?;
        let request_path = run_dir.join("export_bindings_index_jsonl.request.json");
        let response_path = run_dir.join("export_bindings_index_jsonl.response.json");
        remove_if_exists(&response_path).map_err(ExportBindingsIndexError::WriteRequest)?;

        let req = ExportBindingsIndexRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_dir: resolve_path_string_from_cwd(cwd, &req.output_dir),
            ..req.clone()
        };

        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ExportBindingsIndexError::ParseJson)?,
        )
        .map_err(ExportBindingsIndexError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;
        let _ = result;
        let bytes =
            std::fs::read(&response_path).map_err(ExportBindingsIndexError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ExportBindingsIndexResponse> =
            serde_json::from_slice(&bytes).map_err(ExportBindingsIndexError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ExportBindingsIndexError::ScriptError("missing result".into()))
        } else {
            Err(ExportBindingsIndexError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

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

const TRIGGER_CAPTURE_PY: &str = include_str!("../scripts/trigger_capture.py");

const FIND_EVENTS_JSON_PY: &str = include_str!("../scripts/find_events_json.py");

const EXPORT_ACTIONS_JSONL_PY: &str = include_str!("../scripts/export_actions_jsonl.py");

const EXPORT_BINDINGS_INDEX_JSONL_PY: &str =
    include_str!("../scripts/export_bindings_index_jsonl.py");
