use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CaptureInput, DrawcallScope, EventFilter, ExportOutput, FindEventsError, FindEventsLimit,
    FindEventsRequest, FindEventsResponse, RenderDocInstallation, ReplaySaveOutputsPngError,
    ReplaySaveOutputsPngRequest, ReplaySaveOutputsPngResponse, normalize_capture_path,
};

fn default_true() -> bool {
    true
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
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(default)]
    pub selection: FindEventSelection,
    #[serde(default = "default_true")]
    pub only_drawcalls: bool,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub limit: FindEventsLimit,
    #[serde(flatten)]
    pub output: ExportOutput,
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
    Replay(ReplaySaveOutputsPngError),
}

impl From<ReplaySaveOutputsPngError> for FindEventsAndSaveOutputsPngError {
    fn from(value: ReplaySaveOutputsPngError) -> Self {
        match value {
            ReplaySaveOutputsPngError::CreateOutputDir(err) => Self::CreateOutputDir(err),
            other => Self::Replay(other),
        }
    }
}

impl RenderDocInstallation {
    pub fn find_events_and_save_outputs_png(
        &self,
        cwd: &Path,
        req: &FindEventsAndSaveOutputsPngRequest,
    ) -> Result<FindEventsAndSaveOutputsPngResponse, FindEventsAndSaveOutputsPngError> {
        let capture_path = normalize_capture_path(cwd, &req.capture.capture_path);

        let find = self.find_events(
            cwd,
            &FindEventsRequest {
                capture: CaptureInput {
                    capture_path: capture_path.clone(),
                },
                drawcall_scope: DrawcallScope {
                    only_drawcalls: req.only_drawcalls,
                },
                filter: req.filter.clone(),
                limit: req.limit,
            },
        )?;

        let selected_event_id = match req.selection {
            FindEventSelection::First => find.first_event_id,
            FindEventSelection::Last => find.last_event_id,
        }
        .ok_or(FindEventsAndSaveOutputsPngError::NoMatchingEvents)?;

        let replay = self.replay_save_outputs_png(
            cwd,
            &ReplaySaveOutputsPngRequest {
                capture_path,
                event_id: Some(selected_event_id),
                output_dir: req.output.output_dir.clone(),
                basename: req.output.basename.clone(),
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
