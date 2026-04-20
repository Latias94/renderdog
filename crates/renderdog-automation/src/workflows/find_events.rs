use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::normalize_capture_path;
use crate::scripting::{QRenderDocJsonJob, define_qrenderdoc_json_job_error};

use super::{FindEventsRequest, FindEventsResponse};
use crate::CaptureInput;

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum FindEventsError {
        create_dir_variant: CreateScriptsDir => "failed to create scripts dir: {0}",
        parse_json_message: "failed to parse JSON: {0}",
    }
}

impl RenderDocInstallation {
    pub fn find_events(
        &self,
        cwd: &Path,
        req: &FindEventsRequest,
    ) -> Result<FindEventsResponse, FindEventsError> {
        let req = FindEventsRequest {
            capture: CaptureInput {
                capture_path: normalize_capture_path(cwd, &req.capture.capture_path),
            },
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(cwd, FIND_EVENTS_JOB, &req)
            .map_err(FindEventsError::from)
    }
}

const FIND_EVENTS_JSON_PY: &str = include_str!("../../scripts/find_events_json.py");

const FIND_EVENTS_JOB: QRenderDocJsonJob =
    QRenderDocJsonJob::new("find_events", "find_events_json.py", FIND_EVENTS_JSON_PY);
