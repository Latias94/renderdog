use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CaptureInput, DrawcallScope, EventFilter, ExportOutput, FindEventsError, FindEventsLimit,
    FindEventsRequest, FindEventsResponse, RenderDocInstallation, ReplaySaveOutputsPngError,
    ReplaySaveOutputsPngRequest, ReplaySaveOutputsPngResponse,
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

impl FindEventSelection {
    fn select_event_id(self, find: &FindEventsResponse) -> Option<u32> {
        match self {
            Self::First => find.first_event_id,
            Self::Last => find.last_event_id,
        }
    }
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

impl FindEventsAndSaveOutputsPngResponse {
    fn from_parts(
        find: FindEventsResponse,
        selected_event_id: u32,
        replay: ReplaySaveOutputsPngResponse,
    ) -> Self {
        Self {
            find,
            selected_event_id,
            replay,
        }
    }
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

impl FindEventsAndSaveOutputsPngRequest {
    fn find_request(&self, capture: CaptureInput) -> FindEventsRequest {
        FindEventsRequest {
            capture,
            drawcall_scope: DrawcallScope {
                only_drawcalls: self.only_drawcalls,
            },
            filter: self.filter.clone(),
            limit: self.limit,
        }
    }

    fn replay_request(
        &self,
        capture_path: String,
        selected_event_id: u32,
    ) -> ReplaySaveOutputsPngRequest {
        ReplaySaveOutputsPngRequest {
            capture_path,
            event_id: Some(selected_event_id),
            output_dir: self.output.output_dir.clone(),
            basename: self.output.basename.clone(),
            include_depth: self.include_depth,
        }
    }
}

impl RenderDocInstallation {
    pub fn find_events_and_save_outputs_png(
        &self,
        cwd: &Path,
        req: &FindEventsAndSaveOutputsPngRequest,
    ) -> Result<FindEventsAndSaveOutputsPngResponse, FindEventsAndSaveOutputsPngError> {
        let capture = req.capture.normalized_in_cwd(cwd);
        let find = self.find_events(cwd, &req.find_request(capture.clone()))?;
        let selected_event_id = req
            .selection
            .select_event_id(&find)
            .ok_or(FindEventsAndSaveOutputsPngError::NoMatchingEvents)?;

        let replay = self.replay_save_outputs_png(
            cwd,
            &req.replay_request(capture.capture_path, selected_event_id),
        )?;

        Ok(FindEventsAndSaveOutputsPngResponse::from_parts(
            find,
            selected_event_id,
            replay,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FindEventSelection, FindEventsAndSaveOutputsPngRequest, FindEventsLimit, FindEventsResponse,
    };
    use crate::{CaptureInput, EventFilter, ExportOutput};

    #[test]
    fn find_event_selection_picks_first_or_last_match() {
        let find = FindEventsResponse {
            capture_path: "/tmp/capture.rdc".to_string(),
            total_matches: 2,
            truncated: false,
            first_event_id: Some(11),
            last_event_id: Some(42),
            matches: Vec::new(),
        };

        assert_eq!(FindEventSelection::First.select_event_id(&find), Some(11));
        assert_eq!(FindEventSelection::Last.select_event_id(&find), Some(42));
    }

    #[test]
    fn workflow_request_builds_find_and_replay_requests() {
        let req = FindEventsAndSaveOutputsPngRequest {
            capture: CaptureInput {
                capture_path: "captures/frame.rdc".to_string(),
            },
            selection: FindEventSelection::Last,
            only_drawcalls: true,
            filter: EventFilter {
                marker_contains: Some("fret".to_string()),
                ..EventFilter::default()
            },
            limit: FindEventsLimit {
                max_results: Some(5),
            },
            output: ExportOutput {
                output_dir: Some("artifacts/replay".to_string()),
                basename: Some("frame".to_string()),
            },
            include_depth: true,
        };

        let capture = CaptureInput {
            capture_path: "/tmp/project/captures/frame.rdc".to_string(),
        };
        let find = req.find_request(capture.clone());
        let replay = req.replay_request(capture.capture_path, 99);

        assert!(find.drawcall_scope.only_drawcalls);
        assert_eq!(find.filter.marker_contains.as_deref(), Some("fret"));
        assert_eq!(find.limit.max_results, Some(5));
        assert_eq!(replay.capture_path, "/tmp/project/captures/frame.rdc");
        assert_eq!(replay.event_id, Some(99));
        assert_eq!(replay.output_dir.as_deref(), Some("artifacts/replay"));
        assert_eq!(replay.basename.as_deref(), Some("frame"));
        assert!(replay.include_depth);
    }
}
