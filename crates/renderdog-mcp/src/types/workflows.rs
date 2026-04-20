use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type CaptureAndExportBundleRequest =
    CwdRequest<renderdog::CaptureAndExportBundleRequest>;

pub(crate) type CaptureAndExportBundleResponse = renderdog::CaptureAndExportBundleResponse;
