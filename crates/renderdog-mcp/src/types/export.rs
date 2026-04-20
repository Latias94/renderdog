use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type ExportBundleRequest = CwdRequest<renderdog::ExportBundleRequest>;
pub(crate) type ExportBundleResponse = renderdog::ExportBundleResponse;
