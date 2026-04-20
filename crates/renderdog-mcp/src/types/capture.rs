use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type SaveThumbnailRequest = CwdRequest<renderdog::SaveThumbnailRequest>;
pub(crate) type SaveThumbnailResponse = renderdog::SaveThumbnailResponse;
pub(crate) type OpenCaptureUiRequest = CwdRequest<renderdog::OpenCaptureUiRequest>;
pub(crate) type OpenCaptureUiResponse = renderdog::OpenCaptureUiResponse;
