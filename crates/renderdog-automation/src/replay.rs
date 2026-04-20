use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::RenderDocInstallation;
use crate::qrenderdoc_jobs::{
    REPLAY_LIST_TEXTURES_JOB, REPLAY_PICK_PIXEL_JOB, REPLAY_SAVE_OUTPUTS_PNG_JOB,
    REPLAY_SAVE_TEXTURE_PNG_JOB,
};
use crate::{
    QRenderDocJsonError, normalize_capture_path, prepare_export_target,
    resolve_path_string_from_cwd,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
}

impl ReplayListTexturesRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture_path: normalize_capture_path(cwd, &self.capture_path),
            ..self.clone()
        }
    }
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

impl ReplayPickPixelRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture_path: normalize_capture_path(cwd, &self.capture_path),
            ..self.clone()
        }
    }
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

impl ReplaySaveTexturePngRequest {
    pub(crate) fn resolved_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture_path: resolve_path_string_from_cwd(cwd, &self.capture_path),
            output_path: resolve_path_string_from_cwd(cwd, &self.output_path),
            ..self.clone()
        }
    }
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
    #[serde(default)]
    pub event_id: Option<u32>,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
    #[serde(default)]
    pub include_depth: bool,
}

impl ReplaySaveOutputsPngRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Result<Self, std::io::Error> {
        let prepared = prepare_export_target(
            cwd,
            &self.capture_path,
            self.output_dir.as_deref(),
            self.basename.as_deref(),
        )?;

        Ok(Self {
            capture_path: prepared.capture_path,
            output_dir: Some(prepared.output_dir),
            basename: Some(prepared.basename),
            ..self.clone()
        })
    }
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

pub type ReplayListTexturesError = QRenderDocJsonError;
pub type ReplayPickPixelError = QRenderDocJsonError;
pub type ReplaySaveTexturePngError = QRenderDocJsonError;

#[derive(Debug, Error)]
pub enum ReplaySaveOutputsPngError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error(transparent)]
    Replay(#[from] QRenderDocJsonError),
}

impl RenderDocInstallation {
    pub fn replay_list_textures(
        &self,
        cwd: &Path,
        req: &ReplayListTexturesRequest,
    ) -> Result<ReplayListTexturesResponse, ReplayListTexturesError> {
        let req = req.normalized_in_cwd(cwd);

        self.run_qrenderdoc_json_job(cwd, REPLAY_LIST_TEXTURES_JOB, &req)
    }

    pub fn replay_pick_pixel(
        &self,
        cwd: &Path,
        req: &ReplayPickPixelRequest,
    ) -> Result<ReplayPickPixelResponse, ReplayPickPixelError> {
        let req = req.normalized_in_cwd(cwd);

        self.run_qrenderdoc_json_job(cwd, REPLAY_PICK_PIXEL_JOB, &req)
    }

    pub fn replay_save_texture_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveTexturePngRequest,
    ) -> Result<ReplaySaveTexturePngResponse, ReplaySaveTexturePngError> {
        let req = req.resolved_in_cwd(cwd);

        self.run_qrenderdoc_json_job(cwd, REPLAY_SAVE_TEXTURE_PNG_JOB, &req)
    }

    pub fn replay_save_outputs_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveOutputsPngRequest,
    ) -> Result<ReplaySaveOutputsPngResponse, ReplaySaveOutputsPngError> {
        let req = req
            .normalized_in_cwd(cwd)
            .map_err(ReplaySaveOutputsPngError::CreateOutputDir)?;

        self.run_qrenderdoc_json_job(cwd, REPLAY_SAVE_OUTPUTS_PNG_JOB, &req)
            .map_err(ReplaySaveOutputsPngError::from)
    }
}
