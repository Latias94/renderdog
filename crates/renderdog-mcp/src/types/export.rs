use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type ExportActionsRequest = CwdRequest<renderdog::ExportActionsRequest>;
pub(crate) type ExportBindingsIndexRequest = CwdRequest<renderdog::ExportBindingsIndexRequest>;
pub(crate) type ExportBundleRequest = CwdRequest<renderdog::ExportBundleRequest>;
pub(crate) type ExportBundleResponse = renderdog::ExportBundleResponse;
