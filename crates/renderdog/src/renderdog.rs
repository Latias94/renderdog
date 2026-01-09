use std::ops::Deref;

use crate::{InAppError, RenderDocInApp};

pub struct RenderDog {
    inner: RenderDocInApp,
}

impl RenderDog {
    pub fn new() -> Result<Self, InAppError> {
        Ok(Self {
            inner: RenderDocInApp::try_connect_or_load_default()?,
        })
    }

    #[cfg(all(unix, target_os = "linux"))]
    pub fn new_noload_first() -> Result<Self, InAppError> {
        Ok(Self {
            inner: RenderDocInApp::try_connect_noload_or_load_default()?,
        })
    }

    pub fn connect_injected() -> Result<Self, InAppError> {
        Ok(Self {
            inner: RenderDocInApp::try_connect()?,
        })
    }

    pub fn load(path_or_name: &str) -> Result<Self, InAppError> {
        Ok(Self {
            inner: RenderDocInApp::try_load_and_connect(path_or_name)?,
        })
    }

    pub fn inner(&self) -> &RenderDocInApp {
        &self.inner
    }
}

impl Deref for RenderDog {
    type Target = RenderDocInApp;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
