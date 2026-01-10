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
pub struct ReplayListTexturesRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayTextureInfo {
    pub index: u32,
    pub resource_id: u64,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mips: u32,
    pub arraysize: u32,
    pub ms_samp: u32,
    pub byte_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub textures: Vec<ReplayTextureInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayPickPixelRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayPickPixelResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub x: u32,
    pub y: u32,
    pub rgba: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveTexturePngRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveTexturePngResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveOutputsPngRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub output_dir: String,
    pub basename: String,
    pub include_depth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySavedImage {
    pub kind: String,
    pub index: Option<u32>,
    pub resource_id: u64,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveOutputsPngResponse {
    pub capture_path: String,
    pub event_id: u32,
    pub outputs: Vec<ReplaySavedImage>,
}

#[derive(Debug, Error)]
pub enum ReplayListTexturesError {
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

impl From<crate::QRenderDocPythonError> for ReplayListTexturesError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ReplayPickPixelError {
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

impl From<crate::QRenderDocPythonError> for ReplayPickPixelError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ReplaySaveTexturePngError {
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

impl From<crate::QRenderDocPythonError> for ReplaySaveTexturePngError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

#[derive(Debug, Error)]
pub enum ReplaySaveOutputsPngError {
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

impl From<crate::QRenderDocPythonError> for ReplaySaveOutputsPngError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

fn remove_if_exists(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

impl RenderDocInstallation {
    pub fn replay_list_textures(
        &self,
        cwd: &Path,
        req: &ReplayListTexturesRequest,
    ) -> Result<ReplayListTexturesResponse, ReplayListTexturesError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ReplayListTexturesError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_list_textures_json.py");
        write_script_file(&script_path, REPLAY_LIST_TEXTURES_JSON_PY)
            .map_err(ReplayListTexturesError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_list_textures")
            .map_err(ReplayListTexturesError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_list_textures_json.request.json");
        let response_path = run_dir.join("replay_list_textures_json.response.json");
        remove_if_exists(&response_path).map_err(ReplayListTexturesError::WriteRequest)?;

        let req = ReplayListTexturesRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplayListTexturesError::ParseJson)?,
        )
        .map_err(ReplayListTexturesError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(ReplayListTexturesError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplayListTexturesResponse> =
            serde_json::from_slice(&bytes).map_err(ReplayListTexturesError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplayListTexturesError::ScriptError("missing result".into()))
        } else {
            Err(ReplayListTexturesError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    pub fn replay_pick_pixel(
        &self,
        cwd: &Path,
        req: &ReplayPickPixelRequest,
    ) -> Result<ReplayPickPixelResponse, ReplayPickPixelError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ReplayPickPixelError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_pick_pixel_json.py");
        write_script_file(&script_path, REPLAY_PICK_PIXEL_JSON_PY)
            .map_err(ReplayPickPixelError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_pick_pixel")
            .map_err(ReplayPickPixelError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_pick_pixel_json.request.json");
        let response_path = run_dir.join("replay_pick_pixel_json.response.json");
        remove_if_exists(&response_path).map_err(ReplayPickPixelError::WriteRequest)?;

        let req = ReplayPickPixelRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplayPickPixelError::ParseJson)?,
        )
        .map_err(ReplayPickPixelError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(ReplayPickPixelError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplayPickPixelResponse> =
            serde_json::from_slice(&bytes).map_err(ReplayPickPixelError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplayPickPixelError::ScriptError("missing result".into()))
        } else {
            Err(ReplayPickPixelError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    pub fn replay_save_texture_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveTexturePngRequest,
    ) -> Result<ReplaySaveTexturePngResponse, ReplaySaveTexturePngError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir)
            .map_err(ReplaySaveTexturePngError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_save_texture_png_json.py");
        write_script_file(&script_path, REPLAY_SAVE_TEXTURE_PNG_JSON_PY)
            .map_err(ReplaySaveTexturePngError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_save_texture_png")
            .map_err(ReplaySaveTexturePngError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_save_texture_png_json.request.json");
        let response_path = run_dir.join("replay_save_texture_png_json.response.json");
        remove_if_exists(&response_path).map_err(ReplaySaveTexturePngError::WriteRequest)?;

        let req = ReplaySaveTexturePngRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_path: resolve_path_string_from_cwd(cwd, &req.output_path),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplaySaveTexturePngError::ParseJson)?,
        )
        .map_err(ReplaySaveTexturePngError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes =
            std::fs::read(&response_path).map_err(ReplaySaveTexturePngError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplaySaveTexturePngResponse> =
            serde_json::from_slice(&bytes).map_err(ReplaySaveTexturePngError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplaySaveTexturePngError::ScriptError("missing result".into()))
        } else {
            Err(ReplaySaveTexturePngError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    pub fn replay_save_outputs_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveOutputsPngRequest,
    ) -> Result<ReplaySaveOutputsPngResponse, ReplaySaveOutputsPngError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir)
            .map_err(ReplaySaveOutputsPngError::CreateScriptsDir)?;

        let script_path = scripts_dir.join("replay_save_outputs_png_json.py");
        write_script_file(&script_path, REPLAY_SAVE_OUTPUTS_PNG_JSON_PY)
            .map_err(ReplaySaveOutputsPngError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "replay_save_outputs_png")
            .map_err(ReplaySaveOutputsPngError::CreateScriptsDir)?;
        let request_path = run_dir.join("replay_save_outputs_png_json.request.json");
        let response_path = run_dir.join("replay_save_outputs_png_json.response.json");
        remove_if_exists(&response_path).map_err(ReplaySaveOutputsPngError::WriteRequest)?;

        let req = ReplaySaveOutputsPngRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_dir: resolve_path_string_from_cwd(cwd, &req.output_dir),
            ..req.clone()
        };
        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ReplaySaveOutputsPngError::ParseJson)?,
        )
        .map_err(ReplaySaveOutputsPngError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;

        let _ = result;
        let bytes =
            std::fs::read(&response_path).map_err(ReplaySaveOutputsPngError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ReplaySaveOutputsPngResponse> =
            serde_json::from_slice(&bytes).map_err(ReplaySaveOutputsPngError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ReplaySaveOutputsPngError::ScriptError("missing result".into()))
        } else {
            Err(ReplaySaveOutputsPngError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }
}

const REPLAY_LIST_TEXTURES_JSON_PY: &str = include_str!("../scripts/replay_list_textures_json.py");

const REPLAY_PICK_PIXEL_JSON_PY: &str = include_str!("../scripts/replay_pick_pixel_json.py");

const REPLAY_SAVE_TEXTURE_PNG_JSON_PY: &str =
    include_str!("../scripts/replay_save_texture_png_json.py");

const REPLAY_SAVE_OUTPUTS_PNG_JSON_PY: &str =
    include_str!("../scripts/replay_save_outputs_png_json.py");
