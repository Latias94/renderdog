use std::path::PathBuf;

use schemars::JsonSchema;
use serde::Deserialize;

pub(crate) mod capture;
pub(crate) mod defaults;
pub(crate) mod diagnostics;
pub(crate) mod export;
pub(crate) mod find;
pub(crate) mod replay;
pub(crate) mod workflows;

pub(crate) use capture::*;
pub(crate) use diagnostics::*;
pub(crate) use export::*;
pub(crate) use find::*;
pub(crate) use replay::*;
pub(crate) use workflows::*;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CwdRequest<T> {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: T,
}

impl<T> CwdRequest<T> {
    pub(crate) fn resolve_cwd(&self) -> Result<PathBuf, String> {
        crate::paths::resolve_base_cwd(self.cwd.clone())
    }

    pub(crate) fn into_parts(self) -> Result<(PathBuf, T), String> {
        Ok((crate::paths::resolve_base_cwd(self.cwd)?, self.inner))
    }
}
