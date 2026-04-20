use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJob, define_qrenderdoc_json_job_error};
use crate::{normalize_capture_path, prepare_export_target, resolve_path_string_from_cwd};

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
    #[serde(default)]
    pub event_id: Option<u32>,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
    #[serde(default)]
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

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum ReplayListTexturesError {
        create_dir_variant: CreateScriptsDir => "failed to create scripts dir: {0}",
        parse_json_message: "failed to parse JSON: {0}",
    }
}

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum ReplayPickPixelError {
        create_dir_variant: CreateScriptsDir => "failed to create scripts dir: {0}",
        parse_json_message: "failed to parse JSON: {0}",
    }
}

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum ReplaySaveTexturePngError {
        create_dir_variant: CreateScriptsDir => "failed to create scripts dir: {0}",
        parse_json_message: "failed to parse JSON: {0}",
    }
}

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum ReplaySaveOutputsPngError {
        create_dir_variant: CreateScriptsDir => "failed to create scripts dir: {0}",
        parse_json_message: "failed to parse JSON: {0}",
        extra_variant: CreateOutputDir(std::io::Error) => "failed to create output dir: {0}",
    }
}

impl RenderDocInstallation {
    pub fn replay_list_textures(
        &self,
        cwd: &Path,
        req: &ReplayListTexturesRequest,
    ) -> Result<ReplayListTexturesResponse, ReplayListTexturesError> {
        let req = ReplayListTexturesRequest {
            capture_path: normalize_capture_path(cwd, &req.capture_path),
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(cwd, REPLAY_LIST_TEXTURES_JOB, &req)
            .map_err(ReplayListTexturesError::from)
    }

    pub fn replay_pick_pixel(
        &self,
        cwd: &Path,
        req: &ReplayPickPixelRequest,
    ) -> Result<ReplayPickPixelResponse, ReplayPickPixelError> {
        let req = ReplayPickPixelRequest {
            capture_path: normalize_capture_path(cwd, &req.capture_path),
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(cwd, REPLAY_PICK_PIXEL_JOB, &req)
            .map_err(ReplayPickPixelError::from)
    }

    pub fn replay_save_texture_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveTexturePngRequest,
    ) -> Result<ReplaySaveTexturePngResponse, ReplaySaveTexturePngError> {
        let req = ReplaySaveTexturePngRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_path: resolve_path_string_from_cwd(cwd, &req.output_path),
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(cwd, REPLAY_SAVE_TEXTURE_PNG_JOB, &req)
            .map_err(ReplaySaveTexturePngError::from)
    }

    pub fn replay_save_outputs_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveOutputsPngRequest,
    ) -> Result<ReplaySaveOutputsPngResponse, ReplaySaveOutputsPngError> {
        let prepared = prepare_export_target(
            cwd,
            &req.capture_path,
            req.output_dir.as_deref(),
            req.basename.as_deref(),
        )
        .map_err(ReplaySaveOutputsPngError::CreateOutputDir)?;

        let req = ReplaySaveOutputsPngRequest {
            capture_path: prepared.capture_path,
            output_dir: Some(prepared.output_dir),
            basename: Some(prepared.basename),
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(cwd, REPLAY_SAVE_OUTPUTS_PNG_JOB, &req)
            .map_err(ReplaySaveOutputsPngError::from)
    }
}

const REPLAY_LIST_TEXTURES_JSON_PY: &str = include_str!("../scripts/replay_list_textures_json.py");

const REPLAY_LIST_TEXTURES_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "replay_list_textures",
    "replay_list_textures_json.py",
    REPLAY_LIST_TEXTURES_JSON_PY,
);

const REPLAY_PICK_PIXEL_JSON_PY: &str = include_str!("../scripts/replay_pick_pixel_json.py");

const REPLAY_PICK_PIXEL_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "replay_pick_pixel",
    "replay_pick_pixel_json.py",
    REPLAY_PICK_PIXEL_JSON_PY,
);

const REPLAY_SAVE_TEXTURE_PNG_JSON_PY: &str =
    include_str!("../scripts/replay_save_texture_png_json.py");

const REPLAY_SAVE_TEXTURE_PNG_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "replay_save_texture_png",
    "replay_save_texture_png_json.py",
    REPLAY_SAVE_TEXTURE_PNG_JSON_PY,
);

const REPLAY_SAVE_OUTPUTS_PNG_JSON_PY: &str =
    include_str!("../scripts/replay_save_outputs_png_json.py");

const REPLAY_SAVE_OUTPUTS_PNG_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "replay_save_outputs_png",
    "replay_save_outputs_png_json.py",
    REPLAY_SAVE_OUTPUTS_PNG_JSON_PY,
);
