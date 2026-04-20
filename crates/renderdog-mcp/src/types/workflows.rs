use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type CaptureAndExportActionsRequest =
    CwdRequest<renderdog::CaptureAndExportActionsRequest>;
pub(crate) type CaptureAndExportBindingsIndexRequest =
    CwdRequest<renderdog::CaptureAndExportBindingsIndexRequest>;
pub(crate) type CaptureAndExportBundleRequest =
    CwdRequest<renderdog::CaptureAndExportBundleRequest>;

pub(crate) type CaptureAndExportActionsResponse = renderdog::CaptureAndExportActionsResponse;
pub(crate) type CaptureAndExportBindingsIndexResponse =
    renderdog::CaptureAndExportBindingsIndexResponse;
pub(crate) type CaptureAndExportBundleResponse = renderdog::CaptureAndExportBundleResponse;
