use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type FindEventsRequest = CwdRequest<renderdog::FindEventsRequest>;
pub(crate) type FindEventsAndSaveOutputsPngRequest =
    CwdRequest<renderdog::FindEventsAndSaveOutputsPngRequest>;

pub(crate) type FindEventsAndSaveOutputsPngResponse =
    renderdog::FindEventsAndSaveOutputsPngResponse;
