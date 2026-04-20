use std::path::Path;

use crate::qrenderdoc_jobs::FIND_EVENTS_JOB;
use crate::{QRenderDocJsonError, RenderDocInstallation};

use super::{FindEventsRequest, FindEventsResponse};

pub type FindEventsError = QRenderDocJsonError;

impl RenderDocInstallation {
    pub fn find_events(
        &self,
        cwd: &Path,
        req: &FindEventsRequest,
    ) -> Result<FindEventsResponse, FindEventsError> {
        self.run_prepared_qrenderdoc_json_job(cwd, FIND_EVENTS_JOB, req)
    }
}
