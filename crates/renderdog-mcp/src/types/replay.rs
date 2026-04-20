use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type ReplayListTexturesRequest = CwdRequest<renderdog::ReplayListTexturesRequest>;
pub(crate) type ReplayPickPixelRequest = CwdRequest<renderdog::ReplayPickPixelRequest>;
pub(crate) type ReplaySaveTexturePngRequest = CwdRequest<renderdog::ReplaySaveTexturePngRequest>;
pub(crate) type ReplaySaveOutputsPngRequest = CwdRequest<renderdog::ReplaySaveOutputsPngRequest>;
