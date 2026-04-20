use std::path::Path;

use crate::scripting::QRenderDocJsonJob;
use crate::{QRenderDocJsonError, RenderDocInstallation};

use super::{FindEventsRequest, FindEventsResponse};

pub type FindEventsError = QRenderDocJsonError;

impl RenderDocInstallation {
    pub fn find_events(
        &self,
        cwd: &Path,
        req: &FindEventsRequest,
    ) -> Result<FindEventsResponse, FindEventsError> {
        let req = req.normalized_in_cwd(cwd);

        self.run_qrenderdoc_json_job(cwd, FIND_EVENTS_JOB, &req)
    }
}

const FIND_EVENTS_JSON_PY: &str = include_str!("../../scripts/find_events_json.py");

const FIND_EVENTS_JOB: QRenderDocJsonJob =
    QRenderDocJsonJob::new("find_events", "find_events_json.py", FIND_EVENTS_JSON_PY);
