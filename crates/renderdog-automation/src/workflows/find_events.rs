use std::path::Path;

use crate::qrenderdoc_jobs::FIND_EVENTS_JOB;
use crate::{QRenderDocJobError, RenderDocInstallation};

use super::{FindEventsRequest, FindEventsResponse};

pub type FindEventsError = QRenderDocJobError;

impl RenderDocInstallation {
    pub fn find_events(
        &self,
        cwd: &Path,
        req: &FindEventsRequest,
    ) -> Result<FindEventsResponse, FindEventsError> {
        self.run_qrenderdoc_job_in_cwd(cwd, FIND_EVENTS_JOB, req)
    }
}
