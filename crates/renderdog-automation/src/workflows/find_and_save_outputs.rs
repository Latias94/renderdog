use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    FindEventsError, FindEventsRequest, FindEventsResponse, RenderDocInstallation,
    ReplaySaveOutputsPngError, ReplaySaveOutputsPngRequest, ReplaySaveOutputsPngResponse,
    default_capture_basename, resolve_export_output_dir_from_cwd, resolve_path_string_from_cwd,
};

fn default_true() -> bool {
    true
}

fn default_max_results() -> Option<u32> {
    Some(200)
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FindEventSelection {
    First,
    #[default]
    Last,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsAndSaveOutputsPngRequest {
    pub capture_path: String,
    #[serde(default)]
    pub selection: FindEventSelection,
    #[serde(default = "default_true")]
    pub only_drawcalls: bool,
    #[serde(default)]
    pub marker_prefix: Option<String>,
    #[serde(default)]
    pub event_id_min: Option<u32>,
    #[serde(default)]
    pub event_id_max: Option<u32>,
    #[serde(default)]
    pub name_contains: Option<String>,
    #[serde(default)]
    pub marker_contains: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default = "default_max_results")]
    pub max_results: Option<u32>,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
    #[serde(default)]
    pub include_depth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsAndSaveOutputsPngResponse {
    pub find: FindEventsResponse,
    pub selected_event_id: u32,
    pub replay: ReplaySaveOutputsPngResponse,
}

#[derive(Debug, Error)]
pub enum FindEventsAndSaveOutputsPngError {
    #[error("find events failed: {0}")]
    Find(#[from] FindEventsError),
    #[error("no matching events found")]
    NoMatchingEvents,
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("save outputs PNG failed: {0}")]
    Replay(#[from] ReplaySaveOutputsPngError),
}

impl RenderDocInstallation {
    pub fn find_events_and_save_outputs_png(
        &self,
        cwd: &Path,
        req: &FindEventsAndSaveOutputsPngRequest,
    ) -> Result<FindEventsAndSaveOutputsPngResponse, FindEventsAndSaveOutputsPngError> {
        let capture_path = resolve_path_string_from_cwd(cwd, &req.capture_path);

        let find = self.find_events(
            cwd,
            &FindEventsRequest {
                capture_path: capture_path.clone(),
                only_drawcalls: req.only_drawcalls,
                marker_prefix: req.marker_prefix.clone(),
                event_id_min: req.event_id_min,
                event_id_max: req.event_id_max,
                name_contains: req.name_contains.clone(),
                marker_contains: req.marker_contains.clone(),
                case_sensitive: req.case_sensitive,
                max_results: req.max_results,
            },
        )?;

        let selected_event_id = match req.selection {
            FindEventSelection::First => find.first_event_id,
            FindEventSelection::Last => find.last_event_id,
        }
        .ok_or(FindEventsAndSaveOutputsPngError::NoMatchingEvents)?;

        let output_dir = resolve_export_output_dir_from_cwd(cwd, req.output_dir.as_deref());
        std::fs::create_dir_all(&output_dir)
            .map_err(FindEventsAndSaveOutputsPngError::CreateOutputDir)?;

        let basename = req
            .basename
            .clone()
            .unwrap_or_else(|| default_capture_basename(&capture_path));

        let replay = self.replay_save_outputs_png(
            cwd,
            &ReplaySaveOutputsPngRequest {
                capture_path,
                event_id: Some(selected_event_id),
                output_dir: Some(output_dir.display().to_string()),
                basename: Some(basename),
                include_depth: req.include_depth,
            },
        )?;

        Ok(FindEventsAndSaveOutputsPngResponse {
            find,
            selected_event_id,
            replay,
        })
    }
}
